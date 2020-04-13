mod archive;
mod github;
mod music;

use nfd2::Response;
use orbtk::prelude::*;
use std::cell::{Cell, RefCell};
use std::path::PathBuf;

#[derive(Debug, Clone)]
enum Action {
    SelectDestination(PathBuf),
}

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
    fn update(&self, ctx: &mut Context<'_>) {
        if let Some(action) = self.action.take() {
            match action {
                Action::SelectDestination(path) => {
                    ctx.child("destination-text")
                        .set("text", String16::from(String::from(path.to_string_lossy())));
                    ctx.child("install-button").set("enabled", true);
                    self.destination.replace(path);
                }
            }
        }
    }
}

widget!(MainView<MainViewState>);

impl Template for MainView {
    fn template(self, _: Entity, ctx: &mut BuildContext) -> Self {
        let folder_state = self.clone_state();
        let install_state = self.clone_state();
        self.name("MainView").child(
            Grid::create()
                .rows(Rows::create().row(72.0).row("*").build())
                .child(
                    TextBlock::create()
                        .name("DestinationText")
                        .selector(Selector::from("span").id("destination-text"))
                        .attach(Grid::row(0))
                        .text("test")
                        .margin((0.0, 0.0, 0.0, 2.0))
                        .vertical_alignment("center")
                        .build(ctx),
                )
                .child(
                    Button::create()
                        .selector(Selector::from("button").id("folder-button"))
                        .attach(Grid::row(0))
                        .text("Select Folder")
                        .on_click(move |_| {
                            match nfd2::open_pick_folder(None).unwrap() {
                                Response::Okay(path) => {
                                    folder_state.action(Action::SelectDestination(path))
                                }
                                _ => (),
                            };
                            true
                        })
                        .build(ctx),
                )
                .child(
                    Button::create()
                        .selector(Selector::from("button").id("install-button"))
                        .attach(Grid::row(1))
                        .text("Install")
                        .text("Select Folder")
                        .enabled(false)
                        .on_click(move |_| {
                            install(&install_state.destination.borrow());
                            true
                        })
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
                .position((100.0, 100.0))
                .size(420.0, 730.0)
                .resizeable(true)
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
