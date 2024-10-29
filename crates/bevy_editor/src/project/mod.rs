//! This module contains project management functionalities for the Bevy Editor.

use bevy::log::{error, info};
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, time::SystemTime};
use templates::{copy_template, Templates};

mod cache;
pub mod templates;

/// Basic information about a project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInfo {
    /// The name of the project.
    pub name: String,
    /// The path to the root of the project.
    pub path: PathBuf,
    /// The last time the project was opened.
    pub last_opened: SystemTime,
}

/// Create a new project with the given name and path.
/// Copy the blank project template from the local templates folder
pub async fn create_new_project(
    template: Templates,
    name: String,
    path: PathBuf,
) -> std::io::Result<ProjectInfo> {
    let info = ProjectInfo {
        name,
        path,
        last_opened: SystemTime::now(),
    };

    if let Err(error) = copy_template(template, &info.path.as_path()).await {
        error!("Failed to create new project");
        return Err(error);
    }

    let mut projects = get_local_projects();
    projects.push(info.clone());
    if let Err(error) = cache::save_projects(projects) {
        error!("Failed to add new project to project file cache");
        return Err(error);
    }

    Ok(info)
}

/// Get all projects that have been opened in the editor.
pub fn get_local_projects() -> Vec<ProjectInfo> {
    match cache::load_projects() {
        Ok(projects) => projects,
        Err(error) => {
            error!("Failed to load projects from cache file: {:?}", error);
            Vec::new()
        }
    }
}

/// Run a project in editor mode.
pub fn run_project(project: ProjectInfo) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    std::process::Command::new("cmd")
        .current_dir(&project.path)
        .args(&["/C", "cargo", "run"])
        .spawn()
        .map_err(|error| error.to_string())?;

    #[cfg(not(target_os = "windows"))]
    unimplemented!("Run project is not implemented for this platform");

    info!("Project '{}' started successfully", project.name);
    Ok(())
}
