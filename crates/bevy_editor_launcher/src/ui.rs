use std::io::ErrorKind;
use std::path::Path;

use bevy::{prelude::*, ui::RelativeCursorPosition};
use bevy_editor::project::{ProjectInfo, run_project, set_project_list, templates::Templates};
use bevy_editor_styles::Theme;
use bevy_footer_bar::FooterBarNode;

use bevy_scroll_box::spawn_scroll_box;

use crate::ProjectInfoList;

#[derive(Component)]
#[require(Node)]
pub struct ProjectList;

/// Component for notification popup
#[derive(Component)]
pub struct NotificationPopup {
    pub timer: Timer,
}

/// System to handle notification popups
pub fn handle_notification_popups(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut NotificationPopup)>,
) {
    for (entity, mut popup) in query.iter_mut() {
        popup.timer.tick(time.delta());
        if popup.timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

/// Spawn a notification popup with a message
pub fn spawn_notification_popup(commands: &mut Commands, theme: &Theme, message: &str) -> Entity {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                top: Val::Px(50.0),
                margin: UiRect::horizontal(Val::Auto),
                padding: UiRect::all(Val::Px(12.0)),
                width: Val::Auto,
                height: Val::Auto,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.9)),
            BorderRadius::all(Val::Px(8.0)),
            NotificationPopup {
                timer: Timer::from_seconds(3.0, TimerMode::Once),
            },
        ))
        .with_child((
            Text::new(message.to_string()),
            TextFont {
                font: theme.text.font.clone(),
                font_size: 18.0,
                ..default()
            },
            TextColor(Color::WHITE),
            Outline {
                width: Val::Px(0.5),
                color: Color::BLACK,
                ..default()
            },
        ))
        .id()
}

pub fn setup(
    mut commands: Commands,
    theme: Res<Theme>,
    asset_server: Res<AssetServer>,
    project_list: Res<ProjectInfoList>,
) {
    commands.spawn((
        Camera2d,
        Camera {
            order: 10,
            ..default()
        },
    ));

    let root = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            theme.pane.area_background_color,
        ))
        .id();

    let main = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                flex_grow: 1.0,
                ..default()
            },
            ChildOf(root),
        ))
        .id();

    spawn_scroll_box(
        &mut commands,
        &theme,
        Overflow::scroll_y(),
        Some(|commands: &mut Commands, content_box: Entity| {
            let mut content_ec = commands.entity(content_box);
            content_ec.insert(ProjectList);
            content_ec.with_children(|parent| {
                for project in project_list.0.iter() {
                    spawn_project_node(parent, &theme, &asset_server, project);
                }
                parent
                    .spawn((
                        Node {
                            display: Display::Flex,
                            flex_direction: FlexDirection::Column,
                            margin: UiRect::axes(
                                Val::Px((250.0 - 100.0) / 2.0),
                                Val::Px((200.0 - 100.0) / 2.0),
                            ),
                            width: Val::Px(100.0),
                            height: Val::Px(100.0),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            border: UiRect::all(Val::Px(5.0)),
                            ..default()
                        },
                        BorderRadius::all(Val::Px(20.0)),
                        BorderColor::all(theme.button.background_color.0),
                    ))
                    .with_child((
                        Node {
                            width: Val::Px(30.0),
                            height: Val::Px(30.0),
                            ..default()
                        },
                        ImageNode::new(asset_server.load("plus.png")),
                    ))
                    .observe(|_trigger: On<Pointer<Release>>, mut commands: Commands| {
                        let new_project_path = rfd::FileDialog::new().pick_folder();
                        if let Some(path) = new_project_path {
                            crate::spawn_create_new_project_task(
                                &mut commands,
                                Templates::Blank,
                                path,
                            );
                        }
                    });
            });
        }),
    )
    .insert(ChildOf(main));

    let _footer = commands.spawn(FooterBarNode).insert(ChildOf(root)).id();
}

pub(crate) fn spawn_project_node<'a>(
    commands: &'a mut ChildSpawnerCommands,
    theme: &Theme,
    asset_server: &Res<AssetServer>,
    project: &ProjectInfo,
) -> EntityCommands<'a> {
    let mut root_ec = commands.spawn((
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            margin: UiRect::all(Val::Px(5.0)),
            width: Val::Px(250.0),
            height: Val::Px(200.0),
            ..default()
        },
        RelativeCursorPosition::default(),
        BorderRadius::new(Val::Px(15.0), Val::Px(15.0), Val::Px(15.0), Val::Px(15.0)),
        theme.button.background_color,
    ));

    root_ec.observe(
        |trigger: On<Pointer<Release>>,
         mut commands: Commands,
         query_children: Query<&Children>,
         query_text: Query<&Text>,
         mut exit: MessageWriter<AppExit>,
         mut project_list: ResMut<ProjectInfoList>,
         theme: Res<Theme>| {
            let project = {
                let text = {
                    let project_entity = trigger.event().event_target();
                    let project_children = query_children.get(project_entity).unwrap();
                    let text_container = project_children.get(1).expect(
                        "Expected project node to have 2 children, (the second being a container for the name)"
                    );
                    let text_container_children = query_children.get(*text_container).unwrap();
                    let text_entity = text_container_children
                        .first()
                        .expect("Expected text container to have 1 child, the text entity");
                    query_text
                        .get(*text_entity)
                        .expect("Expected text entity to have a Text component")
                };

                project_list
                    .0
                    .iter()
                    .find(|p| p.name().unwrap() == text.0)
                    .unwrap()
                    .clone()
            };

            // Check if project directory exists before trying to run it
            if !Path::new(&project.path).exists() {
                // Show notification popup
                let project_name = project.name().unwrap_or_else(|| "Unknown".to_string());
                spawn_notification_popup(
                    &mut commands,
                    &theme,
                    &format!("Project not found: '{project_name}'"),
                );
                // Remove project from list
                project_list.0.retain(|p| p.path != project.path);
                set_project_list(project_list.0.clone());
                // Remove project node from UI
                let project_entity = trigger.event().event_target();
                commands.entity(project_entity).despawn();
                return;
            }

            // Project exists, try to run it
            match run_project(&project) {
                Ok(_) => {
                    exit.write(AppExit::Success);
                }
                Err(error) => {
                    error!("Failed to run project: {:?}", error);
                    match error.kind() {
                        ErrorKind::NotFound | ErrorKind::InvalidData => {
                            // Show notification popup
                            let project_name = project.name().unwrap_or_else(|| "Unknown".to_string());
                            spawn_notification_popup(
                                &mut commands,
                                &theme,
                                &format!("Failed to run project: '{project_name}'"),
                            );
                            // Remove project from list
                            project_list.0.retain(|p| p.path != project.path);
                            set_project_list(project_list.0.clone());
                            // Remove project node from UI
                            let project_entity = trigger.event().event_target();
                            commands.entity(project_entity).despawn();
                        }
                        _ => {
                            // Show generic error notification
                            spawn_notification_popup(
                                &mut commands,
                                &theme,
                                &format!("Error running project: '{error}'"),
                            );
                        }
                    }
                }
            }
        },
    );

    root_ec.with_children(|parent| {
        // Project preview (TODO: add thumbnail)
        parent
            .spawn((Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                overflow: Overflow::clip(),
                flex_grow: 1.0,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },))
            .with_children(|parent| {
                parent.spawn((
                    ImageNode::new(asset_server.load("image-off.png")),
                    Node {
                        width: Val::Percent(30.0),
                        ..default()
                    },
                ));
            });
        // Project name
        parent
            .spawn((
                Node {
                    display: Display::Flex,
                    min_height: Val::Percent(20.0),
                    width: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(Color::oklch(0.209, 0.0, 0.0)),
                BorderRadius::new(Val::Px(0.0), Val::Px(0.0), Val::Px(15.0), Val::Px(15.0)),
            ))
            .with_child((
                Text::new(project.name().unwrap().to_string()),
                TextFont {
                    font: theme.text.font.clone(),
                    font_size: 16.0,
                    ..default()
                },
            ));
    });

    root_ec
}
