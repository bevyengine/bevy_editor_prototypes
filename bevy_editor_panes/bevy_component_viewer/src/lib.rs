//! Crate for a Bevy Editor Component Viewer pane. This has yet to be blessed, but I thought it might be nice to specifically view components
//!
//! This crate provides a plugin and UI for viewing and interacting with components in a Bevy application.
use bevy::tasks::futures_lite::future;
use bevy::tasks::Task;
use bevy::{
    color::palettes::tailwind, platform::collections::HashMap, prelude::*, tasks::IoTaskPool,
};
use bevy_editor_core::SelectedEntity;
use bevy_i_cant_believe_its_not_bsn::{template, Template, TemplateEntityCommandsExt};
use bevy_pane_layout::prelude::{PaneAppExt, PaneStructure};
use bevy_remote::{
    builtin_methods::{BrpQuery, BrpQueryFilter, BrpQueryParams, BRP_QUERY_METHOD},
    http::{DEFAULT_ADDR, DEFAULT_PORT},
    BrpRequest,
};
use serde::Deserialize;
use serde_json;
/// Plugin for the editor properties pane.
pub struct ComponentViewerPlugin;

impl Plugin for ComponentViewerPlugin {
    fn build(&self, app: &mut App) {
        app.register_pane("Components", setup_pane)
            .init_resource::<RemoteTransforms>()
            .init_resource::<PollTimer>()
            .init_resource::<TransformTaskHandle>()
            .add_systems(Startup, setup_poll_timer)
            .add_systems(
                Update,
                (
                    poll_remote_transforms,
                    update_remote_transforms,
                    update_components_pane,
                ),
            );
    }
}

/// Root UI node of the properties pane.
#[derive(Component)]
struct ComponentViewerRoot;

/// Data structure representing an internal transform component.
#[derive(Reflect, Component, Debug, Deserialize, Clone)]
pub struct TransformData {
    /// The translation vector of the transform.
    pub translation: [f32; 3],
    /// The rotation quaternion of the transform.
    pub rotation: [f32; 4],
    /// The scale vector of the transform.
    pub scale: [f32; 3],
}

/// Represents an entity's transform data in the remote application.
#[derive(Component, Reflect, Debug, Deserialize)]
struct InternalEntityTransformEntry {
    /// The ID of the entity this transform belongs to.
    pub entity: Entity,
    /// The transform data for this entity.
    pub components: HashMap<String, TransformData>,
}

/// Resource to hold the remote transforms data.
#[derive(Resource, Default)]
pub struct RemoteTransforms(pub HashMap<Entity, TransformData>);

#[derive(Resource, Default)]
struct PollTimer(Timer);

fn setup_poll_timer(mut commands: Commands) {
    commands.insert_resource(PollTimer(Timer::from_seconds(0.1, TimerMode::Repeating)));
}

#[derive(Resource, Default)]
struct TransformTaskHandle(Option<Task<HashMap<Entity, TransformData>>>);

fn poll_remote_transforms(
    time: Res<Time>,
    mut timer: ResMut<PollTimer>,
    mut task_handle: ResMut<TransformTaskHandle>,
) {
    if timer.0.tick(time.delta()).just_finished() && task_handle.0.is_none() {
        let task_pool = IoTaskPool::get();
        let task =
            task_pool.spawn(async move {
                let components =
                    vec!["bevy_transform::components::transform::Transform".to_string()];
                let url = format!("http://{}:{}/", DEFAULT_ADDR, DEFAULT_PORT);
                let req = BrpRequest {
                    jsonrpc: String::from("2.0"),
                    method: String::from(BRP_QUERY_METHOD),
                    id: Some(serde_json::to_value(1).expect("Failed to serialize id")),
                    params: Some(
                        serde_json::to_value(BrpQueryParams {
                            data: BrpQuery {
                                components,
                                option: Vec::default(),
                                has: Vec::default(),
                            },
                            strict: false,
                            filter: BrpQueryFilter {
                                without: Vec::default(),
                                with: vec!["bevy_ecs::name::Name".to_string()],
                            },
                        })
                        .unwrap(),
                    ),
                };

                let mut entity_transform_map: HashMap<Entity, TransformData> = HashMap::new();

                if let Ok(mut response) = ureq::post(&url).send_json(req) {
                    if let Ok(json) = response.body_mut().read_json::<serde_json::Value>() {
                        if let Some(results) = json["result"].as_array() {
                            for entry in results {
                                if let Ok(entry) = serde_json::from_value::<
                                    InternalEntityTransformEntry,
                                >(entry.clone())
                                {
                                    if let Some((_key, transform)) = entry.components.iter().next()
                                    {
                                        entity_transform_map
                                            .insert(entry.entity, transform.clone());
                                    }
                                }
                            }
                        }
                    } else {
                        error!("Failed to parse JSON response");
                    }
                } else {
                    error!("Failed to send request to remote application");
                }
                entity_transform_map
            });
        task_handle.0 = Some(task);
    }
}

fn update_remote_transforms(
    mut task_handle: ResMut<TransformTaskHandle>,
    mut transforms: ResMut<RemoteTransforms>,
) {
    if let Some(mut task) = task_handle.0.take() {
        if let Some(result) = future::block_on(future::poll_once(&mut task)) {
            transforms.0 = result;
        } else {
            // Not ready yet, put it back
            task_handle.0 = Some(task);
        }
    }
}

fn setup_pane(pane: In<PaneStructure>, mut commands: Commands) {
    commands.entity(pane.content).insert((
        ComponentViewerRoot,
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
    panes: Query<Entity, With<ComponentViewerRoot>>,
    _selected_entity: Res<SelectedEntity>,
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
    transforms: &HashMap<Entity, TransformData>,
) -> Template {
    if let Some(entity) = selected {
        if let Some(transform) = transforms.get(&entity) {
            return template! {
                Node {
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(8.0)),
                    ..Default::default()
                } => [
                    (Text(format!("Entity: {:?}", entity)), TextFont::from_font_size(16.0), TextColor(tailwind::YELLOW_100.into()));
                    (Text("Transform:".to_string()), TextFont::from_font_size(14.0), TextColor(tailwind::GREEN_100.into()));
                    (Text(format!("x: {:.2}, y: {:.2}, z: {:.2}", transform.translation[0], transform.translation[1], transform.translation[2])), TextFont::from_font_size(12.0));
                    (Text("Rotation:".to_string()), TextFont::from_font_size(14.0), TextColor(tailwind::GREEN_100.into()));
                    (Text(format!("x: {:.2}, y: {:.2}, z: {:.2}, w: {:.2}", transform.rotation[0], transform.rotation[1], transform.rotation[2], transform.rotation[3])), TextFont::from_font_size(12.0));
                    (Text("Scale:".to_string()), TextFont::from_font_size(14.0), TextColor(tailwind::GREEN_100.into()));
                    (Text(format!("x: {:.2}, y: {:.2}, z: {:.2}", transform.scale[0], transform.scale[1], transform.scale[2])), TextFont::from_font_size(12.0));
                ];
            };
        } else {
            return template! {
                Node {
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(8.0)),
                    ..Default::default()
                } => [
                    (Text("No transform data available for this entity.".into()), TextFont::from_font_size(14.0));
                ];
            };
        }
    } else {
        return template! {
            Node {
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(8.0)),
                ..Default::default()
            } => [
                (Text("Select an entity to view its transform data.".into()), TextFont::from_font_size(14.0));
            ];
        };
    }
}
