
use agdb::QueryBuilder;
use directories::ProjectDirs;
use fs_graph::Graph;

/// Setup function for tests. Always stores the db in the data_dir.
fn setup(test_name: &str) -> Graph {
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

/// Cleanup function for tests. Removes the root directory from the data_dir.
fn cleanup(test_name: &str) {
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

    cleanup(func_name);
}

#[test]
fn existing_db_in_directory() {
    // We add a node to the db, then create a new graph with the same name.
    // The new graph should be able to access the node.
    let func_name = "existing_db_in_directory";
    let mut first = setup(func_name);

    let _ = first
        .db
        .exec_mut(&QueryBuilder::insert().nodes().aliases("testalias").query());

    let second = setup(func_name);

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

    cleanup(func_name);
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
