use std::collections::BinaryHeap;

use bevy::{
    asset::{AssetPath, LoadState},
    platform::collections::HashMap,
    prelude::*,
};

use crate::asset::{AssetError, LoadPriority, LoadTask};

/// Active asynchronous loading task.
#[derive(Component)]
pub struct ActiveLoadTask {
    pub task_id: u64,
    pub path: AssetPath<'static>,
    pub handle: Handle<Image>,
    pub priority: LoadPriority,
}

/// Event emitted when an asset finishes loading.
#[derive(Event, BufferedEvent, Debug, Clone)]
pub struct AssetLoadCompleted {
    pub task_id: u64,
    pub path: AssetPath<'static>,
    pub handle: Handle<Image>,
    pub priority: LoadPriority,
}

/// Event emitted when an asset fails to load.
#[derive(Event, BufferedEvent, Debug, Clone)]
pub struct AssetLoadFailed {
    pub task_id: u64,
    pub path: AssetPath<'static>,
    pub error: AssetError,
}

/// Event emitted when an asset is hot-reloaded.
#[derive(Event, BufferedEvent, Debug, Clone)]
pub struct AssetHotReloaded {
    pub task_id: u64,
    pub path: AssetPath<'static>,
    pub handle: Handle<Image>,
}

/// Asynchronous asset loader with priority queue.
#[derive(Resource)]
pub struct AssetLoader {
    queue: BinaryHeap<LoadTask>,
    next_task_id: u64,
    max_concurrent: usize,
    active_tasks: usize,
    task_paths: HashMap<u64, AssetPath<'static>>,
    handle_to_entity: HashMap<AssetId<Image>, Entity>,
    task_to_handle: HashMap<u64, AssetId<Image>>,
}

impl Default for AssetLoader {
    fn default() -> Self {
        Self {
            queue: BinaryHeap::new(),
            next_task_id: 0,
            max_concurrent: 4,
            active_tasks: 0,
            task_paths: HashMap::new(),
            handle_to_entity: HashMap::new(),
            task_to_handle: HashMap::new(),
        }
    }
}

impl AssetLoader {
    /// Creates a new loader with specified max concurrent tasks.
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            queue: BinaryHeap::new(),
            next_task_id: 0,
            max_concurrent,
            active_tasks: 0,
            task_paths: HashMap::new(),
            handle_to_entity: HashMap::new(),
            task_to_handle: HashMap::new(),
        }
    }

    /// Submits a load task to the priority queue.
    pub fn submit<'a>(&mut self, path: impl Into<AssetPath<'a>>, priority: LoadPriority) -> u64 {
        let task_id = self.next_task_id;
        self.next_task_id += 1;
        let asset_path: AssetPath<'static> = path.into().into_owned();
        let task = LoadTask::new(asset_path.clone(), priority, task_id);
        self.queue.push(task);
        self.task_paths.insert(task_id, asset_path.clone());
        task_id
    }

    /// Gets the path for a task ID.
    pub fn get_task_path(&self, task_id: u64) -> Option<&AssetPath<'static>> {
        self.task_paths.get(&task_id)
    }

    /// Removes task path mapping (called when task completes).
    pub fn remove_task_path(&mut self, task_id: u64) {
        self.task_paths.remove(&task_id);
    }

    /// Returns the number of tasks in the queue.
    pub fn queue_len(&self) -> usize {
        self.queue.len()
    }

    /// Peeks at the next task to process without removing it.
    pub fn peek_next(&self) -> Option<&LoadTask> {
        self.queue.peek()
    }

    /// Pops the next task to process from the queue.
    pub fn pop_next(&mut self) -> Option<LoadTask> {
        self.queue.pop()
    }

    /// Checks if a new task can be started.
    pub fn can_start_task(&self) -> bool {
        self.active_tasks < self.max_concurrent
    }

    /// Increments the active task count.
    pub fn start_task(&mut self) {
        self.active_tasks += 1;
    }

    /// Decrements the active task count.
    pub fn finish_task(&mut self) {
        if self.active_tasks > 0 {
            self.active_tasks -= 1;
        }
    }

    /// Returns the current number of active tasks.
    pub fn active_tasks(&self) -> usize {
        self.active_tasks
    }

    /// Spawns an asynchronous load task using AssetServer.
    pub fn spawn_load_task(
        &mut self,
        task: LoadTask,
        asset_server: &AssetServer,
    ) -> ActiveLoadTask {
        let task_id = task.task_id;
        let path = task.path.clone();
        let priority = task.priority;

        let handle = asset_server.load(&path);

        ActiveLoadTask {
            task_id,
            path,
            handle: handle.clone(),
            priority,
        }
    }

    /// Registers task entity and handle mapping.
    pub fn register_task(&mut self, task_id: u64, entity: Entity, handle: Handle<Image>) {
        let handle_id = handle.id();
        self.handle_to_entity.insert(handle_id, entity);
        self.task_to_handle.insert(task_id, handle_id);
    }

    /// Gets task entity by handle ID.
    pub fn get_entity_by_handle(&self, handle_id: AssetId<Image>) -> Option<Entity> {
        self.handle_to_entity.get(&handle_id).copied()
    }

    /// Gets handle ID by task ID.
    pub fn get_handle_id_by_task(&self, task_id: u64) -> Option<AssetId<Image>> {
        self.task_to_handle.get(&task_id).copied()
    }

    /// Cleans up task mappings.
    pub fn cleanup_task(&mut self, task_id: u64, handle_id: AssetId<Image>) {
        self.task_paths.remove(&task_id);
        self.handle_to_entity.remove(&handle_id);
        self.task_to_handle.remove(&task_id);
    }
}

/// Processes the load queue and starts new tasks.
pub fn process_load_queue(
    mut commands: Commands,
    mut loader: ResMut<AssetLoader>,
    asset_server: Res<AssetServer>,
) {
    while loader.can_start_task() {
        if let Some(task) = loader.pop_next() {
            let active_task = loader.spawn_load_task(task, &asset_server);
            let task_id = active_task.task_id;
            let handle = active_task.handle.clone();

            loader.start_task();
            let entity = commands.spawn(active_task).id();
            loader.register_task(task_id, entity, handle);
        } else {
            break;
        }
    }
}

/// Handles asset events (event-driven approach).
pub fn handle_asset_events(
    mut commands: Commands,
    mut loader: ResMut<AssetLoader>,
    mut asset_events: EventReader<AssetEvent<Image>>,
    mut load_failed_events_bevy: EventReader<bevy::asset::AssetLoadFailedEvent<Image>>,
    asset_server: Res<AssetServer>,
    mut load_completed_events: EventWriter<AssetLoadCompleted>,
    mut load_failed_events: EventWriter<AssetLoadFailed>,
    mut hot_reload_events: EventWriter<AssetHotReloaded>,
    task_query: Query<&ActiveLoadTask>,
) {
    // Handle Bevy's AssetLoadFailedEvent
    for event in load_failed_events_bevy.read() {
        if let Some(entity) = loader.get_entity_by_handle(event.id) {
            if let Ok(active_task) = task_query.get(entity) {
                load_failed_events.write(AssetLoadFailed {
                    task_id: active_task.task_id,
                    path: active_task.path.clone(),
                    error: AssetError::AssetLoadFailed {
                        path: active_task.path.clone(),
                        reason: format!("{:?}", event.error),
                    },
                });

                loader.finish_task();
                loader.cleanup_task(active_task.task_id, event.id);
                commands.entity(entity).despawn();
            }
        }
    }

    // Handle AssetEvent
    for event in asset_events.read() {
        match event {
            AssetEvent::LoadedWithDependencies { id } => {
                if let Some(entity) = loader.get_entity_by_handle(*id) {
                    if let Ok(active_task) = task_query.get(entity) {
                        load_completed_events.write(AssetLoadCompleted {
                            task_id: active_task.task_id,
                            path: active_task.path.clone(),
                            handle: active_task.handle.clone(),
                            priority: active_task.priority,
                        });

                        loader.finish_task();
                        loader.cleanup_task(active_task.task_id, *id);
                        commands.entity(entity).despawn();
                    }
                }
            }
            AssetEvent::Removed { id } => {
                if let Some(entity) = loader.get_entity_by_handle(*id) {
                    if let Ok(active_task) = task_query.get(entity) {
                        load_failed_events.write(AssetLoadFailed {
                            task_id: active_task.task_id,
                            path: active_task.path.clone(),
                            error: AssetError::AssetRemoved {
                                path: active_task.path.clone(),
                            },
                        });

                        loader.finish_task();
                        loader.cleanup_task(active_task.task_id, *id);
                        commands.entity(entity).despawn();
                    }
                }
            }
            AssetEvent::Modified { id } => {
                if let Some(entity) = loader.get_entity_by_handle(*id) {
                    if let Ok(active_task) = task_query.get(entity) {
                        hot_reload_events.write(AssetHotReloaded {
                            task_id: active_task.task_id,
                            path: active_task.path.clone(),
                            handle: active_task.handle.clone(),
                        });
                    }
                }
            }
            _ => {}
        }
    }

    // Fallback: Check load state for failed tasks
    let mut failed_entities = Vec::new();
    for active_task in task_query.iter() {
        let load_state = asset_server.load_state(&active_task.handle);
        if let bevy::asset::LoadState::Failed(_) = load_state {
            if let Some(entity) = loader.get_entity_by_handle(active_task.handle.id()) {
                load_failed_events.write(AssetLoadFailed {
                    task_id: active_task.task_id,
                    path: active_task.path.clone(),
                    error: AssetError::AssetLoadFailed {
                        path: active_task.path.clone(),
                        reason: "Asset load failed".to_string(),
                    },
                });

                loader.finish_task();
                loader.cleanup_task(active_task.task_id, active_task.handle.id());
                failed_entities.push(entity);
            }
        }
    }
    for entity in failed_entities {
        commands.entity(entity).despawn();
    }
}

/// Polls active tasks and handles completed ones (fallback approach).
#[allow(dead_code)]
pub fn poll_load_tasks(
    mut commands: Commands,
    mut loader: ResMut<AssetLoader>,
    asset_server: Res<AssetServer>,
    mut task_query: Query<(Entity, &ActiveLoadTask)>,
) {
    for (entity, active_task) in task_query.iter_mut() {
        if asset_server.is_loaded_with_dependencies(&active_task.handle) {
            info!("Asset loaded successfully: {:?}", active_task.path);

            loader.finish_task();
            let handle_id = active_task.handle.id();
            loader.cleanup_task(active_task.task_id, handle_id);
            commands.entity(entity).despawn();
        } else {
            let load_state = asset_server.load_state(&active_task.handle);
            if let LoadState::Failed(_) = load_state {
                warn!("Asset load failed: {:?}", active_task.path);

                loader.finish_task();
                let handle_id = active_task.handle.id();
                loader.cleanup_task(active_task.task_id, handle_id);
                commands.entity(entity).despawn();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_submit_and_queue() {
        let mut loader = AssetLoader::new(4);
        assert_eq!(loader.queue_len(), 0);

        loader.submit("test1.png", LoadPriority::Preload);
        assert_eq!(loader.queue_len(), 1);

        loader.submit("test2.png", LoadPriority::CurrentAccess);
        assert_eq!(loader.queue_len(), 2);
    }

    #[test]
    fn test_priority_ordering() {
        let mut loader = AssetLoader::new(4);

        let id1 = loader.submit("preload.png", LoadPriority::Preload);
        let id2 = loader.submit("current.png", LoadPriority::CurrentAccess);
        let id3 = loader.submit("hotreload.png", LoadPriority::HotReload);

        let task1 = loader.pop_next().unwrap();
        assert_eq!(task1.priority, LoadPriority::CurrentAccess);
        assert_eq!(task1.task_id, id2);

        let task2 = loader.pop_next().unwrap();
        assert_eq!(task2.priority, LoadPriority::HotReload);
        assert_eq!(task2.task_id, id3);

        let task3 = loader.pop_next().unwrap();
        assert_eq!(task3.priority, LoadPriority::Preload);
        assert_eq!(task3.task_id, id1);
    }

    #[test]
    fn test_concurrent_limit() {
        let mut loader = AssetLoader::new(2);
        assert!(loader.can_start_task());

        loader.start_task();
        assert_eq!(loader.active_tasks(), 1);
        assert!(loader.can_start_task());

        loader.start_task();
        assert_eq!(loader.active_tasks(), 2);
        assert!(!loader.can_start_task());

        loader.finish_task();
        assert_eq!(loader.active_tasks(), 1);
        assert!(loader.can_start_task());
    }

    #[test]
    fn test_task_registration_and_cleanup() {
        let mut loader = AssetLoader::new(4);
        let task_id = loader.submit("test.png", LoadPriority::CurrentAccess);

        // Verify task path is stored
        assert_eq!(
            loader.get_task_path(task_id),
            Some(&AssetPath::from("test.png"))
        );

        // Use a mock handle ID for testing
        // Note: In a real scenario, we'd use actual asset handles, but for unit testing
        // we can use default values to test the mapping logic
        let handle_id: AssetId<Image> = AssetId::default();
        let entity = Entity::PLACEHOLDER;
        let handle = Handle::<Image>::default();
        loader.register_task(task_id, entity, handle);

        assert_eq!(loader.get_entity_by_handle(handle_id), Some(entity));
        assert_eq!(loader.get_handle_id_by_task(task_id), Some(handle_id));

        loader.cleanup_task(task_id, handle_id);

        assert_eq!(loader.get_entity_by_handle(handle_id), None);
        assert_eq!(loader.get_handle_id_by_task(task_id), None);
        // Verify task path is also removed
        assert_eq!(loader.get_task_path(task_id), None);
    }

    #[test]
    fn test_get_task_path() {
        let mut loader = AssetLoader::new(4);

        // Test with non-existent task
        assert_eq!(loader.get_task_path(999), None);

        // Test with existing task
        let task_id = loader.submit("test.png", LoadPriority::Preload);
        assert_eq!(
            loader.get_task_path(task_id),
            Some(&AssetPath::from("test.png"))
        );

        // Test with multiple tasks
        let task_id2 = loader.submit("test2.png", LoadPriority::CurrentAccess);
        assert_eq!(
            loader.get_task_path(task_id2),
            Some(&AssetPath::from("test2.png"))
        );

        // Verify both paths are stored
        assert_eq!(
            loader.get_task_path(task_id),
            Some(&AssetPath::from("test.png"))
        );
    }
}
