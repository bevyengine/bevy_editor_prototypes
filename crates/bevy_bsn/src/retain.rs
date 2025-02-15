use core::hash::Hash;
use std::fmt::Display;

use bevy::{
    ecs::component::ComponentId,
    platform_support::{
        collections::{HashMap, HashSet},
        hash::FixedHasher,
    },
    prelude::{Children, Component, Entity, EntityCommands, EntityWorldMut},
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
#[derive(Eq, PartialEq, Hash, Clone)]
pub struct Key(String);

impl<T: Display> From<T> for Key {
    fn from(s: T) -> Self {
        Self(s.to_string())
    }
}

/// Receipts allow retaining of scenes that can be intelligently updated.
#[derive(Default, Component, Clone)]
pub struct Receipt {
    /// The components it inserted.
    components: HashSet<ComponentId>,
    /// The anchors of all the children it spawned/retained.
    anchors: HashMap<Anchor, Entity>,
}

/// Trait implemented for scenes that can be retained.
pub trait RetainScene {
    /// Retains the scene by constructing and inserting the components on the entity,
    ///  removing components that should be removed, and spawning/updating children.
    ///
    /// Maintains [`Receipt`]s to allow for intelligent updates.
    fn retain(self, entity: &mut EntityWorldMut) -> Result<(), ConstructError>;
}

impl RetainScene for DynamicScene {
    fn retain(self, entity: &mut EntityWorldMut) -> Result<(), ConstructError> {
        // Clone the receipt for the targeted entity.
        let receipt = entity
            .get::<Receipt>()
            .map(ToOwned::to_owned)
            .unwrap_or_default();

        let entity_id = entity.id();
        entity.world_scope(|world| {
            // Construct and insert the components
            let mut components =
                HashSet::with_capacity_and_hasher(self.component_props.len(), FixedHasher);
            for (type_id, component_props) in self.component_props {
                component_props
                    .construct(&mut ConstructContext::new(entity_id, world))
                    .unwrap();
                components.insert(world.components().get_id(type_id).unwrap());
            }

            // Remove the components in the previous bundle but not this one
            let mut entity = world.entity_mut(entity_id);
            for component_id in receipt.components.difference(&components) {
                entity.remove_by_id(*component_id);
            }

            // Retain the children
            let anchors = self
                .children
                .retain_children(&mut entity, receipt.anchors)?;

            // Place the new receipt onto the entity
            entity.insert(Receipt {
                components,
                anchors,
            });

            Ok(())
        })
    }
}

/// Trait implemented for collections of scenes that can be retained.
pub trait RetainChildren {
    /// Retains the scenes as children of `entity`, updating the [`Receipt`] in the process.
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
        let children = entity.world_scope(|world| {
            // Get or create an entity for each fragment.
            let mut i = 0;
            let children: Vec<_> = self
                .into_iter()
                .map(|child| {
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
                    (child, anchor, entity_id)
                })
                .collect();

            // Clear any remaining orphans from the previous template. We do this
            // first (before deparenting) so that hooks still see the parent when
            // they run.
            for orphan_id in current_anchors.into_values() {
                world.entity_mut(orphan_id).despawn();
            }

            children
        });

        // Position the entities as children.
        let child_entities: Vec<_> = children.iter().map(|(_, _, entity)| *entity).collect();
        entity.remove::<Children>();
        entity.add_children(&child_entities);

        // Build the children and produce the receipts. It's important that this
        // happens *after* the entities are positioned as children to make hooks
        // work correctly.
        entity.world_scope(|world| {
            children
                .into_iter()
                .map(|(dynamic_scene, anchor, entity_id)| {
                    dynamic_scene.retain(&mut world.entity_mut(entity_id))?;
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
    fn retain_scene(&mut self, scene: impl Scene) -> Result<(), ConstructError>;

    /// Retains the provided scenes as children of self.
    ///
    /// See [`RetainChildren::retain_children`].
    fn retain_child_scenes<T: Scene>(
        &mut self,
        child_scenes: impl IntoIterator<Item = T>,
    ) -> Result<(), ConstructError>;
}

impl RetainSceneExt for EntityWorldMut<'_> {
    fn retain_scene(&mut self, scene: impl Scene) -> Result<(), ConstructError> {
        let mut dynamic_scene = DynamicScene::default();
        scene.dynamic_patch(&mut dynamic_scene);
        dynamic_scene.retain(self)
    }

    fn retain_child_scenes<T: Scene>(
        &mut self,
        child_scenes: impl IntoIterator<Item = T>,
    ) -> Result<(), ConstructError> {
        // Take the receipt from targeted entity.
        let receipt = self.take::<Receipt>().unwrap_or_default();

        // Retain the children
        let anchors = child_scenes
            .into_iter()
            .map(DynamicPatch::into_dynamic_scene)
            .collect::<Vec<_>>()
            .retain_children(self, receipt.anchors)?;

        // Place the receipt back onto the entity
        self.insert(Receipt {
            components: receipt.components,
            anchors,
        });

        Ok(())
    }
}

/// Retain [`Scene`] extension.
pub trait RetainSceneCommandsExt {
    /// Retains the scene on the entity.
    ///
    /// See [`RetainScene::retain`].
    fn retain_scene(&mut self, scene: impl Scene + Send + 'static);

    /// Retains the provided scenes as children of self.
    ///
    /// See [`RetainChildren::retain_children`].
    fn retain_child_scenes<T: Scene>(
        &mut self,
        child_scenes: impl IntoIterator<Item = T> + Send + 'static,
    );
}

impl RetainSceneCommandsExt for EntityCommands<'_> {
    fn retain_scene(&mut self, scene: impl Scene + Send + 'static) {
        self.queue(|mut entity: EntityWorldMut| {
            entity.retain_scene(scene).unwrap();
        });
    }

    fn retain_child_scenes<T: Scene>(
        &mut self,
        child_scenes: impl IntoIterator<Item = T> + Send + 'static,
    ) {
        self.queue(|mut entity: EntityWorldMut| {
            entity.retain_child_scenes(child_scenes).unwrap();
        });
    }
}
