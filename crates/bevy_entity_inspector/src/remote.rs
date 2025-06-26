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
        .init_resource::<RemoteTransforms>()
        .init_resource::<PollTimer>()
        .init_resource::<TransformTaskHandle>()
        .add_systems(Startup, setup_poll_timer)
        .add_systems(Update, (poll_remote_transforms, update_remote_transforms));
    }
}

/// Resource to hold the remote transforms data.
#[derive(Resource, Default)]
pub struct RemoteTransforms(pub HashMap<Entity, Transform>);

#[derive(Resource, Default)]
struct PollTimer(Timer);

fn setup_poll_timer(mut commands: Commands) {
    commands.insert_resource(PollTimer(Timer::from_seconds(0.1, TimerMode::Repeating)));
}

#[derive(Resource, Default)]
struct TransformTaskHandle(Option<Task<HashMap<Entity, Transform>>>);

fn poll_remote_transforms(
    time: Res<Time>,
    mut timer: ResMut<PollTimer>,
    mut task_handle: ResMut<TransformTaskHandle>,
    remote_config: Res<RemoteConfig>,
) {
    if timer.0.tick(time.delta()).just_finished() && task_handle.0.is_none() {
        let components = Vec::default();
        let url = remote_config.remote_url.clone();

        let task_pool = IoTaskPool::get();
        let task = task_pool.spawn(async move {
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

            let mut entity_transform_map: HashMap<Entity, Transform> = HashMap::new();

            if let Ok(mut response) = ureq::post(&url).send_json(req) {
                if let Ok(json) = response.body_mut().read_json::<serde_json::Value>() {
                    if let Some(results) = json["result"].as_array() {
                        for entry in results {
                            let entity = entry["entity"]
                                .as_str()
                                .and_then(|s| s.parse::<u64>().ok())
                                .map(Entity::from_bits)
                                .expect("Failed to parse entity ID");
                            let components = entry["components"]
                                .as_object()
                                .expect("Expected components to be an object");
                            let transform_json = components
                                .get("bevy_transform::components::transform::Transform")
                                .expect("Expected Transform component to be present");
                            if let Ok(transform) =
                                serde_json::from_value::<Transform>(transform_json.clone())
                            {
                                entity_transform_map.insert(entity, transform);
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
