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

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, camera_setup)
        .run();
}

fn camera_setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}
