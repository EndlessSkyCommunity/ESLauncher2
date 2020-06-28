use crate::get_data_dir;
use log::{Level, Log, Metadata, Record};
use simplelog::{
    CombinedLogger, Config, ConfigBuilder, LevelFilter, SharedLogger, TermLogger, TerminalMode,
    WriteLogger,
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

fn open_logfile() -> Option<File> {
    let mut path = std::env::current_dir().unwrap();
    if let Some(data_dir) = get_data_dir() {
        match fs::create_dir_all(&data_dir) {
            Ok(_) => path = data_dir,
            Err(e) => eprintln!(
                "Creation of data dir ({}) failed due to {}! Falling back to logging to the PWD ({})",
                data_dir.to_string_lossy(), e,
                path.to_string_lossy()
            ),
        }
    }
    path.push("ESLauncher2.log");
    match File::create(&path) {
        Err(e) => {
            eprintln!(
                "Failed to create logfile at {}: {}",
                path.to_string_lossy(),
                e
            );
            None
        }
        Ok(f) => Some(f),
    }
}

pub fn init() -> mpsc::Receiver<String> {
    let (log_writer, log_reader) = mpsc::sync_channel(128);
    let channeled = ChanneledLogger {
        channel: log_writer,
    };

    let config = ConfigBuilder::new()
        .add_filter_ignore_str("iced_wgpu::renderer") // STOP
        .add_filter_ignore_str("wgpu_native::device") // SPAMMING
        .add_filter_ignore_str("wgpu_native::command") // AAAAAH
        .build();

    let mut loggers: Vec<Box<dyn SharedLogger>> = vec![
        Box::new(channeled),
        TermLogger::new(LevelFilter::Debug, config.clone(), TerminalMode::Mixed),
    ];

    if let Some(file) = open_logfile() {
        loggers.push(WriteLogger::new(LevelFilter::Debug, config, file));
    }

    CombinedLogger::init(loggers).unwrap();

    log::info!("Initialized logger");
    log_reader
}
