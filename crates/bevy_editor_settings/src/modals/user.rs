//! this is for the user to override workspace settings

use bevy::reflect::Reflect;

#[derive(Debug, Clone, PartialEq, Eq, Reflect)]
/// Settings for the user
pub struct UserSettings {}
