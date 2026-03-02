//! A scroll widget for Bevy applications.

use bevy::{
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
        app.add_systems(Update, (on_scroll, update_scroll_box, update_scroll_bars));
    }
}

/// A `ScrollBox` is a UI component that allows for content to be scrolled within a defined area.
#[derive(Component, Default)]
#[require(Node, RelativeCursorPosition)]
pub struct ScrollBox {
    position: ScrollPosition,
    overflow: Overflow,
}

impl ScrollBox {
    /// Reset the scroll position to 0
    pub fn scroll_to_top(&mut self) {
        self.position = ScrollPosition::default();
    }
}

/// Represents the content within a [`ScrollBox`].
///
/// This [`Node`] can of any size and will be clipped to the size of the [`ScrollBox`].
/// Unless specified otherwise, any content overflowing will be accessible via the scroll bars.
#[derive(Component, Default)]
#[require(Node)]
pub struct ScrollBoxContent;

/// Determine in which direction the [`ScrollBarHandle`] is moving.
#[derive(Default)]
pub enum ScrollBarHandleDirection {
    /// Handle scroll vertically
    #[default]
    Vertical,
    /// Handle scroll horizontally
    Horizontal,
}

/// A component representing the handle of a scroll bar.
///
/// This component is used to visually indicate the current scroll position within a [`ScrollBox`].
/// It is a child of a scroll bar and can be dragged to scroll the [`ScrollBoxContent`].
/// Scroll bar can also be moved using the mouse wheel, or shift + mouse wheel if the scroll bar is horizontal.
#[derive(Component, Default)]
#[require(Node)]
pub struct ScrollBarHandle(pub ScrollBarHandleDirection);

/// Spawn a new [`ScrollBox`]
pub fn spawn_scroll_box<'a>(
    commands: &'a mut Commands,
    theme: &Res<Theme>,
    direction: Overflow,
    populate_content: Option<impl FnOnce(&mut Commands, Entity)>,
) -> EntityCommands<'a> {
    let scrollbox = commands
        .spawn((
            ScrollBox {
                position: ScrollPosition::default(),
                overflow: direction,
            },
            RelativeCursorPosition::default(),
            Node {
                display: Display::Grid,
                grid_template_rows: vec![GridTrack::flex(1.0), GridTrack::auto()],
                grid_template_columns: vec![GridTrack::flex(1.0), GridTrack::auto()],
                overflow: direction,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
        ))
        .id();

    let scrollbox_content = commands
        .spawn((
            ScrollBoxContent,
            Node {
                grid_column: GridPlacement::start(1),
                grid_row: GridPlacement::start(1),
                position_type: PositionType::Absolute,
                min_width: Val::Percent(100.0),
                min_height: Val::Percent(100.0),
                flex_wrap: if direction.x != OverflowAxis::Scroll {
                    FlexWrap::Wrap
                } else {
                    FlexWrap::default()
                },
                ..default()
            },
            ChildOf(scrollbox),
        ))
        .id();

    if let Some(populate_content) = populate_content {
        populate_content(commands, scrollbox_content);
    }

    if direction.y == OverflowAxis::Scroll {
        spawn_scroll_bar(commands, theme, ScrollBarHandleDirection::Vertical)
            .insert(ChildOf(scrollbox));
    }
    if direction.x == OverflowAxis::Scroll {
        spawn_scroll_bar(commands, theme, ScrollBarHandleDirection::Horizontal)
            .insert(ChildOf(scrollbox));
    }

    commands.entity(scrollbox)
}

fn spawn_scroll_bar<'a>(
    commands: &'a mut Commands,
    theme: &Res<Theme>,
    direction: ScrollBarHandleDirection,
) -> EntityCommands<'a> {
    let scrollbar = commands
        .spawn((
            match direction {
                ScrollBarHandleDirection::Vertical => Node {
                    grid_column: GridPlacement::start(2),
                    grid_row: GridPlacement::start(1),
                    width: Val::Px(10.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                ScrollBarHandleDirection::Horizontal => Node {
                    grid_column: GridPlacement::start(1),
                    grid_row: GridPlacement::start(2),
                    width: Val::Percent(100.0),
                    height: Val::Px(10.0),
                    ..default()
                },
            },
            theme.scroll_box.background_color,
            BorderRadius::all(Val::Px(5.0)),
        ))
        .id();

    // Handle
    commands
        .spawn((
            match direction {
                ScrollBarHandleDirection::Vertical => Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(0.0),
                    ..default()
                },
                ScrollBarHandleDirection::Horizontal => Node {
                    width: Val::Percent(0.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
            },
            ScrollBarHandle(direction),
            BackgroundColor(theme.scroll_box.handle_color),
            theme.scroll_box.border_radius,
            ChildOf(scrollbar),
        ))
        .observe(
            |trigger: On<Pointer<Drag>>,
             query_handle: Query<&ScrollBarHandle>,
             query_parent: Query<&ChildOf>,
             mut query_scrollbox: Query<(&mut ScrollBox, &RelativeCursorPosition, &Children)>,
             query_computed_node: Query<&ComputedNode>,
             mut query_node: Query<&mut Node>| {
                let handle_entity = trigger.event().event_target();
                let handle = query_handle.get(handle_entity).unwrap();
                let scrollbox_entity = {
                    let scrollbar_parent = query_parent.get(handle_entity).unwrap();
                    query_parent
                        .get(scrollbar_parent.parent())
                        .unwrap()
                        .parent()
                };
                let (mut scrollbox, cursor_pos, scrollbox_children) =
                    query_scrollbox.get_mut(scrollbox_entity).unwrap();
                if let Some(norm_cursor_pos) = cursor_pos.normalized {
                    let content_computed = query_computed_node
                        .get(scrollbox_children[0])
                        .expect("Scrollbox children 0 should exist and be a ScrollBoxContent");
                    let mut content_node = query_node.get_mut(scrollbox_children[0]).unwrap();
                    match handle.0 {
                        ScrollBarHandleDirection::Vertical => {
                            let content_size = content_computed.size().y;
                            let norm_handle_size = {
                                let scrollbox_size =
                                    query_computed_node.get(scrollbox_entity).unwrap().size().y;
                                query_computed_node.get(handle_entity).unwrap().size().y
                                    / scrollbox_size
                            };
                            scrollbox.position.y =
                                -((norm_cursor_pos.y - norm_handle_size / 2.0) * content_size);
                            scrollbox.position.y = scrollbox
                                .position
                                .y
                                .clamp(-(content_size - (norm_handle_size * content_size)), 0.0);
                            content_node.top = Val::Px(scrollbox.position.y);
                        }
                        ScrollBarHandleDirection::Horizontal => {
                            let content_size = content_computed.size().x;
                            let norm_handle_size = {
                                let scrollbox_size =
                                    query_computed_node.get(scrollbox_entity).unwrap().size().x;
                                query_computed_node.get(handle_entity).unwrap().size().x
                                    / scrollbox_size
                            };
                            scrollbox.position.x =
                                -((norm_cursor_pos.x - norm_handle_size / 2.0) * content_size);
                            scrollbox.position.x = scrollbox
                                .position
                                .x
                                .clamp(-(content_size - (norm_handle_size * content_size)), 0.0);
                            content_node.left = Val::Px(scrollbox.position.x);
                        }
                    }
                }
            },
        );

    commands.entity(scrollbar)
}

fn on_scroll(
    mut mouse_wheel_events: MessageReader<MouseWheel>,
    mut query_scrollbox: Query<(&RelativeCursorPosition, Entity, &mut ScrollBox, &Children)>,
    query_scrollbox_content: Query<&ComputedNode, With<ScrollBoxContent>>,
    query_computed_node: Query<&ComputedNode>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    for mouse_wheel_event in mouse_wheel_events.read() {
        for (cursor_pos, scrollbox_entity, mut scrollbox, children) in
            &mut query_scrollbox.iter_mut()
        {
            // Only scroll the ScrollBox that the cursor is over
            // TODO: Get the scrollbox with the highest z-index
            if !cursor_pos.cursor_over() {
                continue;
            }

            let scroll_delta = match mouse_wheel_event.unit {
                MouseScrollUnit::Line => mouse_wheel_event.y * SCROLL_LINE_SIZE_VALUE,
                MouseScrollUnit::Pixel => mouse_wheel_event.y,
            };

            let content_sizes = query_scrollbox_content
                .get(children[0])
                .expect("Scrollbox children 0 should be a ScrollBoxContent")
                .size();
            let scrollbox_sizes = query_computed_node.get(scrollbox_entity).unwrap().size();

            if keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight) {
                let max_scroll = (content_sizes.x - scrollbox_sizes.x).max(0.0);
                scrollbox.position.x =
                    (scrollbox.position.x + scroll_delta).clamp(-max_scroll, 0.0);
            } else {
                let max_scroll = (content_sizes.y - scrollbox_sizes.y).max(0.);
                scrollbox.position.y =
                    (scrollbox.position.y + scroll_delta).clamp(-max_scroll, 0.0);
            }

            return; // We only want to scroll 1 ScrollBox
        }
    }
}

fn update_scroll_box(
    query_scrolllboxes: Query<(&ScrollBox, &Children), Changed<ScrollBox>>,
    mut query_node: Query<&mut Node>,
) {
    for (scrollbox, children) in query_scrolllboxes.iter() {
        if scrollbox.overflow.y == OverflowAxis::Scroll {
            let scroll_content = children
                .first()
                .expect("Scrollbox children 0 should be a ScrollBoxContent");
            let mut content_node = query_node.get_mut(*scroll_content).unwrap();
            content_node.top = Val::Px(scrollbox.position.y);
        }

        if scrollbox.overflow.x == OverflowAxis::Scroll {
            let scroll_content = children
                .first()
                .expect("Scrollbox children 0 should be a ScrollBoxContent");
            let mut content_node = query_node.get_mut(*scroll_content).unwrap();
            content_node.left = Val::Px(scrollbox.position.x);
        }
    }
}

/// Recalacule the scroll bar handle position and size
fn update_scroll_bars(
    query_scrollboxes: Query<
        (&ScrollBox, &ComputedNode, &Children),
        (
            With<ScrollBox>,
            Or<(Changed<ComputedNode>, Changed<ScrollBox>)>,
        ),
    >,
    query_scrollbox_content: Query<&ComputedNode, With<ScrollBoxContent>>,
    query_children: Query<&Children>,
    mut query_node: Query<&mut Node>,
) {
    for (scrollbox, scrollbox_computed, scrollbox_children) in query_scrollboxes.iter() {
        let content_children = query_scrollbox_content
            .get(scrollbox_children[0])
            .expect("Scrollbox children 0 should be a ScrollBoxContent");

        if scrollbox.overflow.y == OverflowAxis::Scroll {
            let scrollbar_children = query_children.get(scrollbox_children[1]).expect(
                "Scrollbox children 1 should be a ScrollBar and have 1 child (ScrollBarHandle)",
            );

            let content_height = content_children.size().y;
            let scrollbox_height = scrollbox_computed.size().y;
            let (handle_height, handle_pos) = if content_height == 0.0 {
                (100.0, 0.0)
            } else {
                let height = (scrollbox_height / content_height * 100.0).clamp(5.0, 100.0);
                let pos = (-scrollbox.position.y / content_height * 100.0).clamp(0.0, 100.0);
                (height, pos)
            };

            {
                let mut scrollbar_node = query_node.get_mut(scrollbox_children[1]).unwrap();
                if handle_height == 100.0 {
                    scrollbar_node.display = Display::None;
                    continue;
                }
                scrollbar_node.display = Display::DEFAULT;
            }

            {
                let mut handle_node = query_node
                    .get_mut(scrollbar_children[0])
                    .expect("ScrollBar should have 1 child (ScrollBarHandle)");
                handle_node.height = Val::Percent(handle_height);
                handle_node.top = Val::Percent(handle_pos);
            }
        }

        if scrollbox.overflow.x == OverflowAxis::Scroll {
            let scrollbar_children = query_children
                .get(
                    scrollbox_children[if scrollbox.overflow.y == OverflowAxis::Scroll {
                        2
                    } else {
                        1
                    }],
                )
                .expect(
                    "Scrollbox children 2 should be a ScrollBar and have 1 child (ScrollBarHandle)",
                );

            let content_width = content_children.size().x;
            let scrollbox_width = scrollbox_computed.size().x;
            let (handle_width, handle_pos) = if content_width == 0.0 {
                (100.0, 0.0)
            } else {
                let width = (scrollbox_width / content_width * 100.0).clamp(5.0, 100.0);
                let pos = (-scrollbox.position.x / content_width * 100.0).clamp(0.0, 100.0);
                (width, pos)
            };

            {
                let mut scrollbar_node = query_node.get_mut(scrollbox_children[1]).unwrap();
                if handle_width == 100.0 {
                    scrollbar_node.display = Display::None;
                    continue;
                }
                scrollbar_node.display = Display::DEFAULT;
            }

            {
                let mut handle_node = query_node
                    .get_mut(scrollbar_children[0])
                    .expect("ScrollBar should have 1 child (ScrollBarHandle)");
                handle_node.width = Val::Percent(handle_width);
                handle_node.left = Val::Percent(handle_pos);
            }
        }
    }
}
