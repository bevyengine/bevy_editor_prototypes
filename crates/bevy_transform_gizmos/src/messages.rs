//! This module contains events/messages used to communicate with the transform gizmo plugin.
use bevy::prelude::Entity;
use bevy::prelude::Message;

/// Viewport size in pixels as a 2D vector.
pub type ViewportSize = [f32; 2];

/// Event emitted when a viewport (pane/editor panel) resizes.
/// Used for dynamic gizmo scaling
#[derive(Message, Clone)]
pub struct ViewportResized {
    /// Entity of the pane/viewport that resized (for multi-pane editors).
    pub pane_entity: Entity,
    /// New viewport size in pixels.
    pub size: ViewportSize,
}
