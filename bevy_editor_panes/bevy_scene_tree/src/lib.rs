//! An interactive, collapsible tree view for hierarchical ECS data in Bevy.

use bevy::{
    app::{Plugin, Update},
    core::Name,
    ecs::{
        component::{ComponentHooks, StorageType},
        entity::Entities,
    },
    prelude::*,
};
use bevy_i_cant_believe_its_not_bsn::WithChild;
use bevy_pane_layout::{PaneContentNode, PaneRegistry};

/// Plugin for the editor scene tree pane.
pub struct SceneTreePlugin;

impl Plugin for SceneTreePlugin {
    fn build(&self, app: &mut App) {
        let mut pane_registry = app.world_mut().resource_mut::<PaneRegistry>();
        pane_registry.register("Scene Tree", |mut commands, pane_root| {
            commands.entity(pane_root).insert(SceneTreeRoot);
        });

        app.init_resource::<SelectedEntity>()
            .add_systems(Update, update_scene_tree);
    }
}

#[derive(Component)]
struct SceneTreeRoot;

#[derive(Resource, Default)]
struct SelectedEntity(Option<Entity>);

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

fn update_scene_tree(
    mut commands: Commands,
    scene_tree: Option<Single<Entity, With<SceneTreeRoot>>>,
    children: Query<&Children>,
    content: Query<&PaneContentNode>,
    scene: Query<(Entity, &Name)>,
    has_scene_tree_row: Query<&HasSceneTreeRow>,
    mut selected_entity: ResMut<SelectedEntity>,
    entities: &Entities,
    mut init: Local<bool>,
) {
    // Unselect entity if entity was deleted
    if let Some(e) = selected_entity.0 {
        if !entities.contains(e) {
            selected_entity.0 = None;
        }
    }

    // Get scene tree node
    let Some(scene_tree) = scene_tree else {
        return;
    };
    let tree_node = children
        .iter_descendants(*scene_tree)
        .find(|e| content.contains(*e))
        .unwrap();

    // Setup tree on first run
    if !*init {
        commands
            .entity(tree_node)
            .insert((
                Node {
                    flex_direction: FlexDirection::Column,
                    flex_grow: 1.0,
                    column_gap: Val::Px(2.0),
                    padding: UiRect::all(Val::Px(8.0)),
                    ..Default::default()
                },
                BackgroundColor(Srgba::hex("#181818").unwrap().into()),
            ))
            .observe(
                |mut trigger: Trigger<Pointer<Click>>,
                 mut selected_entity: ResMut<SelectedEntity>| {
                    selected_entity.0 = None;
                    trigger.propagate(false);
                },
            );
        *init = true;
    }

    // Create/update rows for new/changed scene entities
    for (scene_entity, scene_entity_name) in &scene {
        let row_widget = entity_widget(scene_entity_name, selected_entity.0 == Some(scene_entity));

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

        if let Ok(HasSceneTreeRow(row_entity)) = has_scene_tree_row.get(scene_entity) {
            // Update existing row (TODO: Skip if name/is_selected is unchanged)
            commands
                .entity(*row_entity)
                .despawn_descendants()
                .insert(row_widget);
        } else {
            // Create new row
            let row_entity = commands
                .spawn(row_widget)
                .set_parent(tree_node)
                .observe(set_selected_entity_on_click)
                .id();

            commands
                .entity(scene_entity)
                .insert(HasSceneTreeRow(row_entity));
        }
    }
}

fn entity_widget(entity_name: &Name, is_selected: bool) -> impl Bundle {
    (
        Node {
            padding: UiRect::all(Val::Px(4.0)),
            ..Default::default()
        },
        BorderRadius::all(Val::Px(4.0)),
        if is_selected {
            BackgroundColor(Srgba::hex("#252525").unwrap().into())
        } else {
            BackgroundColor::default()
        },
        WithChild((
            Text(entity_name.into()),
            TextFont {
                font_size: 11.0,
                ..Default::default()
            },
            PickingBehavior::IGNORE,
        )),
    )
}
