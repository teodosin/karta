use std::{error::Error, path::PathBuf};

use agdb::QueryBuilder;

use crate::{graph_traits::graph_ntype::GraphNtype, elements::nodetype::NodeType};

use super::{GraphAgdb, StoragePath};

impl GraphNtype for GraphAgdb {
    fn get_node_types(&self) -> Result<Vec<NodeType>, Box<dyn Error>> {
        todo!()
    }

    fn create_nodetype(&mut self, nodetype: NodeType) -> Result<NodeType, Box<dyn Error>> {
        todo!()
    }

    fn instance_nodetype(&self) {
        todo!()
    }
}