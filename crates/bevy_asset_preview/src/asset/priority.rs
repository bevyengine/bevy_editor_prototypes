use core::cmp::Ordering;

/// Load task priority levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LoadPriority {
    /// Current access - highest priority.
    CurrentAccess,
    /// Hot reload - second priority.
    HotReload,
    /// Preload - third priority.
    Preload,
}

impl LoadPriority {
    /// Returns the priority value. Higher value means higher priority.
    pub fn value(self) -> u8 {
        match self {
            LoadPriority::CurrentAccess => 3,
            LoadPriority::HotReload => 2,
            LoadPriority::Preload => 1,
        }
    }
}

impl PartialOrd for LoadPriority {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LoadPriority {
    fn cmp(&self, other: &Self) -> Ordering {
        self.value().cmp(&other.value())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_priority_ordering() {
        assert!(LoadPriority::CurrentAccess > LoadPriority::HotReload);
        assert!(LoadPriority::HotReload > LoadPriority::Preload);
        assert!(LoadPriority::CurrentAccess > LoadPriority::Preload);
    }

    #[test]
    fn test_priority_values() {
        assert_eq!(LoadPriority::CurrentAccess.value(), 3);
        assert_eq!(LoadPriority::HotReload.value(), 2);
        assert_eq!(LoadPriority::Preload.value(), 1);
    }
}
