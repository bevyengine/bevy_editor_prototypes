//! This module provides functionality for traversing events to the first child of an entity.
//!
//! It includes:
//! - `FirstChildTraversalPlugin`: A plugin that sets up the necessary systems for child traversal.
//! - `FirstChildTraversal`: A marker component for entities that should use first-child traversal.
//! - `CachedFirsChild`: A component that caches the first child of an entity for efficient traversal.
//!
//! The module also implements the `Traversal` trait for `CachedFirsChild`, allowing for easy
//! integration with Bevy's event system.

use bevy::{ecs::traversal::Traversal, prelude::*};

/// Plugin for traversing events to the first child of an entity
pub struct FirstChildTraversalPlugin;

impl Plugin for FirstChildTraversalPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, auto_update_cache);
    }
}

/// Marker for traversing events to the first child of an entity
#[derive(Component, Debug, Default)]
pub struct FirstChildTraversal;

/// State for caching the first child of an entity to make Traverse trait easier to implement
#[derive(Component, Debug)]
pub struct CachedFirsChild(pub Entity);

impl Traversal for &'static CachedFirsChild {
    fn traverse(item: Self::Item<'_>) -> Option<Entity> {
        Some(item.0)
    }
}

fn auto_update_cache(
    mut commands: Commands,
    q_changed_children: Query<
        (Entity, &Children),
        (
            With<FirstChildTraversal>,
            Or<(Changed<Children>, Added<FirstChildTraversal>)>,
        ),
    >,
) {
    for (entity, children) in q_changed_children.iter() {
        if let Some(first_child) = children.first() {
            commands
                .entity(entity)
                .insert(CachedFirsChild(*first_child));
        }
    }
}
