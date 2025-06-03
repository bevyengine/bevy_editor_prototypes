//! A consistently-styled, cross-platform Footer bar for Bevy applications.
//!
//! This runs along the top of the screen and provides a list of options to the user,
//! such as "File", "Edit", "View", etc.

use bevy::prelude::*;

use bevy_editor_styles::Theme;

/// The root node for the Footer bar.
#[derive(Component)]
pub struct FooterBarNode;

/// The Bevy Footer Bar Plugin.
pub struct FooterBarPlugin;

impl Plugin for FooterBarPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, footer_setup.in_set(FooterBarSet));
    }
}

/// System Set to set up the Footer bar.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct FooterBarSet;

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The setup system for the Footer bar.
fn footer_setup(
    mut commands: Commands,
    root: Query<Entity, With<FooterBarNode>>,
    theme: Res<Theme>,
) {
    commands
        .entity(root.single().unwrap())
        .insert((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(20.0),
                align_items: AlignItems::Center,
                padding: UiRect::axes(Val::Px(5.0), Val::Px(0.0)),
                ..Default::default()
            },
            theme.general.background_color,
        ))
        .with_children(|parent| {
            parent.spawn(Node {
                width: Val::Percent(50.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::Center,
                ..Default::default()
            });
            parent
                .spawn(Node {
                    width: Val::Percent(50.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::FlexEnd,
                    align_items: AlignItems::Center,

                    ..Default::default()
                })
                .with_child((
                    Text::new(VERSION),
                    TextFont {
                        font: theme.text.font.clone(),
                        font_size: 10.,
                        ..default()
                    },
                    TextColor(theme.text.low_priority),
                ));
        });
}
