//! This module contains the logic allow to drag value stored in a input field

use bevy::prelude::*;

use crate::input_field::{InputField, SetValue, Validable, ValueChanged};

pub struct DragInputPlugin<T: Draggable> {
    _marker: std::marker::PhantomData<T>,
}

impl<T: Draggable> Default for DragInputPlugin<T> {
    fn default() -> Self {
        Self {
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T: Draggable> Plugin for DragInputPlugin<T> {
    fn build(&self, app: &mut App) {
        app.add_observer(on_drag::<T>);
    }
}

/// A trait for values that can be dragged
pub trait Draggable:
    Send + Sync + 'static + Default + PartialEq + Validable + std::ops::Add<Output = Self> + Copy
{
    fn from_f32(value: f32) -> Self;
    fn into_f32(self) -> f32;

    /// The default ratio of the value change per logical pixel drag
    fn default_drag_ratio() -> f32;
}

#[derive(Component)]
#[require(Node, InputField::<T>)]
pub struct DragInput<T: Draggable> {
    drag_accumulate: f32,
    drag_ratio: f32,
    _marker: std::marker::PhantomData<T>,
}

impl<T: Draggable> Default for DragInput<T> {
    fn default() -> Self {
        Self {
            drag_accumulate: 0.0,
            drag_ratio: T::default_drag_ratio(),
            _marker: std::marker::PhantomData,
        }
    }
}


fn on_drag<T: Draggable>(
    trigger: Trigger<Pointer<Drag>>,
    mut commands: Commands,
    mut q_drag_inputs: Query<(&mut DragInput<T>, &mut InputField<T>)>,
) {
    let entity = trigger.entity();

    let Ok((mut drag_input, mut input_field)) = q_drag_inputs.get_mut(entity) else {
        return;
    };

    let delta = trigger.delta.x;
    drag_input.drag_accumulate += delta * drag_input.drag_ratio;

    let from_accumulated : T = T::from_f32(drag_input.drag_accumulate);
    let accumulted_decrese : f32 = from_accumulated.into_f32();
    if accumulted_decrese != 0.0 {
        let new_val = input_field.value + from_accumulated;
        commands.trigger_targets(ValueChanged(new_val), entity);
        if !input_field.controlled {
            commands.trigger_targets(SetValue(new_val), entity);
        }
    }
}


macro_rules! impl_draggable_for_numeric {
    ($($t:ty),*) => {
        $(
            impl Draggable for $t {
                fn default_drag_ratio() -> f32 {
                    1.0
                }

                fn from_f32(value: f32) -> Self {
                    value as Self
                }

                fn into_f32(self) -> f32 {
                    self as f32
                }
            }
        )*
    };
}

impl_draggable_for_numeric!(i8, i16, i32, i64, i128, u8, u16, u32, u64, u128, f32, f64);
