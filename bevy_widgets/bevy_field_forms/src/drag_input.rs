//! This module contains the logic allow to drag value stored in a input field

use bevy::prelude::*;

use crate::input_field::{InputField, Validable, ValueChanged};

/// Plugin for dragging a value stored in an input field
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

        app.add_systems(PreUpdate, on_interaction_changed::<T>);
    }
}

/// A trait for values that can be dragged
pub trait Draggable:
    Send + Sync + 'static + Default + PartialEq + Validable + std::ops::Add<Output = Self> + Copy
{
    /// Converts a f32 value to Self
    fn from_f32(value: f32) -> Self;

    /// Converts Self to a f32 value
    fn into_f32(self) -> f32;

    /// Safely adds another value of the same type, handling potential overflows
    fn safe_add(&self, other: Self) -> Self;

    /// Safely subtracts another value of the same type, handling potential underflows
    fn safe_sub(&self, other: Self) -> Self;

    /// Returns the default ratio of the value change per logical pixel drag
    fn default_drag_ratio() -> f32;
}

/// A component that allows dragging a value stored in an input field
#[derive(Component, Clone)]
#[require(Node, InputField::<T>, Interaction)]
pub struct DragInput<T: Draggable> {
    /// The accumulated drag value
    pub drag_accumulate: f32,

    /// The ratio of the value change per logical pixel drag
    pub drag_ratio: f32,

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
    trigger: On<Pointer<Drag>>,
    mut commands: Commands,
    mut q_drag_inputs: Query<(&mut DragInput<T>, &mut InputField<T>)>,
) {
    let entity = trigger.event().event_target();

    let Ok((mut drag_input, mut input_field)) = q_drag_inputs.get_mut(entity) else {
        return;
    };

    let delta = trigger.delta.x;
    drag_input.drag_accumulate += delta * drag_input.drag_ratio;

    let from_accumulated: T = T::from_f32(drag_input.drag_accumulate.abs());
    let accumulated_decrease: f32 = from_accumulated.into_f32();
    if accumulated_decrease != 0.0 {
        let new_val: T;
        if drag_input.drag_accumulate > 0.0 {
            new_val = input_field.value.safe_add(from_accumulated);
            drag_input.drag_accumulate -= accumulated_decrease;
        } else {
            new_val = input_field.value.safe_sub(from_accumulated);
            drag_input.drag_accumulate += accumulated_decrease;
        }

        commands.trigger(ValueChanged {
            entity,
            value: new_val,
        });
        if !input_field.controlled {
            input_field.value = new_val;
        }
    }
}

fn on_interaction_changed<T: Draggable>(
    mut q_changed_interactions: Query<(&mut DragInput<T>, &Interaction), Changed<Interaction>>,
) {
    for (mut drag_input, interaction) in q_changed_interactions.iter_mut() {
        if *interaction != Interaction::Pressed {
            drag_input.drag_accumulate = 0.0;
        }
    }
}

macro_rules! impl_draggable_for_numeric {
    ($($t:ty),*) => {
        $(
            impl Draggable for $t {
                fn default_drag_ratio() -> f32 {
                    0.1
                }

                fn safe_add(&self, other: Self) -> Self {
                    self.checked_add(other).unwrap_or(Self::MAX)
                }

                fn safe_sub(&self, other: Self) -> Self {
                    self.checked_sub(other).unwrap_or(Self::MIN)
                }

                fn from_f32(value: f32) -> Self {
                    let clamped = value.clamp(Self::MIN as f32, Self::MAX as f32);
                    clamped as Self
                }

                fn into_f32(self) -> f32 {
                    self as f32
                }
            }
        )*
    };
}

impl_draggable_for_numeric!(i8, i16, i32, i64, i128, u8, u16, u32, u64, u128);

macro_rules! impl_draggable_for_float {
    ($($t:ty),*) => {
        $(
            impl Draggable for $t {
                fn default_drag_ratio() -> f32 {
                    0.01
                }

                fn safe_add(&self, other: Self) -> Self {
                    *self + other
                }

                fn safe_sub(&self, other: Self) -> Self {
                    *self - other
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

impl_draggable_for_float!(f32, f64);
