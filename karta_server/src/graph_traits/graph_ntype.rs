use std::error::Error;

use crate::nodetype::TypeName;

pub(crate) trait GraphNtype {
    // -------------------------------------------------------------------
    // Nodetypes

    fn get_node_types(&self) -> Result<Vec<TypeName>, Box<dyn Error>>;

    fn create_nodetype(&mut self, nodetype: TypeName) -> Result<TypeName, Box<dyn Error>>;

    fn instance_nodetype(&self);
}

mod tests {
    #![allow(warnings)]

    use std::path::PathBuf;

    use crate::utils::{cleanup_graph, setup_graph};

    #[test]
    fn new_node_has_type() {
        let func_name = "new_node_has_type";
        let mut graph = setup_graph(func_name);

        todo!();

        cleanup_graph(func_name);
    }
}
