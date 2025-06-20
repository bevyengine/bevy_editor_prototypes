use bevy::prelude::*;

use bevy_component_viewer::ComponentViewerPlugin;
use bevy_editor_styles::Theme;
use bevy_footer_bar::{FooterBarNode, FooterBarPlugin, FooterBarSet};
use bevy_menu_bar::{MenuBarNode, MenuBarPlugin, MenuBarSet};
use bevy_pane_layout::{PaneLayoutPlugin, PaneLayoutSet, RootPaneLayoutNode};
use bevy_properties_pane::PropertiesPanePlugin;
use bevy_scene_tree::SceneTreePlugin;

/// The Bevy Editor UI Plugin.
pub struct EditorUIPlugin;

impl Plugin for EditorUIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, ui_setup.in_set(UISet))
            .configure_sets(
                Startup,
                (PaneLayoutSet, MenuBarSet, FooterBarSet).after(UISet),
            )
            .add_plugins((
                PaneLayoutPlugin,
                MenuBarPlugin,
                FooterBarPlugin,
                SceneTreePlugin,
                PropertiesPanePlugin,
                ComponentViewerPlugin,
            ));
    }
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct UISet;

/// The root node for the UI.
#[derive(Component)]
pub struct RootUINode;

fn ui_setup(mut commands: Commands, theme: Res<Theme>) {
    commands.spawn((
        Camera2d,
        Camera {
            order: 10,
            ..default()
        },
    ));

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),

                display: Display::Flex,
                flex_direction: FlexDirection::Column,

                ..Default::default()
            },
            theme.general.background_color,
            RootUINode,
        ))
        .with_children(|parent| {
            parent.spawn(MenuBarNode);
            parent.spawn(RootPaneLayoutNode);
            parent.spawn(FooterBarNode);
        });
}
