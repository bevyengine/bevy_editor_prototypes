//! Bevy Entity Inspector Integration Tests
//! This file contains integration tests for the Bevy Entity Inspector plugin.
//! These tests ensure that the plugin can be added to a Bevy app with and without the remote feature enabled.
//!
//! To run these tests, use the following command:
//! ```bash
//! cargo test --features "remote"
//! ```

use bevy::prelude::*;
use bevy_entity_inspector::InspectorPlugin;

#[test]
fn test_inspector_plugin_without_remote() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InspectorPlugin));
    // Test passes if we can add the plugin without the remote feature
}

#[cfg(feature = "remote")]
#[test]
fn test_inspector_plugin_with_remote() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InspectorPlugin));
    // Test passes if we can add the plugin with the remote feature
    // The remote plugin is automatically added when the feature is enabled
}
