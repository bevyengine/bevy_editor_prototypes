//! This module provides a system to mirror the `TextChanged` event into a `SetText` event.
//! This is useful to create controlled text widgets with filters on top of this text widget.

use bevy::prelude::*;
use bevy_text_editing::{child_traversal::FirstChildTraversal, SetText, TextChanged};

/// Plugin for the text event mirror.
pub struct TextEventMirrorPlugin;

impl Plugin for TextEventMirrorPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_text_changed);
    }
}

/// Component to be added to the entity that should mirror the text event.
#[derive(Component)]
#[require(Node, FirstChildTraversal)]
pub struct TextEventMirror;

/// Mirror propagating TextChanged event into SetText down to the text field.
/// Allow to easy create controlled text widgets with filters on top of this text widget.
fn on_text_changed(
    mut trigger: Trigger<TextChanged>,
    mut commands: Commands,
    q_mirrors: Query<Entity, With<TextEventMirror>>,
) {
    let entity = trigger.entity();
    let Ok(_) = q_mirrors.get(entity) else {
        return;
    };

    trigger.propagate(false);

    info!("Text mirrored with value {:?}", trigger.new_text);

    commands.trigger_targets(SetText(trigger.new_text.clone()), entity);
}
