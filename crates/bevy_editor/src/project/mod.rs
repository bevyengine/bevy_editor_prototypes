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
    /// The path to the root of the project.
    pub path: PathBuf,
    /// The last time the project was opened.
    pub last_opened: SystemTime,
}

impl PartialEq for ProjectInfo {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

impl ProjectInfo {
    /// Get the name of the project.
    pub fn name(&self) -> Option<String> {
        Some(self.path.file_name()?.to_str()?.to_string())
    }
}

/// Create a new project with the given name and path.
/// Copy the blank project template from the local templates folder
pub async fn create_new_project(
    template: Templates,
    path: PathBuf,
) -> std::io::Result<ProjectInfo> {
    let info = ProjectInfo {
        path,
        last_opened: SystemTime::now(),
    };

    if let Err(error) = copy_template(template, info.path.as_path()).await {
        error!("Failed to create new project");
        return Err(error);
    }
    add_project(info.clone());

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

/// Update the current project info or create new ones if doesn't exist.
pub fn update_project_info() {
    let mut projects = get_local_projects();
    let current_dir = std::env::current_dir().unwrap();

    match projects.iter_mut().find(|p| p.path == current_dir) {
        Some(project) => {
            // Update info
            project.last_opened = SystemTime::now();
        }
        None => {
            // Create new info
            let project = ProjectInfo {
                path: current_dir.clone(),
                last_opened: SystemTime::now(),
            };
            projects.push(project);
        }
    }

    if let Err(error) = cache::save_projects(projects) {
        error!("Failed to update project last opened time: {:?}", error);
    }
}

/// Add project to project list
pub fn add_project(project: ProjectInfo) {
    let mut projects = get_local_projects();
    projects.push(project);

    if let Err(error) = cache::save_projects(projects) {
        error!("Failed to add project to project file cache: {:?}", error);
    }
}

/// Remove project from project list (does not delete any files)
pub fn remove_project(project: &ProjectInfo) {
    let mut projects = get_local_projects();
    projects.retain(|p| p.path != project.path);

    if let Err(error) = cache::save_projects(projects) {
        error!(
            "Failed to remove project from project file cache: {:?}",
            error
        );
    }
}

/// Run a project in editor mode.
pub fn run_project(project: &ProjectInfo) -> std::io::Result<()> {
    // Make sure the project folder exist
    if !project.path.exists() {
        return std::io::Result::Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Project root folder not found",
        ));
    }

    // Make sure it has the minium file to be a valid project
    let cargo_toml = project.path.join("Cargo.toml");
    let src_folder = project.path.join("src");
    let main_rs = src_folder.join("main.rs");
    if !cargo_toml.exists() || !src_folder.exists() || !main_rs.exists() {
        return std::io::Result::Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Project isn't a valid one of the following mising: Cargo.toml, src folder or main.rs file",
        ));
    }

    #[cfg(target_os = "windows")]
    std::process::Command::new("cmd")
        .current_dir(&project.path)
        .args(["/C", "cargo", "run"])
        .spawn()
        .map_err(|error| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to run project: {}", error),
            )
        })?;

    #[cfg(not(target_os = "windows"))]
    std::process::Command::new("sh")
        .current_dir(&project.path)
        .args(["-c", "cargo", "run"])
        .spawn()
        .map_err(|error| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to run project: {}", error),
            )
        })?;

    info!("Project started successfully");
    Ok(())
}
