use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState};
use ratatui::Frame;
use tui_tree_widget::Tree;

use crate::app::App;
use crate::mode::{Focus, SidebarView};
use crate::ui::theme;

pub fn render(app: &mut App, frame: &mut Frame, area: Rect) {
    match app.sidebar_view {
        SidebarView::Collections => render_tree(app, frame, area),
        SidebarView::History => render_history(app, frame, area),
    }
}

fn render_tree(app: &mut App, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .title("Collection")
        .borders(Borders::ALL)
        .border_style(theme::border_style(app.focus == Focus::Sidebar));

    let tree = Tree::new(&app.tree_items)
        .expect("tree item paths are unique")
        .block(block)
        .highlight_style(theme::selection_style());

    frame.render_stateful_widget(tree, area, &mut app.tree_state);
}

fn render_history(app: &mut App, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .title("History")
        .borders(Borders::ALL)
        .border_style(theme::border_style(app.focus == Focus::Sidebar));

    let items: Vec<ListItem> = if app.history_entries.is_empty() {
        vec![ListItem::new("No requests sent yet")]
    } else {
        app.history_entries
            .iter()
            .map(|entry| {
                let status = entry
                    .status_code
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| "ERR".to_string());
                ListItem::new(Line::from(vec![
                    Span::raw(format!("{:<6} ", entry.method)),
                    Span::raw(format!("{status:<4} ")),
                    Span::raw(entry.url.clone()),
                ]))
            })
            .collect()
    };

    let mut state = ListState::default();
    if !app.history_entries.is_empty() {
        state.select(Some(app.history_selected));
    }

    let list = List::new(items).block(block).highlight_style(theme::selection_style());
    frame.render_stateful_widget(list, area, &mut state);
}
