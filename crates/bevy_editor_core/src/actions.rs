//! Editor actions module.

use bevy::{ecs::system::SystemId, prelude::*};

/// Editor selection plugin.
#[derive(Default)]
pub struct ActionsPlugin;

impl Plugin for ActionsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActionRegistry>()
            .init_resource::<ActionBindings>()
            .add_systems(Update, run_actions_on_binding);
    }
}

fn run_actions_on_binding(world: &mut World) {
    world.resource_scope(|world, mut bindings: Mut<ActionBindings>| {
        world.resource_scope(|world, input: Mut<ButtonInput<KeyCode>>| {
            // Sort by the invserse amount of keys in the binding so that simpler keybinds don't prevent more complex ones from triggering.
            bindings.list.sort_by_key(|(v, _)| usize::MAX - v.len());
            'outer: for (binding, action_id) in &bindings.list {
                if let Some(last_key) = binding.last() {
                    for key in &binding[..(binding.len() - 1)] {
                        if !input.pressed(*key) {
                            continue 'outer;
                        }
                    }
                    if input.just_pressed(*last_key) {
                        world.run_action(action_id);
                        break 'outer;
                    }
                }
            }
        });
    });
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

/// List of keybindings to [`Action`]s
#[derive(Resource, Default)]
pub struct ActionBindings {
    list: Vec<(Vec<KeyCode>, String)>,
}

impl ActionBindings {
    /// Add a binding for an action.
    pub fn add_binding(
        &mut self,
        action_id: impl Into<String>,
        binding: impl IntoIterator<Item = KeyCode>,
    ) {
        self.list
            .push((binding.into_iter().collect(), action_id.into()));
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

    /// Register an action binding.
    fn register_action_binding(
        &mut self,
        action_id: impl Into<String>,
        binding: impl IntoIterator<Item = KeyCode>,
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

    fn register_action_binding(
        &mut self,
        action_id: impl Into<String>,
        binding: impl IntoIterator<Item = KeyCode>,
    ) -> &mut Self {
        self.world_mut()
            .get_resource_or_init::<ActionBindings>()
            .add_binding(action_id, binding);
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
