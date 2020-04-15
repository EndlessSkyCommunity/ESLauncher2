mod archive;
mod github;
mod init_view;
mod install;
mod music;

use image;
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
                .rows(
                    Rows::create()
                        .row(215.0)
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
                        .id("content")
                        .attach(Grid::row(1))
                        .child(
                            init_view::create(id, ctx)
                        ).build(ctx)
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
