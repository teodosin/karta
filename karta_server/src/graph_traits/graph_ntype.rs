use std::error::Error;

use crate::elements::nodetype::NodeType;

pub(crate) trait GraphNtype {
    // -------------------------------------------------------------------
    // Nodetypes

    fn get_node_types(&self) -> Result<Vec<NodeType>, Box<dyn Error>>;

    fn create_nodetype(&mut self, nodetype: NodeType) -> Result<NodeType, Box<dyn Error>>;

    fn instance_nodetype(&self);
}

mod tests {
    #![allow(warnings)]

    use std::path::PathBuf;

    // #[test]
    // fn new_node_has_type() {
    //     let func_name = "new_node_has_type";

    //     todo!();
    // }
}
