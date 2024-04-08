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

/// A list of reserved node attribute names that cannot be set by the user directly.
pub const RESERVED_NODE_ATTRS: [&str; 7] = [
    "path", // The full path of the node, name included.
    "name", // The name of the node, without the path. Maybe allows for different characters?
    "ntype", // The type of the node

    "scale", // The absolute scaling of the node, in case it is needed. Vec of 2 f32s
    "rotation", // The absolute rotation of the node, in case it is needed. 
    "color", // The absolute color of the node. Vec of 4 f32s
    "pins", // The state pins of the node. 000 / 001 / 010 / 011 / 100 / 101 / 110 / 111
];

/// A list of reserved edge attribute names that cannot be set by the user directly.
/// Note that they are optional, so default behavior is when they are not set.
pub const RESERVED_EDGE_ATTRS: [&str; 18] = [
    "contains", // Parent-child relationship
    "preview", // Connects a file to a preview image (or gif? Multiple?)

    "from-transition", // Path to an animation file for when the edge is traversed in play mode. 
    "to-transition", // Path to an animation file for when the edge is traversed in play mode.
    "from-preload", // Preload settings for source node when in the target's context & play mode
    "to-preload", // Preload settings for the target node when in source node's context & play mode

    "from-output", // Index of an output socket in source node. Must be validated.
    "to-input", // Index of an input socket in target node. Must be validated. 

    // The following attributes are all Vecs of 2 f32s. 
    "from-position", // Relative position of source node to the target node
    "to-position", // Relative position of the target node to source node
    "from-scale", // Relative scale of source node to the target node
    "to-scale", // Relative scale of the target node to source node

    // The following attributes are all Vecs of 4 f32s.
    "from-color", // Color of the source node when in the target's context
    "to-color", // Color of the target node when in the source node's context

    // The state pins of the node. 000 / 001 / 010 / 011 / 100 / 101 / 110 / 111
    "from-pins", // The state pins of the source node when in the target's context
    "to-pins", // The state pins of the target node when in the source node's context

    // Bezier control points for the edge. 2 f32s for each point, arbitrary number of points.
    // If empty, the edge is a straight line.
    "from-bezier-control", // Control points for the bezier curve between the two nodes. 
    "to-bezier-control", // Control points for the bezier curve between the two nodes.
];
