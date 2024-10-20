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
            BoxShadow {
                blur_radius: Val::Px(3.),
                x_offset: Val::ZERO,
                y_offset: Val::ZERO,
                color: Color::BLACK.with_alpha(0.8),
                ..Default::default()
            },
            theme.context_menu_background_color,
            theme.border_radius,
        ))
        .id();

    for (i, option) in menu.options.iter().enumerate() {
        spawn_option(commands, theme, &option.label, i, target).set_parent(root);
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
            theme.button_border_radius,
        ))
        .observe(
            |trigger: Trigger<Pointer<Over>>,
             theme: Res<Theme>,
             mut query: Query<&mut BackgroundColor>| {
                *query.get_mut(trigger.entity()).unwrap() =
                    theme.context_menu_button_hover_background_color;
            },
        )
        .observe(
            |trigger: Trigger<Pointer<Out>>, mut query: Query<&mut BackgroundColor>| {
                query.get_mut(trigger.entity()).unwrap().0 = Color::NONE;
            },
        )
        .observe(
            move |_trigger: Trigger<Pointer<Over>>,
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
        .observe(
            move |trigger: Trigger<Pointer<Up>>,
                  mut commands: Commands,
                  parent_query: Query<&Parent>,
                  mut query: Query<&mut ContextMenu>| {
                // Despawn the context menu when an option is selected
                let root = parent_query
                    .iter_ancestors(trigger.entity())
                    .last()
                    .unwrap();
                commands.entity(root).despawn_recursive();

                // Run the option callback
                let callback = &mut query.get_mut(target).unwrap().options[index].f;
                (callback)(commands.reborrow(), target);
            },
        )
        .id();

    commands
        .spawn((
            Text::new(label),
            TextFont {
                font_size: 12.,
                ..default()
            },
            PickingBehavior::IGNORE,
        ))
        .set_parent(root);

    commands.entity(root)
}
