//! An interactive, collapsible tree view for hierarchical ECS data in Bevy.

use bevy::{
    app::Plugin,
    color::palettes::tailwind,
    core::Name,
    ecs::{
        component::{ComponentHooks, StorageType},
        entity::Entities,
    },
    picking::focus::PickingInteraction,
    prelude::*,
};
use bevy_i_cant_believe_its_not_bsn::WithChild;
use bevy_pane_layout::prelude::{PaneAppExt, PaneStructure};
use std::ops::Deref;

/// Plugin for the editor scene tree pane.
pub struct SceneTreePlugin;

impl Plugin for SceneTreePlugin {
    fn build(&self, app: &mut App) {
        app.register_pane("Scene Tree", setup_pane);

        app.init_resource::<SelectedEntity>().add_systems(
            PostUpdate,
            (
                reset_selected_entity_if_entity_despawned,
                spawn_new_scene_tree_rows,
                update_scene_tree_rows,
            )
                .chain(),
        );
    }
}

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
            |mut trigger: Trigger<Pointer<Click>>, mut selected_entity: ResMut<SelectedEntity>| {
                selected_entity.0 = None;
                trigger.propagate(false);
            },
        );
}

/// The currently selected entity in the scene.
// TODO: Move to a different crate so that it can be controlled by things like mesh picking
// and accessed by the inspector without depending on this crate.
#[derive(Resource, Default)]
pub struct SelectedEntity(Option<Entity>);

fn reset_selected_entity_if_entity_despawned(
    mut selected_entity: ResMut<SelectedEntity>,
    entities: &Entities,
) {
    if let Some(e) = selected_entity.0 {
        if !entities.contains(e) {
            selected_entity.0 = None;
        }
    }
}

fn spawn_new_scene_tree_rows(
    mut commands: Commands,
    scene_tree: Option<Single<Entity, With<SceneTreeRoot>>>,
    query: Query<(Entity, &Name), Without<HasSceneTreeRow>>,
) {
    // Get scene tree node
    let Some(scene_tree) = scene_tree.as_deref().copied() else {
        return;
    };

    // Create new rows for named entities without any
    for (scene_entity, name) in &query {
        let set_selected_entity_on_click =
            move |mut trigger: Trigger<Pointer<Click>>,
                  mut selected_entity: ResMut<SelectedEntity>| {
                if selected_entity.0 == Some(scene_entity) {
                    selected_entity.0 = None;
                } else {
                    selected_entity.0 = Some(scene_entity);
                }
                trigger.propagate(false);
            };

        let row_entity = commands
            .spawn((
                SceneTreeRow(scene_entity),
                Node {
                    padding: UiRect::all(Val::Px(4.0)),
                    align_items: AlignItems::Center,
                    ..Default::default()
                },
                BorderRadius::all(Val::Px(4.0)),
                WithChild((
                    Text(name.into()),
                    TextFont {
                        font_size: 11.0,
                        ..Default::default()
                    },
                    PickingBehavior::IGNORE,
                )),
            ))
            .set_parent(scene_tree)
            .observe(set_selected_entity_on_click)
            .id();

        commands
            .entity(scene_entity)
            .insert(HasSceneTreeRow(row_entity));
    }
}

fn update_scene_tree_rows(
    scene_query: Query<(Ref<Name>, &HasSceneTreeRow)>,
    mut row_query: Query<(&SceneTreeRow, &mut BackgroundColor, Ref<PickingInteraction>)>,
    children_query: Query<&Children>,
    mut text_query: Query<&mut Text>,
    selected_entity: Res<SelectedEntity>,
) {
    // Update row names if entity name changes
    for (name, scene_tree_row) in &scene_query {
        if name.is_changed() {
            text_query
                .get_mut(children_query.children(scene_tree_row.0)[0])
                .unwrap()
                .0 = name.deref().into();
        }
    }

    // Update row color based on interaction state
    for (scene_tree_row, mut background_color, picking_interaction) in &mut row_query {
        let state = (
            picking_interaction.deref(),
            selected_entity.0 == Some(scene_tree_row.0),
        );
        background_color.0 = match state {
            (_, true) => tailwind::NEUTRAL_700.into(),
            (PickingInteraction::Hovered, _) => tailwind::NEUTRAL_500.into(),
            _ => Color::NONE,
        };
    }
}

/// Root UI node of the scene tree.
#[derive(Component)]
struct SceneTreeRoot;

/// A scene tree row UI node.
///
/// Contains the row's corresponding scene entity.
#[derive(Component)]
struct SceneTreeRow(Entity);

/// A component on an entity in the scene that holds the corresponding scene tree row UI node entity.
///
/// If the entity with this component is despawned, the corresponding scene tree row UI node entity
/// will be automatically desapwned.
struct HasSceneTreeRow(Entity);

impl Component for HasSceneTreeRow {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks.on_remove(|mut world, entity, _| {
            let row_entity = world.entity(entity).get::<Self>().unwrap().0;
            world.commands().entity(row_entity).despawn_recursive();
        });
    }
}
