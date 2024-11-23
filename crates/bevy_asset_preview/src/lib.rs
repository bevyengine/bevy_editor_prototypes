use bevy::{
    app::{App, Last, Plugin, Update},
    asset::{AssetPath, AssetServer, Handle},
    gltf::GltfAssetLabel,
    prelude::{Component, Image},
    scene::Scene,
};

use crate::render::{PrerenderedScenes, PreviewRendered, PreviewSceneState, PreviewSettings};

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
                "jpg" | "jpeg" | "png" | "bmp" | "gif" | "ico" | "pnm" | "pam" | "pbm" | "pgm"
                | "ppm" | "tga" | "webp" => Self::Image(asset_server.load(path)),
                "glb" | "gltf" => Self::Scene(
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
                    ui::preview_handler,
                ),
            )
            // Add to PostUpdate to avoid flashing
            .add_systems(Last, render::change_render_layers)
            .init_resource::<PrerenderedScenes>()
            .init_resource::<PreviewSettings>()
            .init_resource::<PreviewSceneState>();
    }
}
