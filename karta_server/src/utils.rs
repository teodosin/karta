// Utility functions for the library tests.

#![allow(warnings)]

use std::path::PathBuf;

use directories::ProjectDirs;

use crate::{
    graph_agdb::GraphAgdb,
    graph_traits::{graph_core::GraphCore, Graph},
};

pub struct TestContext {
    pub test_name: String,
    pub graph: GraphAgdb,
}

impl TestContext {
    pub fn new(name: &str) -> Self {
        let name = format!("fs_graph_test_{}", name);

        Self {
            test_name: name.to_string(),
            graph: TestContext::setup(&name, None),
        }
    }

    pub fn custom_storage(name: &str) -> Self {
        let name = format!("fs_graph_test_{}", name);

        Self {
            test_name: name.to_string(),
            graph: TestContext::setup(&name, Some(PathBuf::from("storage"))),
        }
    }

    /// Graph setup function for tests. Always stores the db in the data_dir.
    fn setup(test_name: &str, storage: Option<PathBuf>) -> GraphAgdb {
        // let test_name = self.test_name.clone();
        let strg_name = "fs_graph";

        let root = ProjectDirs::from("com", "fs_graph", strg_name)
            .unwrap()
            .data_dir()
            .to_path_buf();

        let full_path = root.join(&test_name);
        let strg_dir = match storage {
            Some(path) => full_path.join(path),
            None => full_path.clone(),
        };

        println!("Trying to create test directory: {:#?}", full_path);

        let graph = GraphAgdb::new_custom_storage(full_path.clone(), &test_name, strg_dir);

        assert_eq!(
            full_path.exists(),
            true,
            "Test directory has not been created"
        );

        graph
    }
}

impl Drop for TestContext {
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
            Ok(_) => {}
            Err(_err) => {
                //println!("Failed to remove test directory: {}", err);
            }
        }
    }
}
