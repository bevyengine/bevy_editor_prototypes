use bevy::{feathers::cursor::EntityCursor, prelude::*, window::SystemCursorIcon};
use bevy_context_menu::{ContextMenu, ContextMenuOption};
use bevy_editor_styles::{Theme, icons};

use crate::{
    Divider, DragState, PaneAreaNode, PaneContentNode, PaneHeaderNode, PaneRootNode, ResizeHandle,
    Size, handlers::*, registry::PaneStructure,
};

pub fn header_context_menu() -> ContextMenu {
    ContextMenu::new([
        ContextMenuOption::new("Close", |mut commands, entity| {
            commands.run_system_cached_with(remove_pane, entity);
        }),
        ContextMenuOption::new("Split - Horizontal", |mut commands, entity| {
            commands.run_system_cached_with(split_pane, (entity, false));
        }),
        ContextMenuOption::new("Split - Vertical", |mut commands, entity| {
            commands.run_system_cached_with(split_pane, (entity, true));
        }),
    ])
}

pub(crate) fn spawn_pane<'a>(
    commands: &'a mut Commands,
    theme: &Theme,
    size: f32,
    name: impl Into<String>,
) -> EntityCommands<'a> {
    let name: String = name.into();
    // Unstyled root node
    let root = commands
        .spawn((
            Node {
                padding: UiRect::all(Val::Px(1.5)),
                ..default()
            },
            Size(size),
            PaneRootNode { name: name.clone() },
        ))
        .id();

    // Area
    let area = commands
        .spawn((
            Node {
                overflow: Overflow::clip(),
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            PaneAreaNode,
            theme.pane.area_background_color,
            theme.general.border_radius,
            ChildOf(root),
        ))
        .id();

    // Header
    let header = commands
        .spawn((
            Node {
                padding: UiRect::axes(Val::Px(5.), Val::Px(3.)),
                width: Val::Percent(100.),
                height: Val::Px(27.),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                flex_shrink: 0.,
                ..default()
            },
            theme.pane.header_background_color,
            theme.pane.header_border_radius,
            header_context_menu(),
            PaneHeaderNode,
            ChildOf(area),
            EntityCursor::System(SystemCursorIcon::Pointer),
        ))
        .with_children(|parent| {
            parent
                .spawn(Node {
                    align_items: AlignItems::Center,
                    flex_shrink: 0.0,
                    ..default()
                })
                .with_children(|parent| {
                    // Drop down button for selecting the pane type.
                    // Once a drop down menu is implemented, this will have that added.
                    parent.spawn((
                        Text::new(icons::CHEVRON_DOWN),
                        TextFont {
                            font: theme.icon.font.clone(),
                            font_size: 16.0,
                            ..default()
                        },
                    ));
                    parent.spawn((
                        Text::new(format!(" {name}")),
                        TextFont {
                            font: theme.text.font.clone(),
                            font_size: 14.0,
                            ..default()
                        },
                    ));
                });

            parent.spawn((
                Text::new(icons::GRIP_VERTICAL),
                TextFont {
                    font: theme.icon.font.clone(),
                    font_size: 16.0,
                    ..default()
                },
            ));
        })
        .id();

    // Content
    let content = commands
        .spawn((
            Node {
                flex_grow: 1.,
                ..default()
            },
            PaneContentNode,
            ChildOf(area),
        ))
        .id();

    commands.entity(root).insert(PaneStructure {
        root,
        area,
        header,
        content,
    });

    commands.entity(root)
}

pub(crate) fn spawn_divider<'a>(
    commands: &'a mut Commands,
    divider: Divider,
    size: f32,
) -> EntityCommands<'a> {
    commands.spawn((
        Node {
            flex_direction: match divider {
                Divider::Horizontal => FlexDirection::Row,
                Divider::Vertical => FlexDirection::Column,
            },
            ..default()
        },
        Size(size),
        divider,
    ))
}

pub(crate) fn spawn_resize_handle<'a>(
    commands: &'a mut Commands,
    divider_parent: Divider,
) -> EntityCommands<'a> {
    const SIZE: f32 = 7.;
    // Add a root node with zero size along the divider axis to avoid messing up the layout
    let mut ec = commands.spawn((
        Node {
            width: match divider_parent {
                Divider::Horizontal => Val::Px(SIZE),
                Divider::Vertical => Val::Percent(100.),
            },
            height: match divider_parent {
                Divider::Horizontal => Val::Percent(100.),
                Divider::Vertical => Val::Px(SIZE),
            },
            // Abuse negative margins because setting width to zero is causing the child to be clipped
            // presumably because of a bug in bevy_ui
            margin: match divider_parent {
                Divider::Horizontal => UiRect::horizontal(Val::Px(-SIZE / 2.)),
                Divider::Vertical => UiRect::vertical(Val::Px(-SIZE / 2.)),
            },
            ..default()
        },
        ZIndex(3),
    ));
    // Add the Resize
    let cursor_icon = match divider_parent {
        Divider::Horizontal => SystemCursorIcon::EwResize,
        Divider::Vertical => SystemCursorIcon::NsResize,
    };
    ec.with_child((
        Node {
            width: match divider_parent {
                Divider::Horizontal => Val::Px(SIZE),
                Divider::Vertical => Val::Percent(100.),
            },
            height: match divider_parent {
                Divider::Horizontal => Val::Percent(100.),
                Divider::Vertical => Val::Px(SIZE),
            },
            ..default()
        },
        ResizeHandle,
        EntityCursor::System(cursor_icon),
    ))
    .observe(
        move |trigger: On<Pointer<DragStart>>,
              mut drag_state: ResMut<DragState>,
              parent_query: Query<&ChildOf>,
              children_query: Query<&Children>,
              computed_node_query: Query<&ComputedNode>,
              size_query: Query<&Size>| {
            if trigger.event().button != PointerButton::Primary {
                return;
            }

            drag_state.is_dragging = true;

            let target = trigger.event().event_target();
            let parent = parent_query.get(target).unwrap().parent();

            let parent_node_size = computed_node_query.get(parent).unwrap().size();
            let parent_node_size = match divider_parent {
                Divider::Horizontal => parent_node_size.x,
                Divider::Vertical => parent_node_size.y,
            };

            let siblings = children_query.get(parent).unwrap();
            // Find the index of this handle among its siblings
            let index = siblings.iter().position(|entity| entity == target).unwrap();

            let size_a = size_query.get(siblings[index - 1]).unwrap().0;
            let size_b = size_query.get(siblings[index + 1]).unwrap().0;

            let min_pane_size = 20.;
            drag_state.offset = 0.;
            drag_state.min = (-size_a * parent_node_size) + min_pane_size;
            drag_state.max = (size_b * parent_node_size) - min_pane_size;
            drag_state.parent_node_size = parent_node_size;
        },
    )
    .observe(
        move |trigger: On<Pointer<Drag>>,
              mut drag_state: ResMut<DragState>,
              parent_query: Query<&ChildOf>,
              children_query: Query<&Children>,
              mut size_query: Query<&mut Size>| {
            if !drag_state.is_dragging {
                return;
            }

            let target = trigger.event().event_target();
            let parent = parent_query.get(target).unwrap().parent();
            let siblings = children_query.get(parent).unwrap();
            // Find the index of this handle among its siblings
            let index = siblings.iter().position(|entity| entity == target).unwrap();

            let delta = trigger.event().delta;
            let delta = match divider_parent {
                Divider::Horizontal => delta.x,
                Divider::Vertical => delta.y,
            };

            let previous_offset = drag_state.offset;

            drag_state.offset += delta;

            drag_state.offset = drag_state.offset.clamp(drag_state.min, drag_state.max);

            let clamped_delta = drag_state.offset - previous_offset;

            size_query.get_mut(siblings[index - 1]).unwrap().0 +=
                clamped_delta / drag_state.parent_node_size;
            size_query.get_mut(siblings[index + 1]).unwrap().0 -=
                clamped_delta / drag_state.parent_node_size;
        },
    )
    .observe(
        move |_trigger: On<Pointer<DragEnd>>, mut drag_state: ResMut<DragState>| {
            drag_state.is_dragging = false;
            drag_state.offset = 0.;
        },
    )
    .observe(
        |_trigger: On<Pointer<Cancel>>, mut drag_state: ResMut<DragState>| {
            drag_state.is_dragging = false;
            drag_state.offset = 0.;
        },
    );
    ec
}
