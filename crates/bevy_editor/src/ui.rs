use bevy::prelude::*;

use bevy_3d_viewport::Bevy3dViewportPane;
use bevy_asset_browser::AssetBrowserPane;
use bevy_editor_styles::Theme;
use bevy_footer_bar::{FooterBarNode, FooterBarPlugin, FooterBarSet};
use bevy_menu_bar::{MenuBarNode, MenuBarPlugin, MenuBarSet};
use bevy_pane_layout::{prelude::*, PaneLayoutPlugin, PaneLayoutSet, RootPaneLayoutNode};
use bevy_properties_pane::PropertiesPane;
use bevy_scene_tree::SceneTreePane;

/// The Bevy Editor UI Plugin.
pub struct EditorUIPlugin;

impl Plugin for EditorUIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            (ui_setup.in_set(UISet), initial_layout.in_set(PaneLayoutSet)),
        )
        .configure_sets(
            Startup,
            (PaneLayoutSet, MenuBarSet, FooterBarSet).after(UISet),
        )
        .add_plugins((PaneLayoutPlugin, MenuBarPlugin, FooterBarPlugin));
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
            Name::new("UI Root"),
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
            parent.spawn((Name::new("Menu Bar"), MenuBarNode));
            parent.spawn((Name::new("Layout Root"), RootPaneLayoutNode));
            parent.spawn((Name::new("Footer Bar"), FooterBarNode));
        });
}

/// This is temporary, until we can load maps from the asset browser
fn initial_layout(
    mut commands: Commands,
    theme: Res<Theme>,
    panes_root: Single<Entity, With<RootPaneLayoutNode>>,
) {
    println!("Creating initial layout.");

    let mut root_divider =
        spawn_root_divider(&mut commands, Divider::Horizontal, Some(*panes_root), 1.);

    let mut sidebar_divider = root_divider.add_divider(0.2);
    sidebar_divider
        .add_pane_group(&theme, 0.4)
        .add_pane(&theme, SceneTreePane);
    sidebar_divider
        .add_pane_group(&theme, 0.6)
        .add_pane(&theme, PropertiesPane)
        .add_pane(&theme, AssetBrowserPane);

    let mut asset_browser_divider = root_divider.add_divider(0.8);
    asset_browser_divider
        .add_pane_group(&theme, 0.7)
        .add_pane(&theme, Bevy3dViewportPane::default());
    asset_browser_divider
        .add_pane_group(&theme, 0.3)
        .add_pane(&theme, AssetBrowserPane);
}
