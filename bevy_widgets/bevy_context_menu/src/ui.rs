use bevy::{prelude::*, window::SystemCursorIcon, winit::cursor::CursorIcon};
use bevy_editor_styles::Theme;

use crate::ContextMenu;

pub(crate) fn spawn_context_menu<'a>(
    commands: &'a mut Commands,
    theme: &Theme,
    menu: &ContextMenu,
    position: Vec2,
    target: Entity,
) -> EntityCommands<'a> {
    let root = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(position.y),
                left: Val::Px(position.x),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(3.)),
                width: Val::Px(300.),
                ..default()
            },
            BoxShadow::from(ShadowStyle {
                blur_radius: Val::Px(3.),
                x_offset: Val::ZERO,
                y_offset: Val::ZERO,
                color: Color::BLACK.with_alpha(0.8),
                ..Default::default()
            }),
            theme.context_menu.background_color,
            theme.general.border_radius,
        ))
        .id();

    for (i, option) in menu.options.iter().enumerate() {
        spawn_option(commands, theme, &option.label, i, target).insert(ChildOf(root));
    }

    commands.entity(root)
}

pub(crate) fn spawn_option<'a>(
    commands: &'a mut Commands,
    theme: &Theme,
    label: &String,
    index: usize,
    target: Entity,
) -> EntityCommands<'a> {
    let root = commands
        .spawn((
            Node {
                padding: UiRect::all(Val::Px(5.)),
                flex_grow: 1.,
                ..default()
            },
            theme.context_menu.option_border_radius,
        ))
        .observe(
            |trigger: Trigger<Pointer<Over>>,
             theme: Res<Theme>,
             mut query: Query<&mut BackgroundColor>| {
                *query.get_mut(trigger.target()).unwrap() = theme.context_menu.hover_color;
            },
        )
        .observe(
            |trigger: Trigger<Pointer<Out>>, mut query: Query<&mut BackgroundColor>| {
                query.get_mut(trigger.target()).unwrap().0 = Color::NONE;
            },
        )
        .observe(
            move |_trigger: Trigger<Pointer<Over>>,
                  window_query: Query<Entity, With<Window>>,
                  mut commands: Commands| {
                let window = window_query.single().unwrap();
                commands
                    .entity(window)
                    .insert(CursorIcon::System(SystemCursorIcon::Pointer));
            },
        )
        .observe(
            |_trigger: Trigger<Pointer<Out>>,
             window_query: Query<Entity, With<Window>>,
             mut commands: Commands| {
                let window = window_query.single().unwrap();
                commands
                    .entity(window)
                    .insert(CursorIcon::System(SystemCursorIcon::Default));
            },
        )
        .observe(
            move |trigger: Trigger<Pointer<Released>>,
                  mut commands: Commands,
                  child_of_query: Query<&ChildOf>,
                  mut query: Query<&mut ContextMenu>| {
                if trigger.event().button != PointerButton::Primary {
                    return;
                }
                // Despawn the context menu when an option is selected
                let root = child_of_query
                    .iter_ancestors(trigger.target())
                    .last()
                    .unwrap();
                commands.entity(root).despawn();

                // Run the option callback
                let callback = &mut query.get_mut(target).unwrap().options[index].f;
                (callback)(commands.reborrow(), target);
            },
        )
        .id();

    commands.spawn((
        Text::new(label),
        TextFont {
            font: theme.text.font.clone(),
            font_size: 12.,
            ..default()
        },
        Pickable::IGNORE,
        ChildOf(root),
    ));

    commands.entity(root)
}
