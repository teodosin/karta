use std::error::Error;

use crate::nodetype::TypeName;

pub(crate) trait GraphNtype {
    // -------------------------------------------------------------------
    // Nodetypes 

    fn get_node_types(&self) -> Result<Vec<TypeName>, Box<dyn Error>>;

    fn create_nodetype(&mut self, nodetype: TypeName) -> Result<TypeName, Box<dyn Error>>;

    fn instance_nodetype(&self);

}