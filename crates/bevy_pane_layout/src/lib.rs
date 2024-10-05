//! Resizable, split-able panes for Bevy.
/// The Bevy Pane Layout system.
/// The intent of this system is to provide a way to create resizable, split-able panes in Bevy.
/// Mimicking the behavior of of Blender's layout system.
///
/// Blender Documentation: <https://docs.blender.org/manual/en/latest/interface/window_system/areas.html>
///
/// Requirements for a valid Pane:
/// - All panes must fit within their bounds, no overflow is allowed.
/// - Panes can not have power over the layout system, their dimensions are controlled by the layout system and should not be modified by anything else.
/// - All panes must have a header, a content area, however a footer is optional.
/// - Panes cannot have min/max sizes, they must be able to be resized to any size.
///   - If a pane can not be sensibly resized, it can overflow under the other panes.
/// - Panes must not interfere with each other, only temporary/absolute positioned elements are allowed to overlap panes.
use bevy::prelude::*;

/// The Bevy Pane Layout Plugin.
pub struct PaneLayoutPlugin;

impl Plugin for PaneLayoutPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, pane_layout_setup.in_set(PaneLayoutSet));
    }
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct PaneLayoutSet;

/// The setup system for the Pane Layout.
fn pane_layout_setup(mut commands: Commands, root: Query<Entity, With<RootPaneLayoutNode>>) {
    // All Panes exist as children of this Node.
    commands.entity(root.single()).insert(NodeBundle {
        style: Style {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            display: Display::Grid,
            grid_template_columns: vec![GridTrack::percent(100.0)],
            grid_template_rows: vec![GridTrack::percent(100.0)],
            ..Default::default()
        },

        ..Default::default()
    });
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
