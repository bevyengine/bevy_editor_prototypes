use std::path::PathBuf;

use bevy::{
    asset::AssetPath,
    prelude::{Handle, Image},
};
use thiserror::Error;

/// Errors that can occur during asset operations.
#[derive(Error, Debug, Clone)]
pub enum AssetError {
    #[error("Image not found: {handle:?}")]
    ImageNotFound { handle: Handle<Image> },

    #[error("Failed to convert image to dynamic format: {0}")]
    ImageConversionFailed(String),

    #[error("Failed to encode image to {format}: {reason}")]
    ImageEncodeFailed { format: String, reason: String },

    #[error("Failed to create directory {path:?}: {reason}")]
    DirectoryCreationFailed { path: PathBuf, reason: String },

    #[error("Failed to write file to {path:?}: {reason}")]
    FileWriteFailed { path: PathBuf, reason: String },

    #[error("Asset load failed for {path:?}: {reason}")]
    AssetLoadFailed {
        path: AssetPath<'static>,
        reason: String,
    },

    #[error("Asset was removed (possibly failed to load): {path:?}")]
    AssetRemoved { path: AssetPath<'static> },

    #[error("IO error: {0}")]
    Io(String),

    #[error("Asset writer error: {0}")]
    AssetWriter(String),
}
