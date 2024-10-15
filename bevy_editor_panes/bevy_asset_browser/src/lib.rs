//! A UI element for browsing assets in the Bevy Editor.
/// The intent of this system is to provide a simple and frictionless way to browse assets in the Bevy Editor.
/// The asset browser is a replica of the your asset directory on disk and get's automatically updated when the directory is modified.
use std::{
    path::{Path, PathBuf},
    time::SystemTime,
};

use bevy::{
    asset::{embedded_asset, io::file::FileAssetReader},
    prelude::*,
};
use bevy_editor_styles::Theme;
use directory_content::{DirectoryContentNode, ScrollingList};
use top_bar::TopBarNode;

mod directory_content;
mod top_bar;

/// The bevy asset browser plugin
pub struct AssetBrowserPlugin;

impl Plugin for AssetBrowserPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "assets/directory_icon.png");
        embedded_asset!(app, "assets/file_icon.png");
        app.insert_resource(AssetBrowserLocation::default())
            .insert_resource(DirectoryLastModifiedTime(SystemTime::UNIX_EPOCH))
            .add_systems(Startup, ui_setup.in_set(AssetBrowserSet))
            .add_systems(
                Startup,
                (top_bar::ui_setup, directory_content::ui_setup).after(AssetBrowserSet),
            )
            .add_systems(Update, button_interaction)
            .add_systems(Update, directory_content::scrolling)
            .add_systems(FixedUpdate, refresh_directory_content);
    }
}

/// System Set to set up the Asset Browser.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct AssetBrowserSet;

/// The current location of the asset browser
#[derive(Resource)]
pub struct AssetBrowserLocation {
    absolute_path: PathBuf,
    start_segment_index: usize,
}

impl Default for AssetBrowserLocation {
    fn default() -> Self {
        let absolute_path = FileAssetReader::get_base_path().join("assets");
        let start_segment_index = absolute_path.iter().count() - 1;
        Self {
            absolute_path,
            start_segment_index,
        }
    }
}

impl AssetBrowserLocation {
    /// Get the path starting from the `start_segment_index`.
    pub fn get_path(&self) -> &Path {
        let mut comps = self.absolute_path.as_path().components();
        for _ in 0..self.start_segment_index {
            comps.next();
        }
        comps.as_path()
    }

    /// Get the absolute path.
    pub fn get_absolute_path(&self) -> &Path {
        self.absolute_path.as_path()
    }

    /// Crops the path from the `start_segment_index` to the given `end_segment_index` (exclusive).
    pub fn trim(&mut self, end_segment_index: usize) {
        self.absolute_path = self
            .absolute_path
            .iter()
            .take(self.start_segment_index + end_segment_index)
            .collect();
    }

    /// Pushes a directory to the current location.
    pub fn push<P: AsRef<Path>>(&mut self, directory_name: P) {
        self.absolute_path.push(directory_name);
    }
}

/// The last time the current directory was modified
/// Used to check if the directory content needs to be refreshed
#[derive(Resource)]
pub struct DirectoryLastModifiedTime(pub SystemTime);

/// The root node for the asset browser.
#[derive(Component)]
pub struct AssetBrowserNode;

fn ui_setup(
    mut commands: Commands,
    root: Query<Entity, With<AssetBrowserNode>>,
    theme: Res<Theme>,
) {
    commands
        .entity(root.single())
        .insert(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(35.0),
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                ..Default::default()
            },
            background_color: theme.background_color,
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn(TopBarNode);
            parent.spawn(DirectoryContentNode);
        });
}

/// A system to automatically refresh the current directory content when the directory is modified.
pub fn refresh_directory_content(
    mut commands: Commands,
    content_list_query: Query<(Entity, Option<&Children>), With<ScrollingList>>,
    mut last_modified_time: ResMut<DirectoryLastModifiedTime>,
    mut location: ResMut<AssetBrowserLocation>,
    theme: Res<Theme>,
    asset_server: Res<AssetServer>,
) {
    directory_content::refresh_content(
        &mut commands,
        &content_list_query,
        &mut last_modified_time,
        &mut location,
        &theme,
        &asset_server,
    );
}

/// Every type of button in the asset browser
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub enum ButtonType {
    /// A Path segment of the current asset browser location
    /// When clicked, the asset browser will navigate to the corresponding directory
    LocationSegment,
    /// An asset button
    /// Used to interact with the assets in the directory content view
    AssetButton(AssetType),
}

/// Every type of asset the asset browser supports
#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum AssetType {
    /// A type of asset that is not supported
    #[default]
    Unknown,
    /// A directory asset
    /// When clicked, the asset browser will step into the directory
    Directory,
}

/// Map the asset type to the corresponding icon
pub fn content_button_to_icon<A: Asset>(
    asset_type: &AssetType,
    asset_server: &Res<AssetServer>,
) -> Handle<A> {
    match asset_type {
        AssetType::Directory => {
            asset_server.load::<A>("embedded://bevy_asset_browser/assets/directory_icon.png")
        }
        _ => asset_server.load::<A>("embedded://bevy_asset_browser/assets/file_icon.png"),
    }
}

/// Handle the asset browser button interactions
#[allow(clippy::too_many_arguments)]
pub fn button_interaction(
    mut commands: Commands,
    mut interaction_query: Query<
        (
            Entity,
            &Interaction,
            &ButtonType,
            &mut BackgroundColor,
            &Children,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    content_list_query: Query<(Entity, Option<&Children>), With<ScrollingList>>,
    path_list_query: Query<(Entity, &Children), With<TopBarNode>>,
    text_query: Query<&Text>,
    theme: Res<Theme>,
    mut location: ResMut<AssetBrowserLocation>,
    mut last_modified_time: ResMut<DirectoryLastModifiedTime>,
    asset_server: Res<AssetServer>,
) {
    for (button_entity, interaction, button_type, mut background_color, button_children) in
        &mut interaction_query
    {
        match *interaction {
            Interaction::Pressed => {
                let location_has_changed = match button_type {
                    ButtonType::LocationSegment => {
                        let (path_list_entity, path_list_children) = path_list_query.single();
                        // Last segment is the current directory, no need to reload
                        if button_entity == *path_list_children.last().unwrap() {
                            return;
                        }
                        let segment_position = path_list_children
                            .iter()
                            .skip(1) // First child is a separator
                            .step_by(2) // Step by 2 to go through each segment, skipping the separators
                            .position(|child| *child == button_entity)
                            .unwrap();
                        location.trim(segment_position + 1);
                        let child_to_remove = &path_list_children[(segment_position + 1) * 2..];
                        for child in child_to_remove {
                            commands.entity(*child).despawn_recursive();
                        }
                        commands
                            .entity(path_list_entity)
                            .remove_children(child_to_remove);
                        true
                    }
                    ButtonType::AssetButton(AssetType::Directory) => {
                        let directory_name =
                            &text_query.get(button_children[1]).unwrap().sections[0].value;
                        location.push(directory_name);
                        let (path_list_entity, _) = path_list_query.single();
                        commands.entity(path_list_entity).with_children(|parent| {
                            top_bar::push_path_segment(
                                parent,
                                directory_name.clone(),
                                theme.as_ref(),
                            );
                        });
                        true
                    }
                    _ => false,
                };

                if location_has_changed {
                    last_modified_time.0 = SystemTime::UNIX_EPOCH; // Force refresh
                    directory_content::refresh_content(
                        &mut commands,
                        &content_list_query,
                        &mut last_modified_time,
                        &mut location,
                        &theme,
                        &asset_server,
                    );
                }
            }
            Interaction::Hovered => match button_type {
                ButtonType::LocationSegment | ButtonType::AssetButton(AssetType::Directory) => {
                    background_color.0 = Color::srgb(0.5, 0.5, 0.5); // TODO: Use theme
                }
                _ => {}
            },
            Interaction::None => match button_type {
                ButtonType::LocationSegment => {
                    background_color.0 = top_bar::PATH_SEGMENT_BACKGROUND_COLOR;
                }
                ButtonType::AssetButton(AssetType::Directory) => {
                    background_color.0 = Color::NONE;
                }
                _ => {}
            },
        }
    }
}
