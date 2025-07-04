//! The main Bevy Editor application.
//!
//! This crate contains a standalone application that can be used to edit Bevy scenes and debug Bevy games.
//! Virtually all of the underlying logic and functionality of the editor should be backed by the assorted crates in the `bevy_editor` workspace;
//! this crate is simply responsible for orchestrating those crates and providing a user interface for them.
//!
//! The exact nature of this crate will be in flux for a while:
//!
//! - Initially, this will be a standard Bevy application that simply edits scenes with `DefaultPlugins`.
//! - Then, it will be a statically linked plugin that can be added to any Bevy game at compile time,
//!   which transforms the user's application into an editor that runs their game.
//! - Finally, it will be a standalone application that communicates with a running Bevy game via the Bevy Remote Protocol.

use std::env;

use bevy::app::App as BevyApp;
use bevy::color::palettes::tailwind;
use bevy::math::ops::cos;
use bevy::prelude::*;
// Re-export Bevy for project use
pub use bevy;

use bevy_context_menu::ContextMenuPlugin;
use bevy_editor_core::EditorCorePlugin;
use bevy_editor_styles::StylesPlugin;

// Panes
use bevy_2d_viewport::Viewport2dPanePlugin;
use bevy_3d_viewport::Viewport3dPanePlugin;
use bevy_asset_browser::AssetBrowserPanePlugin;
use bevy_remote::http::RemoteHttpPlugin;
use bevy_remote::RemotePlugin;

use crate::load_gltf::LoadGltfPlugin;

mod load_gltf;
pub mod project;
pub mod ui;

/// The plugin that handle the bare minimum to run the application
pub struct RuntimePlugin;

impl Plugin for RuntimePlugin {
    fn build(&self, bevy_app: &mut BevyApp) {
        bevy_app.add_plugins(DefaultPlugins);
    }
}

/// The plugin that attach your editor to the application
pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, bevy_app: &mut BevyApp) {
        // Update/register this project to the editor project list
        project::update_project_info();
        info!("Loading Bevy Editor");

        bevy_app
            .add_plugins((
                RemotePlugin::default(),
                RemoteHttpPlugin::default(),
                EditorCorePlugin,
                ContextMenuPlugin,
                StylesPlugin,
                Viewport2dPanePlugin,
                Viewport3dPanePlugin,
                ui::EditorUIPlugin,
                LoadGltfPlugin,
                AssetBrowserPanePlugin,
            ))
            .add_systems(Startup, setup)
            .add_systems(Update, move_cube)
            .register_type::<Cube>()
            .register_type::<MyObject>()
            .register_type::<MoveSpeed>();
    }
}

/// Your game application
/// This appllication allow your game to run, and the editor to be attached to it
#[derive(Default)]
pub struct App;

impl App {
    /// create new instance of [`App`]
    pub fn new() -> Self {
        Self
    }

    /// Run the application
    pub fn run(&self) -> AppExit {
        let args = env::args().collect::<Vec<String>>();
        let editor_mode = !args.iter().any(|arg| arg == "-game");

        let mut bevy_app = BevyApp::new();
        bevy_app.add_plugins(RuntimePlugin);
        if editor_mode {
            bevy_app.add_plugins(EditorPlugin);
        }

        info!(
            "Running bevy editor in {} mode",
            if editor_mode { "editor" } else { "game" }
        );
        bevy_app.run()
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
struct Cube(f32);

#[derive(Component, Reflect)]
#[reflect(Component)]
struct MyObject {
    vec3: Vec3,
    color: Color,
}

#[derive(Resource, Reflect)]
#[reflect(Resource)]
struct MoveSpeed {
    value: f32,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // unnamed object
    commands.spawn((
        Name::new("MyObject1"),
        MyObject {
            vec3: Vec3::new(1.0, 2.0, 3.0),
            color: Color::from(tailwind::BLUE_500),
        },
    ));

    // cube
    let cube_handle = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    commands.spawn((
        Name::new("Cube"),
        Mesh3d(cube_handle.clone()),
        MeshMaterial3d(materials.add(Color::from(tailwind::RED_200))),
        Transform::from_xyz(0.0, 0.5, 0.0),
        Cube(1.0),
        children![(
            Name::new("Sub-cube"),
            Mesh3d(cube_handle.clone()),
            MeshMaterial3d(materials.add(Color::from(tailwind::GREEN_500))),
            Transform::from_xyz(0.0, 1.5, 0.0),
            children![(
                Name::new("Sub-sub-cube"),
                Mesh3d(cube_handle),
                MeshMaterial3d(materials.add(Color::from(tailwind::BLUE_800))),
                Transform::from_xyz(1.5, 0.0, 0.0),
            )]
        )],
    ));

    // circular base
    commands.spawn((
        Name::new("Circular base"),
        Mesh3d(meshes.add(Circle::new(4.0))),
        MeshMaterial3d(materials.add(Color::from(tailwind::GREEN_300))),
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    ));

    // light
    commands.spawn((
        Name::new("Light"),
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));

    // camera
    commands.spawn((
        Name::new("Camera"),
        Camera3d::default(),
        Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::Y, Vec3::Y),
    ));
}

fn move_cube(
    mut query: Query<&mut Transform, With<Cube>>,
    time: Res<Time>,
    move_speed_res: Option<Res<MoveSpeed>>,
) {
    let move_speed = move_speed_res.map(|res| res.value).unwrap_or(1.0);
    for mut transform in &mut query {
        transform.translation.y = -cos(time.elapsed_secs() * move_speed) + 1.5;
    }
}
