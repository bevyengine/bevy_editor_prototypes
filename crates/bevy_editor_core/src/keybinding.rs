//! Editor keybinding module.

use bevy::prelude::*;

use crate::actions::ActionWorldExt;

/// Editor keybinding plugin.
#[derive(Default)]
pub struct KeybindingPlugin;

impl Plugin for KeybindingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Keybindings>()
            .add_systems(Update, process_keybindings);
    }
}

/// The store of keybindings.
#[derive(Resource)]
pub struct Keybindings {
    list: Vec<Keybinding>,
    /// Control whether keybindings are active.
    pub enabled: bool,
}

impl Keybindings {
    /// Add a keybinding to the list.
    pub fn add_keybinding(&mut self, keybinding: Keybinding) {
        self.list.push(keybinding);
    }
}

impl Default for Keybindings {
    fn default() -> Self {
        Self {
            list: Default::default(),
            enabled: true,
        }
    }
}

/// A keybinding for an editor [`Action`](crate::actions::Action).
///
/// # Example
/// This example binds the "load-gltf" action to <kbd>Ctrl</kbd> + <kbd>L</kbd>.
/// ```no_run
/// # use bevy_editor_core::prelude::*;
/// # use bevy::prelude::*;
/// # let mut app = App::new();
/// app.register_keybinding(Keybinding::new(KeyCode::KeyL, "load-gltf").ctrl());
/// ```
#[derive(Clone, Debug, Reflect)]
pub struct Keybinding {
    ctrl: bool,
    shift: bool,
    alt: bool,
    os: bool,
    key: KeyCode,
    action_id: String,
}

impl Keybinding {
    /// Create a new keybind from a key and the id of the action it will be bound to.
    pub fn new(key: KeyCode, action_id: impl Into<String>) -> Self {
        Self {
            ctrl: false,
            shift: false,
            alt: false,
            os: false,
            key,
            action_id: action_id.into(),
        }
    }

    /// Require the <kbd>Ctrl</kbd> or <kbd>Control</kbd> modifier key to be held for this keybind.
    pub fn ctrl(mut self) -> Self {
        self.ctrl = true;
        self
    }

    /// Require the <kbd>Shift</kbd> or <kbd>⇧</kbd> modifier key to be held for this keybind.
    pub fn shift(mut self) -> Self {
        self.shift = true;
        self
    }

    /// Require the <kbd>Alt</kbd> or <kbd>Option</kbd> modifier key to be held for this keybind.
    pub fn alt(mut self) -> Self {
        self.alt = true;
        self
    }

    /// Require the "Windows Logo" key or the <kbd>Command</kbd> or <kbd>⌘</kbd> modifier key to be held for this keybind.
    pub fn os(mut self) -> Self {
        self.os = true;
        self
    }
}

fn process_keybindings(world: &mut World) {
    world.resource_scope(|world, bindings: Mut<Keybindings>| {
        if !bindings.enabled {
            return;
        }
        world.resource_scope(|world, input: Mut<ButtonInput<KeyCode>>| {
            let ctrl = input.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]);
            let shift = input.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
            let alt = input.any_pressed([KeyCode::AltLeft, KeyCode::AltRight]);
            let os = input.any_pressed([KeyCode::SuperLeft, KeyCode::SuperRight]);
            for binding in &bindings.list {
                if (!binding.ctrl || ctrl)
                    && (!binding.alt || alt)
                    && (!binding.shift || shift)
                    && (!binding.os || os)
                    && input.just_pressed(binding.key)
                {
                    world.run_action(&binding.action_id);
                }
            }
        });
    });
}

/// [`Keybindings`] extension trait for [`App`].
pub trait KeybindingAppExt {
    /// Register a keybinding for an action.
    fn register_keybinding(&mut self, binding: Keybinding) -> &mut Self;
}

impl KeybindingAppExt for App {
    fn register_keybinding(&mut self, binding: Keybinding) -> &mut Self {
        self.world_mut()
            .get_resource_or_init::<Keybindings>()
            .add_keybinding(binding);
        self
    }
}
