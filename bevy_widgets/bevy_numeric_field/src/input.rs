use bevy::prelude::*;
use bevy_focus::{Focus, LostFocus};
use bevy_text_field::{render::RenderTextField, LineTextField, TextChanged};

use crate::{InnerNumericField, NewValue, NumericField, NumericFieldValue, SetValue};

pub fn react_to_text_changes<T: NumericFieldValue>(
    trigger: Trigger<TextChanged>,
    mut commands: Commands,
    mut q_fields: Query<(
        &LineTextField,
        &mut NumericField<T>,
        &mut InnerNumericField<T>,
    )>,
) {
    let entity = trigger.entity();

    let Ok((text_field, mut field, mut inner_field)) = q_fields.get_mut(entity) else {
        return;
    };

    if inner_field.ignore_text_changes {
        return;
    }

    if let Ok(val) = T::from_str(text_field.text()) {
        field.value = val;
        inner_field.failed_convert = false;
        if inner_field.last_val != field.value {
            commands.trigger_targets(NewValue(field.value), entity);
            inner_field.last_val = field.value;
        }
    } else {
        field.value = inner_field.last_val;
        inner_field.failed_convert = true;
    }
}

pub fn react_on_self_changes<T: NumericFieldValue>(
    mut q_changed: Query<
        (Entity, &NumericField<T>, &mut LineTextField),
        (Changed<NumericField<T>>, Without<Focus>),
    >,
) {
    for (_, field, mut text_field) in q_changed.iter_mut() {
        text_field.set_text(field.value.to_string());
    }
}

pub fn react_on_set_value<T: NumericFieldValue>(
    trigger: Trigger<SetValue<T>>,
    mut q_fields: Query<(
        &mut NumericField<T>,
        &mut InnerNumericField<T>,
        &mut LineTextField,
    )>,
) {
    if let Ok((mut field, mut inner_field, mut text_field)) = q_fields.get_mut(trigger.entity()) {
        field.set_value(trigger.event().0);
        inner_field.last_val = field.value;
        inner_field.failed_convert = false;
        text_field.set_text(field.value.to_string());
    }
}

pub fn react_on_lost_focus<T: NumericFieldValue>(
    trigger: Trigger<LostFocus>,
    mut commands: Commands,
    mut q_fields: Query<(
        &NumericField<T>,
        &mut InnerNumericField<T>,
        &mut LineTextField,
    )>,
) {
    let entity = trigger.entity();
    let Ok((field, mut inner_field, mut text_field)) = q_fields.get_mut(entity) else {
        return;
    };
    text_field.set_text(field.value.to_string());
    if inner_field.failed_convert {
        commands.trigger_targets(NewValue(field.value), entity);
    }

    inner_field.failed_convert = false;
    inner_field.last_val = field.value;
}

pub fn react_on_drag<T: NumericFieldValue>(
    trigger: Trigger<Pointer<Drag>>,
    mut commands: Commands,
    mut q_fields: Query<(
        &mut NumericField<T>,
        &mut InnerNumericField<T>,
        &mut LineTextField,
    )>,
) {
    let entity = trigger.entity();
    let Ok((mut field, mut inner_field, mut text_field)) = q_fields.get_mut(entity) else {
        return;
    };

    let Some(drag_step) = field.drag_step else {
        return;
    };
    let old_val = field.value;

    // Calculate delta with drag direction
    let delta = drag_step * trigger.event().event.delta.x as f64;

    // Аккумулируем дельту
    inner_field.accumulated_delta += delta;

    let val_from_accum = T::from(inner_field.accumulated_delta);
    if let Some(val_from_accum) = val_from_accum {
        if val_from_accum != T::from(0).unwrap() {
            // Its float we can just add delta
            let new_val = field.value + val_from_accum;
            field.set_value(new_val);
            text_field.set_text(field.value.to_string());
            inner_field.failed_convert = false;
            if inner_field.last_val != field.value {
                commands.trigger_targets(NewValue(field.value), entity);
                inner_field.last_val = field.value;
            }

            inner_field.accumulated_delta = 0.0;

            return;
        }
    }

    // Check if we can add delta to integer
    if inner_field.accumulated_delta.abs() >= 1.0 {
        let change = inner_field.accumulated_delta.trunc() as i64;
        inner_field.accumulated_delta -= change as f64;

        let new_val = if change > 0 {
            (0..change).try_fold(old_val, |v, _| v.checked_add(&T::from(1).unwrap()))
        } else {
            (0..change.abs()).try_fold(old_val, |v, _| v.checked_sub(&T::from(1).unwrap()))
        };

        if let Some(new_val) = new_val {
            field.set_value(new_val);
            text_field.set_text(field.value.to_string());
            inner_field.failed_convert = false;
            if inner_field.last_val != field.value {
                commands.trigger_targets(NewValue(field.value), entity);
                inner_field.last_val = field.value;
            }
        }
    }
}

pub fn react_on_drag_end<T: NumericFieldValue>(
    trigger: Trigger<Pointer<DragEnd>>,
    mut commands: Commands,
    q_fields: Query<(), With<NumericField<T>>>,
) {
    let entity = trigger.entity();
    let Ok(_) = q_fields.get(entity) else {
        return;
    };

    commands.trigger_targets(RenderTextField::default(), entity);
}
