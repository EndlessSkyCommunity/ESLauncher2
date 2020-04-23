#![forbid(unsafe_code)]
#[macro_use]
extern crate log;

mod archive;
mod github;
mod install;
mod install_frame;
mod instance;
mod instances_frame;
mod logger;
mod music;
mod style;
mod update;

use crate::instance::{get_instances_dir, Instance, InstanceMessage};
use iced::{
    scrollable, Align, Application, Column, Command, Container, Element, Font, HorizontalAlignment,
    Length, Row, Scrollable, Settings, Text,
};
use std::path::PathBuf;
use std::sync::mpsc::Receiver;

static LOG_FONT: Font = Font::External {
    name: "DejaVuSansMono-Bold",
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
    log_reader: Receiver<String>,
    log_buffer: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum Message {
    NameChanged(String),
    StartInstallation,
    InstanceMessage(usize, InstanceMessage),
    Installed(Option<Instance>),
    Deleted(Option<PathBuf>),
    Dummy(()),
}

impl Application for ESLauncher {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flag: ()) -> (ESLauncher, Command<Message>) {
        let log_reader = logger::init();
        (
            ESLauncher {
                installation_frame: install_frame::InstallFrameState::default(),
                instances_frame: instances_frame::InstancesFrameState::default(),
                log_scrollable: scrollable::State::default(),
                log_reader,
                log_buffer: vec![],
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("ESLauncher2")
    }

    fn update(&mut self, message: Self::Message) -> Command<Message> {
        match message {
            Message::NameChanged(name) => self.installation_frame.name = name,
            Message::StartInstallation => match get_instances_dir() {
                Some(mut destination) => {
                    destination.push(&self.installation_frame.name);
                    let name = String::from(destination.file_name().unwrap().to_string_lossy());
                    return Command::perform(
                        instance::perform_install(destination, name, true),
                        Message::Installed,
                    );
                }
                None => error!("Could not get instances directory from AppDirs"),
            },
            Message::InstanceMessage(i, msg) => {
                if let Some(instance) = self.instances_frame.instances.get_mut(i) {
                    return instance.update(msg);
                }
            }
            Message::Installed(option) => {
                if let Some(instance) = option {
                    self.instances_frame.instances.push(instance);
                }
            }
            Message::Deleted(option) => {
                if let Some(path) = option {
                    self.instances_frame.instances.retain(|i| !i.path.eq(&path))
                }
            }
            Message::Dummy(_) => (),
        }
        Command::none()
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
                    .padding(20)
                    .align_items(Align::Start),
            ); // TODO: Autoscroll this to bottom. https://github.com/hecrj/iced/issues/307

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}
