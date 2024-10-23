//! This crate provides a collapsing header widget for Bevy.

pub use bevy::prelude::*;


#[derive(Component)]
pub struct CollapsingHeader {
    pub is_collapsed: bool,
}

impl Default for CollapsingHeader {
    fn default() -> Self {
        Self { is_collapsed: true }
    }
}