use bevy::{prelude::*, window::SystemCursorIcon, winit::cursor::CursorIcon};
use bevy_editor_styles::Theme;

use crate::{AssetBrowserLocation, ButtonType, LocationSegmentType};

/// Color of the path segment background when idle
pub const PATH_SEGMENT_BACKGROUND_COLOR: Color = Color::srgb(0.2, 0.2, 0.2);

/// The root node for the asset browser top bar
#[derive(Component)]
pub struct TopBarNode;

pub fn refresh_location_ui(
    mut commands: Commands,
    root: Query<(Entity, Option<&Children>), With<TopBarNode>>,
    theme: Res<Theme>,
    location: Res<AssetBrowserLocation>,
) {
    let (top_bar_entity, top_bar_childrens) = root.single();

    // Clear all children (if any)
    if let Some(childrens) = top_bar_childrens {
        for child in childrens.iter() {
            commands.entity(*child).despawn_recursive();
        }
        commands.entity(top_bar_entity).clear_children();
    }

    // Spawn new children
    commands.entity(top_bar_entity).with_children(|parent| {
        spawn_path_segment_ui(
            parent,
            "Sources".to_string(),
            theme.as_ref(),
            LocationSegmentType::Root,
        );
        if location.source_id.is_none() {
            return;
        }
        parent.spawn(path_separator_ui(theme.as_ref()));
        let source_id = location.source_id.as_ref().unwrap();
        spawn_path_segment_ui(
            parent,
            crate::asset_source_id_to_string(source_id),
            theme.as_ref(),
            LocationSegmentType::Source,
        );
        location.path.iter().for_each(|directory_name| {
            parent.spawn(path_separator_ui(theme.as_ref()));
            spawn_path_segment_ui(
                parent,
                directory_name.to_str().unwrap().to_string(),
                theme.as_ref(),
                LocationSegmentType::Directory,
            );
        });
    });
}

/// push a new path segment UI element
fn spawn_path_segment_ui(
    parent: &mut ChildBuilder,
    directory_name: String,
    theme: &Theme,
    segment_type: LocationSegmentType,
) {
    parent
        .spawn((
            Button,
            Node {
                padding: UiRect::axes(Val::Px(10.0), Val::Px(1.0)),
                margin: UiRect::horizontal(Val::Px(5.0)),
                ..default()
            },
            BackgroundColor(PATH_SEGMENT_BACKGROUND_COLOR),
            theme.border_radius,
            ButtonType::LocationSegment(segment_type),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text(directory_name),
                TextFont {
                    font_size: 10.0,
                    ..default()
                },
                TextColor(theme.text_color),
            ));
        })
        .observe(
            move |_trigger: Trigger<Pointer<Move>>,
                  window_query: Query<Entity, With<Window>>,
                  mut commands: Commands| {
                let window = window_query.single();
                commands
                    .entity(window)
                    .insert(CursorIcon::System(SystemCursorIcon::Pointer));
            },
        )
        .observe(
            move |_trigger: Trigger<Pointer<Out>>,
                  window_query: Query<Entity, With<Window>>,
                  mut commands: Commands| {
                let window = window_query.single();
                commands
                    .entity(window)
                    .insert(CursorIcon::System(SystemCursorIcon::Default));
            },
        );
}

fn path_separator_ui(theme: &Theme) -> impl Bundle {
    (
        Text("/".to_string()),
        TextFont {
            font_size: 10.0,
            ..default()
        },
        TextColor(theme.text_color),
    )
}
