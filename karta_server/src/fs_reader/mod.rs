use std::{error::Error, ops::Index, path::PathBuf};

use crate::prelude::{DataNode, NodePath, NodeTypeId};





fn destructure_file_path(
    path: &PathBuf,
    include_self: bool,
) -> Result<Vec<DataNode>, Box<dyn Error>> {
    if !path.exists() {
        return Err("Path does not exist".into());
    }
    let mut nodes: Vec<DataNode> = Vec::new();

    // This is where we would redirect to different file type handling
    // Right now files don't get destructured, so if we don't include
    // self, we don't get any nodes.
    if path.is_file() && include_self {
        let node_path = NodePath::new(path.to_path_buf());
        let ntype = NodeTypeId::file_type();
        let node = DataNode::new(&node_path, ntype);
        nodes.push(node);
    }

    if path.is_dir() {
        let dir_entries = std::fs::read_dir(path)?;

        for entry in dir_entries {
            let entry = entry?;
            let path = entry.path();

            let node_path = NodePath::new(path.clone());
            let mut ntype: NodeTypeId;

            if path.is_dir() {
                ntype = NodeTypeId::dir_type();
            } else if path.is_file() {
                ntype = NodeTypeId::file_type();
            } else { continue; }

            let node = DataNode::new(&node_path, ntype);
            nodes.push(node);
        }
    }

    return Ok(nodes);
}

// Todo: some tests for this


