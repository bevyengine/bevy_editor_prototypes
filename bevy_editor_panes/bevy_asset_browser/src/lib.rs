//! A UI element for browsing assets in the Bevy Editor.
/// The intent of this system is to provide a simple and frictionless way to browse assets in the Bevy Editor.
/// The asset browser is a replica of the your asset directory on disk and get's automatically updated when the directory is modified.
use std::path::PathBuf;

use bevy::{
    asset::{
        embedded_asset,
        io::{file::FileAssetReader, AssetSourceId},
        AssetPlugin,
    },
    prelude::*,
};
use bevy_pane_layout::{pane::Pane, prelude::*};
use bevy_scroll_box::ScrollBoxPlugin;
use ui::top_bar::location_as_changed;

mod io;
mod ui;

/// The bevy asset browser plugin
#[derive(Component)]
pub struct AssetBrowserPane;

impl Pane for AssetBrowserPane {
    const NAME: &str = "Asset Browser";
    const ID: &str = "asset_browser";

    fn build(app: &mut App) {
        embedded_asset!(app, "assets/directory_icon.png");
        embedded_asset!(app, "assets/source_icon.png");
        embedded_asset!(app, "assets/file_icon.png");

        // Fetch the AssetPlugin file path, this is used to create assets at the correct location
        let default_source_absolute_file_path = {
            let asset_plugins: Vec<&AssetPlugin> = app.get_added_plugins();
            let asset_plugin_file_path = match asset_plugins.first() {
                Some(plugin) => plugin.file_path.clone(),
                None => {
                    app.add_plugins(AssetPlugin::default());
                    AssetPlugin::default().file_path
                }
            };
            let mut absolute_path = FileAssetReader::get_base_path();
            absolute_path.push(asset_plugin_file_path);
            absolute_path
        };

        app.add_plugins(ScrollBoxPlugin)
            .insert_resource(DefaultSourceFilePath(default_source_absolute_file_path))
            .insert_resource(AssetBrowserLocation::default())
            .insert_resource(DirectoryContent::default())
            .add_systems(Startup, io::task::fetch_directory_content)
            // .add_systems(Update, button_interaction)
            .add_systems(
                Update,
                io::task::poll_task.run_if(io::task::fetch_task_is_running),
            )
            .add_systems(
                Update,
                ui::directory_content::refresh_ui
                    .run_if(directory_content_as_changed)
                    .after(io::task::poll_task),
            )
            .add_systems(
                Update,
                (
                    ui::top_bar::refresh_ui,
                    ui::directory_content::refresh_context_menu,
                )
                    .run_if(location_as_changed),
            );
    }

    fn creation_system() -> impl System<In = In<PaneStructure>, Out = ()> {
        IntoSystem::into_system(ui::on_pane_creation)
    }
}

/// One entry of [`DirectoryContent`]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Entry {
    /// Represent an [`AssetSourceId`]
    Source(AssetSourceId<'static>),
    /// Represent a directory
    Folder(String),
    /// Represent a file
    File(String),
}

/// The content of the directory pointed by [`AssetBrowserLocation`]
#[derive(Resource, Default, Debug, Clone, PartialEq, Eq)]
pub struct DirectoryContent(pub Vec<Entry>);

/// Check if the [`DirectoryContent`] has changed, which relate to the content of the current [`AssetBrowserLocation`]
pub(crate) fn directory_content_as_changed(directory_content: Res<DirectoryContent>) -> bool {
    directory_content.is_changed()
}

#[derive(Resource)]
struct DefaultSourceFilePath(pub PathBuf);

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
