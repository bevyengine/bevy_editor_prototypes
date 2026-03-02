//! This module provides a validated input field with a generic value type.

use bevy::prelude::*;
use bevy_text_editing::*;

/// Plugin for validated input fields with a generic value type
pub struct InputFieldPlugin<T: Validable> {
    _marker: std::marker::PhantomData<T>,
}

impl<T: Validable> Default for InputFieldPlugin<T> {
    fn default() -> Self {
        Self {
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T: Validable> Plugin for InputFieldPlugin<T> {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<EditableTextLinePlugin>() {
            app.add_plugins(EditableTextLinePlugin);
        }

        app.add_systems(PostUpdate, on_value_changed::<T>);
        app.add_systems(PreUpdate, on_created::<T>);

        app.add_observer(on_text_changed::<T>);
    }
}

/// A text field with input validation
/// It will not contain special style updates for validation state, because it's expected that it will be
/// combined with other widgets to form a custom UI.
#[derive(Component, Clone)]
#[require(EditableTextLine::controlled(""))]
pub struct InputField<T: Validable> {
    /// The last valid value
    pub value: T,
    /// The current validation state
    pub validation_state: ValidationState,
    /// If true, this text field will not update its value automatically
    /// and will require an external update call to update the value.
    pub controlled: bool,
    /// Old value
    pub old_value: T,
}

impl<T: Validable> Default for InputField<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T: Validable> InputField<T> {
    /// Create a new validated input field with the given value
    pub fn new(value: T) -> Self {
        Self {
            value: value.clone(),
            validation_state: ValidationState::Unchecked,
            controlled: false,
            old_value: value,
        }
    }
}

/// A trait for types that can be validated from a string input.
///
/// Types implementing this trait can be used with `ValidatedInputField`.
pub trait Validable: Send + Sync + Default + PartialEq + Clone + ToString + 'static {
    /// Attempts to validate and convert a string into this type.
    ///
    /// # Arguments
    ///
    /// * `text` - The input string to validate and convert.
    ///
    /// # Returns
    ///
    /// * `Ok(Self)` if the input is valid and can be converted to this type.
    /// * `Err(String)` with an error message if the input is invalid.
    fn validate(text: &str) -> Result<Self, String>;
}

impl Validable for String {
    fn validate(text: &str) -> Result<Self, String> {
        Ok(text.to_string())
    }
}

/// The current state of the text field validation
#[derive(Default, Clone, Debug)]
pub enum ValidationState {
    /// No validation has been performed yet
    #[default]
    Unchecked,
    /// The field content is valid
    Valid,
    /// The field content is invalid
    Invalid(String),
}

/// Event that is emitted when the validation state changes
#[derive(EntityEvent)]
pub struct ValidationChanged {
    /// The target entity.
    pub entity: Entity,
    /// The new validation state.
    pub state: ValidationState,
}

/// Event that is emitted when the value changes
#[derive(EntityEvent)]
pub struct ValueChanged<T: Validable> {
    /// The target entity.
    pub entity: Entity,
    /// The value updated.
    pub value: T,
}

/// This event is used to set the value of the validated input field.
#[derive(EntityEvent)]
pub struct SetValue<T: Validable> {
    /// The target entity.
    pub entity: Entity,
    /// The value set.
    pub value: T,
}

fn on_text_changed<T: Validable>(
    mut trigger: On<TextChanged>,
    mut commands: Commands,
    mut q_validated_input_fields: Query<&mut InputField<T>>,
) {
    let entity = trigger.event().event_target();
    let Ok(mut field) = q_validated_input_fields.get_mut(entity) else {
        return;
    };

    let new_text = trigger.new_text.clone();
    trigger.propagate(false);

    match T::validate(&new_text) {
        Ok(value) => {
            commands.trigger(ValueChanged {
                entity,
                value: value.clone(),
            });
            commands.trigger(ValidationChanged {
                entity,
                state: ValidationState::Valid,
            });
            // As editable label is controlled, we need to set the text manually
            commands.trigger(SetText {
                entity,
                text: new_text,
            });
            // Update the value only if the field is not controlled
            if !field.controlled {
                // Update the text in the EditableTextLine
                field.old_value = value.clone(); // We need to save the old value too for field change detection
                field.value = value;
            }
        }
        Err(error) => {
            // As editable label is controlled, we need to set the text manually
            commands.trigger(SetText {
                entity,
                text: new_text,
            });
            commands.trigger(ValidationChanged {
                entity,
                state: ValidationState::Invalid(error),
            });
        }
    }
}

fn on_value_changed<T: Validable>(
    mut commands: Commands,
    mut q_changed_inputs: Query<(Entity, &mut InputField<T>), Changed<InputField<T>>>,
) {
    for (entity, mut field) in q_changed_inputs.iter_mut() {
        if field.value != field.old_value {
            // info!("Trigger value rerender by field change for {:?}", entity);
            field.old_value = field.value.clone();

            // We will not trigger ValueChanged because it must be triggered only by input change
            // If value field was changed by external code, we will not trigger it again
            commands.trigger(SetText {
                entity,
                text: field.value.to_string(),
            });
            commands.trigger(ValidationChanged {
                entity,
                state: ValidationState::Valid,
            });
        }
    }
}

fn on_created<T: Validable>(
    mut commands: Commands,
    q_created_inputs: Query<(Entity, &InputField<T>), Added<InputField<T>>>,
) {
    for (entity, field) in q_created_inputs.iter() {
        // Set start state
        commands.trigger(SetText {
            entity,
            text: field.value.to_string(),
        });
        commands.trigger(ValidationChanged {
            entity,
            state: ValidationState::Valid,
        });
    }
}

macro_rules! impl_validable_for_numeric {
    ($($t:ty),*) => {
        $(
            impl Validable for $t {
                fn validate(text: &str) -> Result<Self, String> {
                    text.parse().map_err(|_| format!("Invalid {} number", stringify!($t)))
                }
            }
        )*
    };
}

impl_validable_for_numeric!(i8, i16, i32, i64, i128, u8, u16, u32, u64, u128, f32, f64);
