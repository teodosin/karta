use std::{path::PathBuf, time::SystemTime};

use agdb::{DbElement, DbError, DbId, DbKeyValue, DbUserValue, DbValue, QueryId, UserValue};

use crate::{nodetype::{NodePhysicality, TypeName}, path_ser::{alias_to_buf, buf_to_alias}};

/// The universal node type. 
/// Nodes loaded for users of this crate should be in this type. 
/// 
/// Bevy_fs_graph destructures this type into components on an entity
/// to be then later used by bevy_overlay_graph. 
/// 
/// How exactly the other direction, the saving of data, should work, is, 
/// as of writing this, undetermined. Likely in most cases Graph's methods will
/// be used directly to make modifictions rather than creating a Node instance.
#[derive(Debug)]
pub struct Node {
    /// The id of the node in the database.
    db_id: Option<DbId>,
    /// The path of the node relative to the root of the graph.
    /// The path is stored as a string in the database, but is converted to a PathBuf when
    /// the node is loaded.
    path: NodePath,

    ntype: TypeName,
    nphys: NodePhysicality,
    alive: bool, 

    created_time: SysTime,
    modified_time: SysTime,

    attributes: Vec<Attribute>,
}

impl DbUserValue for Node {
    fn db_id(&self) -> Option<QueryId> {
        match self.db_id {
            Some(id) => Some(id.into()),
            None => None,
        }
    }
    
    fn db_keys() -> Vec<DbValue> {
        let mut keys = Vec::new();
        keys.push(DbValue::from("path"));
        keys.push(DbValue::from("ntype"));
        keys.push(DbValue::from("nphys"));
        keys.push(DbValue::from("alive"));
        keys.push(DbValue::from("created_time"));
        keys.push(DbValue::from("modified_time"));

        // Why on earth does this function not have a self parameter?
        // for attribute in &self.attributes {
        //     keys.push(DbValue::from(attribute.name.clone()));
        // }

        keys
    }

    fn from_db_element(element: &DbElement) -> Result<Self, DbError> {
        let elem = element.clone();
        let node = Node::try_from(elem);
        node
    }

    fn to_db_values(&self) -> Vec<DbKeyValue> {
        let mut values = Vec::new();
        values.push(DbKeyValue::from(("path", self.path.clone())));
        values.push(DbKeyValue::from(("ntype", self.ntype.clone())));
        values.push(DbKeyValue::from(("nphys", self.nphys.clone())));
        values.push(DbKeyValue::from(("alive", self.alive)));
        values.push(DbKeyValue::from(("created_time", self.created_time.clone())));
        values.push(DbKeyValue::from(("modified_time", self.modified_time.clone())));

        for attr in &self.attributes {
            values.push(attr.clone().into());
        }

        values
    }
}

/// Implementation block for node. 
impl Node {
    pub fn new(path: NodePath, ntype: TypeName) -> Self {
        let nphys: NodePhysicality;

        match path.0.exists() {
            true => nphys = NodePhysicality::Physical,
            false => nphys = NodePhysicality::Virtual,
            _ => nphys = NodePhysicality::Virtual,
        }

        let now = SysTime(SystemTime::now());

        Node {
            db_id: None,
            path,
            ntype,
            nphys,
            alive: true,
            created_time: now.clone(),
            modified_time: now,

            attributes: Vec::new(),
        }
    }

    /// Perhaps it would be better to update this through Graph? Opportunity for bulk 
    /// insertion?
    pub fn update_modified_time(&mut self) {
        self.modified_time = SysTime(SystemTime::now());
    }

    /// Insert a vector of attibutes into the node. Not for library use. 
    /// Though perhaps not even this crate needs this function.
    pub(crate) fn insert_attributes(&mut self, attributes: Vec<Attribute>) {
        unimplemented!();
    }
    // Getters
    
    pub fn id(&self) -> Option<DbId> {
        self.db_id
    }

    /// Get the NodePath of the node. 
    pub fn path(&self) -> &NodePath {
        &self.path
    }

    pub fn ntype(&self) -> TypeName {
        todo!();
    }

    pub fn nphys(&self) -> NodePhysicality {
        self.nphys.clone()
    }

    pub fn created_time(&self) -> &SysTime {
        &self.created_time
    }

    pub fn modified_time(&self) -> &SysTime {
        &self.modified_time
    }

    pub fn attributes(&self) -> &Vec<Attribute> {
        &self.attributes
    }
}

impl TryFrom<DbElement> for Node {
    type Error = DbError;

    fn try_from(value: DbElement) -> Result<Self, Self::Error> {
        let fixed: [&str; 6] = ["path", "ntype", "nphys", "alive", "created_time", "modified_time"];
        let rest = value.values.iter().filter(|v|!fixed.contains(&v.key.string().unwrap().as_str())).collect::<Vec<_>>();

        let db_id = value.id;
        let path = value.values.iter().find(|v| v.key == "path".into());
        let ntype = value.values.iter().find(|v| v.key == "ntype".into());
        let nphys = value.values.iter().find(|v| v.key == "nphys".into());
        let alive = value.values.iter().find(|v| v.key == "alive".into());
        let created_time = value.values.iter().find(|v| v.key == "created_time".into());
        let modified_time = value.values.iter().find(|v| v.key == "modified_time".into());

        let attrs: Vec<Attribute> = rest.iter().map(|v| {
            Attribute {
                name: v.key.to_string(),
                value: v.value.to_f64().unwrap().to_f64() as f32,
            }
        }).collect();

        let node = Node {
            db_id: Some(db_id),
            path: NodePath(PathBuf::from(path.unwrap().value.to_string())),
            ntype: TypeName::try_from(ntype.unwrap().value.clone())?,
            nphys: NodePhysicality::try_from(nphys.unwrap().value.clone())?,
            alive: alive.unwrap().value.to_bool().unwrap(),
            created_time: SysTime::try_from(created_time.unwrap().value.clone())?,
            modified_time: SysTime::try_from(modified_time.unwrap().value.clone())?,
            attributes: attrs,
        };

        Ok(node)
    }
}

/// Newtype wrapper for the node path. 
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct NodePath(PathBuf);

impl NodePath {
    /// Create a new NodePath from a pathbuf. 
    /// Supplying an empty pathbuf will create a NodePath to the root. 
    pub fn new(path: PathBuf) -> Self {
        NodePath(path)
    }

    /// Get the path as a pathbuf, excluding the root. 
    pub fn buf(&self) -> &PathBuf {
        &self.0
    }

    /// Get the path as a string, excluding the root path but prefixed with root/.
    pub fn alias(&self) -> String {
        let str: String = self.0.to_str().unwrap().into();

        let mut alias;
        
        // Add root/ prefix to path if not empty. If empty, just root
        if str.len() > 0 {
            alias = format!("root/{}", str);
        } else {
            alias = String::from("root");
        }
    
        alias
    }

    // Turn alias (root/path) into NodePath
    pub fn from_alias(alias: &str) -> Self {
        let buf = PathBuf::from(alias);

        // Remove root/ prefix from path
        let newbuf =  match buf.strip_prefix("root/") {
            Ok(buf) => PathBuf::from(buf),
            Err(_) => buf,
        };
    
        NodePath(newbuf)
    }

    /// Get the full path, including the root. Root path must be provided. 
    pub fn full(&self, root_path: &PathBuf) -> PathBuf {
        let full_path = root_path.clone();
        full_path.join(self.0.clone())
    }

    pub fn parent(&self) -> Option<NodePath> {
        let parent = self.0.parent();
        match parent {
            Some(p) => Some(NodePath(p.to_path_buf())),
            None => None,
        }
    }
}

impl From<String> for NodePath {
    fn from(path: String) -> Self {
        NodePath(PathBuf::from(path))
    }
}

impl From<&str> for NodePath {
    fn from(path: &str) -> Self {
        NodePath(PathBuf::from(path))
    }
}

impl TryFrom<DbValue> for NodePath {
    type Error = DbError;

    fn try_from(value: DbValue) -> Result<Self, Self::Error> {


        Ok(NodePath(alias_to_buf(&value.to_string())))
    }
}

impl From<NodePath> for DbValue {
    fn from(path: NodePath) -> Self {
        buf_to_alias(&path.0).into()
    }
}

#[derive(Debug, Clone)]
pub struct SysTime(SystemTime);

impl From<SysTime> for DbValue {
    fn from(time: SysTime) -> Self {
        time.0.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs().into()
    }
}

impl TryFrom<DbValue> for SysTime {
    type Error = DbError;

    fn try_from(value: DbValue) -> Result<Self, Self::Error> {
        Ok(SysTime(SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(value.to_u64().unwrap())))
    }
}

pub struct Edge {
    pub db_id: Option<DbId>,
    pub source: NodePath,
    pub target: NodePath,
    attributes: Vec<Attribute>,
}


#[derive(Clone, Debug)]
pub struct Attribute {
    pub name: String,
    pub value: f32,
}

impl Into<Vec<DbKeyValue>> for Attribute {
    fn into(self) -> Vec<DbKeyValue> {
        vec![
            DbKeyValue::from((self.name, self.value)),
        ]
    }
}

impl Into<DbKeyValue> for Attribute {
    fn into(self) -> DbKeyValue {
        DbKeyValue::from((self.name, self.value))
    }
}

impl Into<Attribute> for DbKeyValue {
    fn into(self) -> Attribute {
        Attribute {
            name: self.key.to_string(),
            value: self.value.to_f64().unwrap().to_f64() as f32,
        }
    }
}


/// A list of reserved node attribute names that cannot be set by the user directly.
pub const RESERVED_NODE_ATTRS: [&str; 12] = [
    "path", // The full path of the node, name included. Implemented as an alias, but still reserved.
    "name", // The name of the node, without the path. Maybe allows for different characters?

    "ntype", // The type of the node
    "nphys", // The physicality of the node

    "created_time", // The time when the node was created.
    "modified_time", // The time when the node was last modified.

    "preview", // Connects a file to a preview file, or stores it in this attribute in base64 for example. 

    "scale", // The absolute scaling of the node, in case it is needed. Vec of 2 f32s
    "rotation", // The absolute rotation of the node, in case it is needed. 
    "color", // The absolute color of the node. Vec of 4 f32s
    "pins", // The absolute state pins of the node. 

    // Reserved names with an underscore at the end are prefixes. 
    // All attributes with a name that starts with one of these prefixes are reserved.
    "param_", // Reserved prefix for any attribute that is a parameter, for operators for example. 
];

/// A list of reserved edge attribute names that cannot be set by the user directly.
/// Note that they are optional, so default behavior is when they are not set.
pub const RESERVED_EDGE_ATTRS: [&str; 22] = [
    "contains", // Physical parent_child relationship

    "text", // Text that is displayed on the edge, additional description

    "created_time", // The time the edge was created.
    "modified_time", // The time the edge was last modified.

    // Alternatively, transition animations could be stored in their own nodes. Might make it more 
    // explicit and ergonomic to edit them, and more easily support multiple animations. 
    // Or just have a vector of paths in the attributes below. 
    "from_transition", // Path to an animation file for when the edge is traversed in play mode. 
    "to_transition", // Path to an animation file for when the edge is traversed in play mode.

    "from_preload", // Preload settings for source node when in the target's context & play mode
    "to_preload", // Preload settings for the target node when in source node's context & play mode

    "from_output", // ID of an output socket in source node. Must be validated.
    "to_input", // ID of an input socket in target node. Must be validated. 

    // The following attributes are all Vecs of 2 f32s. 
    "from_position", // Relative position of source node to the target node
    "to_position", // Relative position of the target node to source node
    "from_scale", // Relative scale of source node to the target node
    "to_scale", // Relative scale of the target node to source node
    "from_rotation", // Relative rotation of source node to the target node
    "to_rotation", // Relative rotation of the target node to source node

    // The following attributes are all Vecs of 4 f32s. Or single hex values?
    "from_color", // Color of the source node when in the target's context
    "to_color", // Color of the target node when in the source node's context

    // The state pins of the node. 
    "from_pins", // The state pins of the source node when in the target's context
    "to_pins", // The state pins of the target node when in the source node's context

    // Bezier control points for the edge. 2 f32s for each point, arbitrary number of points.
    // If empty, the edge is a straight line.
    "from_bezier_control", // Control points for the bezier curve between the two nodes. 
    "to_bezier_control", // Control points for the bezier curve between the two nodes.

    // Karta needs a way to track whether a string of edges belongs to the same "sequence", indicating
    // that there is a preferred order to them. Use cases are for compiling 
    // "sequence"
];
