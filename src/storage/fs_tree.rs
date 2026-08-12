use std::fs;
use std::io;
use std::path::Path;

use crate::model::collection::CollectionNode;

/// Walks a collection directory into a [`CollectionNode`] tree. Folder
/// nesting *is* the collection hierarchy — no manifest file is required.
/// Only the filename is used as the label; full request parsing is
/// deferred until a request is actually opened.
pub fn build_tree(root: &Path) -> io::Result<Vec<CollectionNode>> {
    if !root.exists() {
        return Ok(Vec::new());
    }

    let mut entries: Vec<_> = fs::read_dir(root)?.filter_map(|e| e.ok()).collect();
    entries.sort_by_key(|e| e.file_name());

    let mut folders = Vec::new();
    let mut files = Vec::new();

    for entry in entries {
        let path = entry.path();
        let file_type = entry.file_type()?;

        // Skip dotfiles/dotdirs — notably our own `.callr/` history db dir,
        // which isn't part of the user's collection.
        if entry.file_name().to_string_lossy().starts_with('.') {
            continue;
        }

        if file_type.is_dir() {
            let name = entry.file_name().to_string_lossy().into_owned();
            let children = build_tree(&path)?;
            folders.push(CollectionNode::Folder {
                name,
                path,
                children,
            });
        } else if path.extension().and_then(|e| e.to_str()) == Some("toml") {
            let name = path
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_default();
            files.push(CollectionNode::Request { name, path });
        }
    }

    folders.extend(files);
    Ok(folders)
}

/// Creates a new, empty collection directory. Folder nesting is the
/// collection hierarchy (see [`build_tree`]), so this is nothing more than
/// making the directory — no manifest file to write.
pub fn create_collection(path: &Path) -> io::Result<()> {
    fs::create_dir_all(path)
}
