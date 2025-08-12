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

use std::f32::consts::TAU;
use std::time::Duration;

use bevy::app::App as BevyApp;
use bevy::asset::UnapprovedPathMode;
use bevy::color::palettes::tailwind;
use bevy::prelude::*;
use bevy::{
    core_widgets::CoreWidgetsPlugins,
    feathers::{FeathersPlugin, dark_theme::create_dark_theme, theme::UiTheme},
    input_focus::{InputDispatchPlugin, tab_navigation::TabNavigationPlugin},
};
// Re-export Bevy for project use
pub use bevy;

use bevy::winit::{UpdateMode, WinitSettings};
use bevy_context_menu::ContextMenuPlugin;
use bevy_editor_core::EditorCorePlugin;
use bevy_editor_core::selection::Selectable;
use bevy_editor_styles::StylesPlugin;
use bevy_toolbar::ActiveTool;
use bevy_transform_gizmos::{GizmoTransformable, TransformGizmoPlugin};

// Panes
use bevy_2d_viewport::Viewport2dPanePlugin;
use bevy_3d_viewport::Viewport3dPanePlugin;
use bevy_asset_browser::AssetBrowserPanePlugin;

use crate::load_gltf::LoadGltfPlugin;

mod load_gltf;
pub mod project;
mod ui;

/// The plugin that handle the bare minimum to run the application
pub struct RuntimePlugin;

impl Plugin for RuntimePlugin {
    fn build(&self, bevy_app: &mut BevyApp) {
        bevy_app.add_plugins(DefaultPlugins.set(AssetPlugin {
            unapproved_path_mode: UnapprovedPathMode::Deny,
            ..default()
        }));
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
                EditorCorePlugin,
                ContextMenuPlugin,
                StylesPlugin,
                Viewport2dPanePlugin,
                Viewport3dPanePlugin,
                ui::EditorUIPlugin,
                AssetBrowserPanePlugin,
                LoadGltfPlugin,
                MeshPickingPlugin,
                TransformGizmoPlugin,
                CoreWidgetsPlugins,
                InputDispatchPlugin,
                TabNavigationPlugin,
                FeathersPlugin,
            ))
            .insert_resource(UiTheme(create_dark_theme()))
            .insert_resource(WinitSettings {
                focused_mode: UpdateMode::reactive(Duration::from_secs_f64(1.0 / 60.0)),
                unfocused_mode: UpdateMode::reactive_low_power(Duration::from_secs(1)),
            })
            .init_resource::<ActiveTool>()
            .add_systems(Startup, dummy_setup);
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
        let args = std::env::args().collect::<Vec<String>>();
        let editor_mode = !args.iter().any(|arg| arg == "-game");

        let mut bevy_app = BevyApp::new();
        bevy_app.add_plugins(RuntimePlugin);
        if editor_mode {
            bevy_app.add_plugins(EditorPlugin);
        }

        bevy_app.run()
    }
}

/// This is temporary, until we can load maps from the asset browser
fn dummy_setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials_3d: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(2.5)))),
        MeshMaterial3d(materials_3d.add(Color::WHITE)),
        Name::new("Plane"),
        Selectable,
        GizmoTransformable,
    ));

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::from_size(Vec3::splat(1.)))),
        MeshMaterial3d(materials_3d.add(Color::from(tailwind::BLUE_500))),
        Transform::from_translation(vec3(1.1, 0.5, -1.3))
            .with_rotation(Quat::from_rotation_y(TAU * 0.05)),
        Name::new("Box"),
        Selectable,
        GizmoTransformable,
    ));

    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::default().looking_to(vec3(-1., -1., 1.), Vec3::Y),
        GizmoTransformable,
        Name::new("DirectionalLight"),
    ));
}
