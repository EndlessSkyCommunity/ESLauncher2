use env_logger::Logger;
use log::{Level, LevelFilter, Log, Metadata, Record};
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

pub fn init() -> mpsc::Receiver<String> {
    let (log_writer, log_reader) = mpsc::sync_channel(128);

    let channeled = Box::new(ChanneledLogger {
        channel: log_writer,
    });
    let env = Box::new(env_logger::builder().build());
    multi_log::MultiLogger::init(vec![channeled, env], log::Level::Debug).unwrap();

    log::info!("Initialized logger");
    log_reader
}
