//! Resizable, draggable, collapsible and dockable panes for Bevy.
use bevy::prelude::*;

/// Basic add function, will be removed later.
pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}

/// Root node to capture all editor UI elements, nothing but the layout system should modify this.
#[derive(Component)]
pub struct RootPaneLayoutNode;

/// Root node for each pane, holds all event nodes for layout and the basic structure for all Panes.
#[derive(Component)]
pub struct PaneRootNode;

/// Node to denote the content area of the Pane.
#[derive(Component)]
pub struct PaneAreaNode;

/// Node to add widgets into the header of a Pane.
#[derive(Component)]
pub struct PaneHeaderNode;

/// Button to open up Pane selection menu.
#[derive(Component)]
pub struct PaneMenuButtonNode;

/// Represents the corners and holds an observer to split/merge panes.
#[derive(Component)]
pub struct PaneSplitterNode;

/// Resizing bar for top of the pane.
#[derive(Component)]
pub struct TopResizeBarNode;

/// Resizing bar for bottom of the pane.
#[derive(Component)]
pub struct BottomResizeBarNode;

/// Left hand resize bar.
#[derive(Component)]
pub struct LeftResizeBarNode;

/// Right hand resize bar.
#[derive(Component)]
pub struct RightResizeBarNode;
