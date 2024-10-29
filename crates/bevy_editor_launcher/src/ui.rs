use bevy::{prelude::*, ui::RelativeCursorPosition};
use bevy_editor::project::{get_local_projects, run_project, templates::Templates, ProjectInfo};
use bevy_editor_styles::Theme;
use bevy_footer_bar::FooterBarNode;

use bevy_scroll_box::spawn_scroll_box;

#[derive(Component)]
#[require(Node)]
pub struct ProjectList;

pub fn setup(mut commands: Commands, theme: Res<Theme>, asset_server: Res<AssetServer>) {
    commands.spawn((
        Camera2d,
        Camera {
            order: 10,
            ..default()
        },
    ));

    let ui_root = commands
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

    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            flex_grow: 1.0,
            ..default()
        })
        .set_parent(ui_root)
        .with_children(|parent| {
            spawn_scroll_box(
                parent,
                &theme,
                Overflow::scroll_y(),
                Some(|content_ec: &mut EntityCommands| {
                    content_ec.insert(ProjectList);
                    content_ec.with_children(|parent| {
                        for project in get_local_projects().iter() {
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
                                    align_self: AlignSelf::Center,
                                    justify_self: JustifySelf::Center,
                                    border: UiRect::all(Val::Px(5.0)),
                                    ..default()
                                },
                                BorderRadius::new(
                                    Val::Px(20.0),
                                    Val::Px(20.0),
                                    Val::Px(20.0),
                                    Val::Px(20.0),
                                ),
                                BorderColor(theme.button.background_color.0),
                            ))
                            .with_child((
                                Node {
                                    width: Val::Px(30.0),
                                    height: Val::Px(30.0),
                                    ..default()
                                },
                                UiImage::new(asset_server.load("plus.png")),
                            ))
                            .observe(
                                |_trigger: Trigger<Pointer<Up>>,
                                 mut commands: Commands,
                                 theme: Res<Theme>,
                                 asset_server: Res<AssetServer>| {
                                    let new_project_path = rfd::FileDialog::new().pick_folder();
                                    if let Some(path) = new_project_path {
                                        crate::spawn_create_new_project_task(
                                            &mut commands,
                                            Templates::Blank,
                                            "New Project".to_string(),
                                            path,
                                        );
                                    }
                                },
                            );
                    });
                }),
            );
        });

    let _footer = commands.spawn(FooterBarNode).set_parent(ui_root).id();
}

pub(crate) fn spawn_project_node<'a>(
    commands: &'a mut ChildBuilder,
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
        |trigger: Trigger<Pointer<Up>>, query_children: Query<&Children>, query: Query<&Text>, mut exit: EventWriter<AppExit>| {
            let project = {
                let text = {
                    let project_entity = trigger.entity();
                    let project_children = query_children.get(project_entity).unwrap();
                    let text_container = project_children.get(1).expect("Expected project node to have 2 children, (the second being a container for the name)");
					let text_container_children = query_children.get(*text_container).unwrap();
                    let text_entity = text_container_children.get(0).expect("Expected text container to have 1 child, the text entity");
                    query.get(*text_entity).expect("Expected text entity to have a Text component")
                };

                let projects = get_local_projects();
                projects
                    .iter()
                    .find(|p| p.name == text.0)
                    .unwrap()
                    .clone()
            };

			match run_project(project) {
				Ok(_) => { exit.send(AppExit::Success); },
				Err(error) => {
					error!("Failed to run project: {:?}", error);
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
                    UiImage::new(asset_server.load("image-off.png")),
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
                Text::new(project.name.clone()),
                TextFont {
                    font: theme.text.font.clone(),
                    font_size: 16.0,
                    ..default()
                },
            ));
    });

    root_ec
}
