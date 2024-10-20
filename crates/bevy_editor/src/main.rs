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

use bevy::prelude::*;

use bevy_context_menu::ContextMenuPlugin;
use bevy_editor_styles::StylesPlugin;

// Panes
use bevy_2d_viewport::Viewport2dPanePlugin;
use bevy_3d_viewport::Viewport3dPanePlugin;
use bevy_asset_browser::AssetBrowserPanePlugin;

mod ui;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Bevy Editor".to_string(),
                    ..default()
                }),
                ..default()
            }),
            ContextMenuPlugin,
            StylesPlugin,
            Viewport2dPanePlugin,
            Viewport3dPanePlugin,
            ui::EditorUIPlugin,
            AssetBrowserPanePlugin,
        ))
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        Mesh2d(meshes.add(Circle::new(50.0))),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(Color::WHITE))),
    ));
}
