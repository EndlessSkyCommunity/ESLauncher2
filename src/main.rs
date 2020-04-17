#![forbid(unsafe_code)]

//mod archive;
//mod github;
//mod init_view;
mod install;
//mod music;

use iced::{button, executor, Align, Application, Button, Column, Command, Element, Image, Sandbox, Settings, Subscription, Text, ProgressBar, Container, Length};
use nfd2::Response;
use std::path::PathBuf;

static WIDTH: u16 = 460;
static HEIGHT: u16 = 500;

pub fn main() {
    ESLauncher::run(Settings::default())
}

#[derive(Debug)]
enum ESLauncher {
    PreInstall {
        destination: PathBuf,
        destination_chooser: button::State,
        install_button: button::State,
    },
    Installing {
        progress: f32,
    },
}

#[derive(Debug, Clone, Copy)]
enum Message {
    SelectDestination,
    StartInstallation,
}

impl Application for ESLauncher {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (ESLauncher, Command<Message>) {
        (
            ESLauncher::PreInstall {
                destination: PathBuf::default(),
                destination_chooser: button::State::default(),
                install_button: button::State::default(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("ESLauncher2")
    }

    fn update(&mut self, message: Self::Message) -> Command<Message> {
        match message {
            Message::SelectDestination => {
                match self {
                    ESLauncher::PreInstall { destination, .. } => {
                        match nfd2::open_pick_folder(None).unwrap() {
                            Response::Okay(path) =>  *destination = path,
                            _ => (),
                        }
                    }
                    _ => {}
                }
            },
            Message::StartInstallation => {
                match self {
                    ESLauncher::PreInstall { destination, .. } => {
                        *self = ESLauncher::Installing {
                            progress: 0.0
                        }
                    }
                    _ => {}
                }
            },
        }
        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        match self {
            ESLauncher::Installing { .. } => Subscription::none(),
            _ => Subscription::none(),
        }
    }

    fn view(&mut self) -> Element<'_, Self::Message> {
        let content = match self {
            ESLauncher::PreInstall {destination, destination_chooser, install_button } => {
                 Column::new()
                    .padding(20)
                    .align_items(Align::Center)
                    .push(Text::new(destination.to_string_lossy()))
                    .push(
                        Button::new(destination_chooser, Text::new("Pick Folder"))
                            .on_press(Message::SelectDestination),
                    )
                    .push(
                        Button::new(install_button, Text::new("Install"))
                            .on_press(Message::StartInstallation),
                    )
            }

            ESLauncher::Installing { progress } => {
                Column::new()
                    .padding(20)
                    .align_items(Align::Center)
                    .push(ProgressBar::new(0.0..=100.0, *progress))
            }
        };

        Container::new(content)
            .width(Length::from(WIDTH))
            .height(Length::from(HEIGHT))
            .center_x().center_y().into()
    }
}

/*
impl MainViewState {
    fn action(&self, action: impl Into<Option<Action>>) {
        self.action.set(action.into());
    }
}

impl State for MainViewState {
    fn update(&mut self, _: &mut Registry, ctx: &mut Context<'_>) {
        if let Some(action) = self.action.take() {
            match action {
                Action::SelectDestination(path) => {
                    ctx.child("install-button").set("enabled", true);
                    ctx.child("destination-text")
                        .set("text", String16::from(String::from(path.to_string_lossy())));
                    self.destination.replace(path);
                }
                Action::StartInstallation() => {
                    ctx.child("install-button").set("enabled", false);
                    println!("test");
                    install::install(&self.destination.borrow());
                }
                Action::Log(msg) => {
                    let child = TextBlock::create().class("log-line").text(msg);
                    let entity = ctx.entity_of_child("log-box").unwrap();
                    ctx.append_child_to(child, entity);
                    ctx.child("log-scroll")
                        .set("scroll_offset", Point::new(0.0, std::f64::MIN));
                }
            }
        }
    }
}

widget!(MainView<MainViewState>);

impl Template for MainView {
    fn template(self, id: Entity, ctx: &mut BuildContext) -> Self {
        self.name("MainView").child(
            Grid::create()
                .rows(Rows::create().row(215.0).row("*").build())
                .id("content")
                .child(
                    ImageWidget::create()
                        .attach(Grid::row(0))
                        .image(Image::from_data(460, 215, header_image()).unwrap())
                        .build(ctx),
                )
                .child(
                    Container::create()
                        .id("content")
                        .attach(Grid::row(1))
                        .child(init_view::create(id, ctx))
                        .build(ctx),
                )
                .build(ctx),
        )
    }
}

fn main() {
    music::play();

    Application::new()
        .window(|ctx| {
            Window::create()
                .title("ESLauncher2")
                .position((500.0, 100.0))
                .size(WIDTH, HEIGHT)
                .resizeable(true)
                .theme(
                    ThemeValue::create_from_css(DEFAULT_THEME_CSS)
                        .extension_css(include_str!("../assets/style.css"))
                        .build(),
                )
                .child(MainView::create().build(ctx))
                .build(ctx)
        })
        .run();
}

fn header_image() -> Vec<u32> {
    let array: &[u8] = include_bytes!("../assets/header.jpg");
    let img = image::load_from_memory_with_format(array, image::ImageFormat::Jpeg).unwrap();
    img.into_rgba()
        .pixels()
        .map(|p| {
            ((p[3] as u32) << 24) | ((p[0] as u32) << 16) | ((p[1] as u32) << 8) | (p[2] as u32)
        })
        .collect()
}

// helper to request MainViewState
fn state<'a>(id: Entity, states: &'a mut StatesContext) -> &'a mut MainViewState {
    states.get_mut(id)
}
 */
