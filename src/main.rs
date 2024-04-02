#![forbid(unsafe_code)]
#![windows_subsystem = "windows"] // Don't show console on windows

#[macro_use]
extern crate log;
#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate version;

use std::hash::Hash;
use std::path::PathBuf;
use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;

use iced::advanced::subscription::EventStream;
use iced::advanced::Hasher;
use iced::widget::{Button, Column, Container, Row, Scrollable, Space, Text};
use iced::{
    alignment, font, Alignment, Application, Command, Element, Font, Length, Subscription, Theme,
};
use iced_aw::{TabLabel, Tabs};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use crate::install_frame::InstallFrameMessage;
use crate::instance::{Instance, InstanceMessage, InstanceState, Progress};
use crate::music::{MusicCommand, MusicState};
use crate::plugins_frame::PluginMessage;
use crate::settings::Settings;
use crate::style::{icon_button, log_container, tab_bar};

mod archive;
mod github;
mod install;
mod install_frame;
mod instance;
mod instances_frame;
mod jenkins;
mod logger;
mod music;
mod plugins_frame;
mod settings;
mod style;
mod update;

// Yes, this is terrible abuse of globals.
// I spent hours and hours trying to find a better solution:
// - async_channel
//      Somehow blocks during long operations, so all progress logs arrive after the installation has completed.
//      The messages are sent in time, and the receiving end is a future, so this makes no sense.
// - crossbeam_channel and std::sync::mpsc
//      When receiving in a blocking fashion, closing the window doesn't work; instead, it turns unresponsive.
//      When trying to use try_recv() repeatedly inside futures::stream::poll(),
//      the first non-log that is returned seems to end the stream (no matter what Poll variant it's wrapped in).
// - futures::channel::mpsc
//      send() is async and thus can't be used in the logger.
//      try_send() requires the sender to be mutable (??) and thus can't be used inside log().
//
// Bottom line, this is terrible design with comparatively bad performance, but also the only solution that
// - delivers messages immediately
// - Doesn't block the receiving thread and thus screws over iced's internals
// - Doesn't randomly break after some messages
// so here it will stay.
static MESSAGE_QUEUE: Mutex<VecDeque<Message>> = Mutex::new(VecDeque::new());

pub fn main() -> iced::Result {
    ESLauncher::run(iced::Settings::default())
}

#[derive(Debug)]
struct ESLauncher {
    music_sender: Sender<MusicCommand>,
    install_frame: install_frame::InstallFrame,
    instances_frame: instances_frame::InstancesFrame,
    plugins_frame: plugins_frame::PluginsFrameState,
    message_receiver: MessageReceiver,
    log_buffer: Vec<String>,
    active_tab: Tab,
    settings: Settings,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tab {
    Instances,
    Plugins,
}

#[derive(Debug, Clone)]
pub enum Message {
    InstallFrameMessage(InstallFrameMessage),
    InstanceMessage(String, InstanceMessage),
    PluginMessage(String, PluginMessage),
    AddInstance(Box<Instance>),
    RemoveInstance(Option<String>),
    Dummy(()),
    FontLoaded(Result<(), font::Error>),
    MusicMessage(MusicCommand),
    TabSelected(Tab),
    PluginFrameLoaded(Vec<plugins_frame::Plugin>),
    Log(String),
}

impl Application for ESLauncher {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flag: ()) -> (Self, Command<Message>) {
        logger::init();
        info!("Starting ESLauncher2 v{}", version!());
        if cfg!(target_os = "macos") {
            info!("  running on target environment macos");
        } else if cfg!(target_os = "windows") {
            info!("  running on target environment windows");
        } else if cfg!(target_os = "linux") {
            info!("  running on target environment linux");
        } else {
            info!("  running on target environment other");
        }

        let settings = Settings::load();
        let music_sender = music::spawn(settings.music_state);

        check_for_update();

        let (plugins_frame_state, plugins_frame_cmd) = plugins_frame::PluginsFrameState::new();
        (
            Self {
                music_sender,
                install_frame: install_frame::InstallFrame::default(),
                instances_frame: instances_frame::InstancesFrame::default(),
                plugins_frame: plugins_frame_state,
                message_receiver: MessageReceiver {},
                log_buffer: vec![],
                active_tab: Tab::Instances,
                settings,
            },
            Command::batch(vec![
                plugins_frame_cmd,
                font::load(include_bytes!("../assets/IcoMoon-Free.ttf").as_slice())
                    .map(Message::FontLoaded),
                font::load(include_bytes!("../assets/DejaVuSansMono.ttf").as_slice())
                    .map(Message::FontLoaded),
            ]),
        )
    }

    fn title(&self) -> String {
        format!("ESLauncher2 v{}", version!())
    }

    type Theme = Theme;

    fn update(&mut self, message: Self::Message) -> Command<Message> {
        match message {
            Message::InstallFrameMessage(msg) => return self.install_frame.update(msg),
            Message::InstanceMessage(name, msg) => {
                match self.instances_frame.instances.get_mut(&name) {
                    None => error!("Failed to find internal Instance with name {}", &name),
                    Some(instance) => return instance.update(msg),
                }
            }
            Message::PluginMessage(name, msg) => {
                if let plugins_frame::PluginsFrameState::Ready { plugins, .. } =
                    &mut self.plugins_frame
                {
                    match plugins.iter_mut().find(|p| p.name == name) {
                        None => error!("Failed to find internal Plug-In with name {}", name),
                        Some(p) => return p.update(msg),
                    }
                }
            }
            Message::AddInstance(instance) => {
                let is_ready = instance.state.is_ready();
                self.instances_frame
                    .instances
                    .insert(instance.name.clone(), *instance);
                if is_ready {
                    instance::perform_save_instances(self.instances_frame.instances.clone());
                };
            }
            Message::RemoveInstance(option) => {
                if let Some(name) = option {
                    self.instances_frame.instances.remove(&name);
                    instance::perform_save_instances(self.instances_frame.instances.clone());
                }
            }
            Message::MusicMessage(cmd) => {
                self.music_sender.send(cmd).ok();
                self.settings.music_state = match cmd {
                    MusicCommand::Pause => MusicState::Paused,
                    MusicCommand::Play => MusicState::Playing,
                    _ => self.settings.music_state,
                };
                if let Err(e) = self.settings.save() {
                    error!("Failed to save settings.json: {:#?}", e);
                };
            }
            Message::TabSelected(active_tab) => self.active_tab = active_tab,
            Message::PluginFrameLoaded(plugins) => {
                self.plugins_frame = plugins_frame::PluginsFrameState::from(plugins);
            }
            Message::Log(line) => self.log_buffer.push(line),
            Message::Dummy(()) => (),
            Message::FontLoaded(_) => (),
        }
        Command::none()
    }

    /// Subscriptions are created from Recipes.
    /// Each Recipe has a hash function, which is used to identify it.
    ///
    /// This function is called on each event loop;
    /// if a Recipe already has a running Subscription (as identified by the hash),
    /// the old Subscription will keep running, otherwise a new one will be created.
    ///
    /// Having to clone the receiver is unfortunate, but there aren't actually multiple receivers being used:
    /// the first the Subscription never stops returning values (unless something catastrophic happens),
    /// so the cloned Recipe just gets dropped without being turned into a Subscription.
    fn subscription(&self) -> Subscription<Message> {
        Subscription::from_recipe(self.message_receiver.clone())
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let tabs = Tabs::new(Message::TabSelected)
            .push::<Element<'_, Message>>(
                Tab::Instances,
                TabLabel::Text("Instances".into()),
                Row::new()
                    .push(self.instances_frame.view())
                    .push(self.install_frame.view().map(Message::InstallFrameMessage))
                    .spacing(50)
                    .padding(15)
                    .into(),
            )
            .push(
                Tab::Plugins,
                TabLabel::Text("Plugins".into()),
                self.plugins_frame.view(),
            )
            .set_active_tab(&self.active_tab)
            .tab_bar_style(tab_bar());

        let logbox = self.log_buffer.iter().fold(
            Column::new()
                .spacing(1)
                .padding(15)
                .align_items(Alignment::Start),
            |column, log| {
                column.push(
                    Container::new(
                        Text::new(log)
                            .size(11)
                            .font(Font::with_name("DejaVu Sans Mono"))
                            .horizontal_alignment(alignment::Horizontal::Left),
                    )
                    .style(log_container(log))
                    .width(Length::Fill),
                )
            },
        );

        let content = Column::new()
            .align_items(Alignment::Center)
            .push(tabs.height(Length::FillPortion(3)))
            .push(
                Scrollable::new(logbox)
                    .width(Length::Fill)
                    .height(Length::FillPortion(1)),
            ); // TODO: Autoscroll this to bottom. https://github.com/hecrj/iced/issues/307

        let music_controls = Row::new()
            .width(Length::Fill)
            .align_items(Alignment::Center)
            .padding(8)
            .push(Space::new(Length::Fill, Length::Shrink))
            .push(
                Button::new(match self.settings.music_state {
                    MusicState::Playing => style::pause_icon(),
                    MusicState::Paused => style::play_icon(),
                })
                .style(icon_button())
                .on_press(Message::MusicMessage(
                    match self.settings.music_state {
                        MusicState::Playing => MusicCommand::Pause,
                        MusicState::Paused => MusicCommand::Play,
                    },
                )),
            )
            .push(Text::new("Endless Sky Prototype by JimmyZenith").size(13));

        Container::new(
            Column::new()
                .align_items(Alignment::Start)
                .push(
                    Container::new(content)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .center_x()
                        .center_y(),
                )
                .push(music_controls.height(Length::Shrink)),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    fn theme(&self) -> Self::Theme {
        iced::Theme::custom("LightModified".into(), {
            let mut palette = iced::theme::Palette::LIGHT;
            palette.primary = iced::Color::from_rgb(0.2, 0.2, 0.2);
            palette
        })
    }
}

fn check_for_update() {
    thread::spawn(
        || match github::get_latest_release("EndlessSkyCommunity/ESLauncher2") {
            Ok(tag) => {
                info!("The latest version of ESLauncher2 is {}", tag);
            }
            Err(e) => error!("Failed to fetch latest ESLauncher2 release: {}", e),
        },
    );
}

fn get_data_dir() -> Option<PathBuf> {
    Some(platform_dirs::AppDirs::new(Some("ESLauncher2"), false)?.data_dir)
}

#[derive(Debug, Clone)]
pub struct MessageReceiver {}

impl iced::advanced::subscription::Recipe for MessageReceiver {
    type Output = Message;

    fn hash(&self, state: &mut Hasher) {
        std::any::TypeId::of::<Self>().hash(state);
    }

    fn stream(
        self: Box<Self>,
        _input: EventStream,
    ) -> futures::stream::BoxStream<'static, Self::Output> {
        Box::pin(futures::stream::unfold(0, |state| async move {
            let mut interval = tokio::time::interval(Duration::from_millis(10));
            loop {
                interval.tick().await;
                if let Some(msg) = MESSAGE_QUEUE
                    .try_lock()
                    .ok()
                    .and_then(|mut q| q.pop_front())
                {
                    return Some((msg, state));
                }
            }
        }))
    }
}

pub fn send_message(message: Message) {
    match crate::MESSAGE_QUEUE.lock() {
        Ok(mut queue) => queue.push_back(message),
        Err(e) => {
            // Don't use an error log here because that may cause an endless loop of logs
            eprintln!(
                "Failed to lock message queue:\n{e}\nThe message was as follows:\n{message:#?}"
            );
        }
    }
}

pub fn send_progress_message(instance_name: &str, progress: Progress) {
    send_message(Message::InstanceMessage(
        instance_name.into(),
        InstanceMessage::StateChanged(InstanceState::Working(progress)),
    ));
}
