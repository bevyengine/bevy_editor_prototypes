use bevy::{ecs::traversal::Traversal, prelude::*};

pub struct FirstChildTraversalPlugin;

impl Plugin for FirstChildTraversalPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, auto_update_cache);
    }
}

#[derive(Component, Debug, Default)]
pub struct FirstChildTraversal;

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
