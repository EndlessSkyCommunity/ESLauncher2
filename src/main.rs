#![forbid(unsafe_code)]
#[macro_use]
extern crate log;

mod archive;
mod github;
mod install;
mod install_frame;
mod logger;
mod music;
mod worker;

use crate::worker::{Work, Worker};
use iced::{
    scrollable, Align, Column, Container, Element, Font, HorizontalAlignment, Length, Sandbox,
    Scrollable, Settings, Text,
};
use platform_dirs::{AppDirs, AppUI};
use std::sync::mpsc::Receiver;

static LOG_FONT: Font = Font::External {
    name: "DejaVuSansMono",
    bytes: include_bytes!("../assets/DejaVuSansMono-Bold.ttf"),
};

pub fn main() {
    music::play();
    ESLauncher::run(Settings::default())
}

#[derive(Debug)]
struct ESLauncher {
    installation_frame: install_frame::InstallFrameState,
    log_scrollable: scrollable::State,
    worker: Option<worker::Worker>,
    log_reader: Receiver<String>,
    log_buffer: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum Message {
    NameChanged(String),
    StartInstallation,
}

impl Sandbox for ESLauncher {
    type Message = Message;

    fn new() -> ESLauncher {
        let log_reader = logger::init();
        ESLauncher {
            installation_frame: install_frame::InstallFrameState::default(),
            log_scrollable: scrollable::State::default(),
            worker: None,
            log_reader,
            log_buffer: vec![],
        }
    }

    fn title(&self) -> String {
        String::from("ESLauncher2")
    }

    fn update(&mut self, message: Self::Message) {
        match message {
            Message::NameChanged(name) => self.installation_frame.name = name,
            Message::StartInstallation => {
                let mut destination = AppDirs::new(Some("ESLauncher2"), AppUI::Graphical)
                    .unwrap()
                    .data_dir;
                destination.push("instances");
                destination.push(&self.installation_frame.name);
                self.worker = Some(Worker::new(Work::Install { destination }));
            }
        }
    }

    fn view(&mut self) -> Element<'_, Self::Message> {
        // Update logs
        while let Ok(line) = self.log_reader.try_recv() {
            self.log_buffer.push(line);
        }

        let logbox = self.log_buffer.iter().fold(
            Column::new().padding(20).align_items(Align::Center),
            |column, log| {
                column.push(
                    Text::new(log)
                        .size(14)
                        .font(LOG_FONT)
                        .horizontal_alignment(HorizontalAlignment::Left),
                )
            },
        );

        let content = Column::new()
            .padding(20)
            .align_items(Align::Center)
            .push(install_frame::view(&mut self.installation_frame))
            .push(Scrollable::new(&mut self.log_scrollable).push(logbox)); // TODO: Autoscroll this to bottom. https://github.com/hecrj/iced/issues/307

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}
