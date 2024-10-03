/// Load a type implementing `serde::Deserialize` from a TOML file.
pub fn load<T>(path: impl AsRef<std::path::Path>) -> Result<T, PersistantError>
where
    T: serde::de::DeserializeOwned,
{
    let path = path.as_ref();
    let file = std::fs::read_to_string(path).unwrap();
    Ok(toml::from_str(&file)?)
}

#[inline]
/// TODO: when the editor is an external applcation this should be moved to the user's configuration directory
fn user_settings_path() -> Result<std::path::PathBuf, PersistantError> {
    Ok(std::env::var("CARGO_MANIFEST_DIR")
        .map(std::path::PathBuf::from)
        .map_err(|_| PersistantError::WorkspaceConfigDirs)?
        .join("user.toml"))
}

/// Save the user settings to the default location.
pub fn save_user_settings(
    settings: &crate::modals::user::UserSettings,
) -> Result<(), PersistantError> {
    let path = user_settings_path()?;
    let toml_string = toml::to_string(settings)?;

    std::fs::write(path, toml_string)?;

    Ok(())
}

/// Load the user settings from the default location.
pub fn load_user_settings() -> Result<crate::modals::user::UserSettings, PersistantError> {
    let path = user_settings_path()?;

    load(path)
}

/// Load the workspace settings from the default location.
pub fn load_workspace_settings(
) -> Result<crate::modals::workspace::WorkspaceSettings, PersistantError> {
    let path = std::env::var("CARGO_MANIFEST_DIR")
        .map(std::path::PathBuf::from)
        .map_err(|_| PersistantError::WorkspaceConfigDirs)?;

    load(path.join("Bevy.toml"))
}

/// Errors that can occur when loading a TOML file.
#[derive(Debug, thiserror::Error)]
pub enum PersistantError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("TOML deserialization error: {0}")]
    TomlDe(#[from] toml::de::Error),
    #[error("TOML serialization error: {0}")]
    TomlSer(#[from] toml::ser::Error),
    #[error("Error reading CARGO_MANIFEST_DIR required for workspace settings")]
    WorkspaceConfigDirs,
}
