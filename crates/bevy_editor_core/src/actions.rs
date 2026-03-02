//! Editor actions module.

use bevy::{ecs::system::SystemId, prelude::*};

/// Editor actions plugin.
#[derive(Default)]
pub struct ActionsPlugin;

impl Plugin for ActionsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActionRegistry>();
    }
}

/// The registry for [`Action`]s
#[derive(Resource, Default)]
pub struct ActionRegistry {
    actions: Vec<Action>,
}

impl ActionRegistry {
    /// Register an action.
    pub fn register(
        &mut self,
        id: impl Into<String>,
        label: impl Into<String>,
        system_id: SystemId<(), ()>,
    ) {
        self.actions.push(Action {
            id: id.into(),
            label: label.into(),
            system_id,
        });
    }

    /// Run an action.
    pub fn run(&mut self, world: &mut World, action_id: impl Into<String>) {
        let action_id = action_id.into();
        if let Some(action) = self.actions.iter_mut().find(|ac| ac.id == action_id)
            && let Err(error) = world.run_system(action.system_id)
        {
            error!("Failed to run action '{}': {}", action.id, error);
        }
    }
}

/// Defines some action with an id and a label for display.
pub struct Action {
    id: String,
    #[expect(dead_code)]
    label: String,
    system_id: SystemId,
}

/// [`ActionRegistry`] extension trait for [`App`].
pub trait ActionAppExt {
    /// Register an action.
    fn register_action<M>(
        &mut self,
        id: impl Into<String>,
        label: impl Into<String>,
        system: impl IntoSystem<(), (), M> + 'static,
    ) -> &mut Self;
}

impl ActionAppExt for App {
    fn register_action<M>(
        &mut self,
        id: impl Into<String>,
        label: impl Into<String>,
        system: impl IntoSystem<(), (), M> + 'static,
    ) -> &mut Self {
        let system_id = self.world_mut().register_system(system);
        self.world_mut()
            .get_resource_or_init::<ActionRegistry>()
            .register(id, label, system_id);
        self
    }
}

/// [`ActionRegistry`] extension trait for [`World`].
pub trait ActionWorldExt {
    /// Run an action.
    fn run_action(&mut self, id: impl Into<String>) -> &mut Self;
}

impl ActionWorldExt for World {
    fn run_action(&mut self, id: impl Into<String>) -> &mut Self {
        self.resource_scope(|world, mut registry: Mut<ActionRegistry>| registry.run(world, id));
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_action() {
        #[derive(Resource)]
        struct Counter(u32);
        let mut app = App::new();
        app.insert_resource(Counter(0));
        app.insert_resource(ActionRegistry::default());

        app.register_action("action", "Action", |mut counter: ResMut<Counter>| {
            counter.0 += 1;
        });

        app.world_mut().run_action("action");

        assert_eq!(app.world().resource::<Counter>().0, 1);
    }
}
