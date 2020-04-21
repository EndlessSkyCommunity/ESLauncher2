use crate::install;
use serde::de::Unexpected::Bool;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::thread::JoinHandle;

#[derive(Debug, Clone)]
pub enum Work {
    Install { destination: PathBuf },
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
            Work::Install { destination } => {
                let destination = destination.clone();
                move || {
                    if let Err(e) = install::install(destination) {
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
