//! A consistently-styled, cross-platform menu bar for Bevy applications.
//!
//! This runs along the top of the screen and provides a list of options to the user,
//! such as "File", "Edit", "View", etc.

use bevy::prelude::*;

use bevy_editor_palette::Theme;

/// The root node for the menu bar.
#[derive(Component)]
pub struct MenuBarNode;

/// The Bevy Menu Bar Plugin.
pub struct MenuBarPlugin;

impl Plugin for MenuBarPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, menu_setup.in_set(MenuBarSet));
    }
}

/// System Set to set up the menu bar.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct MenuBarSet;

/// The setup system for the menu bar.
fn menu_setup(mut commands: Commands, root: Query<Entity, With<MenuBarNode>>, theme: Res<Theme>) {
    commands
        .entity(root.single())
        .insert(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Px(30.0),
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                flex_basis: Val::Px(30.0),
                justify_items: JustifyItems::Start,
                align_items: AlignItems::Center,
                padding: UiRect {
                    left: Val::Px(5.0),
                    right: Val::Px(5.0),
                    top: Val::Px(0.0),
                    bottom: Val::Px(0.0),
                },
                ..Default::default()
            },
            background_color: BackgroundColor(theme.background_color),
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn(NodeBundle {
                style: Style {
                    width: Val::Px(30.0),
                    height: Val::Px(20.0),

                    ..Default::default()
                },
                background_color: BackgroundColor(Color::Oklaba(Oklaba {
                    lightness: 0.090,
                    a: 0.0,
                    b: 0.0,
                    alpha: 1.0,
                })),
                ..Default::default()
            });
        });
}
