//! A scroll widget for Bevy applications.

use bevy::{
    a11y::{
        accesskit::{NodeBuilder, Role},
        AccessibilityNode,
    },
    input::mouse::{MouseScrollUnit, MouseWheel},
    prelude::*,
    ui::RelativeCursorPosition,
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

/// A `ScrollBox` is a UI component that allows for content to be scrolled within a defined area.
#[derive(Component, Default)]
#[require(Node)]
pub struct ScrollBox(ScrollPosition);

/// Represents the content within a [`ScrollBox`].
///
/// This [`Node`] can of any size and will be clipped to the size of the [`ScrollBox`].
/// Unless specified otherwise, any content overflowing will be accessible via the [`ScrollBar`].
#[derive(Component, Default)]
#[require(Node)]
pub struct ScrollBoxContent;

/// A component representing a scroll bar in a [`ScrollBox`].
///
/// This component is used to visually indicate the scrollable area within a [`ScrollBoxContent`].
/// It contains a [`ScrollBarHandle`] which represents the draggable part of the scroll bar.
#[derive(Component, Default)]
#[require(Node)]
pub struct ScrollBar;

/// A component representing the handle of a scroll bar.
///
/// This component is used to visually indicate the current scroll position within a [`ScrollBox`].
/// It is a child of a [`ScrollBar`] and can be dragged to scroll the [`ScrollBoxContent`].
/// Scroll bar can also be moved using the mouse wheel, or shift + mouse wheel if the [`ScrollBar`] is horizontal.
#[derive(Component, Default)]
#[require(Node)]
pub struct ScrollBarHandle;

/// Spawn a new [`ScrollBox`]
pub fn spawn_scroll_box<'a>(
    parent: &'a mut ChildBuilder,
    theme: &Res<Theme>,
    populate_content: Option<impl FnOnce(&mut EntityCommands)>,
) -> EntityCommands<'a> {
    let mut scrollbox_ec = parent.spawn((
        ScrollBox::default(),
        RelativeCursorPosition::default(),
        Node {
            display: Display::Grid,
            grid_template_rows: vec![GridTrack::flex(1.0), GridTrack::auto()],
            grid_template_columns: vec![GridTrack::flex(1.0), GridTrack::auto()],
            overflow: Overflow::clip_y(),
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
    ));
    scrollbox_ec.with_children(|parent| {
        let mut scrollbox_content_ec = parent.spawn((
            ScrollBoxContent::default(),
            Node {
                grid_column: GridPlacement::start(1),
                grid_row: GridPlacement::start(1),
                position_type: PositionType::Absolute,
                flex_wrap: FlexWrap::Wrap,
                ..default()
            },
            BackgroundColor(Color::srgb(0.5, 0.5, 0.0)),
        ));
        if let Some(populate_content) = populate_content {
            populate_content(&mut scrollbox_content_ec);
        }

        parent
            .spawn((
                ScrollBar,
                Node {
                    grid_column: GridPlacement::start(2),
                    grid_row: GridPlacement::start(1),
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
                        height: Val::Percent(0.0),
                        ..default()
                    },
                    BackgroundColor(theme.scroll_bar_handle_color),
                    BorderRadius::all(Val::Px(5.0)),
                ));
            });
    });
    scrollbox_ec
}

fn on_scroll(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut query_scrollbox: Query<(&RelativeCursorPosition, Entity, &mut ScrollBox, &Children)>,
    mut query_scrollbox_content: Query<(&mut Node, &ComputedNode), With<ScrollBoxContent>>,
    query_computed_node: Query<&ComputedNode>,
) {
    for mouse_wheel_event in mouse_wheel_events.read() {
        for (cursor_pos, scrollbox_entity, mut scrollbox, children) in
            &mut query_scrollbox.iter_mut()
        {
            // Only scroll the ScrollBox that the cursor is over
            // TODO: Get the scrollbox with the highest z-index
            if !cursor_pos.mouse_over() {
                continue;
            }

            let (mut content, content_computed) = query_scrollbox_content
                .get_mut(children[0])
                .expect("Scrollbox children 0 should be a ScrollBoxContent");
            let content_height = content_computed.size().y;
            let scrollbox_height = query_computed_node.get(scrollbox_entity).unwrap().size().y;
            let max_scroll = (content_height - scrollbox_height).max(0.);
            let delta_y = match mouse_wheel_event.unit {
                MouseScrollUnit::Line => mouse_wheel_event.y * SCROLL_LINE_SIZE_VALUE,
                MouseScrollUnit::Pixel => mouse_wheel_event.y,
            };
            scrollbox.0.offset_y += delta_y;
            scrollbox.0.offset_y = scrollbox.0.offset_y.clamp(-max_scroll, 0.);
            content.top = Val::Px(scrollbox.0.offset_y);

            return; // We only want to scroll 1 ScrollBox
        }
    }
}

// TODO: try Changed<ComputedNode>
fn update_scroll_bars(
    query_scrollboxes: Query<(&ScrollBox, &ComputedNode, &Children), With<ScrollBox>>,
    query_scrollbox_content: Query<&ComputedNode, With<ScrollBoxContent>>,
    mut query_scrollbar_components: ParamSet<(
        Query<&mut Node, With<ScrollBar>>,
        Query<&mut Node, With<ScrollBarHandle>>,
    )>,
) {
    for (scrollbox, scrollbox_computed, scrollbox_children) in query_scrollboxes.iter() {
        let content_computed = query_scrollbox_content
            .get(scrollbox_children[0])
            .expect("Scrollbox children 0 should be a ScrollBoxContent");
        let content_height = content_computed.size().y;
        let scrollbox_height = scrollbox_computed.size().y;
        let handle_height = (scrollbox_height / content_height * 100.0).clamp(5.0, 100.0);
        let handle_position = (-scrollbox.0.offset_y / content_height * 100.0).clamp(0.0, 100.0);

        {
            let mut query_scrollbars = query_scrollbar_components.p0();
            let mut scrollbar = query_scrollbars.single_mut();
            if handle_height == 100.0 {
                scrollbar.display = Display::None;
                return;
            }
            scrollbar.display = Display::DEFAULT;
        }

        {
            let mut query_scrollbar_handle = query_scrollbar_components.p1();
            let mut scrollbar_handle = query_scrollbar_handle.single_mut();
            scrollbar_handle.height = Val::Percent(handle_height);
            scrollbar_handle.top = Val::Percent(handle_position);
        }
    }
}
