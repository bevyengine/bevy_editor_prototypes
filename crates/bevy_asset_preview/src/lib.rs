mod asset;
mod preview;
mod ui;

pub use asset::*;
pub use preview::*;
pub use ui::*;

use bevy::prelude::*;

/// This crate is a work in progress and is not yet ready for use.
/// The intention is to provide a way to load/render/unload assets in the background and provide previews of them in the Bevy Editor.
/// For 2d assets this will be a simple sprite, for 3d assets this will require a quick render of the asset at a low resolution, just enough for a user to be able to tell quickly what it is.
/// This code may be reused for the Bevy Marketplace Viewer to provide previews of assets and plugins.
/// So long as the assets are unchanged, the previews will be cached and will not need to be re-rendered.
/// In theory this can be done passively in the background, and the previews will be ready when the user needs them.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AssetPreviewType {
    Image,
    GLTF,
    Scene,
    Other,
}

pub struct AssetPreviewPlugin;

impl Plugin for AssetPreviewPlugin {
    fn build(&self, app: &mut App) {
        // Initialize resources
        app.init_resource::<asset::AssetLoader>();
        app.init_resource::<preview::PreviewCache>();
        app.init_resource::<preview::PreviewConfig>();
        app.init_resource::<preview::PreviewTaskManager>();

        // Register events
        app.add_event::<asset::AssetLoadCompleted>();
        app.add_event::<asset::AssetLoadFailed>();
        app.add_event::<asset::AssetHotReloaded>();
        app.add_event::<preview::PreviewReady>();
        app.add_event::<preview::PreviewFailed>();

        // Register systems
        // Process preview requests and submit to loader
        app.add_systems(Update, ui::preview_handler);

        // Process load queue (starts new load tasks)
        app.add_systems(Update, asset::process_load_queue);

        // Handle asset events (completion, failures, hot reloads)
        app.add_systems(Update, asset::handle_asset_events);

        // Handle preview load completion and update UI
        app.add_systems(
            Update,
            (
                ui::handle_preview_load_completed,
                ui::handle_preview_load_failed,
                ui::check_failed_loads,
            )
                .after(asset::handle_asset_events),
        );

        // Handle image preview events
        app.add_systems(Update, preview::handle_image_preview_events);

        // Process 3D preview requests
        app.add_systems(
            Update,
            (
                preview::process_3d_preview_requests,
                preview::wait_for_asset_load,
            )
                .chain(),
        );

        // Capture screenshots for image previews
        app.add_systems(Update, preview::capture_preview_screenshot);

        // Handle screenshot events (currently placeholder - requires observers for EntityEvent)
        // app.add_systems(Update, preview::handle_preview_screenshots);

        // Update entity preview camera (temporarily disabled)
        // app.add_systems(
        //     Update,
        //     (
        //         preview::update_preview_camera,
        //         preview::update_preview_camera_bounds,
        //     )
        //         .chain(),
        // );
    }
}
