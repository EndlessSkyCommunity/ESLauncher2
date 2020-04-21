use platform_dirs::{AppDirs, AppUI};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Instance {
    pub path: PathBuf,
    pub name: String,
}

pub fn get_instances_dir() -> Option<PathBuf> {
    let mut dir = AppDirs::new(Some("ESLauncher2"), AppUI::Graphical)?.data_dir;
    dir.push("instances");
    Some(dir)
}

pub fn get_instances() -> Option<Vec<Instance>> {
    let buf = get_instances_dir()?;
    let dir = buf.as_path();
    let mut vec = vec![];
    if dir.exists() {
        match dir.read_dir() {
            Ok(readdir) => {
                for result in readdir {
                    match result {
                        Ok(entry) => match entry.file_type() {
                            Ok(file_type) => {
                                if file_type.is_dir() {
                                    match entry.file_name().into_string() {
                                        Ok(name) => vec.push(Instance {
                                            path: entry.path(),
                                            name,
                                        }),
                                        Err(_) => error!(
                                            "Failed to convert filename of {} to String",
                                            entry.path().to_string_lossy(),
                                        ),
                                    };
                                }
                            }
                            Err(e) => error!(
                                "Failed to get filetype of {}: {}",
                                entry.path().to_string_lossy(),
                                e
                            ),
                        },
                        Err(e) => error!("Failed to read entry from instances folder: {}", e),
                    }
                }
            }
            Err(e) => error!("Failed to read from instances folder: {}", e),
        };
    } else if let Err(e) = fs::create_dir_all(dir) {
        error!("Failed to create instances dir: {}", e);
    }
    Some(vec)
}
