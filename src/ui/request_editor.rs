use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::App;
use crate::mode::{Focus, Mode};
use crate::ui::theme;

pub fn render(app: &App, frame: &mut Frame, area: Rect) {
    let method = app
        .current_request
        .as_ref()
        .map(|r| r.method.to_string())
        .unwrap_or_else(|| "GET".to_string());
    let name = app
        .current_request
        .as_ref()
        .map(|r| r.meta.name.as_str())
        .unwrap_or("scratch");
    let title = if app.in_flight {
        format!("{method} · {name} (sending...)")
    } else {
        format!("{method} · {name}")
    };
    let border_style = theme::border_style(app.focus == Focus::UrlBar);

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(border_style);
    let paragraph = Paragraph::new(app.url_input.value()).block(block);
    frame.render_widget(paragraph, area);

    if app.mode == Mode::Insert && app.focus == Focus::UrlBar {
        let cursor_x = area.x + 1 + app.url_input.visual_cursor() as u16;
        let cursor_y = area.y + 1;
        frame.set_cursor_position((cursor_x, cursor_y));
    }
}
