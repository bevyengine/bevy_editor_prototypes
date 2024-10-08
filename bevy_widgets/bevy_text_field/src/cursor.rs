//! Cursor plugin for text field

use bevy::prelude::*;

pub(crate) struct CursorPlugin;

impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_cursor);
    }
}

#[derive(Component)]
pub(crate) struct Cursor {
    timer: Timer,
    visible: bool,
}

impl Default for Cursor {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.5, TimerMode::Repeating),
            visible: true,
        }
    }
}

pub(crate) fn update_cursor(time: Res<Time>, mut q_cursors: Query<(&mut Cursor, &mut Visibility)>) {
    for (mut cursor, mut visibility) in q_cursors.iter_mut() {
        if cursor.timer.tick(time.delta()).just_finished() {
            cursor.visible = !cursor.visible;
            if cursor.visible {
                *visibility = Visibility::Visible;
            } else {
                *visibility = Visibility::Hidden;
            }
        }
    }
}
