//! This module provides functionality to filter and restrict input to allowed characters in text fields.
//!
//! It includes:
//! - `AllowedCharsFilterPlugin`: A plugin to add character filtering functionality to the app.
//! - `AllowedCharsFilter`: A component to define allowed characters for text input.
//! - Systems to handle text changes and filtering based on allowed characters.

use bevy::{prelude::*, ui::experimental::GhostNode, utils::HashSet};
use bevy_text_editing::{
    child_traversal::FirstChildTraversal, CharPosition, SetCursorPosition, SetText, TextChanged,
};

/// Plugin for the allowed chars filter.
pub struct AllowedCharsFilterPlugin;

impl Plugin for AllowedCharsFilterPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_text_change);
        app.add_observer(on_text_set);
    }
}

/// Create a filter on top of an editable text widget to prevent any unallowed characters from being added into controlled text.
/// It will only allow characters that are in the `allowed_chars` set.
/// Must be used with something on top of it that will handle the text change.
#[derive(Component)]
#[require(Node, FirstChildTraversal)]
pub struct AllowedCharsFilter {
    /// The characters that are allowed to be in the text.
    pub allowed_chars: HashSet<char>,
    /// Whether to filter the `SetText` event.
    pub filter_set_text_event: bool,
}

impl AllowedCharsFilter {
    /// Create a new allowed chars filter.
    pub fn new(allowed_chars: HashSet<char>) -> Self {
        Self {
            allowed_chars,
            filter_set_text_event: false,
        }
    }
}

fn on_text_change(
    mut trigger: Trigger<TextChanged>,
    mut commands: Commands,
    q_filters: Query<&AllowedCharsFilter>,
) {
    let entity = trigger.entity();
    let Ok(filter) = q_filters.get(entity) else {
        return;
    };

    {
        let change_text = trigger.event_mut();
        change_text.new_text = change_text
            .new_text
            .chars()
            .filter(|c| filter.allowed_chars.contains(c))
            .collect();
        change_text.change.new_text = change_text
            .change
            .new_text
            .chars()
            .filter(|c| filter.allowed_chars.contains(c))
            .collect();

        info!("Text was filtered to {:?}", change_text.new_text);
    }

    if trigger.change.is_nop() {
        info!("Text change is nop");
        trigger.propagate(false);
        if let Some(old_cursor_position) = trigger.old_cursor_position {
            commands.trigger_targets(SetCursorPosition(old_cursor_position), entity);
        }
    } else {
        trigger.propagate(true);
    }
}

fn on_text_set(mut trigger: Trigger<SetText>, q_filters: Query<&AllowedCharsFilter>) {
    let entity = trigger.entity();
    let Ok(filter) = q_filters.get(entity) else {
        return;
    };

    if filter.filter_set_text_event {
        let set_text = trigger.event_mut();
        set_text.0 = set_text
            .0
            .chars()
            .filter(|c| filter.allowed_chars.contains(c))
            .collect();
    }

    trigger.propagate(true);
}
