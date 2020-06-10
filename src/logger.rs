use crate::instance::{get_instances_dir};
use log::{Level, Log, Metadata, Record};
use simplelog::{
    CombinedLogger, Config, LevelFilter, SharedLogger, TermLogger, TerminalMode, WriteLogger,
};
use std::fs;
use std::fs::File;
use std::sync::mpsc;

const BLACKLIST: [&str; 4] = ["gfx_backend_", "winit", "wgpu_", "iced_"];

struct ChanneledLogger {
    channel: mpsc::SyncSender<String>,
}

impl Log for ChanneledLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) && should_log(record) {
            let line = format!(
                "{:<7} [{}] {}",
                record.metadata().level(),
                record.module_path().unwrap_or("unknown"),
                record.args()
            );
            let _ = self.channel.try_send(line);
        }
    }

    fn flush(&self) {}
}

impl SharedLogger for ChanneledLogger {
    fn level(&self) -> LevelFilter {
        LevelFilter::Warn
    }

    fn config(&self) -> Option<&Config> {
        None
    }

    fn as_log(self: Box<Self>) -> Box<dyn Log> {
        unimplemented!()
    }
}

fn should_log(record: &Record) -> bool {
    match record.module_path() {
        Some(path) => {
            for x in BLACKLIST.iter() {
                if path.contains(x) {
                    return false;
                }
            }
            true
        }
        None => true,
    }
}

fn open_logfile() -> File {
    let mut path = std::env::current_dir().unwrap();
    if cfg!(target_os = "macos") {
        match get_instances_dir() {
            Some(instance_path) => {
                path = instance_path;
                fs::create_dir_all(path.clone()).expect("Creation of instance directories failed.");
            }
            None => {}
        }
    }
    path.push("ESLauncher2.log");
    File::create(path).unwrap()
}

pub fn init() -> mpsc::Receiver<String> {
    let (log_writer, log_reader) = mpsc::sync_channel(128);

    let channeled = ChanneledLogger {
        channel: log_writer,
    };
    CombinedLogger::init(vec![
        Box::new(channeled),
        TermLogger::new(LevelFilter::Info, Config::default(), TerminalMode::Mixed),
        WriteLogger::new(LevelFilter::Debug, Config::default(), open_logfile()),
    ])
    .unwrap();

    log::info!("Initialized logger");
    log_reader
}
