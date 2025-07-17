use std::{str::FromStr, time::SystemTime};

use agdb::{DbElement, DbError, DbId, DbKeyValue, DbUserValue, DbValue, QueryId};
use uuid::Uuid;

use crate::elements::attribute::RESERVED_EDGE_ATTRS;

use super::{attribute::Attribute, node_path::NodePath, SysTime};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Edge {
    uuid: Uuid,
    db_id: Option<DbId>,
    source: Uuid,
    target: Uuid,
    contains: bool,
    created_time: SysTime,
    modified_time: SysTime,
    attributes: Vec<Attribute>,
}

impl Edge {
    pub fn new(source_uuid: Uuid, target_uuid: Uuid) -> Self {
        let now = SysTime(SystemTime::now());
        let name_to_hash = format!(
            "{}:{}:{}:{}",
            source_uuid,
            target_uuid,
            now.0.duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis(),
            "edge"
        );
        let new_uuid = Uuid::new_v5(&Uuid::NAMESPACE_URL, name_to_hash.as_bytes());
        Self {
            uuid: new_uuid,
            db_id: None,
            source: source_uuid,
            target: target_uuid,
            contains: false,
            attributes: Vec::new(),
            created_time: now.clone(),
            modified_time: now,
        }
    }

    pub fn new_cont(source_uuid: Uuid, target_uuid: Uuid) -> Self {
        let attrs: Vec<Attribute> = vec![Attribute::new_contains()];
        let now = SysTime(SystemTime::now());
        let name_to_hash = format!(
            "{}:{}:{}:{}",
            source_uuid,
            target_uuid,
            now.0.duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis(),
            "edge_cont"
        );
        let new_uuid = Uuid::new_v5(&Uuid::NAMESPACE_URL, name_to_hash.as_bytes());
        Self {
            uuid: new_uuid,
            db_id: None,
            source: source_uuid,
            target: target_uuid,
            contains: true,
            attributes: attrs,
            created_time: now.clone(),
            modified_time: now,
        }
    }

    pub fn uuid(&self) -> Uuid {
        self.uuid
    }

    pub fn db_id(&self) -> Option<DbId> {
        self.db_id
    }

    pub fn source(&self) -> &Uuid {
        &self.source
    }

    pub fn target(&self) -> &Uuid {
        &self.target
    }

    pub fn contains(&self) -> bool {
        self.contains
    }

    pub fn created_time(&self) -> SysTime {
        self.created_time.clone()
    }

    pub fn modified_time(&self) -> SysTime {
        self.modified_time.clone()
    }

    pub fn attributes(&self) -> &Vec<Attribute> {
        &self.attributes
    }

    pub fn set_attributes(&mut self, attributes: Vec<Attribute>) {
        self.attributes = attributes;
    }
}

impl DbUserValue for Edge {
    type ValueType = Self;
    
    fn db_id(&self) -> Option<QueryId> {
        match self.db_id {
            Some(id) => Some(id.into()),
            None => None,
        }
    }

    fn db_keys() -> Vec<DbValue> {
        let mut keys = Vec::new();
        keys.push(DbValue::from("uuid")); // Added uuid
        keys.push(DbValue::from("source"));
        keys.push(DbValue::from("target"));
        keys.push(DbValue::from("created_time"));
        keys.push(DbValue::from("modified_time"));
        // Note: 'contains' and other attributes are handled by iterating 'rest' in TryFrom
        keys
    }

    fn from_db_element(element: &DbElement) -> Result<Self, DbError> {
        let elem = element.clone();
        let edge = Edge::try_from(elem);
        edge
    }

    fn to_db_values(&self) -> Vec<DbKeyValue> {
        let mut values = Vec::new();
        values.push(DbKeyValue::from(("uuid", self.uuid.to_string())));
        values.push(DbKeyValue::from(("source", self.source.to_string())));
        values.push(DbKeyValue::from(("target", self.target.to_string())));
        values.push(DbKeyValue::from(("created_time", self.created_time.clone())));
        values.push(DbKeyValue::from(("modified_time", self.modified_time.clone())));
        // Note: 'contains' is implicitly handled by attributes if it's stored as one.
        // The TryFrom logic reconstructs 'contains' based on attribute presence.
        // If 'contains' should be a direct DbValue, it needs to be added here and in db_keys.
        // Based on current TryFrom, 'contains' is derived, so not adding it as a direct DbValue here.

        for attr in &self.attributes {
            values.push(attr.into());
        }

        values
    }
}

impl TryFrom<DbElement> for Edge {
    type Error = DbError;
    
    fn try_from(value: DbElement) -> Result<Self, Self::Error> {
        // let fixed: [&str; 5] = ["source", "target", "contains", "created_time", "modified_time"];
        let fixed = RESERVED_EDGE_ATTRS;
        let rest = value.values.iter().filter(|v|!fixed.contains(&v.key.string().unwrap().as_str())).collect::<Vec<_>>();

        let db_id = value.id;
        let uuid = value.values.iter().find(|v| v.key == "uuid".into());
        let source = value.values.iter().find(|v| v.key == "source".into());
        let target = value.values.iter().find(|v| v.key == "target".into());
        let contains = value.values.iter().find(|v| v.key == "contains".into());
        let created_time = value.values.iter().find(|v| v.key == "created_time".into());
        let modified_time = value.values.iter().find(|v| v.key == "modified_time".into());

        // println!("source: {:?}", source);
        // println!("target: {:?}", target);

        if source.is_none() || target.is_none() {
            return Err(DbError::from("Edge must have source and target"));
        }


        let attrs: Vec<Attribute> = rest.iter().map(|attr| {
            // println!("Creating attribute - {:#?}", attr);

            Attribute::try_from(*attr).unwrap()
        }).collect();

        let uuid_val = uuid.ok_or_else(|| DbError::from("Edge DbElement missing 'uuid' value"))?.value.clone();
        let source_val = source.ok_or_else(|| DbError::from("Edge DbElement missing 'source' value"))?.value.clone();
        let target_val = target.ok_or_else(|| DbError::from("Edge DbElement missing 'target' value"))?.value.clone();
        let created_time_val = created_time.ok_or_else(|| DbError::from("Edge DbElement missing 'created_time' value"))?.value.clone();
        let modified_time_val = modified_time.ok_or_else(|| DbError::from("Edge DbElement missing 'modified_time' value"))?.value.clone();

        let uuid = Uuid::from_str(&uuid_val.to_string())
            .map_err(|e| DbError::from(format!("Failed to parse Edge UUID: {}", e)))?;

        let edge = Edge {
            db_id: Some(db_id),
            uuid,
            source: Uuid::from_str(&source_val.to_string())
                .map_err(|e| DbError::from(format!("Failed to parse source UUID: {}", e)))?,
            target: Uuid::from_str(&target_val.to_string())
                .map_err(|e| DbError::from(format!("Failed to parse target UUID: {}", e)))?,
            contains: contains.is_some(), // 'contains' is optional, derived from attribute presence
            created_time: SysTime::try_from(created_time_val)?,
            modified_time: SysTime::try_from(modified_time_val)?,
            attributes: attrs,
        };

        Ok(edge)
    }
}

