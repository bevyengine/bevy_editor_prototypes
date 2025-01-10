use core::marker::PhantomData;

use bevy::ecs::{
    component::{ComponentHooks, ComponentId, StorageType},
    prelude::*,
    world::{Command, DeferredWorld},
};

/// A component that when added to an entity, will be removed from the entity and replaced with its contents if [`Some`].
///
/// Under the hood, this is done using component lifecycle hooks.
/// The component is removed from the entity when it is added, and contents are extracted.
/// If the inner value is [`Some`], the contents are then readded to the entity.
///
/// # Example
///
/// ```rust
/// use bevy::ecs::prelude::*;
/// use bevy::ecs::system::RunSystemOnce;
/// use bevy_i_cant_believe_its_not_bsn::Maybe;
///
/// #[derive(Component)]
/// struct A;
///
/// #[derive(Bundle)]
/// struct TestBundle {
///    maybe_a: Maybe<A>,
/// }
///
/// let mut world = World::new();
///
/// let entity_with_component = world.run_system_once(|mut commands: Commands| -> Entity {
///     commands
///         .spawn(TestBundle {
///             maybe_a: Maybe::new(A),
///         })
///         .id()
/// }).unwrap();
/// let entity_ref = world.get_entity(entity_with_component).unwrap();
/// assert!(entity_ref.contains::<A>());
/// assert!(!entity_ref.contains::<Maybe<A>>());
///
/// let entity_without_component = world.run_system_once(|mut commands: Commands| -> Entity {
///     commands
///         .spawn(TestBundle {
///             maybe_a: Maybe::NONE,
///         })
///         .id()
/// }).unwrap();
/// let entity_ref = world.get_entity(entity_without_component).unwrap();
/// assert!(!entity_ref.contains::<A>());
/// assert!(!entity_ref.contains::<Maybe<A>>());
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Maybe<B: Bundle>(pub Option<B>);

impl<B: Bundle> Component for Maybe<B> {
    /// This is a sparse set component as it's only ever added and removed, never iterated over.
    const STORAGE_TYPE: StorageType = StorageType::SparseSet;

    fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks.on_add(maybe_hook::<B>);
    }
}

impl<B: Bundle> Maybe<B> {
    /// Creates a new `Maybe` component of type `B` with no bundle.
    pub const NONE: Self = Self(None);

    /// Creates a new `Maybe` component with the given bundle.
    pub const fn new(bundle: B) -> Self {
        Self(Some(bundle))
    }

    /// Returns the contents of the `Maybe` component, if any.
    pub fn into_inner(self) -> Option<B> {
        self.0
    }
}

impl<B: Bundle> Default for Maybe<B> {
    /// Defaults to [`Maybe::NONE`].
    fn default() -> Self {
        Self::NONE
    }
}

/// A hook that runs whenever [`Maybe`] is added to an entity.
///
/// Generates a [`MaybeCommand`].
fn maybe_hook<B: Bundle>(mut world: DeferredWorld<'_>, entity: Entity, _component_id: ComponentId) {
    // Component hooks can't perform structural changes, so we need to rely on commands.
    world.commands().queue(MaybeCommand {
        entity,
        _phantom: PhantomData::<B>,
    });
}

struct MaybeCommand<B> {
    entity: Entity,
    _phantom: PhantomData<B>,
}

impl<B: Bundle> Command for MaybeCommand<B> {
    fn apply(self, world: &mut World) {
        let Ok(mut entity_mut) = world.get_entity_mut(self.entity) else {
            #[cfg(debug_assertions)]
            panic!("Entity with Maybe component not found");

            #[cfg(not(debug_assertions))]
            return;
        };

        let Some(maybe_component) = entity_mut.take::<Maybe<B>>() else {
            #[cfg(debug_assertions)]
            panic!("Maybe component not found");

            #[cfg(not(debug_assertions))]
            return;
        };

        if let Some(bundle) = maybe_component.into_inner() {
            entity_mut.insert(bundle);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Component)]
    struct A;

    #[derive(Bundle)]
    struct TestBundle {
        maybe_a: Maybe<A>,
    }

    #[test]
    fn maybe_some() {
        let mut world = World::new();
        let entity = world
            .spawn(TestBundle {
                maybe_a: Maybe::new(A),
            })
            .id();

        // FIXME: this should not be needed!
        world.flush();

        assert!(world.get::<A>(entity).is_some());
        assert!(world.get::<Maybe<A>>(entity).is_none());
    }

    #[test]
    fn maybe_none() {
        let mut world = World::new();
        let entity = world
            .spawn(TestBundle {
                maybe_a: Maybe::NONE,
            })
            .id();

        // FIXME: this should not be needed!
        world.flush();

        assert!(world.get::<A>(entity).is_none());
        assert!(world.get::<Maybe<A>>(entity).is_none());
    }

    #[test]
    fn maybe_system() {
        use bevy::ecs::system::RunSystemOnce;

        let mut world = World::new();

        let entity_with_component = world
            .run_system_once(|mut commands: Commands| -> Entity {
                commands
                    .spawn(TestBundle {
                        maybe_a: Maybe::new(A),
                    })
                    .id()
            })
            .unwrap();

        let entity_ref = world.get_entity(entity_with_component).unwrap();
        assert!(entity_ref.contains::<A>());
        assert!(!entity_ref.contains::<Maybe<A>>());

        let entity_without_component = world
            .run_system_once(|mut commands: Commands| -> Entity {
                commands
                    .spawn(TestBundle {
                        maybe_a: Maybe::NONE,
                    })
                    .id()
            })
            .unwrap();

        let entity_ref = world.get_entity(entity_without_component).unwrap();
        assert!(!entity_ref.contains::<A>());
        assert!(!entity_ref.contains::<Maybe<A>>());
    }
}
