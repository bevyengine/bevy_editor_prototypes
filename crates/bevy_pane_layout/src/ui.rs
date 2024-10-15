use bevy::{prelude::*, window::SystemCursorIcon, winit::cursor::CursorIcon};
use bevy_editor_styles::Theme;

use crate::{
    handlers::*, Divider, DragState, PaneAreaNode, PaneContentNode, PaneHeaderNode, PaneRootNode,
    ResizeHandle, Size,
};

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
            NodeBundle {
                style: Style {
                    padding: UiRect::all(Val::Px(1.5)),
                    ..default()
                },
                ..default()
            },
            Size(size),
            PaneRootNode { name: name.clone() },
        ))
        .observe(on_pane_drag_drop)
        .id();

    // Area
    let area = commands
        .spawn((
            NodeBundle {
                background_color: theme.pane_area_background_color,
                border_radius: theme.border_radius,
                style: Style {
                    overflow: Overflow::clip(),
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                ..default()
            },
            PaneAreaNode,
        ))
        .set_parent(root)
        .id();

    // Header
    commands
        .spawn((
            NodeBundle {
                background_color: theme.pane_header_background_color,
                border_radius: theme.pane_header_border_radius,
                style: Style {
                    padding: UiRect::axes(Val::Px(5.), Val::Px(3.)),
                    width: Val::Percent(100.),
                    height: Val::Px(27.),
                    align_items: AlignItems::Center,
                    ..default()
                },
                ..default()
            },
            PaneHeaderNode,
        ))
        .observe(on_pane_header_right_click)
        .observe(on_pane_header_middle_click)
        .observe(
            move |_trigger: Trigger<Pointer<Move>>,
                  window_query: Query<Entity, With<Window>>,
                  mut commands: Commands| {
                let window = window_query.single();
                commands
                    .entity(window)
                    .insert(CursorIcon::System(SystemCursorIcon::Pointer));
            },
        )
        .observe(
            |_trigger: Trigger<Pointer<Out>>,
             window_query: Query<Entity, With<Window>>,
             mut commands: Commands| {
                let window = window_query.single();
                commands
                    .entity(window)
                    .insert(CursorIcon::System(SystemCursorIcon::Default));
            },
        )
        .set_parent(area)
        .with_child((
            Text::new(name),
            TextFont {
                font_size: 14.,
                ..default()
            },
        ));

    // Content
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    flex_grow: 1.,
                    ..default()
                },
                ..default()
            },
            PaneContentNode,
        ))
        .set_parent(area);

    commands.entity(root)
}

pub(crate) fn spawn_divider<'a>(
    commands: &'a mut Commands,
    divider: Divider,
    size: f32,
) -> EntityCommands<'a> {
    commands.spawn((
        NodeBundle {
            style: Style {
                flex_direction: match divider {
                    Divider::Horizontal => FlexDirection::Row,
                    Divider::Vertical => FlexDirection::Column,
                },
                ..default()
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
    let mut ec = commands.spawn(NodeBundle {
        style: Style {
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
        z_index: ZIndex(3),
        ..default()
    });
    // Add the Resize
    ec.with_child((
        NodeBundle {
            style: Style {
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
            ..default()
        },
        ResizeHandle,
    ))
    .observe(
        move |trigger: Trigger<Pointer<DragStart>>,
              mut drag_state: ResMut<DragState>,
              parent_query: Query<&Parent>,
              children_query: Query<&Children>,
              node_query: Query<&Node>,
              size_query: Query<&Size>| {
            drag_state.is_dragging = true;

            let target = trigger.entity();
            let parent = parent_query.get(target).unwrap().get();

            let parent_node_size = node_query.get(parent).unwrap().size();
            let parent_node_size = match divider_parent {
                Divider::Horizontal => parent_node_size.x,
                Divider::Vertical => parent_node_size.y,
            };

            let siblings = children_query.get(parent).unwrap();
            // Find the index of this handle among its siblings
            let index = siblings
                .iter()
                .position(|entity| *entity == target)
                .unwrap();

            let size_a = size_query.get(siblings[index - 1]).unwrap().0;
            let size_b = size_query.get(siblings[index + 1]).unwrap().0;

            drag_state.offset = 0.;
            drag_state.min = -size_a * parent_node_size;
            drag_state.max = size_b * parent_node_size;
            drag_state.parent_node_size = parent_node_size;
        },
    )
    .observe(
        move |trigger: Trigger<Pointer<Drag>>,
              mut drag_state: ResMut<DragState>,
              parent_query: Query<&Parent>,
              children_query: Query<&Children>,
              mut size_query: Query<&mut Size>| {
            let target = trigger.entity();
            let parent = parent_query.get(target).unwrap().get();
            let siblings = children_query.get(parent).unwrap();
            // Find the index of this handle among its siblings
            let index = siblings
                .iter()
                .position(|entity| *entity == target)
                .unwrap();

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
        move |_trigger: Trigger<Pointer<DragEnd>>, mut drag_state: ResMut<DragState>| {
            drag_state.is_dragging = false;
            drag_state.offset = 0.;
        },
    )
    .observe(
        |_trigger: Trigger<Pointer<Cancel>>, mut drag_state: ResMut<DragState>| {
            drag_state.is_dragging = false;
            drag_state.offset = 0.;
        },
    )
    .observe(
        move |_trigger: Trigger<Pointer<Move>>,
              window_query: Query<Entity, With<Window>>,
              mut commands: Commands| {
            let window = window_query.single();
            commands
                .entity(window)
                .insert(CursorIcon::System(match divider_parent {
                    Divider::Horizontal => SystemCursorIcon::EwResize,
                    Divider::Vertical => SystemCursorIcon::NsResize,
                }));
        },
    )
    .observe(
        |_trigger: Trigger<Pointer<Out>>,
         window_query: Query<Entity, With<Window>>,
         mut commands: Commands| {
            let window = window_query.single();
            commands
                .entity(window)
                .insert(CursorIcon::System(SystemCursorIcon::Default));
        },
    );
    ec
}
