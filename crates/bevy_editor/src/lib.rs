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
//!     which transforms the user's application into an editor that runs their game.
//! - Finally, it will be a standalone application that communicates with a running Bevy game via the Bevy Remote Protocol.

use bevy::app::App as BevyApp;
use bevy::prelude::*;
// Re-export Bevy for project use
pub use bevy;

use bevy::window::WindowResolution;
use bevy_context_menu::ContextMenuPlugin;
use bevy_editor_styles::StylesPlugin;

// Panes
use bevy_2d_viewport::Viewport2dPanePlugin;
use bevy_3d_viewport::Viewport3dPanePlugin;
use bevy_asset_browser::AssetBrowserPanePlugin;

use crate::load_gltf::LoadGltfPlugin;

mod load_gltf;
pub mod project;
mod ui;

/// The main application
pub struct App {
    window: Window,
}

impl Default for App {
    fn default() -> Self {
        App {
            window: Window {
                title: "Bevy Editor".to_string(),
                ..default()
            },
        }
    }
}

impl App {
    /// create new instance of [`App`]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set default window resolution/size
    #[inline]
    pub fn window_resolution(&mut self, resolution: WindowResolution) -> &mut Self {
        self.window.resolution = resolution;
        self
    }

    /// Set the window title
    #[inline]
    pub fn window_name(&mut self, name: &str) -> &mut Self {
        self.window.title = name.to_string();
        self
    }

    /// Run the application
    pub fn run(&self) -> AppExit {
        let args = std::env::args().collect::<Vec<String>>();
        let editor_mode = !args.iter().any(|arg| arg == "-game");

        let mut bevy_app = BevyApp::new();
        bevy_app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(self.window.clone()),
            ..default()
        }));
        if editor_mode {
            bevy_app
                .add_plugins((
                    ContextMenuPlugin,
                    StylesPlugin,
                    Viewport2dPanePlugin,
                    Viewport3dPanePlugin,
                    ui::EditorUIPlugin,
                    AssetBrowserPanePlugin,
                    LoadGltfPlugin,
                ))
                .add_systems(Startup, setup);
        }
        bevy_app.run()
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials_2d: ResMut<Assets<ColorMaterial>>,
    mut materials_3d: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Mesh2d(meshes.add(Circle::new(50.0))),
        MeshMaterial2d(materials_2d.add(Color::WHITE)),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(1.5)))),
        MeshMaterial3d(materials_3d.add(Color::WHITE)),
    ));

    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::default().looking_to(Vec3::NEG_ONE, Vec3::Y),
    ));
}
