//! Module to handle Bevy Editor's project templates.

use std::path::Path;

///	The path to the folder containing the templates project
const TEMPLATE_FOLDER_PATH: &str = "templates/";

/// The names of the templates project
const TEMPLATE_NAMES: &[&str] = &["blank_project", "getting_started"];

/// The available projects template
#[derive(Debug, Default, Clone, Copy)]
#[allow(dead_code)]
pub enum Templates {
    /// Template for a blank project
    #[default]
    Blank = 0,
    /// Template for a project with basic assets to get you started
    GettingStarted = 1,
}

pub(super) async fn copy_template(template: Templates, to: &Path) -> std::io::Result<()> {
    let template_path = Path::new(TEMPLATE_FOLDER_PATH).join(TEMPLATE_NAMES[template as usize]);
    clone_directory(template_path, to)?;
    Ok(())
}

fn clone_directory<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> std::io::Result<()> {
    let from = from.as_ref();
    let to = to.as_ref();
    std::fs::create_dir_all(to)?;
    for entry in std::fs::read_dir(from)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = path.file_name().unwrap();
        let new_path = to.join(file_name);
        if path.is_dir() {
            clone_directory(path, new_path)?;
        } else {
            std::fs::copy(path, new_path)?;
        }
    }
    Ok(())
}
