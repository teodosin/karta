use std::{error::Error, path::PathBuf};

use directories::ProjectDirs;

use crate::prelude::*;

pub mod commands;
pub mod graph_node;
pub mod graph_core;

pub struct GraphCommands {
    command_manager: CommandManager,
    graph: GraphAgdb,
}

impl GraphCommands {    
    pub fn new(name: &str, root_path: PathBuf, custom_storage_path: Option<PathBuf>) -> Self {
        let graph = GraphAgdb::new(name, root_path, custom_storage_path);
        let action_manager = CommandManager::new();
        GraphCommands {
            command_manager: action_manager,
            graph,
        }
    }

    pub fn apply(&mut self, command: Box<dyn CommandAgdb>) -> Result<CommandResult, Box<dyn Error>> {
        self.command_manager.apply(&mut self.graph, command)
    }

    pub fn undo(&mut self) -> Result<CommandResult, Box<dyn Error>> {
        let action = self.command_manager.undo(&mut self.graph);
        action
    }

    pub fn redo(&mut self) -> Result<CommandResult, Box<dyn Error>> {
        let action = self.command_manager.redo(&mut self.graph);
        action
    }
}

pub struct TestCommandContext {
    pub test_name: String,
    pub graph: GraphCommands,
}

impl TestCommandContext {
    pub fn new(name: &str) -> Self {
        let name = format!("fs_graph_test_{}", name);

        Self {
            test_name: name.to_string(),
            graph: TestCommandContext::setup(&name, None),
        }
    }

    pub fn custom_storage(name: &str) -> Self {
        let name = format!("fs_graph_test_{}", name);

        Self {
            test_name: name.to_string(),
            graph: TestCommandContext::setup(&name, Some(PathBuf::from("storage"))),
        }
    }

    /// Graph setup function for tests. Always stores the db in the data_dir.
    fn setup(test_name: &str, storage: Option<PathBuf>) -> GraphCommands {
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

        let graph = GraphCommands::new(&test_name, full_path.clone(), Some(strg_dir));

        assert_eq!(
            full_path.exists(),
            true,
            "Test directory has not been created"
        );

        graph
    }
}

impl Drop for TestCommandContext {
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