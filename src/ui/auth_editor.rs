use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;
use unicode_width::UnicodeWidthStr;

use crate::app::{App, AuthKind};
use crate::mode::{Focus, Mode};
use crate::ui::theme;

pub fn render(app: &App, frame: &mut Frame, area: Rect) {
    let focused = app.focus == Focus::Auth;
    let border_style = theme::border_style(focused);
    let title = format!("Auth: {}", app.auth_kind.label());
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(border_style);

    let (spans, cursor_offset) = build_line(app, focused);
    let paragraph = Paragraph::new(Line::from(spans)).block(block);
    frame.render_widget(paragraph, area);

    if focused && app.mode == Mode::Insert
        && let Some(offset) = cursor_offset
    {
        let cursor_x = area.x + 1 + offset as u16;
        let cursor_y = area.y + 1;
        frame.set_cursor_position((cursor_x, cursor_y));
    }
}

fn push(spans: &mut Vec<Span<'static>>, width: &mut usize, text: String) {
    *width += text.width();
    spans.push(Span::raw(text));
}

fn build_line(app: &App, focused: bool) -> (Vec<Span<'static>>, Option<usize>) {
    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut cursor_offset = None;
    let mut width = 0usize;

    match app.auth_kind {
        AuthKind::None => {
            push(
                &mut spans,
                &mut width,
                "press 'a' to choose an auth type (Bearer / Basic / API Key)".to_string(),
            );
        }
        AuthKind::Bearer => {
            push(&mut spans, &mut width, "token: ".to_string());
            if focused && app.auth_field == 0 {
                cursor_offset = Some(width + app.auth_bearer_token.visual_cursor());
            }
            push(&mut spans, &mut width, app.auth_bearer_token.value().to_string());
        }
        AuthKind::Basic => {
            push(&mut spans, &mut width, "username: ".to_string());
            if focused && app.auth_field == 0 {
                cursor_offset = Some(width + app.auth_basic_username.visual_cursor());
            }
            push(&mut spans, &mut width, app.auth_basic_username.value().to_string());
            push(&mut spans, &mut width, "   password: ".to_string());
            if focused && app.auth_field == 1 {
                cursor_offset = Some(width + app.auth_basic_password.visual_cursor());
            }
            push(
                &mut spans,
                &mut width,
                "*".repeat(app.auth_basic_password.value().chars().count()),
            );
        }
        AuthKind::ApiKey => {
            push(&mut spans, &mut width, "name: ".to_string());
            if focused && app.auth_field == 0 {
                cursor_offset = Some(width + app.auth_apikey_name.visual_cursor());
            }
            push(&mut spans, &mut width, app.auth_apikey_name.value().to_string());
            push(&mut spans, &mut width, "   value: ".to_string());
            if focused && app.auth_field == 1 {
                cursor_offset = Some(width + app.auth_apikey_value.visual_cursor());
            }
            push(&mut spans, &mut width, app.auth_apikey_value.value().to_string());
            push(
                &mut spans,
                &mut width,
                format!("   in: {} (l/Enter to toggle)", app.auth_apikey_location.label()),
            );
        }
    }

    (spans, cursor_offset)
}
