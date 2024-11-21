//! A consistently-styled, cross-platform menu bar for Bevy applications.
//!
//! This runs along the top of the screen and provides a list of options to the user,
//! such as "File", "Edit", "View", etc.

use bevy::{asset::embedded_asset, prelude::*};

use bevy_editor_styles::Theme;

/// The root node for the menu bar.
#[derive(Component)]
pub struct MenuBarNode;

/// The Bevy Menu Bar Plugin.
pub struct MenuBarPlugin;

impl Plugin for MenuBarPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "assets/logo/bevy_logo.png");
        app.add_systems(Startup, menu_setup.in_set(MenuBarSet));
    }
}

/// System Set to set up the menu bar.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct MenuBarSet;

/// The setup system for the menu bar.
fn menu_setup(
    mut commands: Commands,
    root: Query<Entity, With<MenuBarNode>>,
    theme: Res<Theme>,
    asset_server: Res<AssetServer>,
) {
    commands.entity(root.single()).insert((
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(30.0),
            display: Display::Flex,
            flex_direction: FlexDirection::Row,

            justify_items: JustifyItems::Start,
            align_items: AlignItems::Center,
            padding: UiRect {
                left: Val::Px(5.0),
                right: Val::Px(5.0),
                top: Val::Px(0.0),
                bottom: Val::Px(0.0),
            },
            ..Default::default()
        },
        theme.general.background_color,
    ));

    let mut hover_over_observer = Observer::new(
        |trigger: Trigger<Pointer<Over>>,
         theme: Res<Theme>,
         mut query: Query<&mut BackgroundColor>| {
            query.get_mut(trigger.entity()).unwrap().0 = theme.button.hover_color;
        },
    );
    let mut hover_out_observer = Observer::new(
        |trigger: Trigger<Pointer<Out>>,
         theme: Res<Theme>,
         mut query: Query<&mut BackgroundColor>| {
            query.get_mut(trigger.entity()).unwrap().0 = theme.menu.background_color;
        },
    );

    let logo = commands
        .spawn(UiImage {
            image: asset_server.load("embedded://bevy_menu_bar/assets/logo/bevy_logo.png"),
            ..Default::default()
        })
        .id();

    let file_text = commands
        .spawn((
            Text::new("File"),
            TextFont {
                font: theme.text.font.clone(),
                font_size: 12.,
                ..default()
            },
            PickingBehavior::IGNORE,
        ))
        .id();
    let file_container = commands
        .spawn((
            Node {
                padding: UiRect {
                    left: Val::Px(5.0),
                    right: Val::Px(5.0),
                    top: Val::Px(2.0),
                    bottom: Val::Px(2.0),
                },
                align_content: AlignContent::Center,
                justify_content: JustifyContent::Center,

                ..Default::default()
            },
            BorderRadius::all(Val::Px(3.)),
        ))
        .id();

    let edit_text = commands
        .spawn((
            Text::new("Edit"),
            TextFont {
                font: theme.text.font.clone(),
                font_size: 12.,
                ..default()
            },
            PickingBehavior::IGNORE,
        ))
        .id();
    let edit_container = commands
        .spawn((
            Node {
                padding: UiRect {
                    left: Val::Px(5.0),
                    right: Val::Px(5.0),
                    top: Val::Px(2.0),
                    bottom: Val::Px(2.0),
                },
                align_content: AlignContent::Center,
                justify_content: JustifyContent::Center,

                ..Default::default()
            },
            BorderRadius::all(Val::Px(3.)),
        ))
        .id();

    let build_text = commands
        .spawn((
            Text::new("Build"),
            TextFont {
                font: theme.text.font.clone(),
                font_size: 12.,
                ..default()
            },
            PickingBehavior::IGNORE,
        ))
        .id();
    let build_container = commands
        .spawn((
            Node {
                padding: UiRect {
                    left: Val::Px(5.0),
                    right: Val::Px(5.0),
                    top: Val::Px(2.0),
                    bottom: Val::Px(2.0),
                },
                align_content: AlignContent::Center,
                justify_content: JustifyContent::Center,

                ..Default::default()
            },
            BorderRadius::all(Val::Px(3.)),
        ))
        .id();

    let window_text = commands
        .spawn((
            Text::new("Window"),
            TextFont {
                font: theme.text.font.clone(),
                font_size: 12.,
                ..default()
            },
            PickingBehavior::IGNORE,
        ))
        .id();
    let window_container = commands
        .spawn((
            Node {
                padding: UiRect {
                    left: Val::Px(5.0),
                    right: Val::Px(5.0),
                    top: Val::Px(2.0),
                    bottom: Val::Px(2.0),
                },
                align_content: AlignContent::Center,
                justify_content: JustifyContent::Center,

                ..Default::default()
            },
            BorderRadius::all(Val::Px(3.)),
        ))
        .id();

    let help_text = commands
        .spawn((
            Text::new("Help"),
            TextFont {
                font: theme.text.font.clone(),
                font_size: 12.,
                ..default()
            },
            PickingBehavior::IGNORE,
        ))
        .id();

    let help_container = commands
        .spawn((
            Node {
                padding: UiRect {
                    left: Val::Px(5.0),
                    right: Val::Px(5.0),
                    top: Val::Px(2.0),
                    bottom: Val::Px(2.0),
                },
                align_content: AlignContent::Center,
                justify_content: JustifyContent::Center,

                ..Default::default()
            },
            BorderRadius::all(Val::Px(3.)),
        ))
        .id();

    let menu_container = commands
        .spawn((Node {
            width: Val::Px(285.0),
            height: Val::Px(30.0),
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            align_content: AlignContent::Stretch,
            ..Default::default()
        },))
        .id();

    commands.entity(menu_container).set_parent(root.single());

    commands.entity(logo).set_parent(menu_container);
    commands.entity(file_container).set_parent(menu_container);
    commands.entity(file_text).set_parent(file_container);
    commands.entity(edit_container).set_parent(menu_container);
    commands.entity(edit_text).set_parent(edit_container);
    commands.entity(build_container).set_parent(menu_container);
    commands.entity(build_text).set_parent(build_container);
    commands.entity(window_container).set_parent(menu_container);
    commands.entity(window_text).set_parent(window_container);
    commands.entity(help_container).set_parent(menu_container);
    commands.entity(help_text).set_parent(help_container);

    hover_over_observer.watch_entity(file_container);
    hover_out_observer.watch_entity(file_container);
    hover_over_observer.watch_entity(edit_container);
    hover_out_observer.watch_entity(edit_container);
    hover_over_observer.watch_entity(build_container);
    hover_out_observer.watch_entity(build_container);
    hover_over_observer.watch_entity(window_container);
    hover_out_observer.watch_entity(window_container);
    hover_over_observer.watch_entity(help_container);
    hover_out_observer.watch_entity(help_container);

    commands.spawn(hover_out_observer);
    commands.spawn(hover_over_observer);
}
