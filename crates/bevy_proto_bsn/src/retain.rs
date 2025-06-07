use core::hash::Hash;
use std::fmt::Display;

use bevy::{
    ecs::component::{ComponentId, ComponentInfo},
    platform::{collections::HashMap, hash::FixedHasher},
    prelude::{Component, Deref, DerefMut, Entity, EntityCommands, EntityWorldMut},
};

use crate::{Scene, *};

/// An anchor is an identifier for entities in a retained scene.
#[derive(Hash, Eq, PartialEq, Clone)]
pub enum Anchor {
    /// The entity is static and using an automatic incrementing ID.
    Auto(u64),
    /// The entity has been explicitly keyed with a [`Key`].
    Keyed(Key),
}

/// An explicit identifier for an entity in a retained scene.
#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct Key(String);

impl<T: Display> From<T> for Key {
    fn from(s: T) -> Self {
        Self(s.to_string())
    }
}

/// Receipts allow retaining of scenes that can be intelligently updated.
#[derive(Component)]
pub struct Receipt<T = ()> {
    /// The components it inserted.
    components: InsertedComponents,
    /// The anchors of all the children it spawned/retained.
    anchors: HashMap<Anchor, Entity>,
    marker: core::marker::PhantomData<T>,
}

impl<T> Clone for Receipt<T> {
    fn clone(&self) -> Self {
        Self {
            components: self.components.clone(),
            anchors: self.anchors.clone(),
            marker: Default::default(),
        }
    }
}

impl<T> Default for Receipt<T> {
    fn default() -> Self {
        Self {
            components: Default::default(),
            anchors: Default::default(),
            marker: Default::default(),
        }
    }
}

/// Map of inserted component ids to a bool of whether they were explicit or required.
#[derive(Default, Clone, Deref, DerefMut)]
pub struct InsertedComponents(HashMap<ComponentId, bool>);

impl InsertedComponents {
    fn with_capacity(capacity: usize) -> Self {
        Self(HashMap::with_capacity_and_hasher(capacity, FixedHasher))
    }

    /// Returns an iterator of the component ids that should be removed going from `self` to `new`.
    ///
    /// Components are considered removed if they are in `self` but not in `new`, or if they were previously explicit but are now required.
    fn iter_removed<'a>(
        &'a self,
        new: &'a InsertedComponents,
    ) -> impl Iterator<Item = ComponentId> + 'a {
        self.iter()
            .filter(|(id, explicit)| {
                !new.contains_key(*id) || (**explicit && !new.get(*id).unwrap())
            })
            .map(|(id, _)| *id)
    }

    fn insert_required(&mut self, id: ComponentId) {
        self.entry(id).or_insert(false);
    }

    fn insert_explicit(&mut self, component_info: &ComponentInfo) {
        self.insert(component_info.id(), true);
        for required_id in component_info.required_components().iter_ids() {
            self.insert_required(required_id);
        }
    }
}

/// Trait implemented for scenes that can be retained.
pub trait RetainScene {
    /// Retains the scene by constructing and inserting the components on the entity,
    ///  removing components that should be removed, and spawning/updating children.
    ///
    /// Maintains [`Receipt`]s to allow for intelligent updates.
    fn retain<T: Send + Sync + 'static>(
        self,
        entity: &mut EntityWorldMut,
    ) -> Result<(), ConstructError>;
}

impl RetainScene for () {
    fn retain<T: Send + Sync + 'static>(
        self,
        entity: &mut EntityWorldMut,
    ) -> Result<(), ConstructError> {
        // Remove/take the receipt if it exists.
        let Some(receipt) = entity.take::<Receipt<T>>() else {
            return Ok(());
        };

        // Remove the components
        for component_id in receipt.components.keys() {
            entity.remove_by_id(*component_id);
        }

        // Clear children
        entity.world_scope(|world| {
            for orphan_id in receipt.anchors.into_values() {
                world.entity_mut(orphan_id).despawn();
            }
        });

        Ok(())
    }
}

impl RetainScene for DynamicScene {
    fn retain<T: Send + Sync + 'static>(
        self,
        entity: &mut EntityWorldMut,
    ) -> Result<(), ConstructError> {
        // Clone the receipt for the targeted entity.
        let receipt = entity
            .get::<Receipt<T>>()
            .map(ToOwned::to_owned)
            .unwrap_or_default();

        // Collect the full set of inserted components, along with whether they are explicit or required.
        let mut components = InsertedComponents::with_capacity(usize::max(
            self.component_props.len(),
            receipt.components.len(),
        ));
        for type_id in self.component_props.keys() {
            let Some(id) = entity.world().components().get_id(*type_id) else {
                continue;
            };

            #[allow(unsafe_code)]
            // SAFETY: We know that the id is valid because we just got it from the world.
            let info = unsafe { entity.world().components().get_info_unchecked(id) };

            components.insert_explicit(info);
        }

        // Remove the components in the previous bundle but not this one
        for component_id in receipt.components.iter_removed(&components) {
            entity.remove_by_id(component_id);
        }

        // Insert the new bundle
        let entity_id = entity.id();
        entity.world_scope(|world| {
            for (_, component_props) in self.component_props {
                component_props
                    .construct(&mut ConstructContext::new(entity_id, world))
                    .unwrap();
            }
        });

        // Retain the children
        let anchors = self.children.retain_children(entity, receipt.anchors)?;

        // Place the new receipt onto the entity
        entity.insert(Receipt::<T> {
            components,
            anchors,
            marker: Default::default(),
        });

        Ok(())
    }
}

/// Trait implemented for collections of scenes that can be retained.
pub trait RetainChildren {
    /// Retains the scenes as children of `entity`, returning the new [`Anchor`] map.
    ///
    /// See: [`RetainScene::retain`].
    fn retain_children(
        self,
        entity: &mut EntityWorldMut,
        current_anchors: HashMap<Anchor, Entity>,
    ) -> Result<HashMap<Anchor, Entity>, ConstructError>;
}

impl RetainChildren for Vec<DynamicScene> {
    fn retain_children(
        self,
        entity: &mut EntityWorldMut,
        mut current_anchors: HashMap<Anchor, Entity>,
    ) -> Result<HashMap<Anchor, Entity>, ConstructError> {
        let mut children = Vec::with_capacity(self.len());
        let mut children_ids = Vec::with_capacity(self.len());

        entity.world_scope(|world| {
            // Get or create an entity for each fragment.
            let mut i = 0;
            for child in self {
                // Compute the anchor for this fragment, using it's key if supplied
                // or an auto-incrementing counter if not.
                let anchor = match child.key() {
                    Some(name) => Anchor::Keyed(name.clone()),
                    None => {
                        let anchor = Anchor::Auto(i);
                        i += 1;
                        anchor
                    }
                };

                // Find the existing child entity based on the anchor, or spawn a
                // new one.
                let entity_id = current_anchors
                    .remove(&anchor)
                    .unwrap_or_else(|| world.spawn_empty().id());

                // Store the child, it's anchor, and it's entity id.
                children.push((child, anchor));
                children_ids.push(entity_id);
            }

            // Clear any remaining orphans from the previous template. We do this
            // first (before deparenting) so that hooks still see the parent when
            // they run.
            for orphan_id in current_anchors.into_values() {
                if let Ok(entity) = world.get_entity_mut(orphan_id) {
                    entity.despawn();
                }
            }
        });

        // Position the entities as children, not touching any other children.
        entity.add_children(&children_ids);

        // Build the children and produce the receipts. It's important that this
        // happens *after* the entities are positioned as children to make hooks
        // work correctly.
        entity.world_scope(|world| {
            children
                .into_iter()
                .enumerate()
                .map(|(i, (dynamic_scene, anchor))| {
                    let entity_id = children_ids[i];
                    dynamic_scene.retain::<()>(&mut world.entity_mut(entity_id))?;
                    Ok((anchor, entity_id))
                })
                .collect()
        })
    }
}

/// Retain [`Scene`] extension.
pub trait RetainSceneExt {
    /// Retains the provided scene on the entity.
    ///
    /// See [`RetainScene::retain`].
    fn retain_scene(&mut self, scene: impl Scene) -> Result<(), ConstructError> {
        self.retain_scene_with::<()>(scene)
    }

    /// Retains the provided scene on the entity with `T` as a marker for the `Receipt`, allowing for multiple retained scenes on the same entity.
    ///
    /// See [`RetainScene::retain`].
    fn retain_scene_with<T: Send + Sync + 'static>(
        &mut self,
        scene: impl Scene,
    ) -> Result<(), ConstructError>;

    /// Retains the provided scenes as children of self.
    ///
    /// See [`RetainChildren::retain_children`].
    fn retain_child_scenes<S: Scene>(
        &mut self,
        child_scenes: impl IntoIterator<Item = S>,
    ) -> Result<(), ConstructError> {
        self.retain_child_scenes_with::<S, ()>(child_scenes)
    }

    /// Retains the provided scenes as children of self with `T` as a marker for the `Receipt`, allowing for multiple retained scenes on the same entity.
    ///
    /// See [`RetainChildren::retain_children`].
    fn retain_child_scenes_with<S: Scene, T: Send + Sync + 'static>(
        &mut self,
        child_scenes: impl IntoIterator<Item = S>,
    ) -> Result<(), ConstructError>;
}

impl RetainSceneExt for EntityWorldMut<'_> {
    fn retain_scene_with<T: Send + Sync + 'static>(
        &mut self,
        scene: impl Scene,
    ) -> Result<(), ConstructError> {
        let mut dynamic_scene = DynamicScene::default();
        scene.dynamic_patch(&mut dynamic_scene);
        dynamic_scene.retain::<T>(self)
    }

    fn retain_child_scenes_with<S: Scene, T: Send + Sync + 'static>(
        &mut self,
        child_scenes: impl IntoIterator<Item = S>,
    ) -> Result<(), ConstructError> {
        // Take the receipt from targeted entity.
        let receipt = self.take::<Receipt<T>>().unwrap_or_default();

        // Retain the children
        let anchors = child_scenes
            .into_iter()
            .map(|child_scene| child_scene.into_dynamic_scene())
            .collect::<Vec<_>>()
            .retain_children(self, receipt.anchors)?;

        // Place the receipt back onto the entity
        self.insert(Receipt::<T> {
            components: receipt.components,
            anchors,
            marker: Default::default(),
        });

        Ok(())
    }
}

/// Retain [`Scene`] extension.
pub trait RetainSceneCommandsExt {
    /// Retains the scene on the entity.
    ///
    /// See [`RetainScene::retain`].
    fn retain_scene(&mut self, scene: impl Scene + Send + 'static) {
        self.retain_scene_with::<()>(scene);
    }

    /// Retains the provided scene on the entity with `T` as a marker for the `Receipt`, allowing for multiple retained scenes on the same entity.
    ///
    /// See [`RetainScene::retain`].
    fn retain_scene_with<T: Send + Sync + 'static>(&mut self, scene: impl Scene + Send + 'static);

    /// Retains the provided scenes as children of self.
    ///
    /// See [`RetainChildren::retain_children`].
    fn retain_child_scenes<S: Scene>(
        &mut self,
        child_scenes: impl IntoIterator<Item = S> + Send + 'static,
    ) {
        self.retain_child_scenes_with::<S, ()>(child_scenes);
    }

    /// Retains the provided scenes as children of self.
    ///
    /// See [`RetainChildren::retain_children`].
    fn retain_child_scenes_with<S: Scene, T: Send + Sync + 'static>(
        &mut self,
        child_scenes: impl IntoIterator<Item = S> + Send + 'static,
    );
}

impl RetainSceneCommandsExt for EntityCommands<'_> {
    fn retain_scene_with<T: Send + Sync + 'static>(&mut self, scene: impl Scene + Send + 'static) {
        self.queue(|mut entity: EntityWorldMut| {
            entity.retain_scene_with::<T>(scene).unwrap();
        });
    }

    fn retain_child_scenes_with<S: Scene, T: Send + Sync + 'static>(
        &mut self,
        child_scenes: impl IntoIterator<Item = S> + Send + 'static,
    ) {
        self.queue(|mut entity: EntityWorldMut| {
            entity
                .retain_child_scenes_with::<S, T>(child_scenes)
                .unwrap();
        });
    }
}
