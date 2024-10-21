use crate::validated_input_field::*;
use bevy::prelude::*;
use bevy_focus::Focus;

/// A plugin that adds an observer to highlight the border of a text field based on its validation state based on the `SimpleBorderHighlight` component.
pub struct SimpleBorderHighlightPlugin;

impl Plugin for SimpleBorderHighlightPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_validation_changed);
        app.add_observer(on_focus_added);
        app.add_observer(on_focus_lost);
    }
}

/// A component that defines colors for highlighting the border of a text field based on its validation state.
#[derive(Component, Default)]
pub struct SimpleBorderHighlight {
    /// The color of the border when the text field's content is valid.
    pub normal_color: Color,
    /// The color of the border when the text field is in focus.
    pub focused_color: Color,
    /// The color of the border when the text field's content is invalid.
    pub invalid_color: Color,
    /// The last validation state of the text field.
    pub last_validation_state: ValidationState,
}


fn on_validation_changed(
    trigger: Trigger<ValidationChanged>,
    mut commands: Commands,
    mut q_highlights: Query<(&mut SimpleBorderHighlight, Option<&Focus>)>,
) {
    let entity = trigger.entity();
    let Ok((mut highlight, focus)) = q_highlights.get_mut(entity) else {
        return;
    };

    match &trigger.0 {
        ValidationState::Valid => {
            if focus.is_some() {
                commands.entity(entity).insert(BorderColor(highlight.focused_color));
            } else {
                commands.entity(entity).insert(BorderColor(highlight.normal_color));
            }
        }
        ValidationState::Invalid(_) => {
            commands.entity(entity).insert(BorderColor(highlight.invalid_color));
        }
        ValidationState::Unchecked => {
            // Do nothing
        },
    }

    highlight.last_validation_state = trigger.0.clone();
}

fn on_focus_added(
    trigger: Trigger<OnInsert, Focus>,
    mut commands: Commands,
    q_highlights: Query<&SimpleBorderHighlight>,
) {
    let entity = trigger.entity();
    let Ok(highlight) = q_highlights.get(entity) else {
        return;
    };

    commands.trigger_targets(ValidationChanged(highlight.last_validation_state.clone()), entity);
}

fn on_focus_lost(
    trigger: Trigger<OnRemove, Focus>,
    mut commands: Commands,
    q_highlights: Query<&SimpleBorderHighlight>,
) {
    let entity = trigger.entity();
    let Ok(highlight) = q_highlights.get(entity) else {
        return;
    };

    commands.trigger_targets(ValidationChanged(highlight.last_validation_state.clone()), entity);
}