//! Contains numeric field definition and implementation for all basic numeric types

use bevy::prelude::*;
use num_traits::{Bounded, NumCast, One, Zero};
use std::cmp::{max, min, PartialOrd};
use std::ops::{Add, Sub};
use std::str::FromStr;

#[derive(Component)]
pub struct NumericField<T: NumericFieldValue> {
    pub value: T,

    pub min: Option<T>,
    pub max: Option<T>,

    pub drag_step: Option<T>, // Change value by logical pixel mouse movement
    pub arrow_step: Option<T>, // Change value by arrow keys
}

#[derive(Component)]
pub(crate) struct InnerNumericField<T: NumericFieldValue> {
    pub last_val: T,
    pub failed_convert: bool,
    pub ignore_text_changes: bool,
}

pub trait NumericFieldValue:
    Copy
    + PartialOrd
    + Add<Output = Self>
    + Sub<Output = Self>
    + Zero
    + One
    + Bounded
    + NumCast
    + PartialEq
    + Send
    + Sync
    + 'static
    + ToString
    + FromStr
{
    fn default_drag_step() -> Self;
    fn default_arrow_step() -> Self;
    fn allowed_chars() -> Vec<char>;
}

impl<T> NumericField<T>
where
    T: NumericFieldValue,
{
    pub fn new(value: T) -> Self {
        NumericField {
            value,
            min: Some(T::min_value()),
            max: Some(T::max_value()),
            drag_step: Some(T::default_drag_step()),
            arrow_step: Some(T::default_arrow_step()),
        }
    }

    pub(crate) fn set_value(&mut self, new_value: T) {
        let new_value = if let Some(min) = self.min {
            if new_value < min {
                min
            } else {
                new_value
            }
        } else {
            new_value
        };

        let new_value = if let Some(max) = self.max {
            if new_value > max {
                max
            } else {
                new_value
            }
        } else {
            new_value
        };

        self.value = new_value;
    }

    pub fn increment(&mut self) {
        if let Some(step) = self.arrow_step {
            self.set_value(self.value + step);
        }
    }

    pub fn decrement(&mut self) {
        if let Some(step) = self.arrow_step {
            self.set_value(self.value - step);
        }
    }
}

macro_rules! impl_signed_numeric_field_value {
    ($($t:ty),*) => {
        $(
            impl NumericFieldValue for $t {
                fn default_drag_step() -> Self { One::one() }
                fn default_arrow_step() -> Self { One::one() }
                fn allowed_chars() -> Vec<char> {
                    vec!['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '-']
                }
            }
        )*
    }
}

macro_rules! impl_unsigned_numeric_field_value {
    ($($t:ty),*) => {
        $(
            impl NumericFieldValue for $t {
                fn default_drag_step() -> Self { One::one() }
                fn default_arrow_step() -> Self { One::one() }
                fn allowed_chars() -> Vec<char> {
                    vec!['0', '1', '2', '3', '4', '5', '6', '7', '8', '9']
                }
            }
        )*
    }
}

impl_signed_numeric_field_value!(i8, i16, i32, i64, i128);
impl_unsigned_numeric_field_value!(u8, u16, u32, u64, u128);

impl NumericFieldValue for f32 {
    fn default_drag_step() -> Self {
        0.1
    }
    fn default_arrow_step() -> Self {
        1.0
    }
    fn allowed_chars() -> Vec<char> {
        vec!['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '.', '-']
    }
}

impl NumericFieldValue for f64 {
    fn default_drag_step() -> Self {
        0.1
    }
    fn default_arrow_step() -> Self {
        1.0
    }
    fn allowed_chars() -> Vec<char> {
        vec!['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '.', '-']
    }
}
