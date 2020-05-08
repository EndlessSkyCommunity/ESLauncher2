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
mod music;
mod style;
mod update;

use crate::install_frame::InstallFrameMessage;
use crate::instance::{Instance, InstanceMessage};
use crate::music::{MusicCommand, MusicState};
use iced::{
    button, scrollable, Align, Application, Button, Column, Command, Container, Element,
    HorizontalAlignment, Length, Row, Scrollable, Settings, Space, Text,
};
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

pub fn main() {
    ESLauncher::run(Settings::default())
}

#[derive(Debug)]
struct ESLauncher {
    music_sender: Sender<MusicCommand>,
    music_button: button::State,
    music_state: MusicState,
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
    MusicMessage(MusicCommand),
}

impl Application for ESLauncher {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flag: ()) -> (ESLauncher, Command<Message>) {
        let log_reader = logger::init();
        info!("Starting ESLauncher2 v{}", version!());

        let music_sender = music::spawn();

        check_for_update();

        (
            ESLauncher {
                music_sender,
                music_button: button::State::default(),
                music_state: MusicState::Playing,
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
                let instance = self.instances_frame.instances.get_mut(i);
                if let Some(instance) = instance {
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
            Message::MusicMessage(cmd) => {
                self.music_sender.send(cmd).ok();
                self.music_state = match cmd {
                    MusicCommand::Pause => MusicState::Paused,
                    MusicCommand::Play => MusicState::Playing,
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

        let music_controls = Row::new()
            .width(Length::Fill)
            .align_items(Align::Center)
            .padding(8)
            .push(Space::new(Length::Fill, Length::Shrink))
            .push(
                Button::new(
                    &mut self.music_button,
                    match self.music_state {
                        MusicState::Playing => style::pause_icon(),
                        MusicState::Paused => style::play_icon(),
                    },
                )
                .style(style::Button::Icon)
                .on_press(Message::MusicMessage(match self.music_state {
                    MusicState::Playing => MusicCommand::Pause,
                    MusicState::Paused => MusicCommand::Play,
                })),
            )
            .push(Text::new("Playing: Endless Sky Prototype by JimmyZenith").size(14));

        Container::new(
            Column::new()
                .align_items(Align::Start)
                .push(
                    Container::new(content)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .center_x()
                        .center_y(),
                )
                .push(music_controls),
        )
        .width(Length::Fill)
        .height(Length::Fill)
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
