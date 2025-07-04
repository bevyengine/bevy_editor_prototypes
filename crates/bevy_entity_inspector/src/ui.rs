//! This module defines the Entity Inspector pane for the Bevy editor.
//! It provides a UI for inspecting and editing properties of entities in the scene.

use std::any::type_name;

use bevy::{color::palettes::tailwind, platform::collections::HashMap, prelude::*};
use bevy_i_cant_believe_its_not_bsn::{Template, TemplateEntityCommandsExt, template};
use bevy_pane_layout::prelude::{PaneAppExt, PaneStructure};
use bevy_transform::TransformWidget;

use crate::{EntityInspectorRow, EntityInspectorRows};

pub struct EntityInspectorUiPlugin;
impl Plugin for EntityInspectorUiPlugin {
    fn build(&self, app: &mut App) {
        app.register_pane("Entity Inspector", on_pane_creation)
            .add_systems(Update, update_components_pane);
    }
}

/// Root UI node of the properties pane.
#[derive(Component)]
struct EntityInspectorRoot;

fn on_pane_creation(pane: In<PaneStructure>, mut commands: Commands) {
    info!("Creating Entity Inspector pane with structure");
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
    rows: Res<EntityInspectorRows>,
    panes: Query<Entity, With<EntityInspectorRoot>>,
    world: &World,
    mut commands: Commands,
) {
    let registry = world.get_resource::<AppTypeRegistry>().unwrap();

    for entity in &panes {
        // info!("Updating Entity Inspector panes with rows {:?}", rows);
        commands
            .entity(entity)
            .build_children(inspector_ui(&rows.rows, registry));
    }
}

fn inspector_ui(
    rows: &HashMap<Entity, EntityInspectorRow>,
    registry: &AppTypeRegistry,
) -> Template {
    // info!("Building inspector UI for {} rows", rows.len());
    rows.iter()
        .flat_map(|(entity, row)| {
            template! {
                Node {
                    flex_direction: FlexDirection::Column,
                    flex_grow: 1.0,
                    padding: UiRect::all(Val::Px(8.0)),
                    column_gap: Val::Px(4.0),
                    ..Default::default()
                } => [
                    (Text(format!("Entity: {} ({:?})", entity, row.name)), TextFont::from_font_size(14.0));
                    ..transform_widget(&row);
                ];
            }
        })
        .collect()
}

fn transform_widget(row: &EntityInspectorRow) -> Template {
    let transform = row
        .components
        .get(type_name::<Transform>())
        .and_then(|boxed| Transform::from_reflect(boxed.as_partial_reflect()))
        .expect("Expected to deserialize a Transform object");
    TransformWidget::draw_widget(&transform)
}
