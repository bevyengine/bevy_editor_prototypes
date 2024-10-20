use bevy::prelude::*;
use bevy_editor_cam::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            bevy_mod_picking::DefaultPickingPlugins,
            DefaultEditorCamPlugins,
        ))
        .add_systems(Startup, (setup, setup_ui))
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let diffuse_map = asset_server.load("environment_maps/diffuse_rgb9e5_zstd.ktx2");
    let specular_map = asset_server.load("environment_maps/specular_rgb9e5_zstd.ktx2");

    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(10.0, 10.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
            projection: Projection::Orthographic(OrthographicProjection {
                scale: 0.01,
                ..default()
            }),
            ..default()
        },
        EnvironmentMapLight {
            intensity: 1000.0,
            diffuse_map: diffuse_map.clone(),
            specular_map: specular_map.clone(),
        },
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
        // This is an extension made specifically for orthographic cameras. Because an ortho camera
        // projection has no field of view, a skybox can't be sensibly rendered, only a single point
        // on the skybox would be visible to the camera at any given time. While this is technically
        // correct to what the camera would see, it is not visually helpful nor appealing. It is
        // common for CAD software to render a skybox with a field of view that is decoupled from
        // the camera field of view.
        bevy_editor_cam::extensions::independent_skybox::IndependentSkybox::new(diffuse_map, 500.0),
    ));

    spawn_helmets(27, &asset_server, &mut commands);
}

fn spawn_helmets(n: usize, asset_server: &AssetServer, commands: &mut Commands) {
    let half_width = (((n as f32).powf(1.0 / 3.0) - 1.0) / 2.0) as i32;
    let scene = asset_server.load("models/PlaneEngine/scene.gltf#Scene0");
    let width = -half_width..=half_width;
    for x in width.clone() {
        for y in width.clone() {
            for z in width.clone() {
                commands.spawn((SceneBundle {
                    scene: scene.clone(),
                    transform: Transform::from_translation(IVec3::new(x, y, z).as_vec3() * 2.0)
                        .with_scale(Vec3::splat(1.)),
                    ..default()
                },));
            }
        }
    }
}

fn setup_ui(mut commands: Commands) {
    let style = TextStyle {
        font_size: 20.0,
        ..default()
    };
    commands.spawn(
        TextBundle::from_sections(vec![
            TextSection::new("Left Mouse - Pan\n", style.clone()),
            TextSection::new("Right Mouse - Orbit\n", style.clone()),
            TextSection::new("Scroll - Zoom\n", style.clone()),
        ])
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        }),
    );
}
