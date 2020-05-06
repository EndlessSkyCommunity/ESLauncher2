#![forbid(unsafe_code)]
#[macro_use]
extern crate log;
#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate version;

mod archive;
mod github;
mod install;
mod install_frame;
mod instance;
mod instances_frame;
mod logger;
mod style;
mod update;

use crate::install_frame::InstallFrameMessage;
use crate::instance::{Instance, InstanceMessage};
use iced::{
    scrollable, Align, Application, Column, Command, Container, Element, HorizontalAlignment,
    Length, Row, Scrollable, Settings, Text,
};
use std::path::PathBuf;
use std::sync::mpsc::Receiver;
use std::thread;

pub fn main() {
    ESLauncher::run(Settings::default())
}

#[derive(Debug)]
struct ESLauncher {
    install_frame: install_frame::InstallFrame,
    instances_frame: instances_frame::InstancesFrameState,
    log_scrollable: scrollable::State,
    log_reader: Receiver<String>,
    log_buffer: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum Message {
    InstallFrameMessage(InstallFrameMessage),
    InstanceMessage(usize, InstanceMessage),
    Installed(Option<Instance>),
    Deleted(Option<PathBuf>),
    Updated(Option<Instance>),
    Dummy(()),
}

impl Application for ESLauncher {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flag: ()) -> (ESLauncher, Command<Message>) {
        let log_reader = logger::init();
        info!("Starting ESLauncher2 v{}", version!());
        check_for_update();

        (
            ESLauncher {
                install_frame: install_frame::InstallFrame::default(),
                instances_frame: instances_frame::InstancesFrameState::default(),
                log_scrollable: scrollable::State::default(),
                log_reader,
                log_buffer: vec![],
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        format!("ESLauncher2 v{}", version!())
    }

    fn update(&mut self, message: Self::Message) -> Command<Message> {
        match message {
            Message::InstallFrameMessage(msg) => return self.install_frame.update(msg),
            Message::InstanceMessage(i, msg) => {
                if let Some(instance) = self.instances_frame.instances.get_mut(i) {
                    return instance.update(msg);
                }
            }
            Message::Installed(option) => {
                if let Some(instance) = option {
                    self.instances_frame.instances.push(instance);
                    instance::perform_save_instances(self.instances_frame.instances.clone());
                }
            }
            Message::Deleted(option) => {
                if let Some(path) = option {
                    self.instances_frame.instances.retain(|i| !i.path.eq(&path));
                    instance::perform_save_instances(self.instances_frame.instances.clone());
                }
            }
            Message::Updated(option) => {
                if let Some(instance) = option {
                    self.instances_frame
                        .instances
                        .retain(|i| !i.path.eq(&instance.path));
                    self.instances_frame.instances.push(instance);
                    instance::perform_save_instances(self.instances_frame.instances.clone());
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
                        .size(15)
                        .font(style::LOG_FONT)
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
                    .push(self.install_frame.view().map(Message::InstallFrameMessage))
                    .spacing(50),
            )
            .push(
                Scrollable::new(&mut self.log_scrollable)
                    .push(logbox)
                    .padding(20)
                    .align_items(Align::Start)
                    .width(Length::Fill),
            ); // TODO: Autoscroll this to bottom. https://github.com/hecrj/iced/issues/307

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}

fn check_for_update() {
    thread::spawn(|| match github::get_latest_release() {
        Ok(release) => {
            if !format!("v{}", version!()).eq(&release.tag_name) {
                info!("The latest version of ESLauncher2 is {}", release.tag_name)
            }
        }
        Err(e) => error!("Failed to fetch latest ESLauncher2 release: {}", e),
    });
}
