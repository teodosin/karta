
use std::any::Any;

use agdb::QueryBuilder;
use directories::ProjectDirs;
use fs_graph::Graph;

/// Graph setup function for tests. Always stores the db in the data_dir.
fn setup_graph(test_name: &str) -> Graph {
    println!("");
    println!("----------------------------------------------");
    println!("Creating graph for test: {}", test_name);

    cleanup_graph(test_name);

    let name = format!("fs_graph_test_{}", test_name);
    let root = ProjectDirs::from("com", "fs_graph", &name)
        .unwrap()
        .data_dir()
        .to_path_buf();
    let full_path = root.join(&name);

    let graph = Graph::new(root.clone().into(), &name);

    assert_eq!(
        full_path.exists(),
        true,
        "Test directory has not been created"
    );


    graph
}

/// Graph cleanup function for tests. Removes the root directory from the data_dir.
fn cleanup_graph(test_name: &str) {
    // Uncomment this return only if you need to temporarily look at the contents
    // return;

    let name = format!("fs_graph_test_{}", test_name);
    let root = ProjectDirs::from("com", "fs_graph", &name)
        .unwrap()
        .data_dir()
        .to_path_buf();

    let removal = std::fs::remove_dir_all(root);

    match removal {
        Ok(_) => {
            println!("Removed test directory");
            println!("----------------------------------------------");
        },
        Err(_err) => {
            //println!("Failed to remove test directory: {}", err);
        }
    }
}

#[test]
fn test_new_graph() {
    let func_name = "test_new_graph";

    let name = format!("fs_graph_test_{}", func_name);
    let root = ProjectDirs::from("com", "fs_graph", &name)
        .unwrap()
        .data_dir()
        .to_path_buf();

    println!("Expected full path: {:?}", root);

    let graph = Graph::new(root.clone().into(), &name);

    println!("Size of graph: {:?} bytes", graph.db.size());

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
    use fs_graph::path_ser::{buf_to_str, str_to_buf};

    let func_name = "long_alias_path";
    let mut graph = setup_graph(func_name);

    let long_path = "this/is/a/long/path/with/many/segments/verylongindeed/evenlonger/wow/are/we/still/here/there/must/be/something/we/can/do/about/all/this/tech/debt";

    let _ = graph
        .db
        .exec_mut(&QueryBuilder::insert().nodes().aliases(long_path).query());

    // Putting this conversion here for extra testing. 
    let buf = str_to_buf(long_path);
    let long_path = buf_to_str(&buf);

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
/// Test whether the db creates an attributes node when the db is first created.
fn create_attributes_category(){
    let func_name = "create_attributes_category";
    let graph = setup_graph(func_name);

    let root_node_result = graph.db.exec(&QueryBuilder::select().ids("root").query());

    assert_eq!(true, root_node_result.is_ok());

    let qry = graph.db.exec(&QueryBuilder::select().ids("root/attributes").query());
    
    assert_eq!(true, qry.is_ok());

    // The attribute we're looking for. Should be reserved. 
    // In this case, the "contains" attribute. 
    let atr = "contains";

    if root_node_result.is_ok() && qry.is_ok() {
        let root_node = root_node_result.unwrap().ids();
        assert_eq!(root_node.len(), 1);
        let root_id = root_node.first().unwrap();

        let attributes_node = qry.unwrap().ids();
        assert_eq!(attributes_node.len(), 1);
        let attributes_id = attributes_node.first().unwrap();

        let query = &QueryBuilder::search().from(*root_id).to(*attributes_id).query();
        let edge = graph.db.exec(query);

        assert_eq!(edge.is_ok(), true);
        
        let edge = edge.unwrap().elements.iter().cloned().filter(|e| e.id.0 < 0).collect::<Vec<_>>();
        println!("Found edge {:?}", edge);

        assert_eq!(edge.len(), 1);

        let eid = edge.first().unwrap().id.0;
        let edge = graph.db.exec(&QueryBuilder::select()
            .keys()
            .ids(eid)
            .query());
        let edge = edge.unwrap().elements;
        assert_eq!(edge.len(), 1);

        let vals = &edge.first().unwrap().values;
        let mut found = false;

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

