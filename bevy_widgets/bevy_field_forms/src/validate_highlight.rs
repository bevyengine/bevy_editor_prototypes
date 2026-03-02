//! This module provides a simple border highlight for input fields

use crate::input_field::*;
use bevy::prelude::*;
use bevy_text_editing::HasFocus;

/// A plugin that adds an observer to highlight the border of a text field based on its validation state based on the `SimpleBorderHighlight` component.
pub struct SimpleBorderHighlightPlugin;

impl Plugin for SimpleBorderHighlightPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_focus_changed);
        app.add_observer(on_validation_changed);

        app.add_systems(PreUpdate, on_interaction_changed);
    }
}

/// A component that defines colors for highlighting the border of a text field based on its validation state.
#[derive(Component, Clone)]
#[require(Node, Interaction)]
pub struct SimpleBorderHighlight {
    /// The color of the border when the text field's content is valid.
    pub normal_color: Color,
    /// The color of the border when the text field is hovered.
    pub hovered_color: Color,
    /// The color of the border when the text field is in focus.
    pub focused_color: Color,
    /// The color of the border when the text field's content is invalid.
    pub invalid_color: Color,
    /// The last validation state of the text field.
    pub last_validation_state: ValidationState,
}

impl Default for SimpleBorderHighlight {
    fn default() -> Self {
        Self {
            normal_color: Color::srgb(0.5, 0.5, 0.5),
            hovered_color: Color::srgb(0.7, 0.7, 0.7),
            focused_color: Color::srgb(1.0, 1.0, 1.0),
            invalid_color: Color::srgb(1.0, 0.0, 0.0),
            last_validation_state: ValidationState::Unchecked,
        }
    }
}

fn on_validation_changed(
    trigger: On<ValidationChanged>,
    mut commands: Commands,
    mut q_highlights: Query<(&mut SimpleBorderHighlight, &Interaction, &HasFocus)>,
) {
    let entity = trigger.event().event_target();
    let Ok((mut highlight, interaction, has_focus)) = q_highlights.get_mut(entity) else {
        return;
    };

    match &trigger.state {
        ValidationState::Valid | ValidationState::Unchecked => {
            if has_focus.0 {
                commands
                    .entity(entity)
                    .insert(BorderColor::all(highlight.focused_color));
            } else if *interaction == Interaction::Hovered {
                commands
                    .entity(entity)
                    .insert(BorderColor::all(highlight.hovered_color));
            } else {
                commands
                    .entity(entity)
                    .insert(BorderColor::all(highlight.normal_color));
            }
        }
        ValidationState::Invalid(_) => {
            commands
                .entity(entity)
                .insert(BorderColor::all(highlight.invalid_color));
        }
    }

    highlight.last_validation_state = trigger.state.clone();
}

fn on_focus_changed(
    trigger: On<Insert, HasFocus>,
    q_highlights: Query<&SimpleBorderHighlight>,
    mut commands: Commands,
) {
    let entity = trigger.event().event_target();
    let Ok(highlight) = q_highlights.get(entity) else {
        return;
    };

    commands.trigger(ValidationChanged {
        entity,
        state: highlight.last_validation_state.clone(),
    });
}

fn on_interaction_changed(
    mut commands: Commands,
    q_changed_interaction: Query<(Entity, &SimpleBorderHighlight), Changed<Interaction>>,
) {
    for (entity, highlight) in q_changed_interaction.iter() {
        commands.trigger(ValidationChanged {
            entity,
            state: highlight.last_validation_state.clone(),
        });
    }
}
