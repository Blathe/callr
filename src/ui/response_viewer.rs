use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::Text;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;
use tui_tree_widget::{Tree, TreeItem};

use crate::app::App;
use crate::mode::{Focus, SidebarView};
use crate::ui::theme;
use crate::ui::widgets::{self, BodyView};

pub fn render(app: &mut App, frame: &mut Frame, area: Rect) {
    if app.focus == Focus::Sidebar && app.sidebar_view == SidebarView::History {
        render_history_snapshot(app, frame, area);
        return;
    }

    let border_style = theme::border_style(app.focus == Focus::Response);

    if let Some(err) = &app.response_error {
        render_message(frame, area, format!("Error: {err}"), border_style);
        return;
    }

    let Some(response) = app.response.clone() else {
        let message = if app.in_flight {
            "Sending request...".to_string()
        } else {
            "No response yet. Press 'i' to edit the URL, Enter to send.".to_string()
        };
        render_message(frame, area, message, border_style);
        return;
    };

    let header = format!("{} {}  ·  {:?}", response.status, response.status_text, response.elapsed);
    let content_type = response
        .headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case("content-type"))
        .map(|(_, v)| v.as_str());
    let view = widgets::detect(content_type, &response.body, response.is_valid_utf8);

    match view {
        BodyView::Json(value) => {
            let tree = widgets::json_tree::build(&value);
            render_tree(app, frame, area, &header, tree.items, border_style);
        }
        BodyView::Xml(body) => match roxmltree::Document::parse(&body) {
            Ok(doc) => {
                let tree = widgets::xml_tree::build(&doc);
                render_tree(app, frame, area, &header, tree.items, border_style);
            }
            Err(_) => render_raw_text(frame, area, &header, &response.body, border_style),
        },
        BodyView::Text => render_raw_text(frame, area, &header, &response.body, border_style),
        BodyView::Binary => {
            render_message(
                frame,
                area,
                format!("{header}\n\nBinary content, {} bytes", response.byte_len),
                border_style,
            );
        }
    }
}

fn render_tree(
    app: &mut App,
    frame: &mut Frame,
    area: Rect,
    header: &str,
    items: Vec<TreeItem<'static, usize>>,
    border_style: Style,
) {
    let title = format!("Response — {header}  (/ to search)");
    let block = Block::default().title(title).borders(Borders::ALL).border_style(border_style);
    let tree = Tree::new(&items)
        .unwrap_or_else(|_| Tree::new(&[]).expect("empty tree is always valid"))
        .block(block)
        .highlight_style(theme::selection_style());
    frame.render_stateful_widget(tree, area, &mut app.response_tree_state);
}

fn render_raw_text(frame: &mut Frame, area: Rect, header: &str, body: &str, border_style: Style) {
    let block = Block::default()
        .title(format!("Response — {header}"))
        .borders(Borders::ALL)
        .border_style(border_style);
    let paragraph = Paragraph::new(body.to_string()).block(block).wrap(Wrap { trim: false });
    frame.render_widget(paragraph, area);
}

fn render_message(frame: &mut Frame, area: Rect, message: String, border_style: Style) {
    let block = Block::default()
        .title("Response")
        .borders(Borders::ALL)
        .border_style(border_style);
    let paragraph = Paragraph::new(Text::raw(message)).block(block).wrap(Wrap { trim: false });
    frame.render_widget(paragraph, area);
}

fn render_history_snapshot(app: &mut App, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .title("Response (history snapshot — Enter to re-run)")
        .borders(Borders::ALL);

    let text = match app.history_entries.get(app.history_selected) {
        Some(entry) => {
            let status = entry
                .status_code
                .map(|c| c.to_string())
                .unwrap_or_else(|| "ERROR".to_string());
            let duration = entry
                .duration_ms
                .map(|ms| format!("{ms}ms"))
                .unwrap_or_else(|| "-".to_string());

            let mut body = format!(
                "{} {}\nStatus: {}   Time: {}\nSent: {}\n\n",
                entry.method,
                entry.url,
                status,
                duration,
                entry.sent_at.to_rfc3339(),
            );
            if let Some(err) = &entry.error_message {
                body.push_str(&format!("Error: {err}\n"));
            }
            if let Some(response_body) = &entry.response_body {
                body.push_str(response_body);
            }
            Text::raw(body)
        }
        None => Text::raw("No requests sent yet."),
    };

    let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: false });
    frame.render_widget(paragraph, area);
}
