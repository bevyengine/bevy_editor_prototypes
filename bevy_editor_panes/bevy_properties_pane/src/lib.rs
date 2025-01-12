//! An interactive, reflection-based inspector for Bevy ECS data in running applications.
//!
//! Data can be viewed and modified in real-time, with changes being reflected in the application.

use bevy::{color::palettes::tailwind, prelude::*};
use bevy_editor_core::SelectedEntity;
use bevy_i_cant_believe_its_not_bsn::{template, Template, TemplateEntityCommandsExt};
use bevy_pane_layout::prelude::{PaneAppExt, PaneStructure};

/// Plugin for the editor properties pane.
pub struct PropertiesPanePlugin;

impl Plugin for PropertiesPanePlugin {
    fn build(&self, app: &mut App) {
        app.register_pane("Properties", setup_pane)
            .add_systems(PostUpdate, update_properties_pane);
    }
}

/// Root UI node of the properties pane.
#[derive(Component)]
struct PropertiesPaneRoot;

fn setup_pane(pane: In<PaneStructure>, mut commands: Commands) {
    commands.entity(pane.content).insert((
        PropertiesPaneRoot,
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

fn update_properties_pane(
    panes: Query<Entity, With<PropertiesPaneRoot>>,
    selected_entity: Res<SelectedEntity>,
    world: &World,
    mut commands: Commands,
) {
    for pane in &panes {
        commands
            .entity(pane)
            .build_children(properties_pane(&selected_entity, world));
    }
}

fn properties_pane(selected_entity: &SelectedEntity, world: &World) -> Template {
    match selected_entity.0 {
        Some(selected_entity) => component_list(selected_entity, world),
        None => template! {(
            Text("Select an entity to inspect".into()),
            TextFont::from_font_size(14.0),
        );},
    }
}

fn component_list(entity: Entity, world: &World) -> Template {
    world
        .inspect_entity(entity)
        .flat_map(|component| {
            let (_, name) = component.name().rsplit_once("::").unwrap();

            template! {(
                Text(name.into()),
                TextFont::from_font_size(12.0),
            );}
        })
        .collect()
}
