use core::cmp::Ordering;

use bevy::asset::AssetPath;

use crate::asset::LoadPriority;

/// Asset loading task.
#[derive(Debug, Clone)]
pub struct LoadTask {
    /// Asset path.
    pub path: AssetPath<'static>,
    /// Load priority.
    pub priority: LoadPriority,
    /// Task ID for tracking.
    pub task_id: u64,
}

impl LoadTask {
    /// Creates a new load task.
    pub fn new<'a>(path: impl Into<AssetPath<'a>>, priority: LoadPriority, task_id: u64) -> Self {
        Self {
            path: path.into().into_owned(),
            priority,
            task_id,
        }
    }
}

impl PartialEq for LoadTask {
    fn eq(&self, other: &Self) -> bool {
        self.task_id == other.task_id
    }
}

impl Eq for LoadTask {}

impl PartialOrd for LoadTask {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LoadTask {
    fn cmp(&self, other: &Self) -> Ordering {
        // Sort by priority first (higher priority first, BinaryHeap is max-heap)
        match self.priority.cmp(&other.priority) {
            Ordering::Equal => {
                // Same priority: sort by task ID (earlier tasks first)
                // BinaryHeap is max-heap, so reverse ID comparison
                other.task_id.cmp(&self.task_id)
            }
            other => other,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_ordering_by_priority() {
        let task1 = LoadTask::new("test1.png", LoadPriority::Preload, 1);
        let task2 = LoadTask::new("test2.png", LoadPriority::CurrentAccess, 2);
        assert!(task2 > task1);
    }

    #[test]
    fn test_task_ordering_by_id_when_same_priority() {
        let task1 = LoadTask::new("test1.png", LoadPriority::Preload, 1);
        let task2 = LoadTask::new("test2.png", LoadPriority::Preload, 2);
        assert!(task1 > task2);
    }
}
