//! Allow to automatically manage created components and children in tree by storing last minimal tree state

use std::any::TypeId;

use bevy::{ecs::component::ComponentId, prelude::*, reflect::Type, utils::HashSet};

use crate::{construct::Construct, patch::Patch};

/// Represents a tree structure for managing entity differences and patches.
///
/// This structure is designed to hold a collection of patches that can be applied to an entity,
/// as well as a list of child trees that represent the entity's children in the scene hierarchy.
#[derive(Default)]
pub struct EntityDiffTree {
    /// A vector of patches that can be applied to the entity to modify its components.
    /// Each patch is a boxed trait object that implements `EntityComponentDiffPatch`.
    pub patch: Vec<Box<dyn EntityComponentDiffPatch>>,
    /// A vector of child trees, each representing a child entity in the scene hierarchy.
    pub children: Vec<EntityDiffTree>,
}

impl EntityDiffTree {
    /// Creates a new `EntityDiffTree` with empty patch and children.
    pub fn new() -> Self {
        Self {
            patch: Vec::new(),
            children: Vec::new(),
        }
    }

    pub fn add_patch(&mut self, patch: impl EntityComponentDiffPatch) {
        self.patch.push(Box::new(patch));
    }

    pub fn add_patch_fn<C: Component + Default + Clone>(&mut self, func: impl FnMut(&mut C) + Send + Sync + 'static) {
        self.add_patch(<C as Construct>::patch(func));
    }

    pub fn add_child(&mut self, child: EntityDiffTree) {
        self.children.push(child);
    }

    /// Adds a patch to the entity.
    pub fn with_patch(mut self, patch: impl EntityComponentDiffPatch) -> Self {
        self.patch.push(Box::new(patch));
        self
    }

    /// Adds a patch to the entity that is a function that mutate a component.
    pub fn with_patch_fn<C: Component + Default + Clone>(
        mut self,
        func: impl FnMut(&mut C) + Send + Sync + 'static,
    ) -> Self {
        self.with_patch(<C as Construct>::patch(func))
    }

    /// Adds a child to the entity.
    pub fn with_child(mut self, child: EntityDiffTree) -> Self {
        self.children.push(child);
        self
    }

    /// Applies the patch to the entity and its children.
    pub fn apply(&mut self, entity: Entity, world: &mut World) {
        let mut new_component_set = HashSet::new();
        {
            let mut entity_mut = world.entity_mut(entity);
            for patch in self.patch.iter_mut() {
                patch.entity_patch(&mut entity_mut);
                // SAFETY: we are not mutate any component or entity. Only read component id from components and register it if not registered yet
                #[allow(unsafe_code)]
                unsafe {
                    new_component_set.insert(patch.component_id(entity_mut.world_mut()));
                }
            }

            if let Some(last_state) = entity_mut.get::<LastTreeState>().cloned() {
                // Remove all components that was used in previous tree state but not in current
                for c_id in last_state
                    .component_ids
                    .iter()
                    .filter(|c_id| !new_component_set.contains(*c_id))
                {
                    entity_mut.remove_by_id(*c_id);
                    info!("Removed component {:?}", c_id);
                }
            }
        }

        // We will use separate "children" vector to avoid conflicts with inner logic of widgets which also can use children (For example InputField spawn children for self)
        let mut children_entities = if let Some(last_state) = world.entity(entity).get::<LastTreeState>().cloned() {
            last_state.children
        } else {
            Vec::new()
        };

        while children_entities.len() < self.children.len() {
            let child_entity = world.spawn_empty().id();
            world.entity_mut(entity).add_child(child_entity);
            children_entities.push(child_entity);
        }

        for (i, child) in self.children.iter_mut().enumerate() {
            child.apply(children_entities[i], world);
        }

        // Clear unused children
        for i in self.children.len()..children_entities.len() {
            world.entity_mut(children_entities[i]).despawn_recursive();
            info!("Despawned child {:?}", children_entities[i]);
        }

        // Store current state
        world.entity_mut(entity).insert(LastTreeState {
            component_ids: new_component_set,
            children: children_entities,
        });
    }

    fn contains_component<C: Component>(&self) -> bool {
        self.patch.iter().any(|patch| patch.get_type_id() == TypeId::of::<C>())
    }

    pub fn add_cascade_patch_fn<C: Component + Default + Clone, T: Component>(&mut self, func: impl Fn(&mut C) + Send + Sync + 'static + Clone) {
        if self.contains_component::<T>() {
            self.add_patch_fn(func.clone());
        } else {
            for child in self.children.iter_mut() {
                child.add_cascade_patch_fn::<C, T>(func.clone());
            }
        }
    }
}

/// This trait is used to modify an entity's components and store the component's ID for tracking purposes.
pub trait EntityComponentDiffPatch: Send + Sync + 'static {
    /// Applies the patch to the given entity.
    fn entity_patch(&mut self, entity_mut: &mut EntityWorldMut);

    /// Returns the ComponentId of the component that this patch is associated with.
    /// This is used to keep track of the components that were present in an entity during the last update.
    fn component_id(&self, world: &mut World) -> ComponentId;

    /// Returns the TypeId of the component that this patch is associated with.
    fn get_type_id(&self) -> TypeId;
}

impl<C: Component + Default + Clone, T: Patch<Construct = C>> EntityComponentDiffPatch for T {
    fn entity_patch(&mut self, entity_mut: &mut EntityWorldMut) {
        if !entity_mut.contains::<C>() {
            entity_mut.insert(C::default());
        }

        let mut component = entity_mut.get_mut::<C>().unwrap();
        self.patch(&mut component);
    }

    fn component_id(&self, world: &mut World) -> ComponentId {
        if let Some(c_id) = world.components().component_id::<C>() {
            c_id
        } else {
            world.register_component::<C>()
        }
    }

    fn get_type_id(&self) -> TypeId {
        TypeId::of::<C>()
    }
}

/// Represents the state of an entity's component tree from the last update.
///
/// This struct is used to keep track of the components and children that were
/// present in an entity during the last tree update. It helps in efficiently
/// determining what has changed in subsequent updates.
#[derive(Default, Component, Clone)]
pub struct LastTreeState {
    /// A set of ComponentIds representing the components that were present
    /// in the entity during the last update.
    pub component_ids: HashSet<ComponentId>,

    /// The used child entities that the entity had during the last update.
    pub children: Vec<Entity>,
}

/// A trait for applying an `EntityDiffTree` to an entity using `EntityCommands`.
///
/// This trait extends the functionality of `EntityCommands` to allow for
/// the application of an `EntityDiffTree`, which represents a set of changes
/// to be applied to an entity's components and children.
pub trait DiffTreeCommands {
    /// Applies the given `EntityDiffTree` to the entity.
    ///
    /// This method queues a command that will apply all the changes
    /// specified in the `EntityDiffTree` to the entity when the command
    /// is executed.
    ///
    /// # Arguments
    ///
    /// * `tree` - The `EntityDiffTree` containing the changes to apply to the entity.
    fn diff_tree(&mut self, tree: EntityDiffTree);
}

impl DiffTreeCommands for EntityCommands<'_> {
    fn diff_tree(&mut self, mut tree: EntityDiffTree) {
        self.queue(move |entity: Entity, world: &mut World| {
            tree.apply(entity, world);
        });
    }
}

#[cfg(test)]
mod tests {
    use crate::construct::Construct;

    use super::*;
    use bevy::prelude::*;

    #[test]
    fn create_default_component() {
        let mut world = World::default();
        let entity = world.spawn_empty().id();
        let mut tree = EntityDiffTree::new().with_patch(Transform::patch(|transform| {
            transform.translation = Vec3::new(1.0, 2.0, 3.0)
        }));

        tree.apply(entity, &mut world);

        let transform = world.entity(entity).get::<Transform>().unwrap();
        // Check that patch was applied
        assert_eq!(transform.translation, Vec3::new(1.0, 2.0, 3.0));
        // Check that other fields are default
        assert_eq!(transform.rotation, Quat::IDENTITY);
        assert_eq!(transform.scale, Vec3::ONE);
    }

    #[test]
    fn check_component_removal() {
        let mut world = World::default();
        let entity = world.spawn_empty().id();
        let mut tree = EntityDiffTree::new().with_patch(Transform::patch(|transform| {
            transform.translation = Vec3::new(1.0, 2.0, 3.0)
        }));

        tree.apply(entity, &mut world);

        assert!(world.entity(entity).contains::<Transform>());

        let mut second_tree = EntityDiffTree::new().with_patch(Name::patch(|name| {
            name.set("test");
        }));

        second_tree.apply(entity, &mut world);

        assert!(world.entity(entity).contains::<Name>());
        assert!(!world.entity(entity).contains::<Transform>());
    }

    #[test]
    fn check_children_create_and_remove() {
        let mut world = World::default();
        let entity = world.spawn_empty().id();
        let mut tree = EntityDiffTree::new()
            .with_patch(Transform::patch(|transform| {
                transform.translation = Vec3::new(1.0, 2.0, 3.0)
            }))
            .with_child(EntityDiffTree::new().with_patch(Transform::patch(|t| {
                t.translation = Vec3::new(4.0, 5.0, 6.0)
            })));

        tree.apply(entity, &mut world);

        assert_eq!(world.entity(entity).get::<Children>().unwrap().len(), 1);
        let child_entity = world.entity(entity).get::<Children>().unwrap()[0];
        assert_eq!(
            world
                .entity(child_entity)
                .get::<Transform>()
                .unwrap()
                .translation,
            Vec3::new(4.0, 5.0, 6.0)
        );

        let mut second_tree = EntityDiffTree::new();
        second_tree.apply(entity, &mut world);

        assert!(world.get_entity(child_entity).is_err());
        assert_eq!(world.entity(entity).get::<Children>().unwrap().len(), 0);
    }

    #[test]
    fn test_fn_patches() {
        let mut world = World::default();
        let entity = world.spawn_empty().id();
        let mut tree = EntityDiffTree::new().with_patch_fn(|t: &mut Transform| {
            t.translation = Vec3::new(1.0, 2.0, 3.0);
        });

        tree.apply(entity, &mut world);

        let transform = world.entity(entity).get::<Transform>().unwrap();
        assert_eq!(transform.translation, Vec3::new(1.0, 2.0, 3.0));
    }
}
