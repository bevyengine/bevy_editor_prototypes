//! 3D Viewport for Bevy
use bevy::{
    picking::{
        pointer::{Location, PointerId, PointerInput, PointerLocation},
        PickSet,
    },
    prelude::*,
    render::{
        camera::{NormalizedRenderTarget, RenderTarget},
        render_resource::{Extent3d, TextureFormat, TextureUsages},
        view::RenderLayers,
    },
    ui::ui_layout_system,
};
use bevy_editor_cam::prelude::{DefaultEditorCamPlugins, EditorCam};
use bevy_editor_styles::Theme;
use bevy_infinite_grid::{InfiniteGrid, InfiniteGridPlugin, InfiniteGridSettings};
use bevy_pane_layout::{PaneContentNode, PaneRegistry};

/// The identifier for the 3D Viewport.
/// This is present on any pane that is a 3D Viewport.
#[derive(Component)]
pub struct Bevy3dViewport {
    camera: Entity,
}

impl Default for Bevy3dViewport {
    fn default() -> Self {
        Bevy3dViewport {
            camera: Entity::PLACEHOLDER,
        }
    }
}

/// Plugin for the 3D Viewport pane.
pub struct Viewport3dPanePlugin;

impl Plugin for Viewport3dPanePlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<InfiniteGridPlugin>() {
            app.add_plugins(InfiniteGridPlugin);
        }
        app.add_plugins(DefaultEditorCamPlugins)
            .add_systems(Startup, setup)
            .add_systems(
                PreUpdate,
                render_target_picking_passthrough.in_set(PickSet::Last),
            )
            .add_systems(
                PostUpdate,
                update_render_target_size.after(ui_layout_system),
            )
            .add_observer(on_pane_creation)
            .add_observer(
                |trigger: Trigger<OnRemove, Bevy3dViewport>,
                 mut commands: Commands,
                 query: Query<&Bevy3dViewport>| {
                    // Despawn the viewport camera
                    commands
                        .entity(query.get(trigger.entity()).unwrap().camera)
                        .despawn_recursive();
                },
            );

        app.world_mut()
            .get_resource_or_init::<PaneRegistry>()
            .register("Viewport 3D", |mut commands, pane_root| {
                commands.entity(pane_root).insert(Bevy3dViewport::default());
            });
    }
}

#[derive(Component)]
struct Active;

// TODO This does not properly handle multiple windows.
/// Copies picking events and moves pointers through render-targets.
fn render_target_picking_passthrough(
    mut commands: Commands,
    viewports: Query<(Entity, &Bevy3dViewport)>,
    content: Query<&PaneContentNode>,
    children_query: Query<&Children>,
    node_query: Query<(&ComputedNode, &GlobalTransform, &UiImage), With<Active>>,
    mut pointers: Query<(&PointerId, &mut PointerLocation)>,
    mut pointer_input_reader: EventReader<PointerInput>,
) {
    for event in pointer_input_reader.read() {
        // Ignore the events we send to the render-targets
        if !matches!(event.location.target, NormalizedRenderTarget::Window(..)) {
            continue;
        }
        for (pane_root, _viewport) in &viewports {
            let content_node_id = children_query
                .iter_descendants(pane_root)
                .find(|e| content.contains(*e))
                .unwrap();

            let image_id = children_query.get(content_node_id).unwrap()[0];

            let Ok((computed_node, global_transform, ui_image)) = node_query.get(image_id) else {
                // Inactive viewport
                continue;
            };
            let node_rect =
                Rect::from_center_size(global_transform.translation().xy(), computed_node.size());

            let new_location = Location {
                position: event.location.position - node_rect.min,
                target: NormalizedRenderTarget::Image(ui_image.texture.clone()),
            };

            // Duplicate the event
            let mut new_event = event.clone();
            // Relocate the event to the render-target
            new_event.location = new_location.clone();
            // Resend the event
            commands.send_event(new_event);

            if let Some((_id, mut pointer_location)) = pointers
                .iter_mut()
                .find(|(pointer_id, _)| **pointer_id == event.pointer_id)
            {
                // Relocate the pointer to the render-target
                pointer_location.location = Some(new_location);
            }
        }
    }
}

fn setup(mut commands: Commands, theme: Res<Theme>) {
    commands.spawn((
        InfiniteGrid,
        InfiniteGridSettings {
            major_line_color: theme.background_color.0,
            minor_line_color: theme.pane_header_background_color.0,
            ..default()
        },
        RenderLayers::layer(1),
    ));
}

fn on_pane_creation(
    trigger: Trigger<OnAdd, Bevy3dViewport>,
    mut commands: Commands,
    children_query: Query<&Children>,
    mut query: Query<&mut Bevy3dViewport>,
    content: Query<&PaneContentNode>,
    mut images: ResMut<Assets<Image>>,
    theme: Res<Theme>,
) {
    let pane_root = trigger.entity();
    let content_node = children_query
        .iter_descendants(pane_root)
        .find(|e| content.contains(*e))
        .unwrap();

    let mut image = Image::default();

    image.texture_descriptor.usage |= TextureUsages::RENDER_ATTACHMENT;
    image.texture_descriptor.format = TextureFormat::Bgra8UnormSrgb;

    let image_handle = images.add(image);

    commands
        .spawn((
            UiImage {
                texture: image_handle.clone(),
                ..Default::default()
            },
            Node {
                position_type: PositionType::Absolute,
                top: Val::ZERO,
                bottom: Val::ZERO,
                left: Val::ZERO,
                right: Val::ZERO,
                ..default()
            },
        ))
        .set_parent(content_node)
        .observe(|trigger: Trigger<Pointer<Over>>, mut commands: Commands| {
            commands.entity(trigger.entity()).insert(Active);
        })
        .observe(|trigger: Trigger<Pointer<Out>>, mut commands: Commands| {
            commands.entity(trigger.entity()).remove::<Active>();
        });

    let camera_id = commands
        .spawn((
            Camera3d::default(),
            Camera {
                target: RenderTarget::Image(image_handle),
                clear_color: ClearColorConfig::Custom(theme.viewport_background_color),
                ..default()
            },
            EditorCam::default(),
            Transform::from_translation(Vec3::ONE * 5.).looking_at(Vec3::ZERO, Vec3::Y),
            RenderLayers::from_layers(&[0, 1]),
        ))
        .id();

    query.get_mut(pane_root).unwrap().camera = camera_id;
}

fn update_render_target_size(
    query: Query<(Entity, &Bevy3dViewport)>,
    mut camera_query: Query<&Camera>,
    content: Query<&PaneContentNode>,
    children_query: Query<&Children>,
    computed_node_query: Query<&ComputedNode, Changed<ComputedNode>>,
    mut images: ResMut<Assets<Image>>,
) {
    for (pane_root, viewport) in &query {
        let content_node_id = children_query
            .iter_descendants(pane_root)
            .find(|e| content.contains(*e))
            .unwrap();

        let Ok(computed_node) = computed_node_query.get(content_node_id) else {
            continue;
        };
        // TODO Convert to physical pixels
        let content_node_size = computed_node.size();

        let camera = camera_query.get_mut(viewport.camera).unwrap();

        let image_handle = camera.target.as_image().unwrap();
        let size = Extent3d {
            width: u32::max(1, content_node_size.x as u32),
            height: u32::max(1, content_node_size.y as u32),
            depth_or_array_layers: 1,
        };
        images.get_mut(image_handle).unwrap().resize(size);
    }
}
