//! Helper functions for cleaner BSN templates.
use bevy::{
    color::Color,
    ui::{UiRect, Val},
};

/// Shorthand for [`Val::Px`].
pub fn px(value: impl Into<f32>) -> Val {
    Val::Px(value.into())
}

/// Shorthand for [`UiRect::all`] + [`Val::Px`].
pub fn px_all(value: impl Into<f32>) -> UiRect {
    UiRect::all(Val::Px(value.into()))
}

/// Shorthand for [`Color::srgb_u8`].
pub fn rgb8(red: u8, green: u8, blue: u8) -> Color {
    Color::srgb_u8(red, green, blue)
}

/// Shorthand for [`Color::srgba_u8`].
pub fn rgba8(red: u8, green: u8, blue: u8, alpha: u8) -> Color {
    Color::srgba_u8(red, green, blue, alpha)
}
