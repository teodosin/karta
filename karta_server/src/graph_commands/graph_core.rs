use super::{GraphCommands, GraphCore};

impl GraphCore for GraphCommands {
    fn storage_path(&self) -> super::StoragePath {
        todo!()
    }

    fn user_root_dirpath(&self) -> std::path::PathBuf {
        todo!()
    }

    fn root_nodepath(&self) -> super::NodePath {
        todo!()
    }

    fn root_name(&self) -> String {
        todo!()
    }

    fn new(name: &str, root_path: std::path::PathBuf, custom_storage_path: Option<std::path::PathBuf>) -> Self {
        todo!()
    }

    fn init_archetype_nodes(&mut self) {
        todo!()
    }

    fn index_single_node(&mut self, path: &super::NodePath) -> Result<super::Node, Box<dyn std::error::Error>> {
        todo!()
    }

    fn index_node_context(&mut self, path: &super::NodePath) {
        self.graph.index_node_context(path);
    }

    fn cleanup_dead_nodes(&mut self) {
        todo!()
    }

    fn maintain_readable_files(&mut self, maintain: bool) {
        todo!()
    }

    fn get_all_aliases(&self) -> Vec<String> {
        todo!()
    }
}