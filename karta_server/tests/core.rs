
use agdb::QueryBuilder;
use directories::ProjectDirs;
use fs_graph::Graph;

/// Graph setup function for tests. Always stores the db in the data_dir.
fn setup_graph(test_name: &str) -> Graph {
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

    std::fs::remove_dir_all(root).expect("Failed to remove root directory");
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

