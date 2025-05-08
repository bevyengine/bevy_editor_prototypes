//! [`PaneRegistry`] module.

use bevy::{
    ecs::system::{BoxedSystem, SystemId},
    platform::collections::HashMap,
    prelude::*,
};

use crate::{PaneLayoutSet, PaneRootNode};

pub(crate) struct PaneRegistryPlugin;

impl Plugin for PaneRegistryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PaneRegistry>()
            .add_systems(Update, on_pane_creation.in_set(PaneLayoutSet));
    }
}

/// A registry of pane types.
#[derive(Resource, Default)]
pub struct PaneRegistry {
    panes: Vec<Pane>,
}

/// The node structure of a pane.
#[derive(Component, Clone, Copy)]
pub struct PaneStructure {
    /// The root of the pane.
    pub root: Entity,
    /// The area node. Child of the root node.
    pub area: Entity,
    /// The header node. Child of the area node.
    pub header: Entity,
    /// The content node. Child of the area node.
    pub content: Entity,
}

impl PaneRegistry {
    /// Register a new pane type.
    pub fn register<M>(
        &mut self,
        name: impl Into<String>,
        system: impl IntoSystem<In<PaneStructure>, (), M>,
    ) {
        self.panes.push(Pane {
            name: name.into(),
            creation_callback: Some(Box::new(IntoSystem::into_system(system))),
        });
    }
}

struct Pane {
    name: String,
    creation_callback: Option<BoxedSystem<In<PaneStructure>>>,
}

pub(crate) fn on_pane_creation(
    world: &mut World,
    roots_query: &mut QueryState<Entity, Added<PaneRootNode>>,
    pane_root_node_query: &mut QueryState<(&PaneRootNode, &PaneStructure)>,
    mut system_ids: Local<HashMap<String, SystemId<In<PaneStructure>>>>,
) {
    let roots: Vec<_> = roots_query.iter(world).collect();
    for entity in roots {
        world.resource_scope(|world, mut pane_registry: Mut<PaneRegistry>| {
            let (pane_root, &structure) = pane_root_node_query.get(world, entity).unwrap();
            let pane = pane_registry
                .panes
                .iter_mut()
                .find(|pane| pane.name == pane_root.name);

            if let Some(pane) = pane {
                let id = system_ids.entry(pane.name.clone()).or_insert_with(|| {
                    world.register_boxed_system(pane.creation_callback.take().unwrap())
                });

                world.run_system_with(*id, structure).unwrap();
            } else {
                warn!(
                    "No pane found in the registry with name: '{}'",
                    pane_root.name
                );
            }
        });
    }
}

/// Extension trait for [`App`].
pub trait PaneAppExt {
    /// Register a new pane type.
    fn register_pane<M>(
        &mut self,
        name: impl Into<String>,
        system: impl IntoSystem<In<PaneStructure>, (), M>,
    ) -> &mut Self;
}

impl PaneAppExt for App {
    fn register_pane<M>(
        &mut self,
        name: impl Into<String>,
        system: impl IntoSystem<In<PaneStructure>, (), M>,
    ) -> &mut Self {
        self.world_mut()
            .get_resource_or_init::<PaneRegistry>()
            .register(name, system);

        self
    }
}
