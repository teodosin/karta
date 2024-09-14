use std::time::SystemTime;

use agdb::{DbElement, DbError, DbId, DbKeyValue, DbUserValue, DbValue, QueryId};

use super::{attribute::Attribute, node_path::NodePath, SysTime};

#[derive(Clone, Debug)]
pub struct Edge {
    db_id: Option<DbId>,
    source: NodePath,
    target: NodePath,
    contains: bool,
    attributes: Vec<Attribute>,
    created_time: SysTime,
    modified_time: SysTime,
}

impl Edge {
    pub fn new(source: &NodePath, target: &NodePath) -> Self {
        let now = SysTime(SystemTime::now());
        Self {
            db_id: None,
            source: source.clone(),
            target: target.clone(),
            contains: false,
            attributes: Vec::new(),
            created_time: now.clone(),
            modified_time: now,
        }
    }

    pub fn new_cont(source: &NodePath, target: &NodePath) -> Self {
        let attrs: Vec<Attribute> = vec![
            Attribute::new_contains()
        ];
        let now = SysTime(SystemTime::now());
        Self {
            db_id: None,
            source: source.clone(),
            target: target.clone(),
            contains: true,
            attributes: attrs,
            created_time: now.clone(),
            modified_time: now,
        }
    }

    pub fn db_id(&self) -> Option<DbId> {
        self.db_id
    }

    pub fn source(&self) -> &NodePath {
        &self.source
    }

    pub fn target(&self) -> &NodePath {
        &self.target
    }

    pub fn contains(&self) -> bool {
        self.contains
    }

    pub fn attributes(&self) -> &Vec<Attribute> {
        &self.attributes
    }
} 

impl DbUserValue for Edge {
    fn db_id(&self) -> Option<QueryId> {
        match self.db_id {
            Some(id) => Some(id.into()),
            None => None,
        }
    }

    fn db_keys() -> Vec<DbValue> {
        let mut keys = Vec::new();
        keys.push(DbValue::from("source"));
        keys.push(DbValue::from("target"));
        keys.push(DbValue::from("created_time"));
        keys.push(DbValue::from("modified_time"));

        keys
    }

    fn from_db_element(element: &DbElement) -> Result<Self, DbError> {
        let elem = element.clone();
        let edge = Edge::try_from(elem);
        edge
    }

    fn to_db_values(&self) -> Vec<DbKeyValue> {
        let mut values = Vec::new();
        values.push(DbKeyValue::from(("source", self.source.clone())));
        values.push(DbKeyValue::from(("target", self.target.clone())));
        values.push(DbKeyValue::from(("created_time", self.created_time.clone())));
        values.push(DbKeyValue::from(("modified_time", self.modified_time.clone())));

        for attr in &self.attributes {
            values.push(attr.into());
        }

        values
    }
}

impl TryFrom<DbElement> for Edge {
    type Error = DbError;
    
    fn try_from(value: DbElement) -> Result<Self, Self::Error> {
        let fixed: [&str; 2] = ["source", "target"];
        let rest = value.values.iter().filter(|v|!fixed.contains(&v.key.string().unwrap().as_str())).collect::<Vec<_>>();

        let db_id = value.id;
        let source = value.values.iter().find(|v| v.key == "source".into());
        let target = value.values.iter().find(|v| v.key == "target".into());
        let contains = value.values.iter().find(|v| v.key == "contains".into());
        let created_time = value.values.iter().find(|v| v.key == "created_time".into());
        let modified_time = value.values.iter().find(|v| v.key == "modified_time".into());

        println!("source: {:?}", source);
        println!("target: {:?}", target);

        if source.is_none() || target.is_none() {
            return Err(DbError::from("Edge must have source and target"));
        }

        let attrs: Vec<Attribute> = rest.iter().map(|attr| {
            Attribute {
                name: attr.key.to_string(),
                value: attr.value.to_f64().unwrap().to_f64() as f32,
            }
        }).collect();

        let edge = Edge {
            db_id: Some(db_id),
            source: NodePath::try_from(source.unwrap().value.clone())?,
            target: NodePath::try_from(target.unwrap().value.clone())?,
            contains: contains.is_some(),
            attributes: attrs,
            created_time: SysTime::try_from(created_time.unwrap().value.clone())?,
            modified_time: SysTime::try_from(modified_time.unwrap().value.clone())?,
        };

        Ok(edge)
    }
}

