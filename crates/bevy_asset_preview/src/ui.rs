use std::any::TypeId;

use bevy::{
    asset::AssetServer,
    prelude::{Commands, Entity, EventReader, Image, Mesh, Query, Res},
    ui::UiImage,
};

use crate::{render::PrerenderedMesh, RequestPreview};

const FILE_PLACEHOLDER: &'static str = "embedded://bevy_asset_browser/assets/file_icon.png";

pub fn preview_handler(
    mut commands: Commands,
    mut requests_query: Query<(Entity, &RequestPreview, Option<&mut UiImage>)>,
    asset_server: Res<AssetServer>,
    prerendered: Res<PrerenderedMesh>,
) {
    for (entity, request, reuseable_image) in &mut requests_query {
        // let preview = match request {
        //     RequestPreview::Image(handle) => handle.clone(),
        //     RequestPreview::Mesh(handle) => prerendered.get(handle),
        // };
    }
}
