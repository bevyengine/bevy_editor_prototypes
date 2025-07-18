//! An interactive, collapsible tree view for hierarchical ECS data in Bevy.

use bevy::{app::Plugin, color::palettes::tailwind, prelude::*};
use bevy_editor_core::SelectedEntity;
use bevy_i_cant_believe_its_not_bsn::{Template, TemplateEntityCommandsExt, on, template};
use bevy_pane_layout::prelude::{PaneAppExt, PaneStructure};

/// Plugin for the editor scene tree pane.
pub struct SceneTreePlugin;

impl Plugin for SceneTreePlugin {
    fn build(&self, app: &mut App) {
        app.register_pane("Scene Tree", setup_pane)
            .add_systems(PostUpdate, update_scene_tree);
    }
}

/// Root UI node of the scene tree.
#[derive(Component)]
struct SceneTreeRoot;

fn setup_pane(pane: In<PaneStructure>, mut commands: Commands) {
    commands
        .entity(pane.content)
        .insert((
            SceneTreeRoot,
            Node {
                flex_direction: FlexDirection::Column,
                flex_grow: 1.0,
                column_gap: Val::Px(2.0),
                padding: UiRect::all(Val::Px(8.0)),
                ..Default::default()
            },
            BackgroundColor(tailwind::NEUTRAL_600.into()),
        ))
        .observe(
            |mut trigger: On<Pointer<Click>>, mut selected_entity: ResMut<SelectedEntity>| {
                selected_entity.0 = None;
                trigger.propagate(false);
            },
        );
}

fn update_scene_tree(
    scene_trees: Query<Entity, With<SceneTreeRoot>>,
    scene_entities: Query<(Entity, &Name)>,
    selected_entity: Res<SelectedEntity>,
    mut commands: Commands,
) {
    for scene_tree in &scene_trees {
        let tree_rows: Template = scene_entities
            .iter()
            .flat_map(|(entity, name)| scene_tree_row_for_entity(entity, name, &selected_entity))
            .collect();

        commands.entity(scene_tree).build_children(tree_rows);
    }
}

fn scene_tree_row_for_entity(
    entity: Entity,
    name: &Name,
    selected_entity: &SelectedEntity,
) -> Template {
    let set_selected_entity_on_click =
        move |mut trigger: On<Pointer<Click>>, mut selected_entity: ResMut<SelectedEntity>| {
            if selected_entity.0 == Some(entity) {
                selected_entity.0 = None;
            } else {
                selected_entity.0 = Some(entity);
            }
            trigger.propagate(false);
        };

    template! {
        {entity}: (
            Node {
                padding: UiRect::all(Val::Px(4.0)),
                align_items: AlignItems::Center,
                ..Default::default()
            },
            BorderRadius::all(Val::Px(4.0)),
            BackgroundColor(if selected_entity.0 == Some(entity) { tailwind::NEUTRAL_700.into() } else { Color::NONE }),
        ) => [
            on(set_selected_entity_on_click);
            (
                Text(name.into()),
                TextFont::from_font_size(11.0),
                Pickable::IGNORE,
            );
        ];
    }
}
