#![allow(warnings)]

use std::any::Any;

use agdb::QueryBuilder;
use directories::ProjectDirs;
use fs_graph::graph::{self, Graph};

mod utils;
use utils::*;

#[test]
fn test_new_graph() {
    let func_name = "test_new_graph";

    let name = format!("fs_graph_test_{}", func_name);
    let root = ProjectDirs::from("com", "fs_graph", &name)
        .unwrap()
        .data_dir()
        .to_path_buf();

    println!("Expected full path: {:#?}", root);

    let graph = Graph::new(root.clone().into(), &name);

    println!("Size of graph: {:#?} bytes", graph.db.size());

    assert_eq!(root.exists(), true, "Root directory does not exist");

    // Check that there exists a root node
    let root_node_result = graph.db.exec(&QueryBuilder::select().ids("root").query());

    match root_node_result {
        Ok(root_node) => {
            assert_eq!(root_node.result /* expected value */, 1);
        }
        Err(e) => {
            println!("Failed to execute query: {}", e);
        }
    }

    cleanup_graph(func_name);
}

/// Add a node to the db, then create a new graph with the same name.
/// The new graph should be able to access the node.
#[test]
fn existing_db_in_directory() {
    let func_name = "existing_db_in_directory";
    let mut first = setup_graph(func_name);

    let _ = first
        .db
        .exec_mut(&QueryBuilder::insert().nodes().aliases("testalias").query());

    let second = setup_graph(func_name);

    let root_node_result = second
        .db
        .exec(&QueryBuilder::select().ids("testalias").query());

    match root_node_result {
        Ok(root_node) => {
            assert_eq!(root_node.result /* expected value */, 1);
        }
        Err(e) => {
            println!("Failed to execute query: {}", e);
        }
    }

    assert_eq!(true, true);

    cleanup_graph(func_name);
}

#[test]
fn new_custom_storage_directory() {
    let func_name = "new_custom_storage_directory";
    let name = format!("fs_graph_test_{}", func_name);
    let root = ProjectDirs::from("com", "fs_graph", &name)
        .unwrap()
        .config_dir()
        .to_path_buf();
    let storage = root.join("storage");

    let graph = Graph::new_custom_storage(root.clone().into(), &name, storage.clone());

    assert_eq!(
        storage.exists(),
        true,
        "Storage directory has not been created"
    );

    let root_node_result = graph.db.exec(&QueryBuilder::select().ids("root").query());

    match root_node_result {
        Ok(root_node) => {
            assert_eq!(root_node.result /* expected value */, 1);
        }
        Err(e) => {
            println!("Failed to execute query: {}", e);
        }
    }

    // Clean up the custom storage directory
    std::fs::remove_dir_all(storage).expect("Failed to remove storage directory");
}

#[test]
/// Test whether you can add a long path as an alias to a node and retrieve it. 
fn long_alias_path() {
    use fs_graph::path_ser::{buf_to_alias, alias_to_buf};

    let func_name = "long_alias_path";
    let mut graph = setup_graph(func_name);

    let long_path = "root/this/is/a/long/path/with/many/segments/verylongindeed/evenlonger/wow/are/we/still/here/there/must/be/something/we/can/do/about/all/this/tech/debt";

    let _ = graph
        .db
        .exec_mut(&QueryBuilder::insert().nodes().aliases(long_path).query());

    // Putting this conversion here for extra testing. 
    let buf = alias_to_buf(long_path);
    let long_path = buf_to_alias(&buf);

    let root_node_result = graph
        .db
        .exec(&QueryBuilder::select().ids(long_path).query());

    let success: bool;
    match root_node_result {
        Ok(root_node) => {
            success = root_node.result == 1;
        }
        Err(e) => {
            println!("Failed to execute query: {}", e);
            success = false;
        }
    }
    assert_eq!(success, true);

    cleanup_graph(func_name);
} 

#[test]
fn correct_root_name() {
    let func_name = "correct_root_name";
    let graph = setup_graph(func_name);

    let dirname: String = get_graph_dir_name(func_name);

    let root_name: String = graph.root_name();

    assert_eq!(root_name, dirname);

    cleanup_graph(func_name);
}

#[test]
/// Test whether the db creates an attributes node when the db is first created.
/// Could possibly be moved to attr.rs
fn create_attributes_category(){
    let func_name = "create_attributes_category";
    let graph = setup_graph(func_name);

    let root_node_result = graph.db.exec(&QueryBuilder::select().ids("root").query());

    assert_eq!(true, root_node_result.is_ok());

    let qry = graph.db.exec(&QueryBuilder::select().ids("root/attributes").query());
    
    assert_eq!(true, qry.is_ok());

    if root_node_result.is_ok() && qry.is_ok() {

        // Validate root node
        let root_node = root_node_result.unwrap().ids();
        assert_eq!(root_node.len(), 1);
        let root_id = root_node.first().unwrap();

        // Validate attributes node
        let attributes_node = qry.unwrap().ids();
        assert_eq!(attributes_node.len(), 1);
        let attributes_id = attributes_node.first().unwrap();

        // Find edge, validate
        let query = &QueryBuilder::search().from(*root_id).to(*attributes_id).query();
        let edge = graph.db.exec(query);
        assert_eq!(edge.is_ok(), true);
        
        let edge = edge.unwrap().elements.iter().cloned().filter(|e| e.id.0 < 0).collect::<Vec<_>>();
        println!("Found edge {:#?}", edge);

        assert_eq!(edge.len(), 1);

        // Select edge, because values don't appear in above query. 
        // These two queries could probably be merged.
        let eid = edge.first().unwrap().id.0;
        let edge = graph.db.exec(&QueryBuilder::select()
            .keys()
            .ids(eid)
            .query());
        let edge = edge.unwrap().elements;
        assert_eq!(edge.len(), 1);

        let vals = &edge.first().unwrap().values;
        let mut found = false;

        // The attribute we're looking for. Should be reserved. 
        // In this case, the "contains" attribute. 
        let atr = "contains";
        for val in vals.iter() {
            if val.key == atr.into() {
                found = true;
            }
        }

        assert_eq!(found, true);

    }

    cleanup_graph(&func_name); 
}

// Loading an old db with a new root directory!
// Should this be allowed or prevented? For usability it would be nice if you could just 
// change the root directory to beyond or within the previous one.
//
// This has to be handled carefully though. If each node stores its path relative to the root,
// then the path of the node will be incorrect if the root directory is changed. Every node in the 
// entire db would have to be updated. On the other hand, if the path is stored as an absolute path,
// then moving the root folder would break those. And if each node only stores its own name, then finding 
// the path of a node would be a slower operation. Also it seems like agdb doesn't support having the 
// same aliases for multiple nodes, so only storing the names wouldn't be feasible anyway. 
//
// One has to consider also that a large portion of the db could be rearranged without changing the root
// directory, meaning that there would still be a lot of updates needed. 

// Test for what happens when a db is moved to a different directory, but the root directory is the same.

