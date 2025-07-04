//! A Bevy app that you can connect to with the BRP and edit.

use bevy::prelude::*;
use bevy_entity_inspector::EntityInspectorPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EntityInspectorPlugin)
        .run();
}
