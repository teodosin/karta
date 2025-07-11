use std::{error::Error, ops::Index, path::{Path, PathBuf}};

use crate::prelude::{DataNode, NodePath, NodeTypeId};

fn get_node_type_from_path(path: &Path) -> NodeTypeId {
    if path.is_dir() {
        return NodeTypeId::dir_type();
    }

    if path.is_file() {
        return match path.extension().and_then(|s| s.to_str()) {
            Some("png") | Some("jpg") | Some("jpeg") | Some("gif") => NodeTypeId::image_type(),
            _ => NodeTypeId::file_type(),
        };
    }

    // Default fallback, though in practice the calling logic should prevent this.
    NodeTypeId::file_type()
}

pub fn destructure_file_path(
    vault_root_path: &PathBuf, // New parameter: the root of the Karta vault
    path_to_destructure: &PathBuf, // This is the absolute path of the item to destructure
    include_self: bool,
) -> Result<Vec<DataNode>, Box<dyn Error>> {
    if !path_to_destructure.exists() {
        return Err(format!("Path to destructure does not exist: {:?}", path_to_destructure).into());
    }
    let mut nodes: Vec<DataNode> = Vec::new();

    if path_to_destructure.is_file() && include_self {
        // Create NodePath relative to vault_root_path
        let node_path = NodePath::from_dir_path(vault_root_path, path_to_destructure);
        let ntype = get_node_type_from_path(path_to_destructure);
        let node = DataNode::new(&node_path, ntype);
        nodes.push(node);
    }

    if path_to_destructure.is_dir() {
        // If include_self is true for a directory, we might want to add the directory itself.
        // However, the current KartaService logic creates the focal directory node separately.
        // This function, as per its usage in KartaService, primarily returns children for a dir.
        // If include_self for a dir meant to include the dir itself, that logic would be here.
        // For now, it only lists children.

        let dir_entries = std::fs::read_dir(path_to_destructure)?;

        for entry in dir_entries {
            let entry = entry?;
            let absolute_entry_path = entry.path(); // This is an absolute path

            // Ignore .karta directory or files
            if absolute_entry_path.file_name().map_or(false, |name| name == ".karta") {
                continue;
            }

            // Create NodePath relative to vault_root_path
            let node_path = NodePath::from_dir_path(vault_root_path, &absolute_entry_path);
            let ntype: NodeTypeId;

            if absolute_entry_path.is_dir() {
                ntype = NodeTypeId::dir_type();
            } else if absolute_entry_path.is_file() {
                ntype = get_node_type_from_path(&absolute_entry_path);
            } else { continue; }

            let node = DataNode::new(&node_path, ntype);
            nodes.push(node);
        }
    }
Ok(nodes)
}

pub fn get_all_paths(vault_root_path: &PathBuf) -> Result<Vec<String>, Box<dyn Error>> {
    let mut paths = Vec::new();
    let mut walker = walkdir::WalkDir::new(vault_root_path).into_iter();

    while let Some(entry) = walker.next() {
        let entry = entry?;
        let path = entry.path();

        if path.file_name().map_or(false, |name| name == ".karta") {
            walker.skip_current_dir();
            continue;
        }

        if path.is_dir() || path.is_file() {
            let node_path = NodePath::from_dir_path(vault_root_path, &path.to_path_buf());
            paths.push(node_path.alias().to_string());
        }
    }

    Ok(paths)
}
