use std::{error::Error, ops::Index, path::PathBuf};

use crate::prelude::{DataNode, NodePath, NodeTypeId};

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
        let ntype = NodeTypeId::file_type();
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
                ntype = NodeTypeId::file_type();
            } else { continue; }

            let node = DataNode::new(&node_path, ntype);
            nodes.push(node);
        }
    }

    Ok(nodes)
}

// Todo: some tests for this


