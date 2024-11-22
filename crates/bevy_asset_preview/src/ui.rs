use bevy::{
    asset::AssetServer,
    prelude::{Commands, Entity, Query, Res, ResMut},
    ui::UiImage,
};

use crate::{render::PrerenderedMeshes, RequestPreview};

const FILE_PLACEHOLDER: &'static str = "embedded://bevy_asset_browser/assets/file_icon.png";

pub fn preview_handler(
    mut commands: Commands,
    mut requests_query: Query<(Entity, &RequestPreview, Option<&mut UiImage>)>,
    asset_server: Res<AssetServer>,
    mut prerendered: ResMut<PrerenderedMeshes>,
) {
    for (entity, request, reuseable_image) in &mut requests_query {
        let preview = match request {
            RequestPreview::Image(handle) => {
                commands.entity(entity).remove::<RequestPreview>();
                handle.clone()
            }
            RequestPreview::Mesh(handle) => {
                if let Some(handle) = prerendered.get_or_schedule(handle.clone()) {
                    commands.entity(entity).remove::<RequestPreview>();
                    handle
                } else {
                    // Not rendered yet, fall back to default.
                    asset_server.load(FILE_PLACEHOLDER)
                }
            }
        };

        if let Some(mut reuseable) = reuseable_image {
            reuseable.image = preview;
        } else {
            // TODO: sprite atlas.
            commands.entity(entity).insert(UiImage {
                image: preview,
                ..Default::default()
            });
        }
    }
}
