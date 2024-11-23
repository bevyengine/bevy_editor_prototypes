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

// TODO: handle assets modification
pub fn preview_handler(
    mut commands: Commands,
    mut requests_query: Query<(Entity, &PreviewAsset, Option<&mut UiImage>)>,
    asset_server: Res<AssetServer>,
    mut prerendered: ResMut<RenderedScenePreviews>,
) {
    for (entity, request, reuseable_image) in &mut requests_query {
        let preview = match request {
            PreviewAsset::Image(handle) => {
                commands.entity(entity).remove::<PreviewAsset>();
                handle.clone()
            }
            PreviewAsset::Scene(handle) => {
                let path = asset_server.get_path(handle).map(|p| {
                    Path::new("cache")
                        .join("asset_preview")
                        .join(p.path().with_extension("png"))
                });
                let reader = FileAssetReader::new("cache");
                dbg!(&handle, handle.id());

                if path
                    .as_ref()
                    .is_some_and(|p| block_on(reader.read(p)).is_ok())
                {
                    asset_server.load(path.unwrap())
                } else if let Some(handle) = prerendered.get_or_schedule(handle.clone()) {
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
