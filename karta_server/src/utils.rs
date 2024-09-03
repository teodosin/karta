// Utility functions for the library tests.

#![allow(warnings)]

use directories::ProjectDirs;

use crate::{graph_agdb::GraphAgdb, graph_traits::Graph};

/// Graph setup function for tests. Always stores the db in the data_dir.
pub fn setup_graph(test_name: &str) -> impl Graph {
    println!("");
    println!("----------------------------------------------");
    println!("Creating graph for test: {}", test_name);

    cleanup_graph(test_name);

    let name = get_graph_dir_name(test_name);
    let root = ProjectDirs::from("com", "fs_graph", &name)
        .unwrap()
        .data_dir()
        .to_path_buf();
    let full_path = root.join(&name);

    let graph = GraphAgdb::new(root.clone().into(), &name);

    assert_eq!(
        full_path.exists(),
        true,
        "Test directory has not been created"
    );

    graph
}

/// Graph cleanup function for tests. Removes the root directory from the data_dir.
pub fn cleanup_graph(func_name: &str) {
    // Uncomment this return only if you need to temporarily look at the contents
    // return;

    let name = get_graph_dir_name(func_name);
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

/// Compute the name for the test directory
pub fn get_graph_dir_name(test_name: &str) -> String {
    let name = format!("fs_graph_test_{}", test_name);

    name
}