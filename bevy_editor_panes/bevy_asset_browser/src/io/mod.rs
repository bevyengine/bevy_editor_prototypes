//! this module encapsulate all the asset browser IO operations

pub(crate) mod task;

use std::{fs::create_dir_all, path::PathBuf};

/// Create a new folder called "New Folder" in the parent directory
/// If a folder with the same name already exists, it will increment the name until it's unique
pub fn create_new_folder(mut parent: PathBuf) -> std::io::Result<String> {
    parent.push("New Folder");
    // increment name until it's unique
    let mut index = 0;
    while parent.exists() {
        // increment name and rename last part of the path
        index += 1;
        parent.pop();
        parent.push(format!("New Folder {}", index));
    }
    create_dir_all(&parent)?;
    Ok(parent
        .components()
        .last()
        .unwrap()
        .as_os_str()
        .to_str()
        .unwrap()
        .to_string())
}

/// Delete a folder and all its content
pub fn delete_folder(path: PathBuf) -> std::io::Result<()> {
    std::fs::remove_dir_all(path)?;
    Ok(())
}
