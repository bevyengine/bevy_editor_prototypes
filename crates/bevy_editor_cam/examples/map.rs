//! Editor camera example with a city map to traverse.

use bevy::{
    anti_alias::taa::TemporalAntiAliasing, color::palettes, pbr::ScreenSpaceAmbientOcclusion,
    post_process::bloom::Bloom,
};
use bevy::{prelude::*, render::camera::TemporalJitter};
use bevy_editor_cam::{extensions::dolly_zoom::DollyZoomTrigger, prelude::*};
use rand::Rng;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, DefaultEditorCamPlugins))
        .add_systems(Startup, (setup, setup_ui))
        .add_systems(
            Update,
            (toggle_projection, projection_specific_render_config).chain(),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    spawn_buildings(&mut commands, &mut meshes, &mut materials, 20.0);

    commands
        .spawn((
            Camera3d::default(),
            Transform::from_xyz(2.0, 2.0, 2.0).looking_at(Vec3::ZERO, Vec3::Y),
            Bloom::default(),
            Msaa::Off,
            EditorCam {
                last_anchor_depth: 2.0,
                ..Default::default()
            },
        ))
        .insert(ScreenSpaceAmbientOcclusion::default())
        .insert(TemporalAntiAliasing::default());
}

fn spawn_buildings(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    half_width: f32,
) {
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(half_width * 20.0)))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::Srgba(palettes::css::DARK_GRAY),
            ..Default::default()
        })),
        Transform::from_xyz(0.0, -5.0, 0.0),
    ));

    let mut rng = rand::thread_rng();
    let mesh = meshes.add(Cuboid::default());
    let material = [
        materials.add(Color::Srgba(palettes::css::GRAY)),
        materials.add(Color::srgb(0.3, 0.6, 0.8)),
        materials.add(Color::srgb(0.55, 0.4, 0.8)),
        materials.add(Color::srgb(0.8, 0.45, 0.5)),
    ];

    let w = half_width as isize;
    for x in -w..=w {
        for z in -w..=w {
            let x = x as f32 + rng.r#gen::<f32>() - 0.5;
            let z = z as f32 + rng.r#gen::<f32>() - 0.5;
            let y =
                rng.r#gen::<f32>() * rng.r#gen::<f32>() * rng.r#gen::<f32>() * rng.r#gen::<f32>();
            let y_scale = 1.02f32.powf(100.0 * y);

            commands.spawn((
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material[rng.gen_range(0..material.len())].clone()),
                Transform::from_xyz(x, y_scale / 2.0 - 5.0, z).with_scale(Vec3::new(
                    (rng.r#gen::<f32>() + 0.5) * 0.3,
                    y_scale,
                    (rng.r#gen::<f32>() + 0.5) * 0.3,
                )),
            ));
        }
    }
}

fn toggle_projection(
    keys: Res<ButtonInput<KeyCode>>,
    mut dolly: MessageWriter<DollyZoomTrigger>,
    cam: Query<Entity, With<EditorCam>>,
    mut toggled: Local<bool>,
) {
    if keys.just_pressed(KeyCode::KeyP) {
        *toggled = !*toggled;
        let target_projection = if *toggled {
            Projection::Orthographic(OrthographicProjection::default_3d())
        } else {
            Projection::Perspective(PerspectiveProjection::default())
        };
        dolly.write(DollyZoomTrigger {
            target_projection,
            camera: cam.single().unwrap(),
        });
    }
}

fn projection_specific_render_config(
    mut commands: Commands,
    mut cam: Query<(Entity, &Projection, &mut Msaa), With<EditorCam>>,
) {
    let (entity, proj, mut msaa) = cam.single_mut().unwrap();
    match proj {
        Projection::Perspective(_) => {
            *msaa = Msaa::Off;
            commands
                .entity(entity)
                .insert(TemporalAntiAliasing::default())
                .insert(ScreenSpaceAmbientOcclusion::default());
        }
        Projection::Orthographic(_) => {
            *msaa = Msaa::Sample4;
            commands
                .entity(entity)
                .remove::<TemporalJitter>()
                .remove::<ScreenSpaceAmbientOcclusion>();
        }
        _ => {}
    }
}

fn setup_ui(mut commands: Commands) {
    commands.spawn((
        Text::new(
            "Left Mouse - Pan\n\
            Right Mouse - Orbit\n\
            Scroll - Zoom\n\
            P - Toggle projection\n",
        ),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        },
    ));
}
