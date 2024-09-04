use std::time::SystemTime;

use agdb::{DbElement, DbError, DbId, DbKeyValue, DbUserValue, DbValue, QueryId};

use crate::nodetype::{NodePhysicality, TypeName};

use super::{attribute::Attribute, node_path::NodePath, SysTime};

/// The universal node type. 
/// Nodes loaded for users of this crate should be in this type. 
/// 
/// Bevy_fs_graph destructures this type into components on an entity
/// to be then later used by bevy_overlay_graph. 
/// 
/// How exactly the other direction, the saving of data, should work, is, 
/// as of writing this, undetermined. Likely in most cases Graph's methods will
/// be used directly to make modifictions rather than creating a Node instance.
#[derive(Debug, PartialEq, Clone)]
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
            values.push(attr.into());
        }

        values
    }
}

/// Implementation block for node. 
impl Node {
    pub fn new(path: &NodePath, ntype: TypeName) -> Self {
        let nphys: NodePhysicality = NodePhysicality::Virtual;

        let now = SysTime(SystemTime::now());

        Node {
            db_id: None,
            path: path.clone(),
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

    pub fn name(&self) -> &str {
        &self.path.name()
    }

    /// Get the NodePath of the node. 
    pub fn path(&self) -> &NodePath {
        &self.path
    }

    pub fn ntype_name(&self) -> TypeName {
        self.ntype.clone()
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
            path: NodePath::try_from(path.unwrap().value.clone())?,
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