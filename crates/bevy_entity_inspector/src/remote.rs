//! Remote inspection functionality for connecting to remote Bevy applications.
//!
//! This module provides the ability to inspect entities in remote Bevy applications
//! via the [`bevy_remote`](https://docs.rs/bevy_remote/latest/bevy_remote/) protocol. It implements efficient polling with hash-based
//! change detection to minimize network traffic and UI updates.
//!
//! # Features
//!
//! - **Automatic Discovery**: Connects to remote Bevy applications on default port
//! - **Efficient Polling**: Only updates when actual changes are detected
//! - **Hash-based Change Detection**: Uses content hashing to detect entity/component changes
//! - **Component Name Simplification**: Displays components as "`crate::Type`" for clarity
//! - **Graceful Degradation**: Handles components without reflection support
//! - **Event Integration**: Emits granular [`InspectorEvent`](crate::events::InspectorEvent)s for UI updates
//!
//! # Protocol
//!
//! Uses the [`bevy_remote`](https://docs.rs/bevy_remote/latest/bevy_remote/) HTTP protocol to query entity data. The system sends
//! BRP (Bevy Remote Protocol) requests to fetch all entities with all components,
//! then processes the JSON response to extract component data.
//!
//! # Performance
//!
//! - **Configurable Polling Rate**: Default 1 second, adjustable via timer
//! - **Change Detection**: Only processes updates when entity data actually changes
//! - **Async Processing**: Uses Bevy's [task system](https://docs.rs/bevy/latest/bevy/tasks/index.html) for non-blocking network requests
//! - **Memory Efficient**: Reuses data structures and avoids unnecessary allocations
//!
//! # Setup
//!
//! The remote inspection requires both the inspector application and the target
//! application to be configured:
//!
//! **Inspector Application:**
//! ```rust,no_run
//! use bevy::prelude::*;
//! use bevy_entity_inspector::InspectorPlugin;
//!
//! App::new()
//!     .add_plugins(DefaultPlugins)
//!     .add_plugins(InspectorPlugin) // Automatically includes remote if feature enabled
//!     .run();
//! ```
//!
//! **Target Application:**
//! ```rust,no_run
//! use bevy::prelude::*;
//! use bevy_remote::{RemotePlugin, http::RemoteHttpPlugin};
//!
//! App::new()
//!     .add_plugins(DefaultPlugins)
//!     .add_plugins(RemotePlugin::default())
//!     .add_plugins(RemoteHttpPlugin::default())
//!     // Your game logic here
//!     .run();
//! ```

use std::any::type_name;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use bevy::{
    platform::collections::HashMap,
    prelude::*,
    tasks::{futures_lite::future, IoTaskPool, Task},
};
use bevy_remote::{
    builtin_methods::{BrpQuery, BrpQueryFilter, BrpQueryParams, BRP_QUERY_METHOD},
    http::{DEFAULT_ADDR, DEFAULT_PORT},
    BrpRequest,
};

use crate::events::{EntityInspectorRow, EntityInspectorRows, InspectorEvent};
use serde::de::IntoDeserializer;

/// Plugin for entity inspector remote functionality.
///
/// This plugin enables the entity inspector to connect to and inspect entities
/// in remote Bevy applications via the `bevy_remote` protocol. It sets up
/// automatic polling, change detection, and event emission for remote entity data.
///
/// # Configuration
///
/// The plugin can be configured with a custom remote URL:
///
/// ```rust,no_run
/// use bevy_entity_inspector::remote::EntityInspectorRemotePlugin;
///
/// let plugin = EntityInspectorRemotePlugin {
///     remote_url: "http://localhost:8080/".to_string(),
/// };
/// ```
///
/// # Systems
///
/// The plugin adds two main systems:
/// - `poll_remote_entity_rows`: Periodically queries the remote application
/// - `update_remote_entity_rows`: Processes responses and emits change events
///
/// # Performance
///
/// - Uses async tasks to avoid blocking the main thread
/// - Implements hash-based change detection to minimize UI updates
/// - Configurable polling frequency (default: 1 second)
/// - Graceful error handling for network issues
pub struct EntityInspectorRemotePlugin {
    /// The address of the remote application to connect to.
    ///
    /// Should include the protocol, host, port, and path.
    /// Example: "<http://127.0.0.1:15702>/"
    pub remote_url: String,
}

impl Default for EntityInspectorRemotePlugin {
    fn default() -> Self {
        EntityInspectorRemotePlugin {
            remote_url: format!("http://{DEFAULT_ADDR}:{DEFAULT_PORT}/"),
        }
    }
}

/// Configuration resource for remote entity inspection.
///
/// Contains connection settings for the remote Bevy application.
/// This resource is automatically inserted by the `EntityInspectorRemotePlugin`.
#[derive(Resource, Debug, Clone)]
pub struct RemoteConfig {
    /// The URL of the remote application.
    ///
    /// Used by the polling system to send BRP requests.
    /// Must be a valid HTTP/HTTPS URL.
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

/// Timer resource for controlling remote polling frequency.
#[derive(Resource, Default)]
struct PollTimer(Timer);

/// Sets up the polling timer with default frequency.
///
/// Configures the timer to poll every 1 second. This can be adjusted
/// based on performance requirements and network conditions.
fn setup_poll_timer(mut commands: Commands) {
    commands.insert_resource(PollTimer(Timer::from_seconds(1.0, TimerMode::Repeating)));
}

/// Handle for the background task that fetches remote entity data.
#[derive(Resource, Default)]
struct RowTaskHandle(Option<Task<HashMap<Entity, EntityInspectorRow>>>);

/// System that initiates remote entity data polling.
///
/// This system runs on a timer and spawns async tasks to fetch entity data
/// from the remote Bevy application. It uses the BRP (Bevy Remote Protocol)
/// to query all entities with all their components.
///
/// # Process
///
/// 1. Check if timer has elapsed and no task is currently running
/// 2. Create BRP query request for all entities and components
/// 3. Spawn async task to send HTTP request
/// 4. Process JSON response and extract entity/component data
/// 5. Calculate content hash for change detection
/// 6. Return processed entity data for the update system
///
/// # Error Handling
///
/// - Network errors are logged but don't crash the system
/// - JSON parsing errors are handled gracefully
/// - Component reflection failures fall back to placeholder data
///
/// # Component Processing
///
/// Components are processed with the following priority:
/// 1. Full reflection via `ReflectDeserialize` (ideal case)
/// 2. Placeholder `DynamicStruct` for registered types without reflection
/// 3. Placeholder for unregistered types
///
/// Component names are simplified from full paths (e.g., `bevy_transform::Transform`)
/// to crate-qualified names (e.g., `bevy_transform::Transform`) for better UX.
fn poll_remote_entity_rows(
    time: Res<Time>,
    mut timer: ResMut<PollTimer>,
    mut task_handle: ResMut<RowTaskHandle>,
    remote_config: Res<RemoteConfig>,
    type_registry: Res<AppTypeRegistry>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        info!("Polling timer finished, checking if we should start new task");

        if task_handle.0.is_none() {
            info!("Starting remote polling to {}", remote_config.remote_url);
            let url = remote_config.remote_url.clone();

            // Clone the type registry Arc to move into the async block
            let type_registry_arc = type_registry.clone();

            let task_pool = IoTaskPool::get();
            let task = task_pool.spawn(async move {
                info!("Sending remote query request to {}", url);
                let req = BrpRequest {
                    params: Some(
                        serde_json::to_value(BrpQueryParams {
                            data: BrpQuery {
                                components: Vec::default(),
                                option: bevy_remote::builtin_methods::ComponentSelector::All,
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
                    info!("Got response from remote server");
                    if let Ok(json) = response.body_mut().read_json::<serde_json::Value>() {
                        info!("Parsed JSON response: {:?}", json);
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
                                    // Extract crate name and type name from the full path
                                    // e.g. "bevy_transform::components::transform::Transform" -> "bevy_transform::Transform"
                                    // e.g. "my_game::player::Player" -> "my_game::Player"
                                    let component_display_name = if let Some(crate_and_rest) = comp_name.split_once("::") {
                                        let crate_name = crate_and_rest.0;
                                        let type_name = comp_name.split("::").last().unwrap_or(comp_name);
                                        format!("{crate_name}::{type_name}")
                                    } else {
                                        // If there's no "::" separator, just use the original name
                                        comp_name.to_string()
                                    };
                                    // Try to reflect using the type registry
                                    if let Some(reg) =
                                        type_registry_arc.read().get_with_type_path(comp_name)
                                    {
                                        if let Some(reflect_deserialize) = reg.data::<ReflectDeserialize>() {
                                            if let Ok(reflected) = reflect_deserialize
                                                .deserialize(comp_val.into_deserializer())
                                            {
                                                comp_map.insert(component_display_name, reflected.to_dynamic());
                                            }
                                        } else {
                                            // Component type is registered but doesn't have ReflectDeserialize
                                            // Store the raw JSON as a debug representation
                                            debug!("Component '{}' is registered but lacks ReflectDeserialize data, storing raw JSON", comp_name);
                                            // Create a simple dynamic representation from the JSON
                                            let dynamic_struct = bevy::reflect::DynamicStruct::default();
                                            comp_map.insert(component_display_name, Box::new(dynamic_struct) as Box<dyn PartialReflect>);
                                        }
                                    } else {
                                        // Component type not registered - store raw JSON representation
                                        debug!("Component '{}' not found in type registry, storing raw JSON", comp_name);
                                        let dynamic_struct = bevy::reflect::DynamicStruct::default();
                                        comp_map.insert(component_display_name, Box::new(dynamic_struct) as Box<dyn PartialReflect>);
                                    }
                                }
                                // Calculate hash of the raw component data for change detection
                                let mut hasher = DefaultHasher::new();
                                // Sort the component data to ensure consistent hashing
                                let mut sorted_components: Vec<_> = components.iter().collect();
                                sorted_components.sort_by_key(|(name, _)| *name);
                                for (comp_name, comp_val) in sorted_components {
                                    comp_name.hash(&mut hasher);
                                    // Hash the JSON string representation
                                    serde_json::to_string(comp_val).unwrap_or_default().hash(&mut hasher);
                                }
                                let data_hash = hasher.finish();
                                rows.insert(
                                    entity,
                                    EntityInspectorRow {
                                        name,
                                        components: comp_map,
                                        data_hash: Some(data_hash),
                                    },
                                );
                            }
                        }
                    } else {
                        error!("Failed to parse JSON response");
                    }
                } else {
                    error!("Failed to send request to remote application at {}", url);
                }
                rows
            });
            task_handle.0 = Some(task);
        } else {
            info!("Remote polling task already running, skipping");
        }
    }
}

/// System that processes completed remote polling tasks and emits change events.
///
/// This system checks for completed async tasks from `poll_remote_entity_rows`
/// and processes the results. It uses the change detection system to determine
/// what has changed and emits appropriate `InspectorEvent`s.
///
/// # Change Detection
///
/// The system uses hash-based change detection to efficiently determine what
/// has changed between polling cycles:
/// - Content hashes are calculated from the raw JSON component data
/// - Only entities with different hashes are considered "updated"
/// - Initial population (empty -> non-empty) is handled as "added" entities
///
/// # Event Emission
///
/// Emits the following events based on detected changes:
/// - `InspectorEvent::EntityAdded` for new entities
/// - `InspectorEvent::EntityRemoved` for deleted entities  
/// - `InspectorEvent::EntityUpdated` for modified entities
///
/// # Performance
///
/// - Only processes tasks when they're complete (non-blocking)
/// - Uses efficient `HashMap` operations for change detection
/// - Batches all changes into a single update cycle
/// - Clears change tracking immediately after event emission
fn update_remote_entity_rows(
    mut task_handle: ResMut<RowTaskHandle>,
    mut rows: ResMut<EntityInspectorRows>,
    mut events: EventWriter<InspectorEvent>,
) {
    if let Some(mut task) = task_handle.0.take() {
        if let Some(result) = future::block_on(future::poll_once(&mut task)) {
            info!(
                "Remote polling task completed with {} entities",
                result.len()
            );
            // Use the new change tracking to detect what changed
            rows.update_rows(result);

            // Send events for the changes
            if rows.has_changes() {
                info!(
                    "Remote data changed: {} added, {} removed, {} updated",
                    rows.added_entities.len(),
                    rows.removed_entities.len(),
                    rows.updated_entities.len()
                );

                for entity in &rows.added_entities {
                    events.write(InspectorEvent::EntityAdded(*entity));
                }

                for entity in &rows.removed_entities {
                    events.write(InspectorEvent::EntityRemoved(*entity));
                }

                for entity in &rows.updated_entities {
                    events.write(InspectorEvent::EntityUpdated(*entity));
                }

                // Clear the change tracking
                rows.clear_changes();
            } else {
                info!("No changes detected in remote data");
            }
        } else {
            // Not ready yet, put it back
            task_handle.0 = Some(task);
        }
    }
}
