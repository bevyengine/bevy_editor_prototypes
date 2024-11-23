use std::path::Path;

use bevy::{
    asset::{
        io::{file::FileAssetReader, AssetReader},
        AssetServer,
    },
    prelude::{Commands, Entity, Query, Res, ResMut},
    tasks::block_on,
    ui::UiImage,
};

use crate::{render::RenderedScenePreviews, PreviewAsset};

const FILE_PLACEHOLDER: &'static str = "embedded://bevy_asset_browser/assets/file_icon.png";

enum PreviewType {
    Image,
    Scene,
    Other,
}

// TODO: handle assets modification
pub fn preview_handler(
    mut commands: Commands,
    mut requests_query: Query<(Entity, &PreviewAsset, Option<&mut UiImage>)>,
    asset_server: Res<AssetServer>,
    mut prerendered: ResMut<RenderedScenePreviews>,
) {
    for (entity, preview, reuseable_image) in &mut requests_query {
        let ty = match preview.extension() {
            Some(ext) => match ext.to_str().unwrap() {
                "jpeg" | "jpeg" | "png" | "bmp" | "gif" | "ico" | "pnm" | "pam" | "pbm" | "pgm"
                | "ppm" | "tga" | "webp" => PreviewType::Image,
                "glb" | "gltf" => PreviewType::Scene,
                _ => PreviewType::Other,
            },
            None => PreviewType::Other,
        };

        let preview = match ty {
            PreviewType::Image => {
                commands.entity(entity).remove::<PreviewAsset>();
                asset_server.load(preview.as_path())
            }
            PreviewType::Scene => {
                if let Some(handle) =
                    prerendered.get_or_schedule((**preview).clone(), &asset_server)
                {
                    commands.entity(entity).remove::<PreviewAsset>();
                    handle
                } else {
                    // Not rendered yet, fall back to default.
                    asset_server.load(FILE_PLACEHOLDER)
                }
            }
            PreviewType::Other => {
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
