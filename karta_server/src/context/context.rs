use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{prelude::ViewNode, SERVER_VERSION};

use super::context_settings::ContextSettings;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Context {
    karta_version: String,
    focal: Uuid,
    nodes: Vec<ViewNode>,
    settings: ContextSettings
}

impl Context {
    pub fn focal(&self) -> Uuid {
        self.focal
    }

    pub fn viewnodes(&self) -> &Vec<ViewNode> {
        &self.nodes
    }

    pub fn new(focal: Uuid) -> Self {
        Self {
            karta_version: SERVER_VERSION.to_string(),
            focal,
            nodes: Vec::new(),
            settings: ContextSettings::default()
        }
    }

    pub fn with_viewnodes(focal: Uuid, nodes: Vec<ViewNode>) -> Self {
        Self {
            karta_version: SERVER_VERSION.to_string(),
            focal,
            nodes,
            settings: ContextSettings::default()
        }
    }

    pub fn add_node(&mut self, node: ViewNode) {
        self.nodes.push(node);
    }
}