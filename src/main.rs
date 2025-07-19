#![forbid(unsafe_code)]
#![windows_subsystem = "windows"] // Don't show console on windows

#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate log;
#[macro_use]
extern crate version;

use parking_lot::{Mutex, RwLock};
use std::hash::Hash;
use std::path::PathBuf;
use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;

use crate::install_frame::InstallFrameMessage;
use crate::instance::{Instance, InstanceMessage, InstanceState, Progress};
use crate::instances_frame::InstancesFrame;
use crate::music::{MusicCommand, MusicState};
use crate::plugins_frame::PluginMessage;
use crate::settings::{Settings, SettingsFrame, SettingsMessage};
use crate::style::{icon_button, log_container, tab_bar};
use iced::advanced::subscription;
use iced::advanced::subscription::{EventStream, Hasher};
use iced::widget::{text, Button, Column, Container, Row, Scrollable, Space, Text};
use iced::{alignment, font, Alignment, Element, Font, Length, Subscription, Task, Theme};
use iced_aw::{TabLabel, Tabs};
use iced_dialog::dialog;
use std::collections::VecDeque;
use std::fmt::Debug;
use std::sync::Arc;

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

// TODO: investigate use cases for Task::then

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
    iced::application(ESLauncher::title, ESLauncher::update, ESLauncher::view)
        .subscription(ESLauncher::subscription)
        .theme(ESLauncher::theme)
        .run_with(|| ESLauncher::new())
}

type SharedSettings = Arc<RwLock<Settings>>;

#[derive(Debug)]
struct ESLauncher {
    music_sender: Sender<MusicCommand>,
    install_frame: install_frame::InstallFrame,
    instances_frame: InstancesFrame,
    plugins_frame: plugins_frame::PluginsFrameState,
    message_receiver: MessageReceiver,
    log_buffer: Vec<String>,
    active_tab: Tab,
    settings: SharedSettings,
    settings_frame: SettingsFrame,
    dialog: Option<DialogSpec>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tab {
    Instances,
    Plugins,
    Settings,
}

#[derive(Debug, Clone)]
pub enum Message {
    InstallFrameMessage(InstallFrameMessage),
    InstanceMessage(String, InstanceMessage),
    PluginMessage(String, PluginMessage),
    SettingsMessage(SettingsMessage),
    AddInstance(Box<Instance>),
    RemoveInstance(Option<String>),
    ReloadInstances(),
    Dummy(()),
    FontLoaded(Result<(), font::Error>),
    MusicMessage(MusicCommand),
    TabSelected(Tab),
    PluginFrameLoaded(Vec<plugins_frame::Plugin>),
    Log(String),
    OpenDialog(DialogSpec),
    DialogClosed(Box<Message>), // boxed to avoid recursive size calculation
}

#[derive(Debug, Clone)]
pub struct DialogSpec {
    title: Option<String>,
    content: String,
    buttons: Vec<(String, Message)>,
}

impl ESLauncher {
    fn new() -> (Self, Task<Message>) {
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

        let settings = Arc::new(RwLock::new(Settings::load()));
        let music_sender = music::spawn(settings.read().music_state);

        check_for_update();

        let (plugins_frame_state, plugins_frame_cmd) = plugins_frame::PluginsFrameState::new();
        (
            Self {
                music_sender,
                install_frame: install_frame::InstallFrame::default(),
                instances_frame: InstancesFrame::new(settings.clone()),
                plugins_frame: plugins_frame_state,
                message_receiver: MessageReceiver {},
                log_buffer: vec![],
                active_tab: Tab::Instances,
                settings_frame: SettingsFrame::new(settings.clone()),
                settings,
                dialog: None,
            },
            Task::batch(vec![
                plugins_frame_cmd,
                font::load(include_bytes!("../assets/IcoMoon-Free.ttf").as_slice())
                    .map(Message::FontLoaded),
                font::load(include_bytes!("../assets/DejaVuSansMono.ttf").as_slice())
                    .map(Message::FontLoaded),
            ]),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::InstallFrameMessage(msg) => {
                return self.install_frame.update(msg, self.settings.clone())
            }
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
            Message::SettingsMessage(msg) => return self.settings_frame.update(msg),
            Message::AddInstance(instance) => {
                let is_ready = instance.state.is_ready();
                self.instances_frame
                    .instances
                    .insert(instance.name.clone(), *instance);
                if is_ready {
                    instance::perform_save_instances(
                        self.instances_frame.instances.clone(),
                        &self.settings,
                    );
                };
            }
            Message::RemoveInstance(option) => {
                if let Some(name) = option {
                    self.instances_frame.instances.remove(&name);
                    instance::perform_save_instances(
                        self.instances_frame.instances.clone(),
                        &self.settings,
                    );
                }
            }
            Message::ReloadInstances() => {
                self.instances_frame = InstancesFrame::new(self.settings.clone());
            }
            Message::MusicMessage(cmd) => {
                self.music_sender.send(cmd).ok();
                let mut guard = self.settings.write();
                guard.music_state = match cmd {
                    MusicCommand::Pause => MusicState::Paused,
                    MusicCommand::Play => MusicState::Playing,
                    _ => guard.music_state,
                };
                guard.save();
            }
            Message::TabSelected(active_tab) => self.active_tab = active_tab,
            Message::PluginFrameLoaded(plugins) => {
                self.plugins_frame = plugins_frame::PluginsFrameState::from(plugins);
            }
            Message::Log(line) => self.log_buffer.push(line),
            Message::Dummy(()) => (),
            Message::FontLoaded(_) => (),
            Message::OpenDialog(spec) => self.dialog = Some(spec),
            Message::DialogClosed(msg) => {
                self.dialog = None;
                return Task::done(*msg);
            }
        }
        Task::none()
    }

    /// Subscriptions are created from Recipes.
    /// Each Recipe has a hash function, which is used to identify it.
    ///
    /// This function is called on each event loop;
    /// if a Recipe already has a running Subscription (as identified by the hash),
    /// the old Subscription will keep running, otherwise a new one will be created.
    ///
    /// Having to clone the receiver is unfortunate, but there aren't actually multiple receivers being used:
    /// the first Subscription never stops returning values (unless something catastrophic happens),
    /// so the cloned Recipe just gets dropped without being turned into a Subscription.
    fn subscription(&self) -> Subscription<Message> {
        subscription::from_recipe(self.message_receiver.clone())
    }

    fn view(&self) -> Element<'_, Message> {
        let tabs = Tabs::new(Message::TabSelected)
            .push::<Element<'_, Message>>(
                Tab::Instances,
                TabLabel::Text("Instances".into()),
                iced::widget::column([Row::new()
                    .push(self.instances_frame.view())
                    .push(iced::widget::vertical_rule(2))
                    .push(self.install_frame.view().map(Message::InstallFrameMessage))
                    .spacing(10)
                    .padding(iced::Padding {
                        top: 0.0,
                        right: 15.0,
                        bottom: 0.0,
                        left: 15.0,
                    })
                    .into()])
                .into(),
            )
            .push(
                Tab::Plugins,
                TabLabel::Text("Plugins".into()),
                iced::widget::column([self.plugins_frame.view().into()]),
            )
            .push(
                Tab::Settings,
                TabLabel::Text("Settings".into()),
                iced::widget::column([self.settings_frame.view().into()]),
            )
            .set_active_tab(&self.active_tab)
            .tab_bar_style(tab_bar);

        let logbox = self.log_buffer.iter().fold(
            Column::new()
                .spacing(1)
                .padding(15)
                .align_x(Alignment::Start),
            |column, log| {
                column.push(
                    Container::new(
                        Text::new(log)
                            .size(11)
                            .font(Font::with_name("DejaVu Sans Mono"))
                            .align_x(alignment::Horizontal::Left),
                    )
                    .style(log_container(log))
                    .width(Length::Fill),
                )
            },
        );

        let content = Column::new()
            .align_x(Alignment::Center)
            .push(tabs.height(Length::FillPortion(3)))
            .push(
                iced::widget::container(iced::widget::horizontal_rule(2)).padding(iced::Padding {
                    top: 0.0,
                    right: 10.0,
                    bottom: 0.0,
                    left: 10.0,
                }),
            )
            .push(
                Scrollable::new(logbox)
                    .width(Length::Fill)
                    .height(Length::FillPortion(1)),
            ); // TODO: Autoscroll this to bottom. https://github.com/hecrj/iced/issues/307

        let music_controls = Row::new()
            .width(Length::Fill)
            .align_y(Alignment::Center)
            .padding(8)
            .push(Space::new(Length::Fill, Length::Shrink))
            .push(
                Button::new(match self.settings.read().music_state {
                    MusicState::Playing => style::pause_icon(),
                    MusicState::Paused => style::play_icon(),
                })
                .style(icon_button)
                .on_press(Message::MusicMessage(
                    match self.settings.read().music_state {
                        MusicState::Playing => MusicCommand::Pause,
                        MusicState::Paused => MusicCommand::Play,
                    },
                )),
            )
            .push(Text::new("Endless Sky Prototype by JimmyZenith").size(13));

        let base = Container::new(
            Column::new()
                .align_x(Alignment::Start)
                .push(Container::new(content).center(Length::Fill))
                .push(music_controls.height(Length::Shrink)),
        )
        .width(Length::Fill)
        .height(Length::Fill);

        if let Some(spec) = &self.dialog {
            let mut dialog = dialog(true, base, &*spec.content).height(300);
            if let Some(title) = &spec.title {
                dialog = dialog.title(title);
            }
            for (content, msg) in &spec.buttons {
                dialog = dialog.push_button(iced_dialog::button(
                    content,
                    Message::DialogClosed(Box::new(msg.clone())),
                ));
            }
            dialog.into()
        } else {
            dialog(false, base, text("")).into()
        }
    }

    fn title(&self) -> String {
        format!("ESLauncher2 v{}", version!())
    }

    fn theme(&self) -> Theme {
        let theme = &self.settings.read().theme;
        theme.into()
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

impl subscription::Recipe for MessageReceiver {
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
                if let Some(msg) = MESSAGE_QUEUE.try_lock().and_then(|mut q| q.pop_front()) {
                    return Some((msg, state));
                }
            }
        }))
    }
}

pub fn send_message(message: Message) {
    MESSAGE_QUEUE.lock().push_back(message);
}

pub fn send_progress_message(instance_name: &str, progress: Progress) {
    send_message(Message::InstanceMessage(
        instance_name.into(),
        InstanceMessage::StateChanged(InstanceState::Working(progress)),
    ));
}
