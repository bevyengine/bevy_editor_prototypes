use bevy::{prelude::*, window::SystemCursorIcon, winit::cursor::CursorIcon};

use crate::{Divider, DragState, ResizeHandle, Size};

pub(crate) fn spawn_resize_handle<'a>(
    commands: &'a mut Commands,
    divider_parent: Divider,
) -> EntityCommands<'a> {
    const SIZE: f32 = 7.;
    // Add a root node with zero size along the divider axis to avoid messing up the layout
    let mut ec = commands.spawn((
        Name::new("Resize Handle"),
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
    ))
    .observe(
        move |trigger: Trigger<Pointer<DragStart>>,
              mut drag_state: ResMut<DragState>,
              parent_query: Query<&Parent>,
              children_query: Query<&Children>,
              computed_node_query: Query<&ComputedNode>,
              size_query: Query<&Size>| {
            if trigger.event().button != PointerButton::Primary {
                return;
            }

            drag_state.is_dragging = true;

            let target = trigger.entity();
            let parent = parent_query.get(target).unwrap().get();

            let parent_node_size = computed_node_query.get(parent).unwrap().size();
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

            let min_pane_size = 20.;
            drag_state.offset = 0.;
            drag_state.min = (-size_a * parent_node_size) + min_pane_size;
            drag_state.max = (size_b * parent_node_size) - min_pane_size;
            drag_state.parent_node_size = parent_node_size;
        },
    )
    .observe(
        move |trigger: Trigger<Pointer<Drag>>,
              mut drag_state: ResMut<DragState>,
              parent_query: Query<&Parent>,
              children_query: Query<&Children>,
              mut size_query: Query<&mut Size>| {
            if !drag_state.is_dragging {
                return;
            }

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
