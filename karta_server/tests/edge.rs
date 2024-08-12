#![allow(warnings)]


use fs_graph::{elements::NodePath, graph::Graph};
use std::path::PathBuf;

mod utils;
use utils::*;

#[test]
fn create_new_edge() {
    let func_name = "create_new_edge";
    let mut graph = setup_graph(func_name);

    // Create two nodes
    let path1 = NodePath::from("node1");
    let path2 = NodePath::from("node2");
    
    let node1 = graph.create_node_by_path(path1.clone(), None).unwrap();
    let node2 = graph.create_node_by_path(path2.clone(), None).unwrap();

    // Create an edge between the nodes
    let edge = graph.create_edge(&path1, &path2);

    assert!(edge.is_ok(), "Failed to create edge");

    // Verify the edge exists
    // let edge_exists = graph.edge_exists(node1.id, node2.id);
    // assert!(edge_exists, "Edge does not exist after creation");

    cleanup_graph(func_name);
}


// Test creating an Edge with attributes
// Test converting Attribute to DbKeyValue
// Test creating Edge with reserved attribute names
// Test inserting a new edge
// Test reconnecting an edge
// Test deleting an edge (non-parent edge)
// Test deleting a parent edge (should fail or be ignored)
// Test inserting edge attributes (normal and reserved)
// Test deleting edge attributes (normal and reserved)
// Test creating a parent-child relationship between nodes
