use ratatui::style::Style;
use ratatui::text::{Line, Span};
use serde_json::Value;
use tui_tree_widget::TreeItem;

use crate::ui::theme;

pub struct JsonTree {
    pub items: Vec<TreeItem<'static, usize>>,
    /// Every node's path (per-level local index, root to node) paired with
    /// its searchable text, used to resolve `/` search queries to a node.
    pub index: Vec<(Vec<usize>, String)>,
}

pub fn build(value: &Value) -> JsonTree {
    let mut index = Vec::new();
    let items = top_level_nodes(value, &mut index);
    JsonTree { items, index }
}

fn top_level_nodes(value: &Value, index: &mut Vec<(Vec<usize>, String)>) -> Vec<TreeItem<'static, usize>> {
    match value {
        Value::Object(map) => map
            .iter()
            .enumerate()
            .map(|(i, (key, v))| node(&[i], key.clone(), v, index))
            .collect(),
        Value::Array(items) => items
            .iter()
            .enumerate()
            .map(|(i, v)| node(&[i], format!("[{i}]"), v, index))
            .collect(),
        other => vec![node(&[0], "value".to_string(), other, index)],
    }
}

fn node(
    path: &[usize],
    label: String,
    value: &Value,
    index: &mut Vec<(Vec<usize>, String)>,
) -> TreeItem<'static, usize> {
    // The widget composes ancestor chains itself — each node's own
    // identifier only needs to be unique among its siblings.
    let local_id = *path.last().unwrap_or(&0);
    let full_path = path.to_vec();
    let label_span = Span::styled(format!("{label}: "), Style::default().fg(theme::SYNTAX_KEY));

    match value {
        Value::Object(map) => {
            index.push((full_path, format!("{label}: object")));
            let children: Vec<_> = map
                .iter()
                .enumerate()
                .map(|(i, (k, v))| {
                    let mut child_path = path.to_vec();
                    child_path.push(i);
                    node(&child_path, k.clone(), v, index)
                })
                .collect();
            let summary = Span::styled(
                format!("{{ {} }}", pluralize(map.len(), "key")),
                Style::default().fg(theme::MUTED),
            );
            let line = Line::from(vec![label_span, summary]);
            TreeItem::new(local_id, line.clone(), children).unwrap_or_else(|_| TreeItem::new_leaf(local_id, line))
        }
        Value::Array(items) => {
            index.push((full_path, format!("{label}: array")));
            let children: Vec<_> = items
                .iter()
                .enumerate()
                .map(|(i, v)| {
                    let mut child_path = path.to_vec();
                    child_path.push(i);
                    node(&child_path, format!("[{i}]"), v, index)
                })
                .collect();
            let summary = Span::styled(
                format!("[ {} ]", pluralize(items.len(), "item")),
                Style::default().fg(theme::MUTED),
            );
            let line = Line::from(vec![label_span, summary]);
            TreeItem::new(local_id, line.clone(), children).unwrap_or_else(|_| TreeItem::new_leaf(local_id, line))
        }
        Value::String(s) => {
            index.push((full_path, format!("{label}: {s}")));
            let line = Line::from(vec![
                label_span,
                Span::styled(format!("\"{s}\""), Style::default().fg(theme::SYNTAX_STRING)),
            ]);
            TreeItem::new_leaf(local_id, line)
        }
        Value::Number(n) => {
            index.push((full_path, format!("{label}: {n}")));
            let line = Line::from(vec![label_span, Span::styled(n.to_string(), Style::default().fg(theme::SYNTAX_NUMBER))]);
            TreeItem::new_leaf(local_id, line)
        }
        Value::Bool(b) => {
            index.push((full_path, format!("{label}: {b}")));
            let line = Line::from(vec![
                label_span,
                Span::styled(b.to_string(), Style::default().fg(theme::SYNTAX_BOOL)),
            ]);
            TreeItem::new_leaf(local_id, line)
        }
        Value::Null => {
            index.push((full_path, format!("{label}: null")));
            let line = Line::from(vec![label_span, Span::styled("null", Style::default().fg(theme::SYNTAX_NULL))]);
            TreeItem::new_leaf(local_id, line)
        }
    }
}

fn pluralize(count: usize, word: &str) -> String {
    if count == 1 {
        format!("1 {word}")
    } else {
        format!("{count} {word}s")
    }
}
