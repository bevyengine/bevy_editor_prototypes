//! A Bevy app that demonstrates the Entity Inspector plugin.
//!
//! This example shows how to use the [`InspectorPlugin`] with modular data sources.
//! The inspector can be configured to use different data sources:
//! - Remote mode: Connect to a remote Bevy application via [`bevy_remote`](https://docs.rs/bevy_remote/latest/bevy_remote/)
//! - Scene files: Load and inspect scene data
//! - BSN: Load and inspect BSN (Bevy Scene Notation) files
//!
//! To run this example:
//! - With remote data source: `cargo run --example inspector --features remote`
//! - Basic inspector (empty until data source is configured): `cargo run --example inspector`
//!
//! For remote mode, first start the `cube_server` example:
//! `cargo run --example cube_server --features remote`
//!
//! # Related Documentation
//!
//! - [`bevy_entity_inspector::InspectorPlugin`] - Main plugin for the inspector
//! - [`bevy_entity_inspector::create_dark_inspector_theme`] - Dark theme configuration

use bevy::prelude::*;
use bevy_entity_inspector::{create_dark_inspector_theme, InspectorPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(InspectorPlugin)
        .insert_resource(create_dark_inspector_theme())
        .run();
}
