use std::io::Cursor;

use bevy::{
    asset::{AssetPath, io::ErasedAssetWriter},
    platform::collections::HashMap,
    prelude::*,
    tasks::{IoTaskPool, Task, block_on, futures_lite},
};
use image::ImageFormat;

use crate::asset::AssetError;

/// Active save task tracking component.
#[derive(Component)]
pub struct ActiveSaveTask {
    pub task_id: u64,
    pub path: AssetPath<'static>,
    pub target_path: AssetPath<'static>,
    pub task: Task<Result<(), AssetError>>,
}

/// Event emitted when a save task completes.
#[derive(Event, BufferedEvent, Debug, Clone)]
pub struct SaveCompleted {
    pub task_id: u64,
    pub path: AssetPath<'static>,
    pub result: Result<(), AssetError>,
}

/// Resource for save task tracking.
#[derive(Resource, Default)]
pub struct SaveTaskTracker {
    next_task_id: u64,
    pending_saves: HashMap<u64, AssetPath<'static>>,
}

impl SaveTaskTracker {
    /// Creates a new save task ID.
    pub fn create_task_id(&mut self) -> u64 {
        let id = self.next_task_id;
        self.next_task_id += 1;
        id
    }

    /// Registers a pending save.
    pub fn register_pending(&mut self, task_id: u64, path: AssetPath<'static>) {
        self.pending_saves.insert(task_id, path);
    }

    /// Marks a save as completed.
    pub fn mark_completed(&mut self, task_id: u64) {
        self.pending_saves.remove(&task_id);
    }

    /// Returns the number of pending saves.
    pub fn pending_count(&self) -> usize {
        self.pending_saves.len()
    }

    /// Checks if a task ID is in pending saves.
    pub fn is_pending(&self, task_id: u64) -> bool {
        self.pending_saves.contains_key(&task_id)
    }
}

/// Saves an image asset to the specified path asynchronously using AssetWriter abstraction.
pub fn save_image<'a>(
    image: Handle<Image>,
    target_path: impl Into<AssetPath<'a>>,
    images: &Assets<Image>,
    writer: impl ErasedAssetWriter,
) -> Task<Result<(), AssetError>> {
    let target_path: AssetPath<'static> = target_path.into().into_owned();

    let Some(image_data) = images.get(&image) else {
        return IoTaskPool::get()
            .spawn(async move { Err(AssetError::ImageNotFound { handle: image }) });
    };

    // Convert to dynamic image
    let dynamic_image = match image_data.clone().try_into_dynamic() {
        Ok(img) => img,
        Err(e) => {
            return IoTaskPool::get()
                .spawn(async move { Err(AssetError::ImageConversionFailed(e.to_string())) });
        }
    };

    // Convert to RGBA8 format
    let rgba_image = dynamic_image.into_rgba8();

    let task_pool = IoTaskPool::get();
    let target_path_clone = target_path.clone();
    // Ensure the file extension is .webp for WebP format
    let mut target_path_for_writer = target_path.path().to_path_buf();
    if let Some(ext) = target_path_for_writer.extension() {
        if ext.to_str() != Some("webp") {
            target_path_for_writer.set_extension("webp");
        }
    } else {
        target_path_for_writer.set_extension("webp");
    }

    task_pool.spawn(async move {
        // Create directory first
        if let Some(parent) = target_path_for_writer.parent() {
            writer.create_directory(parent).await.map_err(|e| {
                AssetError::DirectoryCreationFailed {
                    path: parent.to_path_buf(),
                    reason: e.to_string(),
                }
            })?;
        }

        // Encode WebP directly to memory
        let mut cursor = Cursor::new(Vec::new());
        rgba_image
            .write_to(&mut cursor, ImageFormat::WebP)
            .map_err(|e| AssetError::ImageEncodeFailed {
                format: "WebP".to_string(),
                reason: e.to_string(),
            })?;

        let webp_bytes = cursor.into_inner();

        // Write via AssetWriter (atomic operation)
        writer
            .write_bytes(&target_path_for_writer, &webp_bytes)
            .await
            .map_err(|e| AssetError::FileWriteFailed {
                path: target_path_clone.path().to_path_buf(),
                reason: e.to_string(),
            })?;

        info!("Image saved successfully to {:?}", target_path_clone);
        Ok(())
    })
}

/// System that monitors save completion and cleans up tasks.
pub fn monitor_save_completion(
    mut save_completed_events: EventWriter<SaveCompleted>,
    mut tracker: ResMut<SaveTaskTracker>,
    mut commands: Commands,
    mut save_task_query: Query<(Entity, &mut ActiveSaveTask)>,
) {
    for (entity, mut active_task) in save_task_query.iter_mut() {
        // Poll the async task
        if let Some(result) = block_on(futures_lite::future::poll_once(&mut active_task.task)) {
            // Task completed, send event
            save_completed_events.write(SaveCompleted {
                task_id: active_task.task_id,
                path: active_task.path.clone(),
                result,
            });

            tracker.mark_completed(active_task.task_id);
            commands.entity(entity).despawn();
        }
    }
}

/// System that handles save completion events.
pub fn handle_save_completed(mut save_completed_events: EventReader<SaveCompleted>) {
    for event in save_completed_events.read() {
        match &event.result {
            Ok(_) => {
                debug!(
                    "Save task {} completed successfully for {:?}",
                    event.task_id, event.path
                );
            }
            Err(err) => {
                warn!(
                    "Save task {} failed for {:?}: {}",
                    event.task_id, event.path, err
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_id_generation() {
        let mut tracker = SaveTaskTracker::default();
        assert_eq!(tracker.create_task_id(), 0);
        assert_eq!(tracker.create_task_id(), 1);
        assert_eq!(tracker.create_task_id(), 2);
    }

    #[test]
    fn test_pending_save_registration() {
        let mut tracker = SaveTaskTracker::default();
        let id1 = tracker.create_task_id();
        let id2 = tracker.create_task_id();

        let path1 = AssetPath::from("test1.png");
        tracker.register_pending(id1, path1.clone());
        assert_eq!(tracker.pending_count(), 1);
        assert!(tracker.is_pending(id1));

        let path2 = AssetPath::from("test2.png");
        tracker.register_pending(id2, path2.clone());
        assert_eq!(tracker.pending_count(), 2);
        assert!(tracker.is_pending(id2));
    }

    #[test]
    fn test_mark_completed() {
        let mut tracker = SaveTaskTracker::default();
        let id1 = tracker.create_task_id();
        let id2 = tracker.create_task_id();

        let path1 = AssetPath::from("test1.png");
        let path2 = AssetPath::from("test2.png");
        tracker.register_pending(id1, path1);
        tracker.register_pending(id2, path2);

        assert_eq!(tracker.pending_count(), 2);
        tracker.mark_completed(id1);
        assert_eq!(tracker.pending_count(), 1);
        assert!(!tracker.is_pending(id1));
        assert!(tracker.is_pending(id2));

        tracker.mark_completed(id2);
        assert_eq!(tracker.pending_count(), 0);
        assert!(!tracker.is_pending(id2));
    }
}
