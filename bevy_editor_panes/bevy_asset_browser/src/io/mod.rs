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
        parent.push(format!("New Folder {index}"));
    }
    create_dir_all(&parent)?;
    Ok(parent
        .components()
        .next_back()
        .unwrap()
        .as_os_str()
        .to_str()
        .unwrap()
        .to_string())
}

/// Create a new rust file with an empty system inside
pub fn create_new_script(mut parent: PathBuf) -> std::io::Result<String> {
    parent.push("script.rs");
    // increment name until it's unique
    let mut index = 0;
    while parent.exists() {
        // increment name and rename last part of the path
        index += 1;
        parent.pop();
        parent.push(format!("script_{index}.rs"));
    }
    std::fs::write(
        &parent,
        "pub fn new_system(mut commands: Commands) {\n    // system\n}\n",
    )?;
    Ok(parent
        .components()
        .next_back()
        .unwrap()
        .as_os_str()
        .to_str()
        .unwrap()
        .to_string())
}

/// Open the folder in the file manager that the target os uses
pub fn open_in_file_manager(path: PathBuf) -> std::io::Result<()> {
    // TODO: test for windows and mac (works on linux)
    #[cfg(target_os = "windows")]
    std::process::Command::new("explorer")
        .arg(path.as_os_str().to_str().unwrap())
        .spawn()?;
    #[cfg(target_os = "linux")]
    std::process::Command::new("xdg-open")
        .arg(path.as_os_str().to_str().unwrap())
        .spawn()?;
    #[cfg(target_os = "macos")]
    std::process::Command::new("open")
        .arg(path.as_os_str().to_str().unwrap())
        .spawn()?;
    Ok(())
}

/// Delete a file
pub fn delete_file(path: PathBuf) -> std::io::Result<()> {
    std::fs::remove_file(path)?;
    Ok(())
}
/// Delete a folder and all its content
pub fn delete_folder(path: PathBuf) -> std::io::Result<()> {
    std::fs::remove_dir_all(path)?;
    Ok(())
}
