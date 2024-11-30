//! 2d Viewport for Bevy
use bevy::{
    prelude::*,
    render::{
        camera::RenderTarget,
        render_resource::{Extent3d, TextureFormat, TextureUsages},
        view::RenderLayers,
    },
    ui::ui_layout_system,
};
use bevy_editor_camera::{EditorCamera2d, EditorCamera2dPlugin};
use bevy_editor_styles::Theme;
use bevy_infinite_grid::{InfiniteGrid, InfiniteGridPlugin, InfiniteGridSettings};
use bevy_pane_layout::prelude::*;

/// The identifier for the 2D Viewport.
/// This is present on any pane that is a 2D Viewport.
#[derive(Component)]
pub struct Bevy2dViewport {
    camera_id: Entity,
}

impl Default for Bevy2dViewport {
    fn default() -> Self {
        Bevy2dViewport {
            camera_id: Entity::PLACEHOLDER,
        }
    }
}

/// Plugin for the 2D Viewport pane.
pub struct Viewport2dPanePlugin;

impl Plugin for Viewport2dPanePlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<InfiniteGridPlugin>() {
            app.add_plugins(InfiniteGridPlugin);
        }
        app.add_plugins(EditorCamera2dPlugin)
            .add_systems(Startup, setup)
            .add_systems(
                PostUpdate,
                update_render_target_size.after(ui_layout_system),
            )
            .add_observer(
                |trigger: Trigger<OnRemove, Bevy2dViewport>,
                 mut commands: Commands,
                 query: Query<&Bevy2dViewport>| {
                    // Despawn the viewport camera
                    commands
                        .entity(query.get(trigger.entity()).unwrap().camera_id)
                        .despawn_recursive();
                },
            );

        app.register_pane("Viewport 2D", on_pane_creation);
    }
}

fn setup(mut commands: Commands, theme: Res<Theme>) {
    commands.spawn((
        InfiniteGrid,
        InfiniteGridSettings {
            scale: 100.,
            dot_fadeout_strength: 0.,
            x_axis_color: theme.viewport.x_axis_color,
            z_axis_color: theme.viewport.y_axis_color,
            major_line_color: theme.viewport.grid_major_line_color,
            minor_line_color: theme.viewport.grid_minor_line_color,
            ..default()
        },
        Transform::from_rotation(Quat::from_rotation_arc(Vec3::Y, Vec3::Z)),
        RenderLayers::layer(2),
    ));
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

    let image_id = commands
        .spawn((
            ImageNode::new(image_handle.clone()),
            Node {
                position_type: PositionType::Absolute,
                top: Val::ZERO,
                bottom: Val::ZERO,
                left: Val::ZERO,
                right: Val::ZERO,
                ..default()
            },
        ))
        .set_parent(structure.content)
        .id();

    let camera_id = commands
        .spawn((
            Camera2d,
            EditorCamera2d {
                enabled: false,
                ..default()
            },
            Camera {
                target: RenderTarget::Image(image_handle),
                clear_color: ClearColorConfig::Custom(theme.viewport.background_color),
                ..default()
            },
            RenderLayers::from_layers(&[0, 2]),
        ))
        .id();

    commands
        .entity(image_id)
        .observe(
            move |_trigger: Trigger<Pointer<Move>>, mut query: Query<&mut EditorCamera2d>| {
                let mut editor_camera = query.get_mut(camera_id).unwrap();
                editor_camera.enabled = true;
            },
        )
        .observe(
            move |_trigger: Trigger<Pointer<Out>>, mut query: Query<&mut EditorCamera2d>| {
                query.get_mut(camera_id).unwrap().enabled = false;
            },
        );

    commands
        .entity(structure.root)
        .insert(Bevy2dViewport { camera_id });
}

fn update_render_target_size(
    query: Query<(Entity, &Bevy2dViewport)>,
    mut camera_query: Query<(&Camera, &mut EditorCamera2d)>,
    content: Query<&PaneContentNode>,
    children_query: Query<&Children>,
    pos_query: Query<
        (&ComputedNode, &GlobalTransform),
        Or<(Changed<ComputedNode>, Changed<GlobalTransform>)>,
    >,
    mut images: ResMut<Assets<Image>>,
) {
    for (pane_root, viewport) in &query {
        let content_node_id = children_query
            .iter_descendants(pane_root)
            .find(|e| content.contains(*e))
            .unwrap();

        let Ok((computed_node, global_transform)) = pos_query.get(content_node_id) else {
            continue;
        };
        // TODO Convert to physical pixels
        let content_node_size = computed_node.size();

        let node_position = global_transform.translation().xy();
        let rect = Rect::from_center_size(node_position, computed_node.size());

        let (camera, mut editor_camera) = camera_query.get_mut(viewport.camera_id).unwrap();

        editor_camera.viewport_override = Some(rect);

        let image_handle = camera.target.as_image().unwrap();
        let size = Extent3d {
            width: u32::max(1, content_node_size.x as u32),
            height: u32::max(1, content_node_size.y as u32),
            depth_or_array_layers: 1,
        };
        images.get_mut(image_handle).unwrap().resize(size);
    }
}
