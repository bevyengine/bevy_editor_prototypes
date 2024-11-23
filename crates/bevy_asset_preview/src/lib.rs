use bevy::{
    app::{App, Plugin, Startup, Update},
    asset::{AssetPath, AssetServer, Handle},
    gltf::GltfAssetLabel,
    prelude::{Component, Image, Mesh},
    scene::Scene,
};

use crate::render::{
    PrerenderedScenes, PreviewRenderLayers, PreviewRendered, PreviewSceneState, PreviewSettings,
};

mod render;
mod ui;

/// This crate is a work in progress and is not yet ready for use.
/// The intention is to provide a way to load/render/unload assets in the background and provide previews of them in the Bevy Editor.
/// For 2d assets this will be a simple sprite, for 3d assets this will require a quick render of the asset at a low resolution, just enough for a user to be able to tell quickly what it is.
/// This code may be reused for the Bevy Marketplace Viewer to provide previews of assets and plugins.
/// So long as the assets are unchanged, the previews will be cached and will not need to be re-rendered.
/// In theory this can be done passively in the background, and the previews will be ready when the user needs them.

#[derive(Component)]
pub enum PreviewAsset {
    Image(Handle<Image>),
    Scene(Handle<Scene>),
    Other,
}

impl PreviewAsset {
    pub fn new<'a>(path: impl Into<AssetPath<'a>>, asset_server: &AssetServer) -> Self {
        let path = <_ as Into<AssetPath<'a>>>::into(path);
        match path.path().extension() {
            Some(ext) => match ext.to_str().unwrap() {
                "png" => Self::Image(asset_server.load(path)),
                "glb" => Self::Scene(
                    asset_server
                        .load(GltfAssetLabel::Scene(0).from_asset(path.path().to_path_buf())),
                ),
                _ => Self::Other,
            },
            None => Self::Other,
        }
    }
}

pub struct AssetPreviewPlugin;

impl Plugin for AssetPreviewPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PreviewRendered>()
            .add_systems(
                Update,
                (
                    render::update_queue,
                    render::update_preview_frames_counter,
                    render::change_render_layers,
                    ui::preview_handler,
                ),
            )
            .init_resource::<PrerenderedScenes>()
            .init_resource::<PreviewSettings>()
            .init_resource::<PreviewSceneState>()
            .init_resource::<PreviewRenderLayers>();
    }
}
