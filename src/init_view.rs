use crate::{state, Action, WIDTH};
use nfd2::Response;

pub fn create(id: Entity, ctx: &mut BuildContext) -> Entity {
    Grid::create()
        .rows(Rows::create().row(100.0).row(100.0).row("*").build())
        .id("init-view")
        .child(
            TextBlock::create()
                .name("DestinationText")
                .id("destination-text")
                .attach(Grid::row(0))
                .margin(10.0)
                .text("No folder chosen")
                .build(ctx),
        )
        .child(
            Button::create()
                .id("folder-button")
                .attach(Grid::row(0))
                .horizontal_alignment("end")
                .width(110.0)
                .text("Pick Folder")
                .on_click(move |states, _| {
                    match nfd2::open_pick_folder(None).unwrap() {
                        Response::Okay(path) => {
                            state(id, states).action(Action::SelectDestination(path))
                        }
                        _ => (),
                    };
                    true
                })
                .build(ctx),
        )
        .child(
            Button::create()
                .id("install-button")
                .attach(Grid::row(1))
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
            ScrollViewer::create()
                .id("log-scroll")
                .attach(Grid::row(2))
                .horizontal_alignment("center")
                .width(WIDTH - 20.0)
                .scroll_viewer_mode(("disabled", "auto"))
                .child(
                    Stack::create()
                        .id("log-box")
                        .attach(Grid::row(3))
                        .build(ctx),
                )
                .build(ctx),
        )
        .build(ctx)
}
