use crate::instance::{load_instances, save_instances};
use crate::music::MusicState;
use crate::{get_data_dir, send_message, style, DialogSpec, Message, SharedSettings};
use anyhow::{Context, Result};
use cp_r::CopyStats;
use futures::join;
use iced::advanced::graphics::core::Element;
use iced::task::{sipper, Sipper};
use iced::widget::{combo_box, container, row, space, text, Text};
use iced::{
    widget::{button, Column, Container, Row},
    Theme,
};
use iced::{Alignment, Padding, Renderer, Task};
use serde::{Deserialize, Serialize};
use sipper::Sender;
use std::fmt::{Debug, Display, Formatter};
use std::iter::Iterator;
use std::sync::LazyLock;
use std::time::Duration;
use std::{fs::File, path::PathBuf};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Settings {
    pub music_state: MusicState,
    #[serde(default)]
    pub theme: SelectableTheme,
    #[serde(skip)]
    pub theme_preview: Option<SelectableTheme>,
    #[serde(default = "default_install_dir")]
    pub install_dir: PathBuf,
}
fn default_install_dir() -> PathBuf {
    get_data_dir().unwrap().join("instances")
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            music_state: Default::default(),
            theme: SelectableTheme::default(),
            theme_preview: None,
            install_dir: default_install_dir(),
        }
    }
}

impl Settings {
    pub fn save(&self) {
        let save = || -> Result<()> {
            let mut settings_file =
                get_data_dir().ok_or_else(|| anyhow!("Failed to get app save dir"))?;
            settings_file.push("settings.json");

            let file = File::create(settings_file)?;
            serde_json::to_writer_pretty(file, self)?;
            Ok(())
        };
        if let Err(e) = save() {
            error!("Failed to save settings.json: {:#?}", e);
        }
    }

    pub fn load() -> Self {
        let mut settings_file = get_data_dir()
            .ok_or_else(|| anyhow!("Failed to get app save dir"))
            .unwrap();
        settings_file.push("settings.json");

        if !settings_file.exists() {
            return Self::default();
        }

        File::open(settings_file)
            .with_context(|| "Failed to open settings.json")
            .and_then(|f| {
                serde_json::from_reader(f).with_context(|| "Failed to deserialize settings.json")
            })
            .unwrap_or_else(|e| {
                warn!("{:#?}", e);
                Self::default()
            })
    }
}

// TODO: do we need this? or can we live with autodetect = None ?
#[derive(Default, Debug, Clone, Deserialize, Serialize)]
#[serde(from = "Option<String>", into = "Option<String>")]
pub enum SelectableTheme {
    #[default]
    Autodetect,
    Preset(Theme),
}

impl From<Option<String>> for SelectableTheme {
    fn from(value: Option<String>) -> Self {
        value
            .map(|s| {
                Theme::ALL
                    .iter()
                    .find(|t| t.to_string() == s)
                    .cloned()
                    .unwrap_or_else(|| {
                        warn!("Got unknown theme {s} from config, falling back to default");
                        Theme::Light
                    })
            })
            .map(|t| Self::Preset(t.clone()))
            .unwrap_or_default()
    }
}

impl From<SelectableTheme> for Option<String> {
    fn from(st: SelectableTheme) -> Option<String> {
        match st {
            SelectableTheme::Autodetect => None,
            SelectableTheme::Preset(t) => Some(t.to_string()),
        }
    }
}

impl Display for SelectableTheme {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SelectableTheme::Autodetect => write!(f, "Detect from System"),
            SelectableTheme::Preset(t) => Display::fmt(&t, f),
        }
    }
}

impl From<&SelectableTheme> for Option<Theme> {
    fn from(st: &SelectableTheme) -> Option<Theme> {
        match st {
            SelectableTheme::Autodetect => None,
            SelectableTheme::Preset(t) => Some(t.clone()),
        }
    }
}
pub static ALL_SELECTABLE_THEMES: LazyLock<Vec<SelectableTheme>> = LazyLock::new(|| {
    let mut vec = vec![SelectableTheme::Autodetect];
    vec.extend(
        Theme::ALL
            .iter()
            .map(|t| SelectableTheme::Preset(t.clone())),
    );
    vec
});

#[derive(Debug)]
pub struct SettingsFrame {
    settings: SharedSettings,
    theme_selector_state: combo_box::State<SelectableTheme>,
}

#[derive(Debug, Clone)]
pub enum SettingsMessage {
    ThemeSelected(SelectableTheme),
    ThemePreviewed(Option<SelectableTheme>),
    RequestInstallPath,
    SetInstallPath(PathBuf),
    MoveInstallPath(PathBuf),
}
impl SettingsFrame {
    pub fn new(settings: SharedSettings) -> Self {
        let theme_selector_state = combo_box::State::new(ALL_SELECTABLE_THEMES.to_vec());
        Self {
            settings,
            theme_selector_state,
        }
    }
    pub fn view(&self) -> Container<Message> {
        fn settings_row<'a>(
            label: &'a str,
            content: impl Into<Element<'a, Message, Theme, Renderer>>,
        ) -> impl Into<Element<'a, Message, Theme, Renderer>> {
            container(
                Column::new()
                    .push(
                        Row::new()
                            .push(Text::new(label))
                            .push(space::horizontal())
                            .push(
                                container(content).align_x(iced::alignment::Horizontal::Right), // .width(Length::Fill),
                            )
                            .align_y(Alignment::Center),
                    )
                    .spacing(10.0),
            )
        }

        let install_dir_picker = button(style::folder_icon().size(12.0))
            .on_press(Message::SettingsMessage(
                SettingsMessage::RequestInstallPath,
            ))
            // .style(icon_button)
            .padding(Padding::from([2, 0]));
        let install_dir_reset_btn = if self.settings.read().install_dir.eq(&default_install_dir()) {
            None
        } else {
            Some(
                button(style::reset_icon().size(12.0))
                    .on_press(Message::OpenDialog(move_dir_dialog_spec(
                        default_install_dir(),
                    )))
                    .padding(Padding::from([2, 0])),
            )
        };

        Container::new(
            Column::new()
                .push(settings_row(
                    "Install directory",
                    row!(
                        text(format!(
                            "Installing to {}",
                            self.settings.read().install_dir.to_string_lossy(),
                        ))
                        .size(12.0),
                        install_dir_picker
                    )
                    .push(install_dir_reset_btn)
                    .align_y(Alignment::Center)
                    .spacing(10.0)
                    .padding(Padding {
                        top: 0.0,
                        right: 10.0,
                        bottom: 0.0,
                        left: 0.0,
                    }),
                ))
                .push(settings_row(
                    "Theme",
                    combo_box(
                        &self.theme_selector_state,
                        "Please select a theme",
                        Some(&self.settings.read().theme),
                        |st| Message::SettingsMessage(SettingsMessage::ThemeSelected(st)),
                    )
                    .on_option_hovered(|st| {
                        Message::SettingsMessage(SettingsMessage::ThemePreviewed(Some(st)))
                    })
                    .on_close(Message::SettingsMessage(
                        SettingsMessage::ThemePreviewed(None),
                    )),
                ))
                .spacing(10.0),
        )
        .padding(100.0)
    }

    pub fn update(&mut self, message: SettingsMessage) -> Task<Message> {
        match message {
            SettingsMessage::RequestInstallPath => {
                return Task::perform(rfd::AsyncFileDialog::new().pick_folder(), |f| match f {
                    Some(handle) => {
                        Message::OpenDialog(move_dir_dialog_spec(handle.path().to_path_buf()))
                    }
                    None => Message::Dummy(()),
                })
            }
            SettingsMessage::SetInstallPath(p) => {
                self.settings.write().install_dir = p;
                return Task::done(Message::ReloadInstances());
            }
            SettingsMessage::MoveInstallPath(new) => {
                let sipper = move_install_dir(self.settings.clone(), new.clone());
                return Task::sip(sipper, Message::OpenDialog, Message::OpenDialog);
            }
            SettingsMessage::ThemeSelected(theme) => {
                let mut guard = self.settings.write();
                guard.theme_preview = None;
                guard.theme = theme;
            }
            SettingsMessage::ThemePreviewed(theme_maybe) => {
                self.settings.write().theme_preview = theme_maybe
            }
        };
        self.settings.read().save();

        Task::none()
    }
}

fn move_install_dir(
    settings: SharedSettings,
    dest: PathBuf,
) -> impl Sipper<DialogSpec, DialogSpec> {
    sipper(move |sender| async move {
        let res = try_move_install_dir(sender, settings, dest).await;
        res.unwrap_or_else(|e| {
            error!("Error while moving install dir: {e:#?}");
            DialogSpec {
                title: Some("Error".into()),
                content: format!("An error occured during the operation:\n{e:#?}"),
                buttons: vec![("Close".into(), Message::Dummy(()))],
            }
        })
    })
}

async fn try_move_install_dir(
    mut sender: Sender<DialogSpec>,
    settings: SharedSettings,
    dest: PathBuf,
) -> Result<DialogSpec> {
    // Users can't choose empty directories, but the reset button can
    if tokio::fs::try_exists(&dest).await? {
        tokio::fs::remove_dir_all(&dest)
            .await
            .with_context(|| "Failed to remove destination directory")?;
    }

    let source = settings.read().install_dir.clone();
    let source_clone = source.clone();
    let dest_clone = dest.clone();

    let (tokio_tx, mut tokio_rx) = tokio::sync::mpsc::channel(1024);
    let cp_r_fut = tokio::task::spawn_blocking(move || {
        cp_r::CopyOptions::new()
            .create_destination(true)
            .after_entry_copied(|_, _, s| {
                tokio_tx
                    .blocking_send(move_in_progress_dialog_spec(Some(s)))
                    .expect("cp_r thread tried to send updates to closed channel");
                Ok(())
            })
            .copy_tree(source_clone, dest_clone)
    });
    let read_updates_fut = async {
        while let Some(s) = tokio_rx.recv().await {
            sender.send(s).await
        }
    };
    let (stats_result, _) = join!(cp_r_fut, read_updates_fut);
    let stats = stats_result
        .with_context(|| "Copy task failed")?
        .with_context(|| "Failed to copy files")?;

    info!("Patching instance paths");
    let mut instances = load_instances(&dest)?;
    for i in &mut instances {
        let name = i.path.file_name().unwrap();
        i.executable = i.executable.strip_prefix(&i.path)
            .map(|exe| dest.join(exe))
            .inspect_err(|e| {
            warn!("Failed to patch instance paths for instance '{}', you might have to re-install it", name.to_string_lossy())
        }).unwrap_or_else(|_| i.executable.clone());
        i.path = dest.join(name);
    }
    save_instances(instances, &dest)?;

    info!("Removing old directory");
    tokio::fs::remove_dir_all(&source)
        .await
        .with_context(|| "Failed to remove former install dir")?;

    settings.write().install_dir = dest;
    info!("Install dir moved successfully");
    Ok(DialogSpec {
        title: None,
        content: format!(
            "Success!\n Moved {} files, {} folders",
            stats.files, stats.dirs
        ),
        buttons: vec![("Close".into(), Message::ReloadInstances())],
    })
}

fn move_dir_dialog_spec(new_dir: PathBuf) -> DialogSpec {
    let items = std::fs::read_dir(&new_dir)
        .map(|r| r.count())
        .unwrap_or_default();
    let warning = if items > 0 {
        format!(
            "\nWARNING: Found {items} existing items in the directory.\n\
            If you select Yes, these will be deleted!\n\
            If you select No, ESLauncher2 might run into problems later."
        )
    } else {
        String::default()
    };

    let buttons = vec![
        (
            "Yes".into(),
            Message::SettingsMessage(SettingsMessage::MoveInstallPath(new_dir.clone())),
        ),
        (
            "No".into(),
            Message::SettingsMessage(SettingsMessage::SetInstallPath(new_dir)),
        ),
        ("Cancel".into(), Message::Dummy(())),
    ];
    DialogSpec {
        title: None,
        content: format!("Should ESLauncher move your instances to the new folder?{warning}"),
        buttons,
    }
}

fn move_in_progress_dialog_spec(stats: Option<&CopyStats>) -> DialogSpec {
    let items = if let Some(stats) = stats {
        stats.dirs + stats.files + stats.symlinks
    } else {
        0
    };
    let content = format!("Moving install dir, please be patient...\n{items} items");
    DialogSpec {
        title: None,
        content,
        buttons: vec![],
    }
}
