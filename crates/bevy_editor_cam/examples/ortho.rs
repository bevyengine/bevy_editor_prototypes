//! Orthographic editor camera example.

use bevy::prelude::*;
use bevy_editor_cam::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, DefaultEditorCamPlugins))
        .add_systems(Startup, (setup, setup_ui))
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(10.0, 10.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        Projection::Orthographic(OrthographicProjection {
            scale: 0.01,
            ..OrthographicProjection::default_3d()
        }),
        // This component makes the camera controllable with this plugin.
        //
        // Important: the `with_initial_anchor_depth` is critical for an orthographic camera. Unlike
        // perspective, we can't rely on distant things being small to hide precision artifacts.
        // This means we need to be careful with the near and far plane of the camera, especially
        // because in orthographic, the depth precision is linear.
        //
        // This plugin uses the anchor (the point in space the user is interested in) to set the
        // orthographic scale, as well as the near and far planes. This can be a bit tricky if you
        // are unfamiliar with orthographic projections. Consider using an pseudo-ortho projection
        // (see `pseudo_ortho` example) if you don't need a true ortho projection.
        EditorCam::default().with_initial_anchor_depth(10.0),
    ));

    let n = 27;
    let half_width = (((n as f32).powf(1.0 / 3.0) - 1.0) / 2.0) as i32;
    let mesh = meshes.add(Cone::default());
    let material = materials.add(Color::WHITE);
    let width = -half_width..=half_width;
    for x in width.clone() {
        for y in width.clone() {
            for z in width.clone() {
                commands.spawn((
                    Mesh3d(mesh.clone()),
                    MeshMaterial3d(material.clone()),
                    Transform::from_translation(IVec3::new(x, y, z).as_vec3() * 2.0)
                        .with_scale(Vec3::splat(1.)),
                ));
            }
        }
    }
}

fn setup_ui(mut commands: Commands) {
    commands.spawn((
        Text::new(
            "Left Mouse - Pan\n\
            Right Mouse - Orbit\n\
            Scroll - Zoom\n",
        ),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        },
    ));
}
