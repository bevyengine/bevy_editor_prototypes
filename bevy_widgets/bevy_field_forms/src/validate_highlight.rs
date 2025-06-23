//! This module provides a simple border highlight for input fields

use crate::input_field::*;
use bevy::{
    input_focus::{InputFocus, IsFocused, IsFocusedHelper},
    prelude::*,
};

/// A plugin that adds an observer to highlight the border of a text field based on its validation state based on the `SimpleBorderHighlight` component.
pub struct SimpleBorderHighlightPlugin;

impl Plugin for SimpleBorderHighlightPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_validation_changed);

        app.add_systems(PreUpdate, (on_interaction_changed, on_focus_changed));
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
    mut q_highlights: Query<(&mut SimpleBorderHighlight, &Interaction)>,
    is_focused_helper: IsFocusedHelper,
) {
    let entity = trigger.target();
    let Ok((mut highlight, interaction)) = q_highlights.get_mut(entity) else {
        return;
    };

    match &trigger.0 {
        ValidationState::Valid | ValidationState::Unchecked => {
            if is_focused_helper.is_focus_within(entity) {
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

    highlight.last_validation_state = trigger.0.clone();
}

fn on_focus_changed(
    input_focus: Res<InputFocus>,
    is_focused_helper: IsFocusedHelper,
    q_highlights: Query<(Entity, &SimpleBorderHighlight)>,
    mut commands: Commands,
    mut previous_focus: Local<Option<Entity>>,
) {
    if input_focus.0 == *previous_focus {
        return;
    }

    if let Some(previous_entity) = previous_focus.take() {
        // If the previous focus is set, we need to trigger the validation change for it
        if let Ok((_, highlight)) = q_highlights.get(previous_entity) {
            info!("Focus lost from {:?}", previous_entity);
            commands.trigger_targets(
                ValidationChanged(highlight.last_validation_state.clone()),
                previous_entity,
            );
        }
    }

    for (entity, highlight) in q_highlights.iter() {
        if is_focused_helper.is_focus_within(entity) {
            info!("Focus added to {:?}", entity);

            commands.trigger_targets(
                ValidationChanged(highlight.last_validation_state.clone()),
                entity,
            );
        }
    }

    *previous_focus = input_focus.0;
}

fn on_interaction_changed(
    mut commands: Commands,
    q_changed_interaction: Query<(Entity, &SimpleBorderHighlight), Changed<Interaction>>,
) {
    for (entity, highlight) in q_changed_interaction.iter() {
        commands.trigger_targets(
            ValidationChanged(highlight.last_validation_state.clone()),
            entity,
        );
    }
}
