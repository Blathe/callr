use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Borders};
use ratatui::Frame;

use crate::app::App;
use crate::mode::{Focus, Mode};
use crate::ui::theme;

pub fn render(app: &mut App, frame: &mut Frame, area: Rect) {
    let focused = app.focus == Focus::Body;
    let border_style = theme::border_style(focused);
    let block = Block::default()
        .title("Body")
        .borders(Borders::ALL)
        .border_style(border_style);

    let cursor_style = if focused && app.mode == Mode::Insert {
        Style::default().add_modifier(Modifier::REVERSED)
    } else {
        Style::default()
    };
    app.body_editor.set_cursor_style(cursor_style);
    app.body_editor.set_block(block);
    frame.render_widget(&app.body_editor, area);
}
