use std::{error::Error, path::PathBuf};

use super::{attribute::{Attribute, RelativePosition}, edge::Edge, node::DataNode, node_path::NodePath, nodetype::NodeTypeId};

pub trait GraphNodes {
    // -------------------------------------------------------------------
    // Nodes

    /// Retrieves a particular node's data from the database.
    /// The path is relative to the root of the graph.
    ///
    /// TODO: This takes a mutable reference because of the indexing requirement, which is
    /// awkward and tech debt.
    fn open_node(&self, path: &NodePath) -> Result<DataNode, Box<dyn Error>>;

    // Retrieves the edges of a particular node.
    // fn get_node_edges(&self, path: &NodePath) -> Vec<Edge>;

    /// Opens the connections of a particular node.
    /// Takes in the path to the node relative to the root of the graph.
    ///
    /// TODO: Add filter argument when Filter is implemented.
    /// Note that possibly Filter could have a condition that nodes
    /// would have to be connected to some node, which would just turn
    /// this into a generic "open_nodes" function, or "search_nodes".
    /// Then filters could just be wrappers around agdb's QueryConditions...
    fn open_node_connections(&self, path: &NodePath) -> Vec<(DataNode, Edge)>;

    /// Inserts a Node.
    fn insert_nodes(&mut self, nodes: Vec<DataNode>);
}

// --------------------------------------------------------------------

#[cfg(test)]
mod tests {
    #![allow(warnings)]

    use crate::{
        elements::{
            self, attribute::{Attribute, RESERVED_NODE_ATTRS}, node, node_path::NodePath, nodetype::ARCHETYPES
        }, graph_agdb::GraphAgdb, graph_traits::graph_edge::GraphEdge, prelude::{DataNode, NodeTypeId}, utils::utils::TestContext
    };
    use agdb::QueryBuilder;

    use std::{
        fs::{create_dir, File},
        path::PathBuf,
        vec,
    };

    use crate::graph_traits::{graph_core::GraphCore, graph_node::GraphNodes};

    #[test]
    fn opening_root_node_connections__returns_vector_of_nodes() {
        let func_name = "opening_root_node_connections__returns_vector_of_nodes";
        let mut ctx = TestContext::new(func_name);

        let root_path = NodePath::root();

        let connections = ctx.graph.open_node_connections(&root_path);

        // Archetypes includes the root, but the source node is excluded from the function so
        // so the length should be one less than the number of archetypes
        let archetypes: Vec<&&str> = ARCHETYPES.iter().filter(|ar| **ar != "").collect();

        assert_eq!(
            connections.len(),
            archetypes.len(),
            "Root path should have its archetype connections: {}",
            archetypes.len()
        );

        // Check that the tuples contain matching edges
        for (i, connection) in connections.iter().enumerate() {
            assert!(
                *connection.1.source() == connection.0.path()
                    || *connection.1.target() == connection.0.path(),
                "Edge should be connected to Node"
            )
        }
    }

    #[test]
    fn inserting_node__node_is_inserted() {
        let func_name = "inserting_node__node_is_inserted";
        let mut ctx = TestContext::new(func_name);

        let path = NodePath::new(PathBuf::from("test"));

        let node = DataNode::new(&path, NodeTypeId::virtual_generic());

        ctx.graph.insert_nodes(vec![node]);

        ctx.graph.open_node(&path).expect("Node should exist");
    }

    #[test]
    fn inserting_modified_same_node__updates_node() {
        let func_name = "inserting_modified_same_node__updates_node";
        let mut ctx = TestContext::new(func_name);

        let path = NodePath::new(PathBuf::from("test"));

        let node = DataNode::new(&path, NodeTypeId::virtual_generic());

        ctx.graph.insert_nodes(vec![node.clone()]);

        let root_path = NodePath::user_root();
        let root_connections = ctx.graph.open_node_connections(&root_path);

        let mut mod_node = node.clone();
        mod_node.set_name("testerer");
        
        ctx.graph.insert_nodes(vec![mod_node]);
        let second_connections = ctx.graph.open_node_connections(&root_path);

        assert_eq!(root_connections.len(), second_connections.len());

        // Check the name too
        let mod_node = ctx.graph.open_node(&path).unwrap();
        assert_eq!(mod_node.name(), "testerer");
    }

    #[test]
    fn inserting_long_path__creates_intermediate_nodes() {
        let func_name = "inserting_long_path__creates_intermediate_nodes";
        let mut ctx = TestContext::new(func_name);

        let first = NodePath::new(PathBuf::from("one"));
        let second = NodePath::new(PathBuf::from("one/two"));
        let third = NodePath::new(PathBuf::from("one/two/three"));

        let third_node = DataNode::new(&third, NodeTypeId::virtual_generic());

        ctx.graph.insert_nodes(vec![third_node]);

        let second_node = ctx.graph.open_node(&second);
        assert!(second_node.is_ok());

        let first_node = ctx.graph.open_node(&first);
        assert!(first_node.is_ok());

        println!("{:#?}", ctx.graph.get_all_aliases());
    }
}
