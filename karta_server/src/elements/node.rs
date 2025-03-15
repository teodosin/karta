use std::{str::FromStr, time::SystemTime};

use agdb::{DbElement, DbError, DbId, DbKeyValue, DbUserValue, DbValue, QueryId};
use uuid::Uuid;

use super::{attribute::Attribute, node_path::NodePath, nodetype::NodeTypeId, SysTime};

/// The universal node type. 
/// Nodes loaded for users of this crate should be in this type. 
/// 
/// bevy_karta_client destructures this type into components on an entity
/// to be then later used by bevy_karta_ui. 
/// 
/// How exactly the other direction, the saving of data, should work, is, 
/// as of writing this, undetermined. Likely in most cases Graph's methods will
/// be used directly to make modifictions rather than creating a Node instance.
#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
pub struct DataNode {
    /// The id of the node in the database.
    db_id: Option<DbId>,
    uuid: Uuid,
    created_time: SysTime,
    modified_time: SysTime,
    /// The path of the node relative to the root of the graph.
    /// The path is stored as a string in the database, but is converted to a PathBuf when
    /// the node is loaded.
    path: NodePath,
    name: String,

    ntype: NodeTypeId,
    alive: bool, 


    // attributes: Vec<Attribute>,
}

impl DbUserValue for DataNode {
    type ValueType = Self;

    fn db_id(&self) -> Option<QueryId> {
        match self.db_id {
            Some(id) => Some(id.into()),
            None => None,
        }
    }
    
    fn db_keys() -> Vec<DbValue> {
        let mut keys = Vec::new();
        keys.push(DbValue::from("uuid"));
        keys.push(DbValue::from("created_time"));
        keys.push(DbValue::from("modified_time"));

        keys.push(DbValue::from("path"));
        keys.push(DbValue::from("name"));
        keys.push(DbValue::from("ntype"));
        keys.push(DbValue::from("alive"));

        // Why on earth does this function not have a self parameter?
        // for attribute in &self.attributes {
        //     keys.push(DbValue::from(attribute.name.clone()));
        // }

        keys
    }

    fn from_db_element(element: &DbElement) -> Result<Self, DbError> {
        let elem = element.clone();
        let node = DataNode::try_from(elem);
        node
    }

    fn to_db_values(&self) -> Vec<DbKeyValue> {
        let mut values = Vec::new();
        values.push(DbKeyValue::from(("uuid", self.uuid.clone().to_string())));
        values.push(DbKeyValue::from(("created_time", self.created_time.clone())));
        values.push(DbKeyValue::from(("modified_time", self.modified_time.clone())));

        values.push(DbKeyValue::from(("path", self.path.clone())));
        values.push(DbKeyValue::from(("name", self.name.clone())));
        values.push(DbKeyValue::from(("ntype", self.ntype.clone())));
        values.push(DbKeyValue::from(("alive", self.alive)));

        // for attr in &self.attributes {
        //     values.push(attr.into());
        // }

        values
    }
}

/// Implementation block for node. 
impl DataNode {
    pub fn new(path: &NodePath, ntype: NodeTypeId) -> Self {

        let now = SysTime(SystemTime::now());

        DataNode {
            db_id: None,
            // TODO: Make the Uuid not depend on the path but somehow just on the node itself.
            // The Uuid should be stable regardless of the location of the node,
            // so that ViewNodes in contexts don't have to get updated when the node is moved.
            uuid: Uuid::new_v5(&Uuid::NAMESPACE_URL, &path.alias().into_bytes()),
            created_time: now.clone(),
            modified_time: now,

            path: path.clone(),
            name: path.name(),
            ntype,
            alive: true,

            // attributes: Vec::new(),
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

    pub fn name(&self) -> String {
        self.path.name()
    }

    /// Get the NodePath of the node. 
    pub fn path(&self) -> NodePath {
        self.path.clone()
    }

    pub fn ntype_name(&self) -> NodeTypeId {
        self.ntype.clone()
    }

    pub fn created_time(&self) -> SysTime {
        self.created_time.clone()
    }

    pub fn modified_time(&self) -> SysTime {
        self.modified_time.clone()
    }

    // pub fn attributes(&self) -> Vec<Attribute> {
    //     self.attributes.clone()
    // }
}

impl TryFrom<DbElement> for DataNode {
    type Error = DbError;

    fn try_from(value: DbElement) -> Result<Self, Self::Error> {
        // let fixed: [&str; 6] = ["path", "ntype", "nphys", "alive", "created_time", "modified_time"];
        let fixed = super::attribute::RESERVED_NODE_ATTRS;
        let rest = value.values.iter().filter(|v|!fixed.contains(&v.key.string().unwrap().as_str())).collect::<Vec<_>>();

        let db_id = value.id;
        let uuid = value.values.iter().find(|v| v.key == "uuid".into());
        let created_time = value.values.iter().find(|v| v.key == "created_time".into());
        let modified_time = value.values.iter().find(|v| v.key == "modified_time".into());
        let path = value.values.iter().find(|v| v.key == "path".into());
        let name = value.values.iter().find(|v| v.key == "name".into());
        let ntype = value.values.iter().find(|v| v.key == "ntype".into());
        let nphys = value.values.iter().find(|v| v.key == "nphys".into());
        let alive = value.values.iter().find(|v| v.key == "alive".into());

        // let attrs: Vec<Attribute> = rest.iter().map(|v| {
        //     Attribute::try_from(*v).unwrap()
        // }).collect();

        let node = DataNode {
            db_id: Some(db_id),
            uuid: Uuid::from_str(uuid.unwrap().value.clone().to_string().as_str()).unwrap(),
            created_time: SysTime::try_from(created_time.unwrap().value.clone())?,
            modified_time: SysTime::try_from(modified_time.unwrap().value.clone())?,

            path: NodePath::try_from(path.unwrap().value.clone())?,
            name: name.unwrap().value.clone().to_string(),
            ntype: NodeTypeId::try_from(ntype.unwrap().value.clone())?,
            alive: alive.unwrap().value.to_bool().unwrap(),
            // attributes: attrs,
        };

        Ok(node)
    }
}