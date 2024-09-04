// Utility functions for the library tests.

#![allow(warnings)]

use directories::ProjectDirs;

use crate::{graph_agdb::GraphAgdb, graph_traits::{graph_core::GraphCore, Graph}};

pub struct TestGraph {
    pub test_name: String,
}

impl TestGraph {
    pub fn new(name: &str) -> Self {
        let name = format!("fs_graph_test_{}", name);

        Self {
            test_name: name.to_string(),
        }
    }

    /// Graph setup function for tests. Always stores the db in the data_dir.
    pub fn setup(&self) -> impl Graph {
        let test_name = self.test_name.clone();
        let strg_name = "fs_graph";

        let root = ProjectDirs::from("com", "fs_graph", strg_name)
            .unwrap()
            .data_dir()
            .to_path_buf();

        let full_path = root.join(&test_name);

        println!("Trying to create test directory: {:#?}", full_path);

        let graph = GraphAgdb::new_custom_storage(full_path.clone(), &test_name, full_path.clone());

        assert_eq!(
            full_path.exists(),
            true,
            "Test directory has not been created"
        );

        graph
    }
}

impl Drop for TestGraph {
    fn drop(&mut self) {
        // Uncomment this return only if you need to temporarily look at the contents
        // return;
        let name = &self.test_name;
        
        let root = ProjectDirs::from("com", "fs_graph", "fs_graph")
            .unwrap()
            .data_dir()
            .to_path_buf();

        let full_path = root.join(name);
        
        let removal = std::fs::remove_dir_all(full_path);
        
        match removal {
            Ok(_) => {

            }
            Err(_err) => {
                //println!("Failed to remove test directory: {}", err);
            }
        }
    }
}

