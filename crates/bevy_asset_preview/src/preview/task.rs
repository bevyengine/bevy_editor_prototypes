use bevy::{asset::AssetPath, platform::collections::HashMap, prelude::*};

use crate::preview::{PreviewMode, PreviewRequestType};

/// Event emitted when a preview is ready.
#[derive(Event, BufferedEvent, Debug, Clone)]
pub struct PreviewReady {
    /// Task ID of the preview request.
    pub task_id: u64,
    /// Asset path.
    pub path: AssetPath<'static>,
    /// Preview image handle.
    pub image_handle: Handle<Image>,
}

/// Event emitted when a preview generation fails.
#[derive(Event, BufferedEvent, Debug, Clone)]
pub struct PreviewFailed {
    /// Task ID of the preview request.
    pub task_id: u64,
    /// Asset path.
    pub path: AssetPath<'static>,
    /// Error message.
    pub error: String,
}

/// Component that tracks a pending preview request.
#[derive(Component, Debug)]
pub struct PendingPreviewRequest {
    /// Task ID.
    pub task_id: u64,
    /// Asset path.
    pub path: AssetPath<'static>,
    /// Preview request type.
    pub request_type: PreviewRequestType,
    /// Preview mode.
    pub mode: PreviewMode,
}

/// Resource that manages preview generation tasks.
#[derive(Resource, Default)]
pub struct PreviewTaskManager {
    /// Next task ID.
    next_task_id: u64,
    /// Maps task ID to request entity.
    task_to_entity: HashMap<u64, Entity>,
}

impl PreviewTaskManager {
    /// Creates a new task manager.
    pub fn new() -> Self {
        Self {
            next_task_id: 0,
            task_to_entity: HashMap::new(),
        }
    }

    /// Creates a new task ID.
    pub fn create_task_id(&mut self) -> u64 {
        let id = self.next_task_id;
        self.next_task_id += 1;
        id
    }

    /// Registers a task entity.
    pub fn register_task(&mut self, task_id: u64, entity: Entity) {
        self.task_to_entity.insert(task_id, entity);
    }

    /// Gets entity for a task ID.
    pub fn get_entity(&self, task_id: u64) -> Option<Entity> {
        self.task_to_entity.get(&task_id).copied()
    }

    /// Removes task registration.
    pub fn remove_task(&mut self, task_id: u64) {
        self.task_to_entity.remove(&task_id);
    }
}
