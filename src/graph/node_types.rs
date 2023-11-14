// 

use std::{fs, fmt, path::PathBuf};

use bevy::prelude::{Component, Plugin, App};
use enum_iterator::Sequence;

mod file_types;
mod filters;
mod forces;
mod operators;
mod panels;
mod query;

pub struct NodeTypesPlugin;

impl Plugin for NodeTypesPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(forces::ForceNodesPlugin)
        ;
    }
}

// For now, all node types will be stored in a single enum
// This will be changed to a more flexible system later
#[derive(Clone, Copy, Debug, PartialEq, Sequence)]
pub enum NodeTypes {
    Folder, 
    FileBase,
    FileImage,
    FileText,
    Text,
}

impl fmt::Display for NodeTypes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            NodeTypes::Folder => write!(f, "Folder"),
            NodeTypes::FileBase => write!(f, "Generic File"),
            NodeTypes::FileImage => write!(f, "Image"),
            NodeTypes::FileText => write!(f, "Text File"),
            NodeTypes::Text => write!(f, "Text Card"),
        }
    }

}

#[derive(Clone, Debug, PartialEq, Sequence)]
pub enum DataTypes {
    Text,
    Image,
}

// A helper function to get the type based on a node path
pub fn get_type_from_path(
    path: &PathBuf, 
) -> Option<NodeTypes> {
    match fs::metadata(&path) {
        Ok(metadata) => {
            if metadata.is_dir() {
                Some(NodeTypes::Folder)
            } else {
                Some(NodeTypes::FileBase)
            }
        },
        Err(_) => {
            println!("Error: Parent path does not exist");
            None
        }
    }
}

trait Node {
    // Path and node are stored in the GraphNode, right?
    // fn get_path(&self) -> String;
    // fn get_name(&self) -> String;
    fn get_data_type(&self) -> NodeDataTypes;

    fn get_data(&self) -> Self;
}

enum NodeDataTypes {
    _Image,
    Text,
}

#[derive(Component, Debug)]
pub struct NodeFileText {
    pub text: String,
}

impl Node for NodeFileText {
    fn get_data_type(&self) -> NodeDataTypes {
        NodeDataTypes::Text
    }

    fn get_data(&self) -> Self {
        NodeFileText {
            text: self.text.clone(),
        }
    }
}