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

use iced::{
    button, scrollable, Align, Application, Button, Column, Command, Container, Element,
    HorizontalAlignment, Length, Row, Scrollable, Settings, Space, Subscription, Text,
};
use std::collections::VecDeque;
use std::sync::Mutex;

use crate::install_frame::InstallFrameMessage;
use crate::instance::{Instance, InstanceMessage};
use crate::music::{MusicCommand, MusicState};
use crate::plugins_frame::PluginMessage;
use lazy_static::lazy_static;

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
mod style;
mod update;

lazy_static! {
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
    static ref MESSAGE_QUEUE: Mutex<VecDeque<Message>> = Mutex::new(VecDeque::new());
}

pub fn main() -> iced::Result {
    ESLauncher::run(Settings::default())
}

#[derive(Debug)]
struct ESLauncher {
    music_sender: Sender<MusicCommand>,
    music_button: button::State,
    music_state: MusicState,
    install_frame: install_frame::InstallFrame,
    instances_frame: instances_frame::InstancesFrame,
    plugins_frame: plugins_frame::PluginsFrameState,
    log_scrollable: scrollable::State,
    message_receiver: MessageReceiver,
    log_buffer: Vec<String>,
    view: MainView,
    instances_view_button: button::State,
    plugins_view_button: button::State,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MainView {
    Instances,
    Plugins,
}

#[derive(Debug, Clone)]
pub enum Message {
    InstallFrameMessage(InstallFrameMessage),
    InstanceMessage(String, InstanceMessage),
    PluginMessage(String, PluginMessage),
    Installed(Option<Instance>),
    Deleted(Option<PathBuf>),
    Updated(Option<Instance>),
    Dummy(()),
    MusicMessage(MusicCommand),
    ViewChanged(MainView),
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

        let music_sender = music::spawn();

        check_for_update();

        let (plugins_frame_state, command) = plugins_frame::PluginsFrameState::new();
        (
            Self {
                music_sender,
                music_button: button::State::default(),
                music_state: MusicState::Playing,
                install_frame: install_frame::InstallFrame::default(),
                instances_frame: instances_frame::InstancesFrame::default(),
                plugins_frame: plugins_frame_state,
                log_scrollable: scrollable::State::default(),
                message_receiver: MessageReceiver {},
                log_buffer: vec![],
                view: MainView::Instances,
                instances_view_button: button::State::default(),
                plugins_view_button: button::State::default(),
            },
            command,
        )
    }

    fn title(&self) -> String {
        format!("ESLauncher2 v{}", version!())
    }

    fn update(&mut self, message: Self::Message) -> Command<Message> {
        match message {
            Message::InstallFrameMessage(msg) => return self.install_frame.update(msg),
            Message::InstanceMessage(name, msg) => {
                match self.instances_frame.find_instance(&name) {
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
            Message::ViewChanged(view) => self.view = view,
            Message::PluginFrameLoaded(plugins) => {
                self.plugins_frame = plugins_frame::PluginsFrameState::from(plugins);
            }
            Message::Log(line) => self.log_buffer.push(line),
            Message::Dummy(_) => (),
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

    fn view(&mut self) -> Element<'_, Self::Message> {
        let view_chooser = Row::new()
            .spacing(100)
            .padding(30)
            .align_items(Align::Center)
            .push(
                Button::new(
                    &mut self.instances_view_button,
                    Container::new(Text::new("Instances")).padding(5),
                )
                .padding(5)
                .on_press(Message::ViewChanged(MainView::Instances))
                .style(style::Button::Tab(self.view == MainView::Instances)),
            )
            .push(
                Button::new(
                    &mut self.plugins_view_button,
                    Container::new(Text::new("Plugins")).padding(5),
                )
                .on_press(Message::ViewChanged(MainView::Plugins))
                .style(style::Button::Tab(self.view == MainView::Plugins)),
            );

        let main_view = match self.view {
            MainView::Instances => Container::new(
                Row::new()
                    .push(self.instances_frame.view())
                    .push(self.install_frame.view().map(Message::InstallFrameMessage))
                    .spacing(50),
            ),
            MainView::Plugins => self.plugins_frame.view(),
        };

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
            .push(view_chooser)
            .push(main_view.height(Length::FillPortion(3)))
            .push(
                Scrollable::new(&mut self.log_scrollable)
                    .push(logbox)
                    .padding(20)
                    .align_items(Align::Start)
                    .width(Length::Fill)
                    .height(Length::FillPortion(1)),
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
    thread::spawn(
        || match github::get_latest_release("EndlessSkyCommunity/ESLauncher2") {
            Ok(release) => {
                if !format!("v{}", version!()).eq(&release.tag_name) {
                    info!("The latest version of ESLauncher2 is {}", release.tag_name)
                }
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

impl<H, I> iced_futures::subscription::Recipe<H, I> for MessageReceiver
where
    H: std::hash::Hasher,
{
    type Output = Message;

    fn hash(&self, state: &mut H) {
        std::any::TypeId::of::<Self>().hash(state);
    }

    fn stream(
        self: Box<Self>,
        _input: futures::stream::BoxStream<'static, I>,
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
                "Failed to lock message queue:\n{}\nThe message was as follows:\n{:#?}",
                e, message
            );
        }
    }
}
