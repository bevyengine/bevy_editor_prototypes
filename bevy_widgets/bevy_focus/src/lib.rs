//! This crate contains input focus management for Bevy widgets.
//! Currently only one entity can hold focus at a time.

use bevy::prelude::*;

/// Plugin for input focus logic
pub struct FocusPlugin;

impl Plugin for FocusPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SetFocus>();
        app.add_event::<ClearFocus>();
        app.add_event::<GotFocus>();
        app.add_event::<LostFocus>();

        app.add_observer(set_focus);
        app.add_observer(clear_focus);
        app.add_observer(mouse_click);

        app.add_systems(
            Last,
            clear_focus_after_click.run_if(resource_exists::<NeedClearFocus>),
        );
    }
}

/// Component which indicates that a widget is focused and can receive input events.
#[derive(Component, Debug)]
pub struct Focus;

/// Mark that a widget can receive input events and can be focused
#[derive(Component, Default)]
pub struct Focusable;

/// Event indicating that a widget has received focus
#[derive(Event)]
pub struct GotFocus(pub Option<Pointer<Click>>);

/// Event indicating that a widget has lost focus
#[derive(Event)]
pub struct LostFocus;

/// Set focus to a widget
#[derive(Event)]
pub struct SetFocus;

/// Clear focus from widgets
#[derive(Event)]
pub struct ClearFocus;

/// Extension trait for [`Commands`]
/// Contains commands to set and clear input focus
pub trait FocusExt {
    /// Set input focus to the given targets
    fn set_focus(&mut self, target: Entity);

    /// Clear input focus
    fn clear_focus(&mut self);
}

impl FocusExt for Commands<'_, '_> {
    fn set_focus(&mut self, target: Entity) {
        self.trigger_targets(SetFocus, target);
    }

    fn clear_focus(&mut self) {
        self.trigger(ClearFocus);
    }
}

#[derive(Resource)]
struct NeedClearFocus(bool);

fn set_focus(
    trigger: Trigger<SetFocus>,
    mut commands: Commands,
    q_focused: Query<Entity, With<Focus>>,
) {
    for entity in q_focused.iter() {
        if entity == trigger.entity() {
            continue;
        }
        commands.entity(entity).remove::<Focus>();
        commands.trigger_targets(LostFocus, entity);
    }
    commands.entity(trigger.entity()).insert(Focus);
    commands.trigger_targets(GotFocus(None), trigger.entity());
}

fn clear_focus(
    _: Trigger<ClearFocus>,
    mut commands: Commands,
    q_focused: Query<Entity, With<Focus>>,
) {
    for entity in q_focused.iter() {
        commands.entity(entity).insert(Focus);
        commands.trigger_targets(LostFocus, entity);
    }
}

fn mouse_click(
    mut click: Trigger<Pointer<Click>>,
    mut commands: Commands,
    q_focusable: Query<Entity, With<Focusable>>,
    q_focused: Query<Entity, With<Focus>>,
) {
    if click.event().button != PointerButton::Primary {
        return;
    }
    let entity = click.entity();
    if q_focusable.contains(entity) {
        commands.insert_resource(NeedClearFocus(false));

        click.propagate(false);
        for e in q_focused.iter() {
            if e == entity {
                continue;
            }
            commands.entity(e).remove::<Focus>();
            commands.trigger_targets(LostFocus, e);
        }
        commands.entity(entity).insert(Focus);
        commands.trigger_targets(GotFocus(Some(click.event().clone())), entity);
    } else {
        commands.insert_resource(NeedClearFocus(true));
    }
}

fn clear_focus_after_click(mut commands: Commands, need_clear_focus: Res<NeedClearFocus>) {
    if need_clear_focus.0 {
        commands.clear_focus();
        commands.remove_resource::<NeedClearFocus>();
    }
}
