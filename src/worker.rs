use crate::install;
use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::thread;
use std::thread::JoinHandle;

#[derive(Debug, Clone)]
pub enum Work {
    Install { destination: PathBuf },
}

#[derive(Debug)]
pub struct Worker {
    receiver: Receiver<String>,
    handle: JoinHandle<()>,
    logs: Vec<String>,
    work: Work,
}

impl Worker {
    pub fn new(work: Work) -> Worker {
        let (sender, receiver) = mpsc::channel();

        let func = match &work {
            Work::Install { destination } => {
                let destination = destination.clone();
                move || install::install(sender, destination)
            }
        };
        let handle = thread::spawn(func);
        Worker {
            receiver,
            handle,
            logs: vec![],
            work,
        }
    }

    pub fn logs(&mut self) -> &Vec<String> {
        let optional = self.receiver.try_recv().ok(); // TODO: React to disconnects
        if let Some(log) = optional {
            self.logs.push(log)
        };
        &self.logs
    }
}
