use std::path::PathBuf;

use bevy::{
    app::{App, Last, Plugin, Update},
    asset::{AssetPath, AssetServer, Handle},
    gltf::GltfAssetLabel,
    prelude::{Component, Deref, Image, IntoSystemConfigs},
    render::{
        extract_resource::ExtractResourcePlugin, graph::CameraDriverLabel,
        render_graph::RenderGraph, Render, RenderApp, RenderSet,
    },
    scene::Scene,
};

use crate::render::{
    receive::{
        MainWorldPreviewImageReceiver, PreviewImageCopies, PreviewTextureToBufferLabel,
        PreviewTextureToBufferNode, RenderWorldPreviewImageSender,
    },
    PreviewRendered, PreviewSceneState, PreviewSettings, RenderedScenePreviews,
};

mod io;
mod render;
mod ui;

/// This crate is a work in progress and is not yet ready for use.
/// The intention is to provide a way to load/render/unload assets in the background and provide previews of them in the Bevy Editor.
/// For 2d assets this will be a simple sprite, for 3d assets this will require a quick render of the asset at a low resolution, just enough for a user to be able to tell quickly what it is.
/// This code may be reused for the Bevy Marketplace Viewer to provide previews of assets and plugins.
/// So long as the assets are unchanged, the previews will be cached and will not need to be re-rendered.
/// In theory this can be done passively in the background, and the previews will be ready when the user needs them.

#[derive(Component, Deref)]
pub struct PreviewAsset(pub PathBuf);

pub struct AssetPreviewPlugin;

impl Plugin for AssetPreviewPlugin {
    fn build(&self, app: &mut App) {
        let (s, r) = crossbeam_channel::unbounded();

        app.add_plugins(ExtractResourcePlugin::<PreviewImageCopies>::default())
            .add_event::<PreviewRendered>()
            .add_systems(
                Update,
                (
                    render::update_queue,
                    render::update_preview_frames_counter,
                    ui::preview_handler,
                    io::receive_preview,
                ),
            )
            .add_systems(Last, render::change_render_layers)
            .init_resource::<RenderedScenePreviews>()
            .init_resource::<PreviewSettings>()
            .init_resource::<PreviewSceneState>()
            .init_resource::<PreviewImageCopies>()
            .insert_resource(MainWorldPreviewImageReceiver(r));

        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .add_systems(
                Render,
                render::receive::receive_image_from_buffer.after(RenderSet::Render),
            )
            .insert_resource(RenderWorldPreviewImageSender(s));

        let mut graph = render_app.world_mut().resource_mut::<RenderGraph>();
        graph.add_node(PreviewTextureToBufferLabel, PreviewTextureToBufferNode);
        graph.add_node_edge(CameraDriverLabel, PreviewTextureToBufferLabel);
    }
}
