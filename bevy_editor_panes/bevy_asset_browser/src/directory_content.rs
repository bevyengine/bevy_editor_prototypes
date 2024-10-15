use std::path::PathBuf;

use bevy::{
    a11y::{
        accesskit::{NodeBuilder, Role},
        AccessibilityNode,
    },
    input::mouse::{MouseScrollUnit, MouseWheel},
    prelude::*,
    text::BreakLineOn,
};
use bevy_editor_styles::Theme;

use crate::AssetBrowserLocation;

/// The root node for the directory content view
#[derive(Component)]
pub struct DirectoryContentNode;

pub fn ui_setup(
    mut commands: Commands,
    root: Query<Entity, With<DirectoryContentNode>>,
    theme: Res<Theme>,
) {
    commands
        .entity(root.single())
        .insert(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_self: AlignSelf::Stretch,
                overflow: Overflow::clip_y(),
                ..default()
            },
            background_color: theme.pane_background_color,
            ..default()
        })
        .with_children(|parent| {
            // Moving panel
            parent.spawn((
                NodeBundle {
                    style: Style {
                        position_type: PositionType::Absolute,
                        flex_wrap: FlexWrap::Wrap,
                        ..default()
                    },
                    ..default()
                },
                ScrollingList::default(),
                AccessibilityNode(NodeBuilder::new(Role::Grid)),
            ));
        });
}

pub enum FetchDirectoryContentResult {
    Success(Vec<PathBuf>),
    UpToDate,
}

pub fn fetch_directory_content(
    location: &mut ResMut<AssetBrowserLocation>,
    last_modified_time: &mut ResMut<crate::DirectoryLastModifiedTime>,
) -> FetchDirectoryContentResult {
    let metadata = {
        if let Ok(metadata) = std::fs::metadata(location.get_absolute_path()) {
            metadata
        } else {
            **location = AssetBrowserLocation::default();
            let content_directory_exist =
                std::fs::exists(location.get_absolute_path()).unwrap_or(false);
            if !content_directory_exist {
                std::fs::create_dir_all(location.get_absolute_path()).unwrap();
            }
            std::fs::metadata(location.get_absolute_path()).unwrap()
        }
    };
    let modified_time = metadata.modified().unwrap();
    if modified_time == last_modified_time.0 {
        return FetchDirectoryContentResult::UpToDate;
    }
    last_modified_time.0 = metadata.modified().unwrap();

    let mut dir_content = std::fs::read_dir(location.get_absolute_path())
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect::<Vec<_>>();
    // Sort, directories first in alphabetical order, then files in alphabetical order toot
    dir_content.sort_by(|a, b| {
        if a.is_dir() && b.is_file() {
            std::cmp::Ordering::Less
        } else if a.is_file() && b.is_dir() {
            std::cmp::Ordering::Greater
        } else {
            a.cmp(b)
        }
    });
    FetchDirectoryContentResult::Success(dir_content)
}

#[derive(Component, Default)]
pub struct ScrollingList {
    position: f32,
}

pub fn scrolling(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut query_list: Query<(&mut ScrollingList, &mut Style, &Parent, &Node)>,
    query_node: Query<&Node>,
) {
    for mouse_wheel_event in mouse_wheel_events.read() {
        for (mut scrolling_list, mut style, parent, list_node) in &mut query_list {
            let items_height = list_node.size().y;
            let container_height = query_node.get(parent.get()).unwrap().size().y;
            let max_scroll = (items_height - container_height).max(0.);

            let dy = match mouse_wheel_event.unit {
                MouseScrollUnit::Line => mouse_wheel_event.y * 20.,
                MouseScrollUnit::Pixel => mouse_wheel_event.y,
            };

            scrolling_list.position += dy;
            scrolling_list.position = scrolling_list.position.clamp(-max_scroll, 0.);

            style.top = Val::Px(scrolling_list.position);
        }
    }
}

pub fn refresh_content(
    commands: &mut Commands,
    content_list_query: &Query<(Entity, Option<&Children>), With<ScrollingList>>,
    last_modified_time: &mut ResMut<crate::DirectoryLastModifiedTime>,
    location: &mut ResMut<AssetBrowserLocation>,
    theme: &Res<Theme>,
    asset_server: &Res<AssetServer>,
) {
    match fetch_directory_content(location, last_modified_time) {
        FetchDirectoryContentResult::Success(directory_content) => {
            let (content_list_entity, content_list_children) = content_list_query.single();
            if let Some(content_list_children) = content_list_children {
                for child in content_list_children {
                    commands.entity(*child).despawn_recursive();
                }
                commands
                    .entity(content_list_entity)
                    .remove_children(content_list_children);
            }
            commands
                .entity(content_list_entity)
                .with_children(|parent| {
                    for entry in directory_content {
                        let asset_type = if entry.is_dir() {
                            crate::AssetType::Directory
                        } else {
                            crate::AssetType::Unknown
                        };
                        parent
                            .spawn((
                                ButtonBundle {
                                    style: Style {
                                        margin: UiRect::all(Val::Px(5.0)),
                                        padding: UiRect::all(Val::Px(5.0)),
                                        height: Val::Px(100.0),
                                        width: Val::Px(100.0),
                                        align_items: AlignItems::Center,
                                        flex_direction: FlexDirection::Column,
                                        border: UiRect::all(Val::Px(3.0)),
                                        justify_content: JustifyContent::SpaceBetween,
                                        ..default()
                                    },
                                    border_radius: theme.border_radius,
                                    ..default()
                                },
                                crate::ButtonType::AssetButton(asset_type),
                            ))
                            .with_children(|parent| {
                                parent.spawn(ImageBundle {
                                    image: UiImage::new(crate::content_button_to_icon(
                                        &asset_type,
                                        asset_server,
                                    )),
                                    style: Style {
                                        height: Val::Px(50.0),
                                        ..default()
                                    },
                                    ..default()
                                });
                                parent.spawn(TextBundle {
                                    text: Text {
                                        sections: vec![TextSection {
                                            value: entry
                                                .file_name()
                                                .unwrap()
                                                .to_str()
                                                .unwrap()
                                                .to_string(),
                                            style: TextStyle {
                                                font_size: 12.0,
                                                color: theme.text_color,
                                                ..default()
                                            },
                                        }],
                                        linebreak_behavior: BreakLineOn::WordBoundary,
                                        justify: JustifyText::Center,
                                    },
                                    ..default()
                                });
                            });
                    }
                });
        }
        FetchDirectoryContentResult::UpToDate => {}
    }
}
