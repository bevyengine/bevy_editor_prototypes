use bevy::{prelude::*, window::SystemCursorIcon, winit::cursor::CursorIcon};
use bevy_editor_styles::Theme;

use crate::{AssetBrowserLocation, ButtonType, LocationSegmentType};

/// Color of the path segment background when idle
pub const PATH_SEGMENT_BACKGROUND_COLOR: Color = Color::srgb(0.2, 0.2, 0.2);

/// The root node for the asset browser top bar
#[derive(Component)]
pub struct TopBarNode;

pub fn refresh_location_path_ui(
    mut commands: Commands,
    root: Query<(Entity, Option<&Children>), With<TopBarNode>>,
    theme: Res<Theme>,
    location: Res<AssetBrowserLocation>,
) {
    for (top_bar_entity, top_bar_childrens) in root.iter() {
        // Clear location path UI
        if let Some(childrens) = top_bar_childrens {
            for child in childrens.iter() {
                commands.entity(*child).despawn_recursive();
            }
            commands.entity(top_bar_entity).clear_children();
        }
        // Regenerate location path UI
        let mut top_bar_ec = commands.entity(top_bar_entity);
        spawn_location_path_ui(&theme, &location, &mut top_bar_ec);
    }
}

pub fn spawn_location_path_ui(
    theme: &Res<Theme>,
    location: &Res<AssetBrowserLocation>,
    parent: &mut EntityCommands,
) {
    // Spawn new children
    parent.with_children(|parent| {
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
            theme.general.border_radius,
            ButtonType::LocationSegment(segment_type),
        ))
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
            font: theme.text.font.clone(),
            font_size: 10.0,
            ..default()
        },
        TextColor(theme.text.text_color),
    )
}
