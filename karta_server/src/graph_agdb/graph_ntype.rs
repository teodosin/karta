use std::{error::Error, path::PathBuf};

use agdb::QueryBuilder;

use crate::{graph_traits::graph_ntype::GraphNtype, elements::nodetype::TypeName};

use super::{GraphAgdb, StoragePath};

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