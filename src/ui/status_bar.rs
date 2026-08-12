use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::app::App;
use crate::mode::Mode;
use crate::ui::theme;

pub fn render(app: &App, frame: &mut Frame, area: Rect) {
    let line = match app.mode {
        Mode::Command => Line::from(vec![
            Span::styled(":", theme::accent_style()),
            Span::raw(app.command_input.value()),
        ]),
        Mode::Search => Line::from(vec![
            Span::styled("/", theme::accent_style()),
            Span::raw(app.search_input.value()),
        ]),
        _ => {
            let mode_label = if app.mode == Mode::Insert { "INSERT" } else { "NORMAL" };
            let mut spans = vec![Span::styled(format!(" {mode_label} "), theme::mode_indicator_style())];
            if let Some(msg) = &app.status_message {
                spans.push(Span::raw(format!("  {msg}")));
            } else {
                let hint = match app.focus {
                    crate::mode::Focus::Sidebar => {
                        "  j/k: move  gg/G: top/bottom  h/l: collapse/expand  Enter: open  Tab/S-Tab: cycle pane  :history  :collections  :mkdir  q: quit"
                    }
                    crate::mode::Focus::UrlBar => {
                        "  i: edit url  m: method  Enter: send  Tab/S-Tab: cycle pane  ':': command  q: quit"
                    }
                    crate::mode::Focus::Auth => {
                        "  a: cycle type  j/k: field  i: edit  l/Enter: toggle in  Tab/S-Tab: cycle pane  q: quit"
                    }
                    crate::mode::Focus::Body => {
                        "  i: edit body  Tab/S-Tab: cycle pane  q: quit"
                    }
                    crate::mode::Focus::Response => {
                        "  j/k: move  gg/G: top/bottom  h/l: collapse/expand  /: search  n/N: next/prev match  Tab/S-Tab: cycle pane  q: quit"
                    }
                };
                spans.push(Span::raw(hint));
            }
            Line::from(spans)
        }
    };

    frame.render_widget(Paragraph::new(line), area);

    match app.mode {
        Mode::Command => {
            let cursor_x = area.x + 1 + app.command_input.visual_cursor() as u16;
            frame.set_cursor_position((cursor_x, area.y));
        }
        Mode::Search => {
            let cursor_x = area.x + 1 + app.search_input.visual_cursor() as u16;
            frame.set_cursor_position((cursor_x, area.y));
        }
        _ => {}
    }
}
