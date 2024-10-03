//! A consistently-styled, cross-platform menu bar for Bevy applications.
//!
//! This runs along the top of the screen and provides a list of options to the user,
//! such as "File", "Edit", "View", etc.

use bevy::prelude::*;

/// The Bevy Menu Bar Plugin.
pub struct MenuBarPlugin;

impl Plugin for MenuBarPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(NodeBundle {
        style: Style {
            display: Display::Flex,
            width: Val::Percent(100.0),
            height: Val::Px(30.0),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            ..Default::default()
        },
        ..Default::default()
    });
}
