use std::path::PathBuf;

use agdb::{DbError, DbId, DbValue, UserValue};

use crate::path_ser::{buf_to_str, str_to_buf};

/// The universal node type. 
#[derive(Debug, UserValue)]
pub struct Node {
    /// The id of the node in the database.
    db_id: Option<DbId>,
    /// The path of the node relative to the root of the graph.
    /// The path is stored as a string in the database, but is converted to a PathBuf when
    /// the node is loaded.
    path: NodePath,
    ntype: NodeType,
    nphys: NodePhysicality,
}

impl Node {
    pub fn new(path: NodePath, ntype: NodeType) -> Self {
        let nphys: NodePhysicality;
        match ntype {
            NodeType::Directory => nphys = NodePhysicality::Physical,
            NodeType::File => nphys = NodePhysicality::Physical,
            _ => nphys = NodePhysicality::Virtual,
        }

        Node {
            db_id: None,
            path,
            ntype,
            nphys,
        }
    }

    pub fn path(&self) -> &PathBuf {
        &self.path.0
    }

    pub fn ntype(&self) -> NodeType {
        self.ntype.clone()
    }
}

/// Newtype wrapper for the node path. 
#[derive(Debug, Clone)]
pub struct NodePath(pub PathBuf);

impl TryFrom<DbValue> for NodePath {
    type Error = DbError;

    fn try_from(value: DbValue) -> Result<Self, Self::Error> {
        Ok(NodePath(str_to_buf(&value.to_string())))
    }
}

impl From<NodePath> for DbValue {
    fn from(path: NodePath) -> Self {
        buf_to_str(&path.0).into()
    }
}

#[derive(Debug, Clone)]
pub enum NodePhysicality {
    /// A node that only exists in the db and not in the file system.
    Virtual,
    /// A node that exists in the file system and the db.
    Physical,
    /// An originally physical node that has been removed from the file system.
    Dead,
}

impl TryFrom<DbValue> for NodePhysicality {
    type Error = DbError;

    fn try_from(value: DbValue) -> Result<Self, Self::Error> {
        match value.to_string().as_str() {
            "Virtual" => Ok(NodePhysicality::Virtual),
            "Physical" => Ok(NodePhysicality::Physical),
            "Dead" => Ok(NodePhysicality::Dead),
            _ => Err(DbError::from("Invalid NodePhysicality")),
        }
    }
}

impl From<NodePhysicality> for DbValue {
    fn from(nphys: NodePhysicality) -> Self {
        match nphys {
            NodePhysicality::Virtual => "Virtual".into(),
            NodePhysicality::Physical => "Physical".into(),
            NodePhysicality::Dead => "Dead".into(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum NodeType {
    Directory,
    File,
}


// TODO: Could a macro be created for this?
impl TryFrom<DbValue> for NodeType {
    type Error = DbError;

    fn try_from(value: DbValue) -> Result<Self, Self::Error> {
        match value.to_string().as_str() {
            "Directory" => Ok(NodeType::Directory),
            "File" => Ok(NodeType::File),
            _ => Err(DbError::from("Invalid NodeType")),
        }
    }
}

impl From<NodeType> for DbValue {
    fn from(ntype: NodeType) -> Self {
        match ntype {
            NodeType::Directory => "Directory".into(),
            NodeType::File => "File".into(),
        }
    }
}

pub struct Edge {
    attributes: Vec<Attribute>,
}

#[derive(Clone, Debug)]
pub struct Attribute {
    name: String,
    value: f32,
}

/// A list of reserved node attribute names that cannot be used by the user.
const RESERVED_NODE_ATTRS: [&str; 2] = [
    "name", // The full path of the node
    "ntype", // The type of the node
];
/// A list of reserved edge attribute names that cannot be used by the user.
const RESERVED_EDGE_ATTRS: [&str; 1] = [
    "contains", // For directories
];
