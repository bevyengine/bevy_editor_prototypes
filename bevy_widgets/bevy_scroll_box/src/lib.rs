//! A scroll widget for Bevy applications.

use bevy::{
    a11y::{
        accesskit::{NodeBuilder, Role},
        AccessibilityNode,
    },
    input::mouse::{MouseScrollUnit, MouseWheel},
    prelude::*,
};
use bevy_editor_styles::Theme;

/// The plugin that handle all the scroll boxes.
pub struct ScrollBoxPlugin;

const SCROLL_LINE_SIZE_VALUE: f32 = 20.0;

impl Plugin for ScrollBoxPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, on_scroll)
            .add_systems(Update, update_scroll_bars);
    }
}

/// The box that's scrollable. (The content of the scroll box)
#[derive(Component, Default)]
#[require(Node)]
pub struct ScrollBox {
    /// The position of the scroll box, relative to the it's container. (`position` < 0)
    pub position: f32,
}

/// The scroll bar of the scroll box.
#[derive(Component, Default)]
#[require(Node)]
pub struct ScrollBar;

/// The handle of the scroll bar.
#[derive(Component, Default)]
#[require(Node)]
pub struct ScrollBarHandle;

/// Spawn and attach a scroll box to the parent.
pub fn spawn_scroll_box<'a>(
    parent: &'a mut ChildBuilder,
    theme: &Res<Theme>,
) -> EntityCommands<'a> {
    let mut commands = parent.spawn(Node {
        flex_direction: FlexDirection::RowReverse,
        overflow: Overflow::clip_y(),
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        ..default()
    });
    commands.with_children(|parent| {
        parent.spawn((
            ScrollBox::default(),
            AccessibilityNode(NodeBuilder::new(Role::List)),
            Node {
                position_type: PositionType::Absolute,
                flex_wrap: FlexWrap::Wrap,
                ..default()
            },
        ));
        parent
            .spawn((
                ScrollBar,
                Node {
                    width: Val::Px(10.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(theme.scroll_bar_color),
                BorderRadius::all(Val::Px(5.0)),
            ))
            .with_children(|parent| {
                parent.spawn((
                    ScrollBarHandle,
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(theme.scroll_bar_handle_color),
                    BorderRadius::all(Val::Px(5.0)),
                ));
            });
    });
    commands
}

fn on_scroll(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut query_list: Query<(&mut ScrollBox, &mut Node, &Parent, &ComputedNode)>,
    query_computed_node: Query<&ComputedNode>,
) {
    for mouse_wheel_event in mouse_wheel_events.read() {
        for (mut scrolling_list, mut list_node, container, list_computed_node) in &mut query_list {
            let list_height = list_computed_node.size().y;
            let container_height = query_computed_node.get(container.get()).unwrap().size().y;
            let max_scroll = (list_height - container_height).max(0.);
            let delta_y = match mouse_wheel_event.unit {
                MouseScrollUnit::Line => mouse_wheel_event.y * SCROLL_LINE_SIZE_VALUE,
                MouseScrollUnit::Pixel => mouse_wheel_event.y,
            };
            scrolling_list.position += delta_y;
            scrolling_list.position = scrolling_list.position.clamp(-max_scroll, 0.);
            list_node.top = Val::Px(scrolling_list.position);
        }
    }
}

fn update_scroll_bars(
    query_list: Query<(&ScrollBox, &Parent, &ComputedNode)>,
    mut query_scroll_bar_style: ParamSet<(
        Query<&mut Node, With<ScrollBar>>,
        Query<&mut Node, With<ScrollBarHandle>>,
    )>,
    query_computed_node: Query<&ComputedNode>,
) {
    for (scrolling_list, container, list_computed_node) in query_list.iter() {
        let list_height = list_computed_node.size().y;
        let container_height = query_computed_node.get(container.get()).unwrap().size().y;
        let handle_height = (container_height / list_height * 100.0).clamp(5.0, 100.0);
        let handle_position = (-scrolling_list.position / list_height * 100.0).clamp(0.0, 100.0);

        {
            let mut bar_style_query = query_scroll_bar_style.p0();
            let mut bar_node = bar_style_query.single_mut();
            if handle_height == 100.0 {
                bar_node.display = Display::None;
                return;
            }
            bar_node.display = Display::DEFAULT;
        }

        {
            let mut handle_node_query = query_scroll_bar_style.p1();
            let mut handle_node = handle_node_query.single_mut();

            handle_node.height = Val::Percent(handle_height);
            handle_node.top = Val::Percent(handle_position);
        }
    }
}
