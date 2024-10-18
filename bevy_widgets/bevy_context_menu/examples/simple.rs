//! Simple context menu example.

use bevy::prelude::*;
use bevy_context_menu::{ContextMenu, ContextMenuOption, ContextMenuPlugin};
use bevy_editor_styles::StylesPlugin;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, StylesPlugin, ContextMenuPlugin))
        .add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Camera2d);

            commands.spawn((
                Node::default(),
                Style {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    ..default()
                },
                ContextMenu::new(vec![
                    ContextMenuOption::new("Turn Red", |mut commands, entity| {
                        commands
                            .entity(entity)
                            .insert(BackgroundColor(Color::srgb(0.2, 0., 0.)));
                    }),
                    ContextMenuOption::new("Turn Blue", |mut commands, entity| {
                        commands
                            .entity(entity)
                            .insert(BackgroundColor(Color::srgb(0., 0., 0.2)));
                    }),
                ]),
            ));
        })
        .run();
}
