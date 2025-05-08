use serde::{Deserialize, Serialize};
use std::{fs::File, io, path::PathBuf};

use super::ProjectInfo;

/// The name of the project cache file
const CACHE_FILE: &str = "projects.ron";
/// The name of the Bevy Editor's cache folder
const CACHE_FOLDER_NAME: &str = "Bevy Editor";

/// This is the structure that is saved in the [`CACHE_FILE`]
#[derive(Debug, Serialize, Deserialize)]
struct ProjectsCache {
    projects: Vec<ProjectInfo>,
}

/// Get Bevy Editor's cache folder path
/// `Windows`: %LOCALAPPDATA%/[`CACHE_FOLDER_NAME`]
/// `MacOS`: ~/Library/Caches/[`CACHE_FOLDER_NAME`]
/// `Linux`: ~/.cache/[`CACHE_FOLDER_NAME`]
fn get_cache_folder() -> PathBuf {
    #[cfg(target_os = "windows")]
    let path = PathBuf::from(std::env::var("LOCALAPPDATA").unwrap());

    #[cfg(target_os = "macos")]
    let path = PathBuf::from(std::env::var("HOME").unwrap()).join("Library/Caches");

    #[cfg(target_os = "linux")]
    let path = PathBuf::from(std::env::var("HOME").unwrap()).join(".cache");

    path.join(CACHE_FOLDER_NAME)
}

/// Load the projects from the cache file
pub(super) fn load_projects() -> io::Result<Vec<ProjectInfo>> {
    let cache_folder = get_cache_folder();
    let cache_file = cache_folder.join(CACHE_FILE);

    if !cache_file.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Project cache file not found",
        ));
    }

    let file = File::open(cache_file)?;
    let cache_value: ProjectsCache = ron::de::from_reader(file).map_err(|error| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Couldn't parse project cache file: {}", error),
        )
    })?;

    Ok(cache_value.projects)
}

/// Save the projects to the cache file
pub(super) fn save_projects(projects: Vec<ProjectInfo>) -> io::Result<()> {
    let cache_folder = get_cache_folder();
    let cache_file = cache_folder.join(CACHE_FILE);
    if !cache_folder.exists() {
        std::fs::create_dir(&cache_folder)?;
    }
    let cache_value = ProjectsCache { projects };
    let file = File::create(cache_file)?;
    ron::Options::default()
        .to_io_writer(file, &cache_value)
        .map_err(|error| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Failed to serialize project file cache: {}", error),
            )
        })
}
