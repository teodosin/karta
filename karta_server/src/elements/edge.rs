use agdb::DbId;

use super::{node_path::NodePath, attribute::Attribute};

pub struct Edge {
    pub db_id: Option<DbId>,
    pub source: NodePath,
    pub target: NodePath,
    attributes: Vec<Attribute>,
}

