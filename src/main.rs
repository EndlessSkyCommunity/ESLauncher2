mod archive;
mod github;
mod music;

use image;
use nfd2::Response;
use orbtk::prelude::*;
use orbtk::theme::DEFAULT_THEME_CSS;
use std::cell::{Cell, RefCell};
use std::path::PathBuf;

static WIDTH: f64 = 460.0;
static HEIGHT: f64 = 500.0;

#[derive(Debug, Clone)]
enum Action {
    SelectDestination(PathBuf),
    StartInstallation(),
    Log(String),
}

#[derive(AsAny)]
pub struct MainViewState {
    destination: RefCell<PathBuf>,
    action: Cell<Option<Action>>,
}

impl Default for MainViewState {
    fn default() -> Self {
        MainViewState {
            destination: RefCell::new(PathBuf::default()),
            action: Cell::new(None),
        }
    }
}

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
                    install(&self.destination.borrow());
                }
                Action::Log(msg) => {
                    println!("{}", msg);
                    let mut text: String = ctx
                        .child("log-box")
                        .get_mut::<String16>("text")
                        .to_string()
                        .clone();
                    text.push_str(&*format!("\n{}", msg));
                    ctx.child("log-box").set("text", String16::from(text));
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
                .rows(
                    Rows::create()
                        .row(215.0)
                        .row(75.0)
                        .row(50.0)
                        .row("*")
                        .build(),
                )
                .id("content")
                .child(
                    ImageWidget::create()
                        .attach(Grid::row(0))
                        .image(Image::from_data(460, 215, header_image()).unwrap())
                        .build(ctx),
                )
                .child(
                    Container::create()
                        .element("container")
                        .class("content")
                        .padding(10.0)
                        .attach(Grid::row(1))
                        .child(
                            TextBlock::create()
                                .name("DestinationText")
                                .id("destination-text")
                                .attach(Grid::row(1))
                                .margin(10.0)
                                .text("No folder chosen")
                                .build(ctx),
                        )
                        .child(
                            Button::create()
                                .id("folder-button")
                                .attach(Grid::row(1))
                                .horizontal_alignment("end")
                                .width(110.0)
                                .text("Pick Folder")
                                .on_click(move |states, _| {
                                    match nfd2::open_pick_folder(None).unwrap() {
                                        Response::Okay(path) => state(id, states)
                                            .action(Action::SelectDestination(path)),
                                        _ => (),
                                    };
                                    true
                                })
                                .build(ctx),
                        )
                        .build(ctx),
                )
                .child(
                    Button::create()
                        .id("install-button")
                        .attach(Grid::row(2))
                        .horizontal_alignment("center")
                        .text("Install")
                        .enabled(false)
                        .on_click(move |states, _| {
                            state(id, states).action(Action::StartInstallation());
                            true
                        })
                        .build(ctx),
                )
                .child(
                    TextBox::create()
                        .id("log-box")
                        .attach(Grid::row(3))
                        .horizontal_alignment("center")
                        .width(WIDTH - 20.0)
                        .text("")
                        .enabled(false)
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

    //install(&destination);
}

pub fn install(destination: &PathBuf) {
    let assets = github::get_release_assets().expect("Failed to get Release Assets");

    let asset_marker: &str;
    if cfg!(windows) {
        asset_marker = "win64";
    } else if cfg!(unix) {
        asset_marker = "x86_64-continuous.tar.gz"; // Don't match the AppImage
    } else {
        asset_marker = "macos";
    }
    for asset in assets {
        if asset.name.contains(asset_marker) {
            github::download(&asset).unwrap();
            archive::unpack(&PathBuf::from(&asset.name), &destination);
        }
    }
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
