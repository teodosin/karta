use fs_graph::Graph;
mod core;
use core::*;
use std::path::PathBuf;

/// Test inserting a new node
#[test]
fn create_new_node(){
    let func_name = "create_new_node";
    let mut graph = setup_graph(func_name);

    let path = PathBuf::from("test");

    // let node = graph.insert_node_by_path(path)

    cleanup_graph(&func_name);
}

///
#[test]
fn creating_deep_path_creates_intermediate_nodes() {
    let func_name = "creating_deep_path_creates_intermediate_nodes";
    let mut graph = setup_graph(func_name);

    let path = PathBuf::from("one/two/three");

    let mut node = graph.insert_node_by_path(path, None);

    cleanup_graph(&func_name);
}

/// Test creating a Node with different NodeTypes
/// Test inserting an existing node (should fail or update)
/// Test opening a node that exists
/// Test opening a node that doesn't exist
/// Test deleting a node
/// Test reparenting a physical node
/// Test reparenting a virtual node
/// Test inserting node attributes (normal and reserved)
/// Test deleting node attributes (normal and reserved)
/// Test merging two nodes
/// Test creating a node with a long path name
/// Test inserting path as alias for a node
/// Test node operations with very deep directory structures
/// Test node operations with many sibling directories/files
/// Test converting NodePath to and from DbValue
/// Test converting NodePhysicality to and from DbValue
/// Test converting NodeType to and from DbValue
#[test]
fn todo_tests() {
    assert_eq!(2 + 2, 4);
}