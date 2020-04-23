use crate::install;
use std::path::PathBuf;
use std::thread;
use std::thread::JoinHandle;

#[derive(Debug, Clone)]
pub enum Work {
    Install {
        destination: PathBuf,
        name: String,
        appimage: bool,
    },
}

#[derive(Debug)]
pub struct Worker {
    handle: JoinHandle<()>,
    logs: Vec<String>,
    work: Work,
}

impl Worker {
    pub fn new(work: Work) -> Worker {
        let func = match &work {
            Work::Install {
                destination,
                name,
                appimage,
            } => {
                let destination = destination.clone();
                let name = name.clone();
                let appimage = *appimage;
                move || {
                    if let Err(e) = install::install(destination, name, appimage) {
                        error!("Install panicked with error: {}", e);
                    }
                }
            }
        };
        let handle = thread::spawn(func);
        Worker {
            handle,
            logs: vec![],
            work,
        }
    }
}
