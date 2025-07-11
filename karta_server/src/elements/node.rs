use std::{str::FromStr, time::SystemTime};

use agdb::{DbElement, DbError, DbId, DbKeyValue, DbUserValue, DbValue, QueryId};
use uuid::Uuid;

use super::{attribute::Attribute, node_path::NodePath, nodetype::NodeTypeId, SysTime};

pub const ROOT_UUID: Uuid = Uuid::from_u128(0);

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

    attributes: Vec<Attribute>,
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

        values.push(DbKeyValue::from(("uuid", self.uuid.to_string())));

        values.push(DbKeyValue::from((
            "created_time",
            self.created_time.clone(),
        )));
        values.push(DbKeyValue::from((
            "modified_time",
            self.modified_time.clone(),
        )));

        values.push(DbKeyValue::from(("path", self.path.clone())));
        values.push(DbKeyValue::from(("name", self.name.clone())));
        values.push(DbKeyValue::from(("ntype", self.ntype.clone())));
        values.push(DbKeyValue::from(("alive", self.alive)));

        for attr in &self.attributes {
            values.push(attr.into());
        }

        values
    }
}

/// Implementation block for node.
impl DataNode {
    pub fn new(path: &NodePath, ntype: NodeTypeId) -> Self {
        // Uuid generation is based on the path and creation time.
        let now = SysTime(SystemTime::now());
        let mut combined: String = path.alias();
        combined.push_str(&now.0.elapsed().unwrap().as_millis().to_string());

        // Hash the combined string
        let hash = blake3::hash(combined.as_bytes());
        let mut uuid = Uuid::new_v5(&Uuid::NAMESPACE_URL, hash.as_bytes());

        // Root node has a special uuid
        if *path == NodePath::root() {
            uuid = ROOT_UUID;
        }

        DataNode {
            db_id: None,
            // Note: The Uuid gets set when the node is inserted into the database.
            uuid,
            created_time: now.clone(),
            modified_time: now,

            path: path.clone(),
            name: path.name(),
            ntype,
            alive: true,

            attributes: Vec::new(),
        }
    }

    /// Perhaps it would be better to update this through Graph? Opportunity for bulk
    /// insertion?
    pub fn update_modified_time(&mut self) {
        self.modified_time = SysTime(SystemTime::now());
    }

    pub fn id(&self) -> Option<DbId> {
        self.db_id
    }

    pub fn uuid(&self) -> Uuid {
        self.uuid.clone()
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }
    pub fn set_name(&mut self, name: &str) {
        self.name = name.to_owned();
    }

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

    pub fn attributes(&self) -> Vec<Attribute> {
        self.attributes.clone()
    }

    pub fn set_attributes(&mut self, attributes: Vec<Attribute>) {
        self.attributes = attributes;
    }
}

impl TryFrom<DbElement> for DataNode {
    type Error = DbError;

    fn try_from(value: DbElement) -> Result<Self, Self::Error> {
        let mut uuid = None;
        let mut created_time = None;
        let mut modified_time = None;
        let mut path = None;
        let mut name = None;
        let mut ntype = None;
        let mut alive = None;
        let mut attributes = Vec::new();

        for kv in value.values {
            if let Ok(key) = kv.key.string() {
                match key.as_str() {
                    "uuid" => uuid = Uuid::from_str(&kv.value.to_string()).ok(),
                    "created_time" => created_time = SysTime::try_from(kv.value).ok(),
                    "modified_time" => modified_time = SysTime::try_from(kv.value).ok(),
                    "path" => path = NodePath::try_from(kv.value).ok(),
                    "name" => name = kv.value.string().ok().map(String::from),
                    "ntype" => ntype = NodeTypeId::try_from(kv.value).ok(),
                    "alive" => alive = kv.value.to_bool().ok(),
                    _ => {
                        if let Ok(attr) = Attribute::try_from(&kv) {
                            attributes.push(attr);
                        }
                    }
                }
            }
        }

        if let (Some(uuid), Some(created_time), Some(modified_time), Some(path), Some(name), Some(ntype), Some(alive)) =
            (uuid, created_time, modified_time, path, name, ntype, alive)
        {
            Ok(DataNode {
                db_id: Some(value.id),
                uuid,
                created_time,
                modified_time,
                path,
                name,
                ntype,
                alive,
                attributes,
            })
        } else {
            Err(DbError::from("Missing required fields for DataNode"))
        }
    }
}
