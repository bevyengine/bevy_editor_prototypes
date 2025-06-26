//! This module defines the Entity Inspector pane for the Bevy editor.
//! It provides a UI for inspecting and editing properties of entities in the scene.

use bevy::{color::palettes::tailwind, platform::collections::HashMap, prelude::*};
use bevy_editor_core::SelectedEntity;
use bevy_i_cant_believe_its_not_bsn::{Template, TemplateEntityCommandsExt};
use bevy_pane_layout::prelude::{PaneAppExt, PaneStructure};
use bevy_transform::TransformWidget;

use crate::remote::RemoteTransforms;
pub struct EntityInspectorPanesPlugin;
impl Plugin for EntityInspectorPanesPlugin {
    fn build(&self, app: &mut App) {
        app.register_pane("Entity Inspector", on_pane_creation)
            .add_systems(Update, update_components_pane);
    }
}

/// Root UI node of the properties pane.
#[derive(Component)]
struct EntityInspectorRoot;

fn on_pane_creation(pane: In<PaneStructure>, mut commands: Commands) {
    commands.entity(pane.content).insert((
        EntityInspectorRoot,
        Node {
            flex_direction: FlexDirection::Column,
            flex_grow: 1.0,
            column_gap: Val::Px(4.0),
            padding: UiRect::all(Val::Px(8.0)),
            ..Default::default()
        },
        BackgroundColor(tailwind::NEUTRAL_600.into()),
    ));
}

fn update_components_pane(
    selected: Res<SelectedEntity>,
    transforms: Res<RemoteTransforms>,
    panes: Query<Entity, With<EntityInspectorRoot>>,
    _world: &World,
    mut commands: Commands,
) {
    for entity in &panes {
        commands
            .entity(entity)
            .build_children(selected_transform(selected.0, &transforms.0));
    }
}

fn selected_transform(
    selected: Option<Entity>,
    transforms: &HashMap<Entity, Transform>,
) -> Template {
    if let Some(entity) = selected {
        if let Some(transform) = transforms.get(&entity) {
            TransformWidget::draw_widget(transform)
        } else {
            TransformWidget::draw_missing_transform()
        }
    } else {
        TransformWidget::draw_empty_selection()
    }
}
