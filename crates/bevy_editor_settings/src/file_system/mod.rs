use bevy::log::{error, warn};

use crate::{GlobalSettingsPath, SettingsType};

pub mod load;

const SETTINGS_BASE_DIR: &str = "bevy_editor";

pub fn global_settings_path() -> Option<std::path::PathBuf> {
    let path = directories::BaseDirs::new()?;
    let config_dir = path.config_dir();
    let path = config_dir.join(SETTINGS_BASE_DIR);

    if !path.exists() {
        if let Err(e) = std::fs::create_dir_all(&path) {
            error!("Failed to create global settings directory: {}", e);
            return None;
        }
    }
    Some(path)
}

pub fn load_settings(app: &mut bevy::app::App) {
    if  app.world().get_resource::<GlobalSettingsPath>().is_some() {
        load_global_settings(app);
    }
}

pub fn load_global_settings(app: &mut bevy::app::App) {
    let path = &app.world().get_resource::<GlobalSettingsPath>().unwrap().0;
    let Ok(file) = load::load_toml_file(path.join("global.toml")) else {
        warn!("Failed to load global settings");
        return;
    };

    load::load_preferences(app.world_mut(), file, SettingsType::Global);
    
}

