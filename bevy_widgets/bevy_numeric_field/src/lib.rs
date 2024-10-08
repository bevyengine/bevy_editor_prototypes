//! This library provides a input numeric field widget for Bevy.

mod events;
mod input;
mod numeric_field_struct;
mod render;
mod spawn;

use bevy::prelude::*;
use bevy_text_field::LineTextFieldPlugin;
use std::marker::PhantomData;

pub use events::*;
pub use numeric_field_struct::*;
pub use spawn::*;

/// Contains default numeric field plugins for all basic numeric types (u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64)
pub struct DefaultNumericFieldPlugin;

impl Plugin for DefaultNumericFieldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            NumericFieldPlugin::<u8>::default(),
            NumericFieldPlugin::<u16>::default(),
            NumericFieldPlugin::<u32>::default(),
            NumericFieldPlugin::<u64>::default(),
            NumericFieldPlugin::<u128>::default(),
            NumericFieldPlugin::<i8>::default(),
            NumericFieldPlugin::<i16>::default(),
            NumericFieldPlugin::<i32>::default(),
            NumericFieldPlugin::<i64>::default(),
            NumericFieldPlugin::<i128>::default(),
            NumericFieldPlugin::<f32>::default(),
            NumericFieldPlugin::<f64>::default(),
        ));
    }
}

/// Plugin for numeric field logic with one numeric type
pub struct NumericFieldPlugin<T: NumericFieldValue> {
    _phantom: PhantomData<T>,
}

impl<T: NumericFieldValue> Default for NumericFieldPlugin<T> {
    fn default() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<T: NumericFieldValue> Plugin for NumericFieldPlugin<T> {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<LineTextFieldPlugin>() {
            app.add_plugins(LineTextFieldPlugin);
        }

        app.add_event::<NewValue<T>>();
        app.add_event::<SetValue<T>>();

        app.add_systems(PreUpdate, spawn::spawn_numeric_field::<T>);
        app.add_systems(Update, render::set_borders::<T>);
        app.add_systems(Update, input::react_on_self_changes::<T>);

        app.observe(input::react_to_text_changes::<T>);
        app.observe(input::react_on_set_value::<T>);
        app.observe(input::react_on_lost_focus::<T>);
        app.observe(input::react_on_drag::<T>);
    }
}
