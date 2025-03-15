use std::{error::Error, path::PathBuf};

use crate::elements::nodetype::NodeType;

use super::{attribute::{Attribute, RelativePosition}, edge::Edge, node::Node, node_path::NodePath, nodetype::NodeTypeId};

pub trait GraphNode {
    // -------------------------------------------------------------------
    // Nodes

    /// Retrieves a particular node's data from the database.
    /// The path is relative to the root of the graph.
    ///
    /// TODO: This takes a mutable reference because of the indexing requirement, which is
    /// awkward and tech debt.
    fn open_node(&self, path: &NodePath) -> Result<Node, Box<dyn Error>>;

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
    fn open_node_connections(&self, path: &NodePath) -> Vec<(Node, Edge)>;

    /// Inserts a Node.
    fn insert_nodes(&mut self, node: Node) -> Result<(), Box<dyn Error>>;
}

// --------------------------------------------------------------------

#[cfg(test)]
mod tests {
    #![allow(warnings)]

    use crate::{
        elements::{
            self, attribute::{Attribute, RESERVED_NODE_ATTRS}, node, node_path::NodePath, nodetype::ARCHETYPES
        },
        graph_agdb::GraphAgdb,
        graph_traits::graph_edge::GraphEdge,
        utils::utils::TestContext,
    };
    use agdb::QueryBuilder;

    use std::{
        fs::{create_dir, File},
        path::PathBuf,
        vec,
    };

    use crate::graph_traits::{graph_core::GraphCore, graph_node::GraphNode};

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
}
