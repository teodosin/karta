use agdb::{DbElement, DbError, DbId, DbKeyValue, DbUserValue, DbValue, QueryId};

use super::{node_path::NodePath, attribute::Attribute};

#[derive(Clone, Debug)]
pub struct Edge {
    pub db_id: Option<DbId>,
    pub source: NodePath,
    pub target: NodePath,
    attributes: Vec<Attribute>,
}

impl Edge {
    pub fn new(source: &NodePath, target: &NodePath) -> Self {
        Self {
            db_id: None,
            source: source.clone(),
            target: target.clone(),
            attributes: Vec::new(),
        }
    }

    pub fn new_cont(source: &NodePath, target: &NodePath) -> Self {
        let attrs: Vec<Attribute> = vec![
            Attribute::new_contains()
        ];
        Self {
            db_id: None,
            source: source.clone(),
            target: target.clone(),
            attributes: attrs,
        }
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
            source: NodePath::from(source.unwrap().value.to_string()),
            target: NodePath::from(target.unwrap().value.to_string()),
            attributes: attrs,
        };

        Ok(edge)
    }
}

