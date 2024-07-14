
use crate::elements::{Node, NodePhysicality};

pub enum NodeCategory {
    Root,
    Directory,
    File,
    Filepiece,
}


// A generic trait for a TYPE of node. Not an instance of a type of node. 
#[derive(Debug, Clone)]
pub struct NodeType {
    type_name: String,
}

impl NodeType {
    pub fn new(type_name: String) -> Self {
        Self {
            type_name,
        }
    }

    /// Returns a node type that represents the root of the graph.
    pub fn root_type() -> Self {
        Self {
            type_name: "Root".to_string(),
        }
    }

    /// Type for root-level virtual nodes. Ie. attributes, nodetypes, settings,
    /// other such archetypes. 
    pub fn archetype_type() -> Self {
        Self {
            type_name: "Archetype".to_string(),
        }
    }

    pub fn other() -> Self {
        Self {
            type_name: "Other".to_string(),
        }
    }

    pub fn name(&self) -> &str {
        &self.type_name
    }
}

impl TryFrom<agdb::DbValue> for NodeType {
    type Error = agdb::DbError;

    fn try_from(value: agdb::DbValue) -> Result<Self, Self::Error> {
        match value.string() {
            Ok(s) => Ok(NodeType::new(s.to_string())),
            Err(e) => Err(agdb::DbError::from("Invalid NodeType")),
        }
    }
}

impl From<NodeType> for agdb::DbValue {
    fn from(ntype: NodeType) -> Self {
        ntype.name().into()
    }
}