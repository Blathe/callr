mod auth_editor;
mod format;
mod request_editor;
mod response_viewer;
mod sidebar;
mod status_bar;
mod theme;
pub mod widgets;

use ratatui::layout::{Constraint, Layout};
use ratatui::Frame;

use crate::app::App;

pub fn draw(app: &mut App, frame: &mut Frame) {
    let area = frame.area();
    let outer = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).split(area);

    let main = Layout::horizontal([Constraint::Percentage(25), Constraint::Percentage(75)]).split(outer[0]);

    sidebar::render(app, frame, main[0]);

    let content =
        Layout::vertical([Constraint::Length(3), Constraint::Length(3), Constraint::Min(0)]).split(main[1]);
    request_editor::render(app, frame, content[0]);
    auth_editor::render(app, frame, content[1]);
    response_viewer::render(app, frame, content[2]);

    status_bar::render(app, frame, outer[1]);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::App;
    use crate::model::response::ResponseData;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    use std::time::Duration;
    use tokio::sync::mpsc;

    fn render_to_string(app: &mut App) -> String {
        let backend = TestBackend::new(120, 30);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| draw(app, frame)).unwrap();

        let buffer = terminal.backend().buffer().clone();
        let mut text = String::new();
        for y in 0..buffer.area.height {
            for x in 0..buffer.area.width {
                text.push_str(buffer[(x, y)].symbol());
            }
            text.push('\n');
        }
        text
    }

    #[test]
    fn empty_state_layout_snapshot() {
        let dir = tempfile::tempdir().unwrap();
        let (tx, _rx) = mpsc::unbounded_channel();
        let mut app = App::new(tx, dir.path().to_path_buf());

        let text = render_to_string(&mut app);
        insta::assert_snapshot!(text);
    }

    #[test]
    fn auth_bearer_layout_snapshot() {
        let dir = tempfile::tempdir().unwrap();
        let (tx, _rx) = mpsc::unbounded_channel();
        let mut app = App::new(tx, dir.path().to_path_buf());
        app.focus = crate::mode::Focus::Auth;
        app.auth_kind = crate::app::AuthKind::Bearer;
        app.auth_bearer_token = tui_input::Input::new("secret-token".to_string());

        let text = render_to_string(&mut app);
        insta::assert_snapshot!(text);
    }

    #[test]
    fn json_response_renders_as_a_collapsible_tree() {
        let dir = tempfile::tempdir().unwrap();
        let (tx, _rx) = mpsc::unbounded_channel();
        let mut app = App::new(tx, dir.path().to_path_buf());
        app.response = Some(ResponseData {
            status: 200,
            status_text: "OK".to_string(),
            headers: vec![("content-type".to_string(), "application/json".to_string())],
            body: r#"{"slideshow":{"title":"Sample"}}"#.to_string(),
            is_valid_utf8: true,
            byte_len: 33,
            elapsed: Duration::from_millis(12),
        });

        let text = render_to_string(&mut app);
        assert!(text.contains("slideshow"), "expected JSON key in rendered output:\n{text}");
    }

    #[test]
    fn xml_response_renders_as_a_collapsible_tree() {
        let dir = tempfile::tempdir().unwrap();
        let (tx, _rx) = mpsc::unbounded_channel();
        let mut app = App::new(tx, dir.path().to_path_buf());
        app.response = Some(ResponseData {
            status: 200,
            status_text: "OK".to_string(),
            headers: vec![("content-type".to_string(), "application/xml".to_string())],
            body: "<root><greeting>hello</greeting></root>".to_string(),
            is_valid_utf8: true,
            byte_len: 40,
            elapsed: Duration::from_millis(8),
        });
        // Trees render collapsed by default; expand the root to see its
        // children, exercising the same TreeState::open path search uses.
        app.response_tree_state.open(vec![0]);

        let text = render_to_string(&mut app);
        assert!(text.contains("greeting"), "expected XML tag in rendered output:\n{text}");
    }

    #[test]
    fn collection_tree_skips_internal_callr_dir() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(".callr")).unwrap();
        let (tx, _rx) = mpsc::unbounded_channel();
        let mut app = App::new(tx, dir.path().to_path_buf());

        let text = render_to_string(&mut app);
        assert!(!text.contains(".callr"), "the internal .callr dir must not appear in the sidebar:\n{text}");
    }

    #[test]
    fn binary_response_shows_byte_count_placeholder() {
        let dir = tempfile::tempdir().unwrap();
        let (tx, _rx) = mpsc::unbounded_channel();
        let mut app = App::new(tx, dir.path().to_path_buf());
        app.response = Some(ResponseData {
            status: 200,
            status_text: "OK".to_string(),
            headers: vec![("content-type".to_string(), "application/octet-stream".to_string())],
            body: String::new(),
            is_valid_utf8: false,
            byte_len: 4096,
            elapsed: Duration::from_millis(5),
        });

        let text = render_to_string(&mut app);
        assert!(text.contains("Binary content"), "expected binary placeholder in output:\n{text}");
        assert!(text.contains("4.00 KB"), "expected formatted byte count in output:\n{text}");
    }
}
