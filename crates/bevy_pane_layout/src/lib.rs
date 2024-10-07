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
use bevy_editor_styles::Theme;

use bevy_3d_viewport::Bevy3DViewport;

/// The Bevy Pane Layout Plugin.
pub struct PaneLayoutPlugin;

impl Plugin for PaneLayoutPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, pane_layout_setup.in_set(PaneLayoutSet));
    }
}

/// System Set to set up the Pane Layout.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct PaneLayoutSet;

/// The setup system for the Pane Layout.
fn pane_layout_setup(
    mut commands: Commands,
    root: Query<Entity, With<RootPaneLayoutNode>>,
    theme: Res<Theme>,
) {
    // All Panes exist as children of this Node.
    commands
        .entity(root.single())
        .insert(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                display: Display::Grid,
                grid_template_columns: vec![GridTrack::percent(100.0)],
                grid_template_rows: vec![GridTrack::percent(100.0)],
                ..Default::default()
            },
            background_color: theme.background_color,
            ..Default::default()
        })
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        display: Display::Grid,
                        grid_template_columns: vec![
                            GridTrack::px(15.0),
                            GridTrack::auto(),
                            GridTrack::px(15.0),
                        ],
                        grid_template_rows: vec![
                            GridTrack::px(15.0),
                            GridTrack::auto(),
                            GridTrack::px(15.0),
                        ],
                        align_content: AlignContent::Stretch,
                        justify_content: JustifyContent::Stretch,
                        ..Default::default()
                    },
                    background_color: theme.background_color,
                    ..Default::default()
                })
                .insert(PaneRootNode)
                .with_children(|parent| {
                    parent
                        .spawn(NodeBundle {
                            style: Style {
                                grid_row: GridPlacement::start(2),
                                grid_column: GridPlacement::start(2),
                                display: Display::Grid,
                                grid_template_columns: vec![GridTrack::percent(100.0)],
                                grid_template_rows: vec![GridTrack::px(25.0), GridTrack::auto()],
                                ..Default::default()
                            },
                            background_color: theme.pane_area_background_color,
                            border_radius: theme.border_radius,
                            ..Default::default()
                        })
                        .insert(PaneAreaNode)
                        .with_children(|parent| {
                            parent
                                .spawn(NodeBundle {
                                    style: Style {
                                        width: Val::Percent(100.0),
                                        height: Val::Px(25.0),
                                        padding: UiRect {
                                            left: Val::Px(10.0),
                                            right: Val::Px(10.0),
                                            top: Val::Px(0.0),
                                            bottom: Val::Px(0.0),
                                        },
                                        ..Default::default()
                                    },
                                    background_color: theme.pane_background_color,
                                    border_radius: theme.pane_header_border_radius,
                                    ..Default::default()
                                })
                                .insert(PaneHeaderNode);
                            parent
                                .spawn(NodeBundle {
                                    style: Style {
                                        ..Default::default()
                                    },
                                    background_color: theme.pane_area_background_color,
                                    border_radius: theme.border_radius,
                                    ..Default::default()
                                })
                                .insert(PaneContentNode)
                                .insert(Bevy3DViewport);
                        });
                });
        });
}

/// Root node to capture all editor UI elements, nothing but the layout system should modify this.
#[derive(Component)]
pub struct RootPaneLayoutNode;

/// Root node for each pane, holds all event nodes for layout and the basic structure for all Panes.
#[derive(Component)]
pub struct PaneRootNode;

/// Node to denote the area of the Pane.
#[derive(Component)]
pub struct PaneAreaNode;

/// Node to add widgets into the header of a Pane.
#[derive(Component)]
pub struct PaneHeaderNode;

/// Node to denote the content space of the Pane.
#[derive(Component)]
pub struct PaneContentNode;

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
