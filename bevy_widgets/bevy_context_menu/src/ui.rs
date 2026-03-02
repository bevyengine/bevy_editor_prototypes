use bevy::{feathers::cursor::EntityCursor, prelude::*, window::SystemCursorIcon};
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
            EntityCursor::System(SystemCursorIcon::Pointer),
        ))
        .observe(
            |trigger: On<Pointer<Over>>,
             theme: Res<Theme>,
             mut query: Query<&mut BackgroundColor>| {
                *query.get_mut(trigger.event().event_target()).unwrap() =
                    theme.context_menu.hover_color;
            },
        )
        .observe(
            |trigger: On<Pointer<Out>>, mut query: Query<&mut BackgroundColor>| {
                query.get_mut(trigger.event().event_target()).unwrap().0 = Color::NONE;
            },
        )
        .observe(
            move |trigger: On<Pointer<Release>>,
                  mut commands: Commands,
                  child_of_query: Query<&ChildOf>,
                  mut query: Query<&mut ContextMenu>| {
                if trigger.event().button != PointerButton::Primary {
                    return;
                }
                // Despawn the context menu when an option is selected
                let root = child_of_query
                    .iter_ancestors(trigger.event().event_target())
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
