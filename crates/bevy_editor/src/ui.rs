use bevy::prelude::*;

use bevy_editor_styles::Theme;
use bevy_menu_bar::{MenuBarNode, MenuBarPlugin, MenuBarSet};
use bevy_pane_layout::{PaneLayoutPlugin, PaneLayoutSet, RootPaneLayoutNode};

/// The Bevy Editor UI Plugin.
pub struct EditorUIPlugin;

impl Plugin for EditorUIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, ui_setup.in_set(UISet))
            .configure_sets(Startup, (PaneLayoutSet, MenuBarSet).after(UISet))
            .add_plugins((PaneLayoutPlugin, MenuBarPlugin));
    }
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct UISet;

/// The root node for the UI.
#[derive(Component)]
pub struct RootUINode;

fn ui_setup(mut commands: Commands, theme: Res<Theme>) {
    commands.spawn(Camera2dBundle::default());

    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),

                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                flex_basis: Val::Percent(100.0),

                ..Default::default()
            },
            background_color: theme.background_color,
            ..Default::default()
        })
        .insert(RootUINode)
        .with_children(|parent| {
            parent.spawn(MenuBarNode);
            parent.spawn(RootPaneLayoutNode);
        });
}
