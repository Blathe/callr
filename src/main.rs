mod action;
mod app;
mod cli;
mod db;
mod event;
mod http;
mod keymap;
mod mode;
mod model;
mod storage;
mod ui;

use std::path::PathBuf;

use clap::Parser;
use color_eyre::Result;
use crossterm::event::{Event as CtEvent, EventStream, KeyEventKind};
use futures::StreamExt;
use tokio::sync::mpsc;
use tokio::time::{interval, Duration};

use app::App;
use cli::Cli;
use event::AppMsg;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    init_panic_hook();

    let cli = Cli::parse();
    let collection_root = cli.collection_path.unwrap_or_else(|| PathBuf::from("collections"));

    let mut terminal = ratatui::init();
    let result = run(&mut terminal, collection_root).await;
    ratatui::restore();
    result
}

/// Ensures a panic doesn't leave the terminal stuck in raw/alternate-screen
/// mode: restore first, then hand off to color-eyre's own hook.
fn init_panic_hook() {
    let previous_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        ratatui::restore();
        previous_hook(panic_info);
    }));
}

async fn run(terminal: &mut ratatui::DefaultTerminal, collection_root: PathBuf) -> Result<()> {
    let (app_tx, mut app_rx) = mpsc::unbounded_channel::<AppMsg>();
    let mut app = App::new(app_tx, collection_root);

    let mut term_events = EventStream::new();
    let mut tick = interval(Duration::from_millis(250));

    while !app.should_quit {
        tokio::select! {
            maybe_event = term_events.next() => {
                if let Some(Ok(CtEvent::Key(key))) = maybe_event
                    && key.kind == KeyEventKind::Press {
                    app.handle_key(key);
                }
            }
            Some(msg) = app_rx.recv() => {
                app.handle_app_msg(msg);
            }
            _ = tick.tick() => {
                app.on_tick();
            }
        }

        terminal.draw(|frame| ui::draw(&mut app, frame))?;
    }

    Ok(())
}
