//! Editor core utils.

use std::time::Duration;

use bevy::{
    picking::backend::HitData,
    platform::{collections::HashMap, time::Instant},
    prelude::*,
    scene2::{PatchContext, ResolvedScene, Scene},
};

/// Editor core utils plugin.
#[derive(Default)]
pub struct CoreUtilsPlugin;

impl Plugin for CoreUtilsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DragCancelClickState>()
            .add_message::<Pointer<DragCancelClick>>()
            .register_type::<Pointer<DragCancelClick>>()
            .add_observer(on_press)
            .add_observer(on_drag_start)
            .add_observer(on_release);
    }
}

fn on_press(trigger: On<Pointer<Press>>, mut state: ResMut<DragCancelClickState>) {
    state
        .0
        .insert(trigger.event().event_target(), Instant::now());
}

fn on_drag_start(trigger: On<Pointer<DragStart>>, mut state: ResMut<DragCancelClickState>) {
    state.0.remove(&trigger.event().event_target());
}

fn on_release(
    trigger: On<Pointer<Release>>,
    mut state: ResMut<DragCancelClickState>,
    mut commands: Commands,
) {
    let now = Instant::now();
    if let Some(instant) = state.remove(&trigger.event().event_target()) {
        let event = Pointer::new(
            trigger.pointer_id,
            trigger.pointer_location.clone(),
            DragCancelClick {
                button: trigger.button,
                hit: trigger.hit.clone(),
                duration: now - instant,
            },
            trigger.event().event_target(),
        );
        commands.trigger(event.clone());
        commands.write_message(event);
    }
}

/// Fires when a pointer sends a pointer pressed event followed by a pointer released event, with the same
/// `target` entity for both events and without a drag start event in between.
#[derive(Clone, PartialEq, Debug, Reflect)]
#[reflect(Clone, PartialEq)]
pub struct DragCancelClick {
    /// Pointer button pressed and lifted to trigger this event.
    pub button: PointerButton,
    /// Information about the picking intersection.
    pub hit: HitData,
    /// Duration between the pointer pressed and lifted for this click
    pub duration: Duration,
}

#[derive(Resource, Deref, DerefMut, Default)]
struct DragCancelClickState(HashMap<Entity, Instant>);

/// A boxed [`Scene`]. Useful when you might need to pass or store scenes of different types.
pub struct BoxedScene(pub Box<dyn Scene>);

impl BoxedScene {
    /// Create a new boxed scene.
    pub fn new(scene: impl Scene) -> Self {
        Self(Box::new(scene))
    }
}

impl Scene for BoxedScene {
    fn patch(&self, context: &mut PatchContext, scene: &mut ResolvedScene) {
        self.0.patch(context, scene);
    }
}

/// Convenience trait for boxing scenes.
pub trait IntoBoxedScene: Scene {
    /// Box this scene.
    fn boxed_scene(self) -> BoxedScene;
}

impl<T> IntoBoxedScene for T
where
    T: Scene,
{
    fn boxed_scene(self) -> BoxedScene {
        BoxedScene::new(self)
    }
}
