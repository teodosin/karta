use agdb::{DbError, DbValue};

use crate::elements::node;

// Some of the structs and enums in this file are currently not used.
// Determining a sound architecture for node types is difficult and
// not urgent quite yet.


pub const ARCHETYPES: [&str; 4] = ["", "attributes", "nodetypes", "settings"];

pub struct NodeData;

pub enum NodeTyppe {
    Phys(PhysCategory),
    Virtual(VirtualCategory),
}

#[derive(Debug, Clone, PartialEq)]
pub enum NodePhysicality {
    /// A node that only exists in the db and not in the file system.
    Virtual,
    /// A node that exists in the file system and the db.
    Physical,
}

impl TryFrom<DbValue> for NodePhysicality {
    type Error = DbError;

    fn try_from(value: DbValue) -> Result<Self, Self::Error> {
        match value.to_string().as_str() {
            "Virtual" => Ok(NodePhysicality::Virtual),
            "Physical" => Ok(NodePhysicality::Physical),
            _ => Err(DbError::from("Invalid NodePhysicality")),
        }
    }
}

impl From<NodePhysicality> for DbValue {
    fn from(nphys: NodePhysicality) -> Self {
        match nphys {
            NodePhysicality::Virtual => "Virtual".into(),
            NodePhysicality::Physical => "Physical".into(),
        }
    }
}

/// Categories of physical nodes.
pub enum PhysCategory {
    Root,
    Directory,
    File,
    Filepiece,
}

/// Categories of virtual nodes.
pub enum VirtualCategory {
    Archetype,
    Data,
    Operator,
}

/// Data types that a node can contain or its socket can output.
pub enum DataType {
    String,
    Int,
    Float,
    Bool,
    Vec2,
    Vec3,
    Vec4,
    Mat4,
    Texture,
    Sound,
    Font,
    Mesh,
    GCloud,
    SDFunction,
    SDField,
    Material,
    Camera,
    Light,
    Script,
    Other,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NodeType {
    type_name: String,
}

impl NodeType {
    pub fn new(type_name: String) -> Self {
        Self { type_name }
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
