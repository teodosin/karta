// 

use bevy::prelude::Component;

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