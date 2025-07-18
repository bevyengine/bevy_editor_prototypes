use bevy::{prelude::*, window::SystemCursorIcon, winit::cursor::CursorIcon};
use bevy_editor_styles::Theme;

use crate::{AssetBrowserLocation, io};

use super::source_id_to_string;

/// Color of the path segment background when idle
pub const PATH_SEGMENT_BACKGROUND_COLOR: Color = Color::srgb(0.2, 0.2, 0.2);

/// The root node for the asset browser top bar
#[derive(Component)]
pub struct TopBarNode;

/// Spawn the top bar of the asset browser
pub(crate) fn spawn_top_bar<'a>(
    commands: &'a mut Commands,
    theme: &Res<Theme>,
    location: &Res<AssetBrowserLocation>,
) -> EntityCommands<'a> {
    let top_bar = commands
        .spawn((
            TopBarNode,
            Node {
                height: Val::Px(30.0),
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                padding: UiRect::horizontal(Val::Px(10.0)),
                ..default()
            },
            theme.pane.header_background_color,
        ))
        .id();
    spawn_location_path_ui(commands, theme, location).insert(ChildOf(top_bar));

    commands.entity(top_bar)
}

pub fn location_as_changed(location: Res<AssetBrowserLocation>) -> bool {
    location.is_changed()
}

/// Clear and regenerate the location path UI
pub fn refresh_ui(
    mut commands: Commands,
    root: Query<(Entity, Option<&Children>), With<TopBarNode>>,
    theme: Res<Theme>,
    location: Res<AssetBrowserLocation>,
) {
    for (top_bar_entity, top_bar_childrens) in root.iter() {
        // Clear location path UI
        if let Some(childrens) = top_bar_childrens {
            for child in childrens.iter() {
                commands.entity(child).despawn();
            }
            commands.entity(top_bar_entity).remove::<Children>();
        }
        // Regenerate location path UI
        spawn_location_path_ui(&mut commands, &theme, &location).insert(ChildOf(top_bar_entity));
    }
}

/// Spawn the location path UI
pub fn spawn_location_path_ui<'a>(
    commands: &'a mut Commands,
    theme: &Res<Theme>,
    location: &Res<AssetBrowserLocation>,
) -> EntityCommands<'a> {
    let location_path = commands
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            ..default()
        })
        .id();

    spawn_path_segment_ui(
        commands,
        "Sources".to_string(),
        theme.as_ref(),
        LocationSegmentType::Root,
    )
    .insert(ChildOf(location_path));

    if location.source_id.is_some() {
        commands
            .spawn(path_separator_ui(theme.as_ref()))
            .insert(ChildOf(location_path));
        let source_id = location.source_id.as_ref().unwrap();
        spawn_path_segment_ui(
            commands,
            source_id_to_string(source_id),
            theme.as_ref(),
            LocationSegmentType::Source,
        )
        .insert(ChildOf(location_path));
        location.path.iter().for_each(|directory_name| {
            commands
                .spawn(path_separator_ui(theme.as_ref()))
                .insert(ChildOf(location_path));
            spawn_path_segment_ui(
                commands,
                directory_name.to_str().unwrap().to_string(),
                theme.as_ref(),
                LocationSegmentType::Directory,
            )
            .insert(ChildOf(location_path));
        });
    }
    commands.entity(location_path)
}

/// Spawn a path segment UI element
/// This segment represent a component of the [`AssetBrowserLocation`] path
/// When clicked, it changes the [`AssetBrowserLocation`] to the corresponding path
fn spawn_path_segment_ui<'a>(
    commands: &'a mut Commands,
    directory_name: String,
    theme: &Theme,
    segment_type: LocationSegmentType,
) -> EntityCommands<'a> {
    let mut segment_ec = commands.spawn((
        Button,
        Node {
            padding: UiRect::axes(Val::Px(10.0), Val::Px(1.0)),
            margin: UiRect::horizontal(Val::Px(5.0)),
            ..default()
        },
        BackgroundColor(PATH_SEGMENT_BACKGROUND_COLOR),
        theme.general.border_radius,
        segment_type,
    ));
    segment_ec
        .with_children(|parent| {
            parent.spawn((
                Text(directory_name),
                TextFont {
                    font: theme.text.font.clone(),
                    font_size: 10.0,
                    ..default()
                },
                TextColor(theme.text.text_color),
            ));
        })
        .observe(
            move |trigger: On<Pointer<Release>>,
                  mut commands: Commands,
                  mut location: ResMut<AssetBrowserLocation>,
                  query_children: Query<&Children>,
                  query_segment_info: Query<(&ChildOf, &LocationSegmentType)>| {
                let segment = trigger.target();
                let (parent, segment_type) = query_segment_info.get(segment).unwrap();
                match segment_type {
                    LocationSegmentType::Root => {
                        location.source_id = None;
                        location.path.clear();
                    }
                    LocationSegmentType::Source => {
                        location.path.clear();
                    }
                    LocationSegmentType::Directory => {
                        let location_segments = query_children.get(parent.parent()).unwrap();
                        // Last segment is the current directory, no need to reload
                        if *location_segments.last().unwrap() == segment {
                            return;
                        }
                        let segment_position = location_segments
                            .iter()
                            .step_by(2) // Step by 2 to go through each segment, skipping the separators
                            .skip(1) // Skip the "Sources" segment
                            .position(|child| child == segment)
                            .expect(
                                "You shouldn't be able to click on a segment that isn't in the asset location path"
                            );
                        location.path = location.path.iter().take(segment_position).collect();
                    }
                };
                commands.run_system_cached(io::task::fetch_directory_content);
            },
        )
        .observe(
            move |_trigger: On<Pointer<Move>>,
                  window_query: Query<Entity, With<Window>>,
                  mut commands: Commands| {
                let window = window_query.single().unwrap();
                commands
                    .entity(window)
                    .insert(CursorIcon::System(SystemCursorIcon::Pointer));
            },
        )
        .observe(
            move |_trigger: On<Pointer<Out>>,
                  window_query: Query<Entity, With<Window>>,
                  mut commands: Commands| {
                let window = window_query.single().unwrap();
                commands
                    .entity(window)
                    .insert(CursorIcon::System(SystemCursorIcon::Default));
            },
        );
    segment_ec
}

/// Spawn a path separator UI element
/// This separator is a visual element that separate path segments
fn path_separator_ui(theme: &Theme) -> impl Bundle {
    (
        Text(">".to_string()),
        TextFont {
            font: theme.text.font.clone(),
            font_size: 10.0,
            ..default()
        },
        TextColor(theme.text.text_color),
    )
}

/// All the types of segment the exist in the [`AssetBrowserLocation`] path
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub enum LocationSegmentType {
    /// Root segment, is the segment representing the very first segment of the path
    /// It doesn't actually represent anything, instead it's just mean your nowhere
    /// We use this to display all the [`bevy::asset::io::AssetSource`] available
    Root,
    /// A source segment, is a segment that represent one of the [`bevy::asset::io::AssetSource`] available
    Source,
    /// A directory segment, is a segment that represent a directory relative to the source root
    Directory,
}
