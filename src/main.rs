#![forbid(unsafe_code)]

mod archive;
mod github;
mod install;
mod music;
mod worker;

use crate::worker::{Work, Worker};
use iced::{button, Align, Button, Column, Container, Element, Length, Sandbox, Settings, Text};
use nfd2::Response;
use std::path::PathBuf;

static WIDTH: u16 = 460;
static HEIGHT: u16 = 500;

pub fn main() {
    ESLauncher::run(Settings::default())
}

#[derive(Debug)]
struct ESLauncher {
    destination: PathBuf,
    destination_chooser: button::State,
    install_button: button::State,
    worker: Option<worker::Worker>,
}

#[derive(Debug, Clone)]
enum Message {
    SelectDestination,
    StartInstallation,
}

impl Sandbox for ESLauncher {
    type Message = Message;

    fn new() -> ESLauncher {
        ESLauncher {
            destination: PathBuf::default(),
            destination_chooser: button::State::default(),
            install_button: button::State::default(),
            worker: None,
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
        let logbox = match &self.worker {
            Some(worker) => self.worker.as_mut().unwrap().logs().iter().cloned().fold(
                Column::new().padding(20).align_items(Align::Center),
                |column, log| column.push(Text::new(log)),
            ),
            None => Column::new(),
        };
        let content = Column::new()
            .padding(20)
            .align_items(Align::Center)
            .push(Text::new(self.destination.to_string_lossy()))
            .push(
                Button::new(&mut self.destination_chooser, Text::new("Pick Folder"))
                    .on_press(Message::SelectDestination),
            )
            .push(
                Button::new(&mut self.install_button, Text::new("Install"))
                    .on_press(Message::StartInstallation),
            )
            .push(logbox);

        Container::new(content)
            .width(Length::from(WIDTH))
            .height(Length::from(HEIGHT))
            .center_x()
            .center_y()
            .into()
    }
}
