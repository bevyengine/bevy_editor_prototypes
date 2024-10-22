//! This module provides the context for the construct trait

use bevy::prelude::*;

pub struct ConstructContext<'a> {
    pub id: Entity,
    pub world: &'a mut World,
}