//! A UI element for browsing assets in the Bevy Editor.
/// The intent of this system is to provide a simple and frictionless way to browse assets in the Bevy Editor.
/// The asset browser is a replica of the your asset directory on disk and get's automatically updated when the directory is modified.
use std::{path::PathBuf, time::SystemTime};

use atomicow::CowArc;
use bevy::{
    a11y::{
        accesskit::{NodeBuilder, Role},
        AccessibilityNode,
    },
    asset::{
        embedded_asset,
        io::{AssetSource, AssetSourceBuilders, AssetSourceId},
    },
    ecs::system::SystemId,
    prelude::*,
};
use bevy_editor_styles::Theme;
use bevy_pane_layout::{PaneContentNode, PaneRegistry};
use directory_content::{DirectoryContentNode, ScrollingList};
use top_bar::TopBarNode;

mod directory_content;
mod top_bar;

/// The bevy asset browser plugin
pub struct AssetBrowserPanePlugin;

impl Plugin for AssetBrowserPanePlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "assets/directory_icon.png");
        embedded_asset!(app, "assets/source_icon.png");
        embedded_asset!(app, "assets/file_icon.png");

        app.world_mut()
            .get_resource_or_init::<PaneRegistry>()
            .register("Asset Browser", |mut commands, pane_root| {
                commands.entity(pane_root).insert(AssetBrowserNode);
            });

        app.insert_resource(AssetBrowserLocation::default())
            .insert_resource(directory_content::DirectoryContent::default())
            .init_resource::<AssetBrowserOneShotSystems>()
            .insert_resource(DirectoryLastModifiedTime(SystemTime::UNIX_EPOCH))
            .add_observer(on_pane_creation)
            .add_systems(Startup, directory_content::fetch_directory_content)
            .add_systems(Update, (button_interaction, directory_content::scrolling))
            .add_systems(
                Update,
                (
                    directory_content::poll_fetch_task
                        .run_if(directory_content::run_if_fetch_task_is_running),
                    directory_content::refresh_ui
                        .after(directory_content::poll_fetch_task)
                        .run_if(directory_content::run_if_content_as_changed),
                ),
            );
    }
}

/// All the asset browser one shot systems
pub enum OneShotSystem {
    /// Refer to the system that fetches the directory content on disk
    FetchDirectoryContent,
    /// Refer to the system that refreshes the top bar UI
    RefreshTopBarUi,
}

/// All the asset browser one shot systems, see [`OneShotSystem`] enum for reference
#[derive(Resource)]
pub struct AssetBrowserOneShotSystems(pub [SystemId; 2]);

impl FromWorld for AssetBrowserOneShotSystems {
    fn from_world(world: &mut World) -> Self {
        // Order is important here! Should match the order of OneShotSystem enum
        Self([
            world.register_system(directory_content::fetch_directory_content),
            world.register_system(top_bar::refresh_location_path_ui),
        ])
    }
}

/// System Set to set up the Asset Browser.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct AssetBrowserSet;

/// The current location of the asset browser
#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct AssetBrowserLocation {
    /// The source id of the asset source to browse
    pub source_id: Option<AssetSourceId<'static>>,
    /// The path of the current directory relative to the asset source root
    pub path: PathBuf,
}

impl Default for AssetBrowserLocation {
    fn default() -> Self {
        Self {
            source_id: Some(AssetSourceId::Default),
            path: PathBuf::from(""),
        }
    }
}

/// The last time the current directory was modified
/// Used to check if the directory content needs to be refreshed
#[derive(Resource)]
pub struct DirectoryLastModifiedTime(pub SystemTime);

/// The root node for the asset browser.
#[derive(Component)]
pub struct AssetBrowserNode;

/// Spawn [`AssetBrowserNode`] once the pane is created
#[allow(clippy::too_many_arguments)]
pub fn on_pane_creation(
    trigger: Trigger<OnAdd, AssetBrowserNode>,
    mut commands: Commands,
    theme: Res<Theme>,
    children_query: Query<&Children>,
    content: Query<&PaneContentNode>,
    location: Res<AssetBrowserLocation>,
    asset_server: Res<AssetServer>,
    directory_content: Res<directory_content::DirectoryContent>,
    mut asset_sources_builder: ResMut<AssetSourceBuilders>,
) {
    let pane_root = trigger.entity();
    let content_node = children_query
        .iter_descendants(pane_root)
        .find(|e| content.contains(*e))
        .unwrap();

    commands
        .entity(content_node)
        .insert((Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            ..default()
        },))
        .with_children(|parent| {
            // Top bar
            let mut top_bar_ec = parent.spawn((
                TopBarNode,
                Node {
                    height: Val::Px(30.0),
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    padding: UiRect::horizontal(Val::Px(10.0)),
                    ..default()
                },
                theme.pane_header_background_color,
            ));
            top_bar::spawn_location_path_ui(&theme, &location, &mut top_bar_ec);

            // Directory content
            parent
                .spawn((
                    DirectoryContentNode,
                    Node {
                        flex_direction: FlexDirection::Column,
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        align_self: AlignSelf::Stretch,
                        overflow: Overflow::clip_y(),
                        ..default()
                    },
                ))
                .with_children(|parent| {
                    // Scroll box moving panel
                    let mut content_list_ec = parent.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            flex_wrap: FlexWrap::Wrap,
                            ..default()
                        },
                        ScrollingList::default(),
                        AccessibilityNode(NodeBuilder::new(Role::Grid)),
                    ));
                    directory_content::spawn_content_list_ui(
                        &mut content_list_ec,
                        &theme,
                        &asset_server,
                        &directory_content,
                        &location,
                        &mut asset_sources_builder,
                    );
                });
        });
}

/// Every type of button in the asset browser
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub enum ButtonType {
    /// A Path segment of the current asset browser location
    LocationSegment(LocationSegmentType),
    /// An asset button
    /// Used to interact with the assets in the directory content view
    AssetButton(AssetType),
}

/// All the types of location segments
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LocationSegmentType {
    /// The root segment, is an extra segment that mean that your nowwhere and wish to see what sources are available
    Root,
    /// A source segment, is a segment that represent a source
    Source,
    /// A directory segment, is a segment that represent a directory relative to the source root
    Directory,
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
    /// Special type of Asset that is used to represent the engine source
    EngineSource,
}

/// Map the asset type to the corresponding icon
pub fn content_button_to_icon(
    asset_type: &AssetType,
    asset_server: &Res<AssetServer>,
) -> Handle<Image> {
    match asset_type {
        AssetType::Directory => {
            asset_server.load("embedded://bevy_asset_browser/assets/directory_icon.png")
        }
        AssetType::EngineSource => {
            asset_server.load("embedded://bevy_asset_browser/assets/source_icon.png")
        }
        _ => asset_server.load("embedded://bevy_asset_browser/assets/file_icon.png"),
    }
}

const DEFAULT_SOURCE_ID_NAME: &str = "Default";

/// Convert the asset source id to a string
pub fn asset_source_id_to_string(asset_source_id: &AssetSourceId) -> String {
    match asset_source_id {
        AssetSourceId::Default => DEFAULT_SOURCE_ID_NAME.to_string(),
        AssetSourceId::Name(name) => name.to_string(),
    }
}

/// Handle the asset browser button interactions
pub fn button_interaction(
    mut commands: Commands,
    mut interaction_query: Query<
        (
            Entity,
            &Parent,
            &Interaction,
            &ButtonType,
            &mut BackgroundColor,
            &Children,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    path_list_query: Query<&Children, With<TopBarNode>>,
    text_query: Query<&Text>,
    mut location: ResMut<AssetBrowserLocation>,
    mut asset_sources_builder: ResMut<AssetSourceBuilders>,
    one_shot_systems: Res<AssetBrowserOneShotSystems>,
) {
    for (
        button_entity,
        button_parent,
        interaction,
        button_type,
        mut background_color,
        button_children,
    ) in &mut interaction_query
    {
        match *interaction {
            Interaction::Pressed => {
                let location_as_changed = match button_type {
                    ButtonType::LocationSegment(LocationSegmentType::Root) => {
                        location.source_id = None;
                        location.path.clear();
                        true
                    }
                    ButtonType::LocationSegment(LocationSegmentType::Source) => {
                        location.path.clear();
                        true
                    }
                    ButtonType::LocationSegment(LocationSegmentType::Directory) => {
                        let path_list_children = path_list_query.get(button_parent.get()).expect(
                            "LocationSegment button should always have a TopBarNode parent",
                        );

                        // Last segment is the current directory, no need to reload
                        if button_entity == *path_list_children.last().unwrap() {
                            return;
                        }
                        let segment_position = path_list_children
                            .iter()
                            .step_by(2) // Step by 2 to go through each segment, skipping the separators
                            .skip(1) // Skip the "Sources" segment
                            .position(|child| *child == button_entity)
                            .unwrap();
                        location.path = location.path.iter().take(segment_position).collect();
                        true
                    }
                    ButtonType::AssetButton(AssetType::EngineSource) => {
                        let source_name = &text_query.get(button_children[1]).unwrap().0;
                        if source_name == DEFAULT_SOURCE_ID_NAME {
                            location.source_id = Some(AssetSourceId::Default);
                        } else {
                            location.source_id = asset_sources_builder
                                .build_sources(false, false)
                                .iter()
                                .find(|source| match source.id() {
                                    AssetSourceId::Name(CowArc::Static(name)) => {
                                        name == source_name
                                    }
                                    _ => false,
                                })
                                .map(AssetSource::id);
                        }
                        true
                    }
                    ButtonType::AssetButton(AssetType::Directory) => {
                        let directory_name = &text_query.get(button_children[1]).unwrap().0;
                        location.path.push(directory_name);
                        true
                    }
                    _ => false,
                };

                if location_as_changed {
                    commands.run_system(
                        one_shot_systems.0[OneShotSystem::FetchDirectoryContent as usize],
                    );
                    commands
                        .run_system(one_shot_systems.0[OneShotSystem::RefreshTopBarUi as usize]);
                }
            }
            Interaction::Hovered => match button_type {
                ButtonType::LocationSegment(_) | ButtonType::AssetButton(AssetType::Directory) => {
                    background_color.0 = Color::srgb(0.5, 0.5, 0.5); // TODO: Use theme
                }
                _ => {}
            },
            Interaction::None => match button_type {
                ButtonType::LocationSegment(_) => {
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
