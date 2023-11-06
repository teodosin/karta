// 

use bevy::prelude::{Component, Plugin, App, Update};

use self::forces::{edge_spring_constraints, repulsion_constraints};

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