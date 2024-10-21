//! Contains spawn logic for numeric fields

use bevy::prelude::*;
use bevy_text_field::LineTextField;

use crate::{InnerNumericField, NumericField, NumericFieldValue};

pub(crate) fn spawn_numeric_field<T: NumericFieldValue>(
    mut commands: Commands,
    q_added: Query<(Entity, &NumericField<T>), Added<NumericField<T>>>,
) {
    for (entity, field) in q_added.iter() {
        let mut text_field = LineTextField::new(field.value.to_string());
        text_field.set_allowed_chars(T::allowed_chars());

        commands
            .entity(entity)
            .insert(text_field)
            .insert(InnerNumericField::<T> {
                last_val: field.value,
                failed_convert: false,
                ignore_text_changes: false,
                accumulated_delta: 0.0,
            });
    }
}
