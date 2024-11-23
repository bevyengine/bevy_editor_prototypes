use bevy::{
    asset::AssetServer,
    gltf::GltfAssetLabel,
    prelude::{Commands, Entity, Query, Res, ResMut},
    ui::UiImage,
};

use crate::{render::PrerenderedScenes, PreviewAsset};

const FILE_PLACEHOLDER: &'static str = "embedded://bevy_asset_browser/assets/file_icon.png";

// TODO: handle assets modification
pub fn preview_handler(
    mut commands: Commands,
    mut requests_query: Query<(Entity, &PreviewAsset, Option<&mut UiImage>)>,
    asset_server: Res<AssetServer>,
    mut prerendered: ResMut<PrerenderedScenes>,
) {
    for (entity, request, reuseable_image) in &mut requests_query {
        let preview = match request {
            PreviewAsset::Image(handle) => {
                commands.entity(entity).remove::<PreviewAsset>();
                handle.clone()
            }
            PreviewAsset::Scene(handle) => {
                if let Some(handle) = prerendered.get_or_schedule(handle.clone()) {
                    commands.entity(entity).remove::<PreviewAsset>();
                    handle
                } else {
                    // Not rendered yet, fall back to default.
                    asset_server.load(FILE_PLACEHOLDER)
                }
            }
            PreviewAsset::Other => {
                commands.entity(entity).remove::<PreviewAsset>();
                asset_server.load(FILE_PLACEHOLDER)
            }
        };

        if let Some(mut reuseable) = reuseable_image {
            reuseable.texture = preview;
        } else {
            // TODO: sprite atlas.
            commands.entity(entity).insert(UiImage {
                texture: preview,
                ..Default::default()
            });
        }
    }
}
