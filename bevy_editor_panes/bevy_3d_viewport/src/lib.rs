//! 3D Viewport for Bevy
use bevy::{
    asset::uuid::Uuid,
    camera::{NormalizedRenderTarget, RenderTarget, visibility::RenderLayers},
    feathers::theme::ThemedText,
    picking::{
        PickingSystems,
        input::{mouse_pick_events, touch_pick_events},
        pointer::{Location, PointerId, PointerInput},
    },
    prelude::*,
    render::render_resource::{Extent3d, TextureFormat, TextureUsages},
    scene2::{CommandsSpawnScene, bsn, on},
    ui::ui_layout_system,
};
use bevy_editor_cam::prelude::{DefaultEditorCamPlugins, EditorCam};
use bevy_editor_styles::Theme;
use bevy_infinite_grid::{InfiniteGrid, InfiniteGridPlugin, InfiniteGridSettings};
use bevy_pane_layout::prelude::*;
use bevy_transform_gizmos::{TransformGizmo, prelude::*};
use view_gizmo::ViewGizmoPlugin;

use crate::{selection_box::SelectionBoxPlugin, view_gizmo::view_gizmo_node};

mod selection_box;
mod view_gizmo;

/// The identifier for the 3D Viewport.
/// This is present on any pane that is a 3D Viewport.
#[derive(Component)]
pub struct Bevy3dViewport {
    camera_id: Entity,
}

impl Default for Bevy3dViewport {
    fn default() -> Self {
        Bevy3dViewport {
            camera_id: Entity::PLACEHOLDER,
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

        app.add_plugins((DefaultEditorCamPlugins, ViewGizmoPlugin, SelectionBoxPlugin))
            .add_systems(Startup, setup)
            .add_systems(
                First,
                render_target_picking_passthrough
                    .in_set(PickingSystems::Input)
                    .after(touch_pick_events)
                    .after(mouse_pick_events),
            )
            .add_systems(
                PostUpdate,
                (
                    update_render_target_size.after(ui_layout_system),
                    disable_editor_cam_during_gizmo_interaction,
                ),
            )
            .add_observer(
                |trigger: On<Remove, Bevy3dViewport>,
                 mut commands: Commands,
                 query: Query<&Bevy3dViewport>| {
                    // Despawn the viewport camera
                    commands
                        .entity(query.get(trigger.event().event_target()).unwrap().camera_id)
                        .despawn();
                },
            );

        app.register_pane("Viewport 3D", on_pane_creation);
    }
}

/// Temporary. We will need a proper design for mutually exclusive controls.
fn disable_editor_cam_during_gizmo_interaction(
    transform_gizmo: Single<Ref<TransformGizmo>>,
    mut query: Query<&mut EditorCam>,
) {
    if !transform_gizmo.is_changed() {
        return;
    }
    let enable = transform_gizmo.interaction().is_none();
    for mut editor_cam in &mut query {
        editor_cam.enabled = enable;
    }
}

/// A viewport is considered active while the mouse is hovering over it.
#[derive(Component)]
struct Active;

// FIXME: This system makes a lot of assumptions and is therefore rather fragile. Does not handle multiple windows.
/// Sends copies of [`PointerInput`] event actions from the mouse pointer to pointers belonging to the viewport panes.
fn render_target_picking_passthrough(
    viewports: Query<(Entity, &Bevy3dViewport)>,
    content: Query<&PaneContentNode>,
    children_query: Query<&Children>,
    node_query: Query<(&ComputedNode, &UiGlobalTransform, &ImageNode), With<Active>>,
    mut pointer_input_reader: MessageReader<PointerInput>,
    // Using commands to output PointerInput events to avoid clashing with the MessageReader
    mut commands: Commands,
) {
    for event in pointer_input_reader.read() {
        // Ignore the events sent from this system by only copying events that come directly from the mouse.
        if event.pointer_id != PointerId::Mouse {
            continue;
        }
        for (pane_root, _viewport) in &viewports {
            let Some(content_node_id) = children_query
                .iter_descendants(pane_root)
                .find(|e| content.contains(*e))
            else {
                continue;
            };

            let image_id = children_query.get(content_node_id).unwrap()[0];
            let Ok((computed_node, global_transform, ui_image)) = node_query.get(image_id) else {
                // Inactive viewport
                continue;
            };
            let node_top_left = global_transform.translation - computed_node.size() / 2.;
            let position = event.location.position - node_top_left;
            let target = NormalizedRenderTarget::Image(ui_image.image.clone().into());

            let event_copy = PointerInput {
                action: event.action,
                location: Location { position, target },
                pointer_id: pointer_id_from_entity(pane_root),
            };

            commands.write_message(event_copy);
        }
    }
}

fn setup(mut commands: Commands, theme: Res<Theme>) {
    commands.spawn((
        InfiniteGrid,
        InfiniteGridSettings {
            x_axis_color: theme.viewport.x_axis_color,
            z_axis_color: theme.viewport.z_axis_color,
            major_line_color: theme.viewport.grid_major_line_color,
            minor_line_color: theme.viewport.grid_minor_line_color,
            ..default()
        },
        RenderLayers::layer(1),
    ));
}

/// Construct a pointer id from an entity. Used to tie the viewport panel root entity to a pointer id.
fn pointer_id_from_entity(entity: Entity) -> PointerId {
    let bits = entity.to_bits();
    PointerId::Custom(Uuid::from_u64_pair(bits, bits))
}

fn on_pane_creation(
    structure: In<PaneStructure>,
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    theme: Res<Theme>,
) {
    let mut image = Image::default();

    image.texture_descriptor.usage |= TextureUsages::RENDER_ATTACHMENT;
    image.texture_descriptor.format = TextureFormat::Bgra8UnormSrgb;

    let image_handle = images.add(image);

    // Spawn the cursor associated with this viewport pane.
    let pointer_id = pointer_id_from_entity(structure.root);
    commands.spawn((pointer_id, ChildOf(structure.root)));

    // Remove the existing structure
    commands.entity(structure.area).despawn();

    let image = image_handle.clone();
    commands
        .spawn_scene(bsn! {
            :editor_pane [
                :editor_pane_header [
                    (Text("3D Viewport") ThemedText),
                ],
                :editor_pane_body [
                    ImageNode::new(image.clone())
                    :fit_to_parent
                    on(|trigger: On<Pointer<Over>>, mut commands: Commands| {
                        commands.entity(trigger.event().event_target()).insert(Active);
                    })
                    on(|trigger: On<Pointer<Out>>, mut commands: Commands| {
                        commands.entity(trigger.event().event_target()).remove::<Active>();
                    })
                    [ :view_gizmo_node ]
                ],
            ]
        })
        .insert(ChildOf(structure.root));

    let camera_id = commands
        .spawn((
            Camera3d::default(),
            Camera {
                target: RenderTarget::Image(image_handle.into()),
                clear_color: ClearColorConfig::Custom(theme.viewport.background_color),
                ..default()
            },
            EditorCam::default(),
            GizmoCamera,
            Transform::from_translation(Vec3::ONE * 5.).looking_at(Vec3::ZERO, Vec3::Y),
            RenderLayers::from_layers(&[0, 1]),
            MeshPickingCamera,
        ))
        .id();

    commands
        .entity(structure.root)
        .insert(Bevy3dViewport { camera_id });
}

fn update_render_target_size(
    query: Query<(Entity, &Bevy3dViewport)>,
    mut camera_query: Query<&Camera>,
    bodies: Query<&PaneContentNode>,
    children_query: Query<&Children>,
    computed_node_query: Query<&ComputedNode, Changed<ComputedNode>>,
    mut images: ResMut<Assets<Image>>,
) {
    for (pane_root, viewport) in &query {
        let Some(pane_body) = children_query
            .iter_descendants(pane_root)
            .find(|e| bodies.contains(*e))
        else {
            continue;
        };

        let Ok(computed_node) = computed_node_query.get(pane_body) else {
            continue;
        };
        // TODO Convert to physical pixels
        let content_node_size = computed_node.size();

        let camera = camera_query.get_mut(viewport.camera_id).unwrap();

        let image_handle = camera.target.as_image().unwrap();
        let size = Extent3d {
            width: u32::max(1, content_node_size.x as u32),
            height: u32::max(1, content_node_size.y as u32),
            depth_or_array_layers: 1,
        };
        images.get_mut(image_handle).unwrap().resize(size);
    }
}
