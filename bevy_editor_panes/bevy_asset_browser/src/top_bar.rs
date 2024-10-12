use bevy::prelude::*;
use bevy_editor_styles::Theme;

use crate::AssetBrowserLocation;

/// Color of the path segment background when idle
pub const PATH_SEGMENT_BACKGROUND_COLOR: Color = Color::srgb(0.2, 0.2, 0.2);

/// The root node for the asset browser top bar
#[derive(Component)]
pub struct TopBarNode;

pub fn ui_setup(
    mut commands: Commands,
    root: Query<Entity, With<TopBarNode>>,
    theme: Res<Theme>,
    location: Res<AssetBrowserLocation>,
) {
    commands
        .entity(root.single())
        .insert(NodeBundle {
            style: Style {
                height: Val::Px(50.0),
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                padding: UiRect::horizontal(Val::Px(10.0)),
                ..default()
            },
            background_color: theme.menu_bar_color,
            ..default()
        })
        .with_children(|parent| {
            // Skip 1, not interested about everything that goes before "/assets"
            location.0.iter().skip(1).for_each(|directory_name| {
                push_path_segment(
                    parent,
                    directory_name.to_str().unwrap().to_string(),
                    theme.as_ref(),
                );
            });
        });
}

/// push a new path segment UI element
pub fn push_path_segment(
    parent: &mut ChildBuilder, // TODO: should be TopBarNode
    directory_name: String,
    theme: &Theme,
) -> Entity {
    parent.spawn(TextBundle {
        text: Text {
            sections: vec![TextSection {
                value: "/".to_string(),
                style: TextStyle {
                    font_size: 10.0,
                    color: theme.text_color,
                    ..default()
                },
            }],
            ..default()
        },
        style: Style { ..default() },
        ..default()
    });
    parent
        .spawn((
            ButtonBundle {
                style: Style {
                    padding: UiRect::axes(Val::Px(10.0), Val::Px(2.0)),
                    margin: UiRect::horizontal(Val::Px(5.0)),
                    ..default()
                },
                background_color: BackgroundColor(PATH_SEGMENT_BACKGROUND_COLOR),
                border_radius: theme.border_radius,
                ..default()
            },
            crate::ButtonType::LocationSegment,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle {
                text: Text {
                    sections: vec![TextSection {
                        value: directory_name,
                        style: TextStyle {
                            font_size: 10.0,
                            color: theme.text_color,
                            ..default()
                        },
                    }],
                    ..default()
                },
                style: Style { ..default() },
                ..default()
            });
        })
        .id()
}
