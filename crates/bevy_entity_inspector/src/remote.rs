use std::any::type_name;

use bevy::{
    platform::collections::HashMap,
    prelude::*,
    tasks::{IoTaskPool, Task, futures_lite::future},
};
use bevy_remote::{
    BrpRequest,
    builtin_methods::{BRP_QUERY_METHOD, BrpQuery, BrpQueryFilter, BrpQueryParams},
    http::{DEFAULT_ADDR, DEFAULT_PORT},
};

use crate::{EntityInspectorRow, EntityInspectorRows};
use serde::de::IntoDeserializer;

/// Plugin for the entity inspector remote functionality.
/// This plugin allows the entity inspector to connect to a remote Bevy application
pub struct EntityInspectorRemotePlugin {
    /// The address of the remote application to connect to.
    pub remote_url: String,
}

impl Default for EntityInspectorRemotePlugin {
    fn default() -> Self {
        EntityInspectorRemotePlugin {
            remote_url: format!("http://{}:{}/", DEFAULT_ADDR, DEFAULT_PORT),
        }
    }
}

#[derive(Resource, Debug, Clone)]
pub struct RemoteConfig {
    /// The URL of the remote application.
    pub remote_url: String,
}

impl Plugin for EntityInspectorRemotePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(RemoteConfig {
            remote_url: self.remote_url.clone(),
        })
        .init_resource::<PollTimer>()
        .init_resource::<RowTaskHandle>()
        .add_systems(Startup, setup_poll_timer)
        .add_systems(Update, (poll_remote_entity_rows, update_remote_entity_rows));
    }
}

#[derive(Resource, Default)]
struct PollTimer(Timer);

fn setup_poll_timer(mut commands: Commands) {
    commands.insert_resource(PollTimer(Timer::from_seconds(0.1, TimerMode::Repeating)));
}

#[derive(Resource, Default)]
struct RowTaskHandle(Option<Task<HashMap<Entity, EntityInspectorRow>>>);

fn poll_remote_entity_rows(
    time: Res<Time>,
    mut timer: ResMut<PollTimer>,
    mut task_handle: ResMut<RowTaskHandle>,
    remote_config: Res<RemoteConfig>,
    type_registry: Res<AppTypeRegistry>,
) {
    if timer.0.tick(time.delta()).just_finished() && task_handle.0.is_none() {
        let url = remote_config.remote_url.clone();

        // Clone the type registry Arc to move into the async block
        let type_registry_arc = type_registry.clone();

        let task_pool = IoTaskPool::get();
        let task = task_pool.spawn(async move {
            let req = BrpRequest {
                params: Some(
                    serde_json::to_value(BrpQueryParams {
                        data: BrpQuery {
                            components: vec![type_name::<Transform>().to_string()],
                            option: Vec::default(),
                            has: Vec::default(),
                        },
                        strict: false,
                        filter: BrpQueryFilter {
                            without: Vec::default(),
                            with: Vec::default(),
                        },
                    })
                    .unwrap(),
                ),
                jsonrpc: String::from("2.0"),
                method: String::from(BRP_QUERY_METHOD),
                id: Some(serde_json::to_value(1).expect("Failed to serialize id")),
            };

            let mut rows = HashMap::new();

            if let Ok(mut response) = ureq::post(&url).send_json(req) {
                if let Ok(json) = response.body_mut().read_json::<serde_json::Value>() {
                    if let Some(results) = json["result"].as_array() {
                        for entry in results {
                            let entity = serde_json::from_value::<Entity>(entry["entity"].clone())
                                .expect("Failed to parse entity ID");
                            let components = entry["components"]
                                .as_object()
                                .expect("Failed to parse components");
                            let name = components
                                .get(type_name::<Name>())
                                .and_then(|v| v.get("name"))
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string();

                            // Build component map
                            let mut comp_map = HashMap::new();
                            for (comp_name, comp_val) in components {
                                // Try to reflect using the type registry
                                if let Some(reg) =
                                    type_registry_arc.read().get_with_type_path(comp_name)
                                {
                                    if let Ok(reflected) = reg
                                        .data::<ReflectDeserialize>()
                                        .unwrap()
                                        .deserialize((&comp_val).into_deserializer())
                                    {
                                        comp_map.insert(comp_name.clone(), reflected);
                                    }
                                }
                            }
                            rows.insert(
                                entity,
                                EntityInspectorRow {
                                    name,
                                    components: comp_map,
                                },
                            );
                        }
                    }
                } else {
                    error!("Failed to parse JSON response");
                }
            } else {
                error!("Failed to send request to remote application");
            }
            rows
        });
        task_handle.0 = Some(task);
    }
}

fn update_remote_entity_rows(
    mut task_handle: ResMut<RowTaskHandle>,
    mut rows: ResMut<EntityInspectorRows>,
) {
    if let Some(mut task) = task_handle.0.take() {
        if let Some(result) = future::block_on(future::poll_once(&mut task)) {
            rows.rows = result;
        } else {
            // Not ready yet, put it back
            task_handle.0 = Some(task);
        }
    }
}
