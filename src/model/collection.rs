use std::path::PathBuf;

use tui_tree_widget::TreeItem;

#[derive(Debug, Clone)]
pub enum CollectionNode {
    Folder {
        name: String,
        path: PathBuf,
        children: Vec<CollectionNode>,
    },
    Request {
        name: String,
        path: PathBuf,
    },
}

impl CollectionNode {
    /// Converts the domain tree into the widget-facing shape the sidebar
    /// renders and navigates, keyed by file path (unique and stable across
    /// a session).
    pub fn to_tree_items(nodes: &[CollectionNode]) -> Vec<TreeItem<'static, PathBuf>> {
        nodes
            .iter()
            .filter_map(|node| match node {
                CollectionNode::Folder {
                    name,
                    path,
                    children,
                } => TreeItem::new(path.clone(), name.clone(), Self::to_tree_items(children)).ok(),
                CollectionNode::Request { name, path } => {
                    Some(TreeItem::new_leaf(path.clone(), name.clone()))
                }
            })
            .collect()
    }
}
