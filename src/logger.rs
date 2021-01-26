use crate::{get_data_dir, Message};
use lazy_static::lazy_static;
use log::{Level, Log, Metadata, Record};
use simplelog::{
    CombinedLogger, Config, ConfigBuilder, LevelFilter, SharedLogger, TermLogger, TerminalMode,
    WriteLogger,
};
use std::collections::VecDeque;
use std::fs;
use std::fs::File;
use std::hash::Hash;
use std::sync::Mutex;
use std::time::Duration;

const BLACKLIST: [&str; 4] = ["gfx_backend_", "winit", "wgpu_", "iced_"];

lazy_static! {
    static ref LOG_QUEUE: Mutex<VecDeque<String>> = Mutex::new(VecDeque::new());
}

struct ChanneledLogger {}

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
            match LOG_QUEUE.lock() {
                Ok(mut logs) => logs.push_back(line),
                Err(e) => {
                    // Don't use an error log here because that will likely cause an endless loop of logs
                    eprintln!("Failed to lock log vector:\n{}\nThis message will should have been logged in the UI:\n{}", e, line)
                }
            }
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

#[derive(Debug, Clone)]
pub struct LogReceiver {}

impl<H, I> iced_native::subscription::Recipe<H, I> for LogReceiver
where
    H: std::hash::Hasher,
{
    type Output = crate::Message;

    fn hash(&self, state: &mut H) {
        std::any::TypeId::of::<Self>().hash(state);
    }

    fn stream(
        self: Box<Self>,
        _input: futures::stream::BoxStream<'static, I>,
    ) -> futures::stream::BoxStream<'static, Self::Output> {
        Box::pin(futures::stream::unfold(0, |state| async move {
            loop {
                tokio::time::delay_for(Duration::from_millis(10)).await;
                if let Some(msg) = LOG_QUEUE
                    .try_lock()
                    .ok()
                    .map(|mut q| q.pop_front())
                    .flatten()
                {
                    return Some((Message::Log(msg), state));
                }
            }
        }))
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

pub fn init() -> LogReceiver {
    let channeled = ChanneledLogger {};

    let config = ConfigBuilder::new()
        .add_filter_ignore_str("iced_wgpu::renderer") // STOP
        .add_filter_ignore_str("wgpu_native::device") // SPAMMING
        .add_filter_ignore_str("wgpu_native::command") // AAAAAH
        .add_filter_ignore_str("gfx_backend_metal::device") // spammy thing on mac
        .add_filter_ignore_str("hyper::proto")
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
    LogReceiver {}
}
