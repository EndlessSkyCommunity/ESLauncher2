#![forbid(unsafe_code)]
#[macro_use]
extern crate log;

mod archive;
mod github;
mod install;
mod logger;
mod music;
mod worker;

use crate::worker::{Work, Worker};
use iced::{
    button, scrollable, Align, Button, Column, Container, Element, Font, HorizontalAlignment,
    Length, Row, Sandbox, Scrollable, Settings, Text,
};
use nfd2::Response;
use std::path::PathBuf;
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
    destination: PathBuf,
    destination_chooser: button::State,
    install_button: button::State,
    log_scrollable: scrollable::State,
    worker: Option<worker::Worker>,
    log_reader: Receiver<String>,
    log_buffer: Vec<String>,
}

#[derive(Debug, Clone)]
enum Message {
    SelectDestination,
    StartInstallation,
}

impl Sandbox for ESLauncher {
    type Message = Message;

    fn new() -> ESLauncher {
        let log_reader = logger::init();
        ESLauncher {
            destination: PathBuf::default(),
            destination_chooser: button::State::default(),
            install_button: button::State::default(),
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
            Message::SelectDestination => {
                if let Response::Okay(path) = nfd2::open_pick_folder(None).unwrap() {
                    self.destination = path;
                }
            }
            Message::StartInstallation => {
                if self.worker.is_none() {
                    self.worker = Some(Worker::new(Work::Install {
                        destination: self.destination.clone(),
                    }));
                }
            }
        };
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

        let mut install_button = Button::new(&mut self.install_button, Text::new("Install"));
        if !self.destination.eq(&PathBuf::default()) {
            install_button = install_button.on_press(Message::StartInstallation)
        }

        let content = Column::new()
            .padding(20)
            .align_items(Align::Center)
            .push(
                Row::new()
                    .padding(20)
                    .align_items(Align::Center)
                    .push(Text::new(self.destination.to_string_lossy()))
                    .push(
                        Button::new(&mut self.destination_chooser, Text::new("Pick Folder"))
                            .on_press(Message::SelectDestination),
                    ),
            )
            .push(install_button)
            .push(Scrollable::new(&mut self.log_scrollable).push(logbox)); // TODO: Autoscroll this to bottom. https://github.com/hecrj/iced/issues/307

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}
