// 

use std::fs;

use bevy::prelude::{Component, Plugin, App, Update};

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
pub enum NodeTypes {
    Folder, 
    FileBase,
}

// A helper function to get the type based on a node path
pub fn get_type_from_path(
    path: &String, 
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
    Image,
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