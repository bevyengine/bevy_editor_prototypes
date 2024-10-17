//! A context menu for the Bevy Editor

mod ui;

use bevy::prelude::*;
use bevy_editor_styles::Theme;

use crate::ui::spawn_context_menu;

/// A context menu for the Bevy Editor
pub struct ContextMenuPlugin;

impl Plugin for ContextMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_secondary_button_down_entity_with_context_menu);
    }
}

fn on_secondary_button_down_entity_with_context_menu(
    trigger: Trigger<Pointer<Down>>,
    mut commands: Commands,
    query: Query<&ContextMenu>,
    theme: Res<Theme>,
) {
    let event = trigger.event();
    if event.button != PointerButton::Secondary {
        return;
    }
    let target = trigger.entity();

    let Ok(menu) = query.get(target) else {
        return;
    };

    // Prevent all other entities from being picked by placing a node over the entire window.
    let root = commands
        .spawn((
            Node::default(),
            Style {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                ..default()
            },
        ))
        .observe(|trigger: Trigger<Pointer<Down>>, mut commands: Commands| {
            commands.entity(trigger.entity()).despawn_recursive();
        })
        .id();

    spawn_context_menu(
        &mut commands,
        &theme,
        menu,
        event.pointer_location.position,
        target,
    )
    .observe(|mut trigger: Trigger<Pointer<Down>>| {
        // Prevent the context menu root from despawning the context menu when clicking on the menu
        trigger.propagate(false);
    })
    .set_parent(root);
}

/// Entities with this component will
#[derive(Component)]
pub struct ContextMenu {
    options: Vec<ContextMenuOption>,
}

impl ContextMenu {
    /// Create a new [`ContextMenu`] from a list of [`ContextMenuOption`]s.
    pub fn new(options: impl IntoIterator<Item = ContextMenuOption>) -> Self {
        let options = options.into_iter().collect();
        ContextMenu { options }
    }
}

/// An option inside a [`ContextMenu`].
pub struct ContextMenuOption {
    label: String,
    f: Box<dyn FnMut(Commands, Entity) + Send + Sync>,
}

impl ContextMenuOption {
    /// Create a new [`ContextMenuOption`].
    pub fn new(
        label: impl Into<String>,
        f: impl FnMut(Commands, Entity) + Send + Sync + 'static,
    ) -> Self {
        ContextMenuOption {
            label: label.into(),
            f: Box::new(f),
        }
    }
}
