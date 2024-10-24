//! Entity patch is a patch that is used to update the entity tree

use std::sync::Arc;

use bevy::prelude::*;

use crate::{construct::Construct, patch::Patch};

#[derive(Default)]
pub struct EntityPatch {
    pub patch: Vec<Box<dyn EntityComponentPatch>>,
    pub children: Vec<EntityPatch>,
}

impl EntityPatch {
    pub fn new() -> Self {
        Self {
            patch: Vec::new(),
            children: Vec::new(),
        }
    }

    pub fn with_patch(mut self, patch: impl EntityComponentPatch) -> Self {
        self.patch.push(Box::new(patch));
        self
    }

    pub fn with_child(mut self, child: EntityPatch) -> Self {
        self.children.push(child);
        self
    }

    pub fn apply(&mut self, entity: Entity, world: &mut World) {
        {
            let mut entity_mut = world.entity_mut(entity);
            for patch in self.patch.iter_mut() {
                patch.entity_patch(&mut entity_mut);
            }
        }

        let mut children_entities = Vec::new();
        if let Some(children) = world.entity(entity).get::<Children>() {
            children_entities = children.iter().map(|e| *e).collect();
        }

        while children_entities.len() < self.children.len() {
            let child_entity = world.spawn_empty().id();
            world.entity_mut(entity).add_child(child_entity);
            children_entities.push(child_entity);
        }

        for (i, child) in self.children.iter_mut().enumerate() {
            child.apply(children_entities[i], world);
        }
    }
}

pub trait EntityComponentPatch: Send + Sync + 'static {
    fn entity_patch(&mut self, entity_mut: &mut EntityWorldMut);
}

impl<C: Component + Default + Clone, T: Patch<Construct = C>> EntityComponentPatch for T {
    fn entity_patch(&mut self, entity_mut: &mut EntityWorldMut) {
        let mut component = entity_mut.get_mut::<C>().unwrap();
        self.patch(&mut component);
    }
}
