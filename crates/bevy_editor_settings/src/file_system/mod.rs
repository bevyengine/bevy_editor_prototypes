use std::path::PathBuf;

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
        load_global_settings(app.world_mut());
    }
    load_project_settings(app.world_mut());
}

pub fn load_project_settings(world: &mut bevy::prelude::World) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let Ok(file) = load::load_toml_file(path.join("Bevy.toml")) else {
        warn!("Failed to load project settings");
        return;
    };

    load::load_preferences(world, file, SettingsType::Project);
}

pub fn load_global_settings(world: &mut bevy::prelude::World) {
    let path = &world.get_resource::<GlobalSettingsPath>().unwrap().0;
    let Ok(file) = load::load_toml_file(path.join("global.toml")) else {
        warn!("Failed to load global settings");
        return;
    };

    load::load_preferences(world, file, SettingsType::Global);
    
}


