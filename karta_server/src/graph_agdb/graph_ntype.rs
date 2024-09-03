use std::{error::Error, path::PathBuf};

use agdb::QueryBuilder;

use crate::{graph_traits::GraphNtype, nodetype::TypeName};

use super::{GraphAgdb, Node, NodePath, StoragePath};

impl GraphNtype for GraphAgdb {
    fn get_node_types(&self) -> Result<Vec<TypeName>, Box<dyn Error>> {
        todo!()
    }

    fn create_nodetype(&mut self, nodetype: TypeName) -> Result<TypeName, Box<dyn Error>> {
        todo!()
    }

    fn instance_nodetype(&self) {
        todo!()
    }
}