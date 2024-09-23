//! Resizable, draggable, collapsible and dockable panes for Bevy.
use bevy::prelude::*;

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

#[derive(Component)]
pub struct RootPaneLayoutNode;

#[derive(Component)]
pub struct PaneRootNode;

#[derive(Component)]
pub struct PaneAreaNode;

#[derive(Component)]
pub struct PaneHeaderNode;

#[derive(Component)]
pub struct PaneMenuButtonNode;

#[derive(Component)]
pub struct PaneSplitterNode;

#[derive(Component)]
pub struct TopResizeBarNode;

#[derive(Component)]
pub struct BottomResizeBarNode;

#[derive(Component)]
pub struct LeftResizeBarNode;

#[derive(Component)]
pub struct RightResizeBarNode;
