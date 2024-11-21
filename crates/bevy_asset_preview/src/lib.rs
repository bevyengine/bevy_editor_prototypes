use bevy::prelude::*;

mod render;
mod ui;

/// This crate is a work in progress and is not yet ready for use.
/// The intention is to provide a way to load/render/unload assets in the background and provide previews of them in the Bevy Editor.
/// For 2d assets this will be a simple sprite, for 3d assets this will require a quick render of the asset at a low resolution, just enough for a user to be able to tell quickly what it is.
/// This code may be reused for the Bevy Marketplace Viewer to provide previews of assets and plugins.
/// So long as the assets are unchanged, the previews will be cached and will not need to be re-rendered.
/// In theory this can be done passively in the background, and the previews will be ready when the user needs them.

#[derive(Component)]
pub enum RequestPreview {
    Image(Handle<Image>),
    Mesh(Handle<Mesh>),
}

pub struct AssetPreviewPlugin;

impl Plugin for AssetPreviewPlugin {
    fn build(&self, app: &mut App) {}
}
