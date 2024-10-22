use crate::validated_input_field::*;
use bevy::prelude::*;
use bevy_focus::{Focus, LostFocus};

/// A plugin that adds an observer to highlight the border of a text field based on its validation state based on the `SimpleBorderHighlight` component.
pub struct SimpleBorderHighlightPlugin;

impl Plugin for SimpleBorderHighlightPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_validation_changed);
        app.add_observer(on_focus_added);
        app.add_observer(on_focus_lost);

        app.add_systems(PreUpdate, on_interaction_changed);
    }
}

/// A component that defines colors for highlighting the border of a text field based on its validation state.
#[derive(Component)]
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
    trigger: Trigger<ValidationChanged>,
    mut commands: Commands,
    mut q_highlights: Query<(&mut SimpleBorderHighlight, &Interaction, Option<&Focus>)>,
) {
    let entity = trigger.entity();
    let Ok((mut highlight, interaction, focus)) = q_highlights.get_mut(entity) else {
        return;
    };

    info!("Validation changed to {:?}", trigger.0);
    info!("Focus: {:?}", focus);

    match &trigger.0 {
        ValidationState::Valid => {
            if focus.is_some() {
                commands
                    .entity(entity)
                    .insert(BorderColor(highlight.focused_color));
            } else if *interaction == Interaction::Hovered {
                commands
                    .entity(entity)
                    .insert(BorderColor(highlight.hovered_color));
            } else {
                commands
                    .entity(entity)
                    .insert(BorderColor(highlight.normal_color));
            }
        }
        ValidationState::Invalid(_) => {
            commands
                .entity(entity)
                .insert(BorderColor(highlight.invalid_color));
        }
        ValidationState::Unchecked => {
            if focus.is_some() {
                commands
                    .entity(entity)
                    .insert(BorderColor(highlight.focused_color));
            } else if *interaction == Interaction::Hovered {
                commands
                    .entity(entity)
                    .insert(BorderColor(highlight.hovered_color));
            } else {
                commands
                    .entity(entity)
                    .insert(BorderColor(highlight.normal_color));
            }
        }
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

    info!("Focus added to {:?}", entity);

    commands.trigger_targets(
        ValidationChanged(highlight.last_validation_state.clone()),
        entity,
    );
}

fn on_focus_lost(
    trigger: Trigger<LostFocus>,
    mut commands: Commands,
    q_highlights: Query<&SimpleBorderHighlight>,
) {
    let entity = trigger.entity();
    let Ok(highlight) = q_highlights.get(entity) else {
        return;
    };

    info!("Focus lost from {:?}", entity);

    commands.trigger_targets(
        ValidationChanged(highlight.last_validation_state.clone()),
        entity,
    );
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
