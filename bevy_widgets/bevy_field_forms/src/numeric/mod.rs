use crate::validated_input_field::*;
use bevy::prelude::*;

#[derive(Component)]
pub struct NumericField<T: Numeric> {
    pub value: T,
    pub controlled: bool,
}

pub trait Numeric: Validable {}

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
