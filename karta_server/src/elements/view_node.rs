use uuid::Uuid;

use crate::prelude::Attribute;

use super::node::DataNode;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ViewNodeStatus {
    Generated,
    Modified,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ViewNode {
    pub uuid: Uuid,

    pub status: ViewNodeStatus,

    pub is_name_visible: bool,

    pub relX: f32,
    pub relY: f32,
    pub width: f32,
    pub height: f32,
    pub relScale: f32,
    pub rotation: f32,

    pub attributes: Vec<Attribute>,
}

impl ViewNode {
    pub fn uuid(&self) -> Uuid {
        self.uuid
    }

    pub fn from_data_node(data_node: DataNode) -> Self {
        let viewnode_attributes: Vec<Attribute> = Vec::new();

        Self {
            uuid: data_node.uuid(),
            status: ViewNodeStatus::Generated,
            is_name_visible: true,
            relX: 0.0,
            relY: 0.0,
            width: 200.0,
            height: 200.0,
            relScale: 1.0,
            rotation: 0.0,
            attributes: viewnode_attributes,
        }
    
    }

    pub fn status(&self) -> &ViewNodeStatus {
        &self.status
    }

    pub fn set_status(&mut self, status: ViewNodeStatus) {
        self.status = status;
    }

    pub fn sized(mut self, width: f32, height: f32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    pub fn positioned(mut self, relX: f32, relY: f32) -> Self {
        self.relX = relX;
        self.relY = relY;
        self
    }
}