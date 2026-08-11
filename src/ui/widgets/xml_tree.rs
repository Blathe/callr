use ratatui::style::Style;
use ratatui::text::{Line, Span};
use roxmltree::Node;
use tui_tree_widget::TreeItem;

use crate::ui::theme;

pub struct XmlTree {
    pub items: Vec<TreeItem<'static, usize>>,
    pub index: Vec<(Vec<usize>, String)>,
}

pub fn build(doc: &roxmltree::Document) -> XmlTree {
    let mut index = Vec::new();
    let items = vec![node(&[0], doc.root_element(), &mut index)];
    XmlTree { items, index }
}

fn node(path: &[usize], element: Node, index: &mut Vec<(Vec<usize>, String)>) -> TreeItem<'static, usize> {
    let local_id = *path.last().unwrap_or(&0);
    let full_path = path.to_vec();
    let tag = element.tag_name().name().to_string();
    let attrs: String = element
        .attributes()
        .map(|a| format!(" {}=\"{}\"", a.name(), a.value()))
        .collect();

    let text_content: Option<String> = element
        .children()
        .find(|c| c.is_text())
        .and_then(|c| c.text())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    let children: Vec<_> = element
        .children()
        .filter(|c| c.is_element())
        .enumerate()
        .map(|(i, c)| {
            let mut child_path = path.to_vec();
            child_path.push(i);
            node(&child_path, c, index)
        })
        .collect();

    let open_tag = Span::styled(format!("<{tag}{attrs}>"), Style::default().fg(theme::SYNTAX_KEY));

    if children.is_empty() {
        let line = match &text_content {
            Some(text) => Line::from(vec![
                open_tag,
                Span::styled(text.clone(), Style::default().fg(theme::SYNTAX_STRING)),
                Span::styled(format!("</{tag}>"), Style::default().fg(theme::SYNTAX_KEY)),
            ]),
            None => Line::from(vec![Span::styled(
                format!("<{tag}{attrs}/>"),
                Style::default().fg(theme::SYNTAX_KEY),
            )]),
        };
        index.push((full_path, format!("{tag}{attrs} {}", text_content.unwrap_or_default())));
        TreeItem::new_leaf(local_id, line)
    } else {
        index.push((full_path, format!("{tag}{attrs}")));
        let line = Line::from(vec![open_tag]);
        TreeItem::new(local_id, line.clone(), children).unwrap_or_else(|_| TreeItem::new_leaf(local_id, line))
    }
}
