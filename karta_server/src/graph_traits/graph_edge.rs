use std::error::Error;

use super::{Attribute, Edge, NodePath};

pub(crate) trait GraphEdge {
    // -------------------------------------------------------------------
    // Edges

    fn create_edge(
        &mut self,
        source_path: &NodePath,
        target_path: &NodePath,
    ) -> Result<(), Box<dyn Error>>;

    /// Changes the parent directory of a node. If the node is physical, it will be moved in the file system.
    /// If the node is virtual, the parent will be changed in the db.
    /// Note that due to the implementation, all children of the node will have to be reindexed, recursively.
    fn reparent_node(
        &self,
        node_path: &NodePath,
        new_parent_path: &NodePath,
    ) -> Result<(), Box<dyn Error>>;

    /// Moves an edge and all its attributes to a new source and target. Parent edges can't be reconnected this way,
    /// use the reparent_node function instead.
    fn reconnect_edge(
        &self,
        edge: Edge,
        from: &NodePath,
        to: &NodePath,
    ) -> Result<(), Box<dyn Error>>;

    fn insert_edge(&self, edge: Edge) -> Result<(), Box<dyn Error>>;

    /// Delete an edge from the graph. Edges with the attribute "contains" refer to the parent-child relationship
    /// between nodes and will be ignored. All other attributes will be cleared from them instead.
    fn delete_edge(&self, edge: Edge) -> Result<(), Box<dyn Error>>;

    /// Insert attributes to an edge. Ignore reserved attribute names. Update attributes that already exist.
    fn insert_edge_attr(&self, edge: Edge, attr: Attribute) -> Result<(), Box<dyn Error>>;

    /// Delete attributes from an edge. Ignore reserved attribute names.
    fn delete_edge_attr(&self, edge: Edge, attr: Attribute) -> Result<(), Box<dyn Error>>;
}

// mod tests {
//     #![allow(warnings)]

//     use crate::{
//         elements::NodePath,
//         graph_agdb::GraphAgdb,
//         graph_traits::{graph_edge::GraphEdge, graph_node::GraphNode},
//         utils::{cleanup_graph, setup_graph},
//     };
//     use std::path::PathBuf;

//     #[test]
//     fn create_new_edge() {
//         let func_name = "create_new_edge";
//         let mut graph = setup_graph(func_name);

//         // Create two nodes
//         let path1 = NodePath::from("node1");
//         let path2 = NodePath::from("node2");

//         let node1 = graph.create_node_by_path(path1.clone(), None).unwrap();
//         let node2 = graph.create_node_by_path(path2.clone(), None).unwrap();

//         // Create an edge between the nodes
//         let edge = graph.create_edge(&path1, &path2);

//         assert!(edge.is_ok(), "Failed to create edge");

//         // Verify the edge exists
//         // let edge_exists = graph.edge_exists(node1.id, node2.id);
//         // assert!(edge_exists, "Edge does not exist after creation");

//         cleanup_graph(func_name);
//     }

//     // Test creating an Edge with attributes
//     // Test converting Attribute to DbKeyValue
//     // Test creating Edge with reserved attribute names
//     // Test inserting a new edge
//     // Test reconnecting an edge
//     // Test deleting an edge (non-parent edge)
//     // Test deleting a parent edge (should fail or be ignored)
//     // Test inserting edge attributes (normal and reserved)
//     // Test deleting edge attributes (normal and reserved)
//     // Test creating a parent-child relationship between nodes
// }
