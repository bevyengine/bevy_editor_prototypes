//! A Viewer for the Future Bevy Marketplace, a place where you can find Bevy plugins and assets that members of the community have created.
/// This crate is a work in progress and is not yet ready for use.
/// The intention is to provide a way to browse the marketplace and install plugins and assets directly from the Bevy Editor.
/// Particularly, this crate will connect to the Bevy Marketplace API to fetch the latest plugins and assets, and provide previews, overviews, and (If any) prices of them.
/// If the crate is present in the editor (aka not been removed, or disabled).
/// The crate will pass this data to the asset browser crate to be displayed in the Bevy Editor.
/// The UI/UX of the content will live in the asset browser crate, and this crate will only be responsible for fetching the data.
/// As a security measure purchasing of assets/plugins will be done through the Bevy Marketplace website.
use bevy::prelude::*;

/// The Bevy Marketplace Viewer Plugin.
pub struct MarketplaceViewerPlugin;

impl Plugin for MarketplaceViewerPlugin {
    fn build(&self, _app: &mut App) {}
}
