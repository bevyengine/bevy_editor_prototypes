//! [`PaneRegistry`] module.

use derive_more::derive::{Display, Error};

use bevy::{
    ecs::system::{BoxedSystem, SystemId},
    prelude::*,
    utils::HashMap,
};

use crate::{
    pane::{Pane, PanePlugin},
    prelude::PaneStructure,
    ui::pane::PaneNode,
    PaneLayoutSet,
};

pub(crate) struct PaneRegistryPlugin;

impl Plugin for PaneRegistryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PaneRegistry>()
            .add_systems(Update, on_pane_creation.in_set(PaneLayoutSet));
    }
}

/// Returned when Pane registry fails
#[derive(Debug, Error, Display)]
enum PaneRegistryError {
    DuplicatePane,
}

/// A registry of pane types.
#[derive(Resource, Default)]
pub struct PaneRegistry {
    panes: HashMap<String, PaneCreationHandle>,
}

impl PaneRegistry {
    /// Register a new pane type.
    fn register<T: Pane>(&mut self) -> Result<(), PaneRegistryError> {
        let key = T::ID.into();

        if self.panes.contains_key(key) {
            Err(PaneRegistryError::DuplicatePane)
        } else {
            self.panes.insert(
                T::ID.into(),
                PaneCreationHandle::new(Box::new(IntoSystem::into_system(T::on_create))),
            );
            Ok(())
        }
    }
}

/// Contains the Creation system for a Pane.
/// At all times, one (and only one) of the two fields should be None.
struct PaneCreationHandle {
    callback: Option<BoxedSystem<In<PaneStructure>>>,
    callback_id: Option<SystemId<In<PaneStructure>>>,
}

impl PaneCreationHandle {
    fn new(callback: BoxedSystem<In<PaneStructure>>) -> Self {
        Self {
            callback: Some(callback),
            callback_id: None,
        }
    }

    fn id_for(&mut self, world: &mut World) -> SystemId<In<PaneStructure>> {
        if let Some(id) = self.callback_id {
            id
        } else {
            let id = world.register_boxed_system(self.callback.take().unwrap());
            self.callback_id = Some(id);
            id
        }
    }
}

pub(crate) fn on_pane_creation(
    world: &mut World,
    pane_entity_query: &mut QueryState<Entity, Added<PaneNode>>,
    pane_node_query: &mut QueryState<&PaneNode>,
) {
    let new_panes: Vec<_> = pane_entity_query.iter(world).collect();
    for entity in new_panes {
        world.resource_scope(|world, mut pane_registry: Mut<PaneRegistry>| {
            let pane = pane_node_query.get(world, entity).unwrap();
            let creation_handle = pane_registry.panes.get_mut(&pane.id);
            let pane_structure = PaneStructure::new(entity, pane.container, pane.header);

            if let Some(creation_handle) = creation_handle {
                let system_id = creation_handle.id_for(world);
                world
                    .run_system_with_input(system_id, pane_structure)
                    .unwrap();
            } else {
                warn!("No pane found in the registry with id: '{}'", pane.id);
            }
        });
    }
}

/// Extension trait for [`App`].
pub trait PaneAppExt {
    /// Register a new pane type.
    fn register_pane<T: Pane>(&mut self) -> &mut Self;
}

impl PaneAppExt for App {
    fn register_pane<T: Pane>(&mut self) -> &mut Self {
        let result = self
            .world_mut()
            .get_resource_or_init::<PaneRegistry>()
            .register::<T>();

        if matches!(result, Ok(..)) {
            self.add_plugins(PanePlugin::<T>::new());
        }

        self
    }
}
