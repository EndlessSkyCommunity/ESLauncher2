#![forbid(unsafe_code)]
#[macro_use]
extern crate log;

mod archive;
mod github;
mod install;
mod install_frame;
mod instances;
mod instances_frame;
mod logger;
mod music;
mod worker;

use crate::instances::get_instances_dir;
use crate::worker::{Work, Worker};
use iced::{
    scrollable, Align, Column, Container, Element, Font, HorizontalAlignment, Length, Row, Sandbox,
    Scrollable, Settings, Text,
};
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
    instances_frame: instances_frame::InstancesFrameState,
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
            instances_frame: instances_frame::InstancesFrameState::default(),
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
            Message::StartInstallation => match get_instances_dir() {
                Some(mut destination) => {
                    destination.push(&self.installation_frame.name);
                    self.worker = Some(Worker::new(Work::Install { destination }));
                }
                None => error!("Could not get instances directory from AppDirs"),
            },
        }
    }

    fn view(&mut self) -> Element<'_, Self::Message> {
        // Update logs
        while let Ok(line) = self.log_reader.try_recv() {
            self.log_buffer.push(line);
        }

        let logbox = self.log_buffer.iter().fold(
            Column::new().padding(20).align_items(Align::Start),
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
            .push(
                Row::new()
                    .push(instances_frame::view(&mut self.instances_frame))
                    .push(install_frame::view(&mut self.installation_frame))
                    .spacing(100),
            )
            .push(
                Scrollable::new(&mut self.log_scrollable)
                    .push(logbox)
                    .padding(20),
            ); // TODO: Autoscroll this to bottom. https://github.com/hecrj/iced/issues/307

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}
