use std::{error::Error, path::PathBuf, sync::Arc};

use tokio::sync::RwLock;

use crate::{context::{context::Context, context_db::ContextDb}, elements::node_path::NodeHandle, prelude::*};


pub struct KartaService {
    root_path: PathBuf,
    storage_dir: PathBuf,
    data: GraphAgdb,
    view: ContextDb,
}

impl KartaService {
    pub fn new(
        name: &str,
        root_path: PathBuf,
        storage_dir: PathBuf,
    ) -> Self {

        // Check if the storage dir is called .karta.
        // If not, create it.
        // This might be a bit crude, but it will do for now.
        let mut storage_dir = storage_dir;
        if storage_dir.file_name().unwrap() != ".karta" {
            storage_dir = storage_dir.join(".karta");
            std::fs::create_dir_all(&storage_dir).unwrap();
        }

        let data = GraphAgdb::new(
            name,
            root_path.clone(),
            storage_dir.clone(),
        );
        let view = ContextDb::new(
                name.to_owned(),
                root_path.clone(),
                storage_dir.clone(),
        );

        Self { 
            root_path,
            storage_dir,
            data,
            view
        }
    }

    pub fn root_path(&self) -> &PathBuf {
        &self.root_path
    }

    pub fn storage_path(&self) -> &PathBuf {
        &self.storage_dir
    }

    
    pub fn data(&self) -> &GraphAgdb {
        &self.data
    }

    
    pub fn view(&self) -> &ContextDb {
        &self.view
    }

    
    pub fn data_mut(&mut self) -> &mut GraphAgdb {
        &mut self.data
    }

    
    pub fn view_mut(&mut self) -> &mut ContextDb {
        &mut self.view
    }

    /// Opens a context's Data and View.
    /// This is the main function for opening a context.
    /// Reconciles the indexed data with the physical data.
    pub fn open_context_from_path(&self, path: NodePath) 
        -> Result<(Vec<DataNode>, Vec<Edge>, Context), Box<dyn Error>> {

        let mut finaldatanodes: Vec<DataNode> = Vec::new();
        let mut finaledges: Vec<Edge> = Vec::new();
           
        let focal_handle: NodeHandle = NodeHandle::Path(path.clone());
        let focal_node = self.data().open_node(&focal_handle)?;
        let focal_uuid = focal_node.uuid();

        finaldatanodes.push(focal_node);

        let datanodes = self.data().open_node_connections(&path);
        for (node, edge) in datanodes {
            finaldatanodes.push(node);
            finaledges.push(edge);
        }

        let context = self.view.generate_context(
            focal_uuid,
            finaldatanodes.clone(),
        );

        return Ok((finaldatanodes, finaledges, context));
    }
}

#[cfg(test)]
mod tests {
    use crate::{prelude::NodePath, utils::utils::KartaServiceTestContext};

    #[test]
    fn opening_directory_spawns_viewnodes_without_indexing() {
        let func_name = "opening_directory_spawns_viewnodes_without_indexing";
        let ctx = KartaServiceTestContext::new(func_name);

        // Create a bunch of files and directories in the test vault
        let root_path = ctx.get_vault_root();

        println!("Root path: {:?}", root_path);

        let dir_path = root_path.join("test_dir");
        let file_path = root_path.join("test_file.txt");

        println!("Creating directory: {:?}", dir_path);
        println!("Creating file: {:?}", file_path);

        
        std::fs::create_dir_all(&dir_path).unwrap();
        std::fs::File::create(&file_path).unwrap();

        println!("----------------");

        let (datanodes, edges, context) = ctx.get_service().open_context_from_path(NodePath::user_root()).unwrap();

        // Collect datanode uuids and names
        let datanode_uuids: Vec<_> = datanodes.iter().map(|n| n.uuid()).collect();
        let datanode_names: Vec<_> = datanodes.iter().map(|n| n.name().to_string()).collect();

        // Collect viewnode uuids (from method) and names (by matching uuid to datanode)
        let viewnode_uuids: Vec<_> = context.viewnodes().iter().map(|vn| vn.uuid()).collect();
        let viewnode_names: Vec<_> = context.viewnodes().iter().filter_map(|vn| {
            datanodes.iter().find(|dn| dn.uuid() == vn.uuid()).map(|dn| dn.name().to_string())
        }).collect::<Vec<_>>();

        // Assert that all viewnode uuids match datanode uuids
        assert_eq!(
            viewnode_uuids.iter().collect::<std::collections::HashSet<_>>(),
            datanode_uuids.iter().collect::<std::collections::HashSet<_>>()
        );

        // Print viewnode names
        println!("Viewnodes: {:?}", viewnode_names);

        // Print edges as Name -> Name
        let edge_strs: Vec<String> = edges.iter().map(|e| {
            let from_name = datanodes.iter().find(|n| n.path() == *e.source()).map(|n| n.name()).unwrap_or("<unknown>".to_string());
            let to_name = datanodes.iter().find(|n| n.path() == *e.target()).map(|n| n.name()).unwrap_or("<unknown>".to_string());
            format!("{} -> {}", from_name, to_name)
        }).collect();
        println!("Edges: {:?}", edge_strs);

        // --- Additional asserts ---
        // Find datanodes for test_dir and test_file.txt
        let test_dir_node = datanodes.iter().find(|n| format!("{:?}", n.path()).ends_with("test_dir"));
        let test_file_node = datanodes.iter().find(|n| format!("{:?}", n.path()).ends_with("test_file.txt"));
        assert!(test_dir_node.is_some(), "No datanode with path ending in test_dir");
        assert!(test_file_node.is_some(), "No datanode with path ending in test_file.txt");
        let test_dir_node = test_dir_node.unwrap();
        let test_file_node = test_file_node.unwrap();

        // Each should have a corresponding viewnode
        let test_dir_viewnode = context.viewnodes().iter().find(|vn| vn.uuid() == test_dir_node.uuid());
        let test_file_viewnode = context.viewnodes().iter().find(|vn| vn.uuid() == test_file_node.uuid());
        assert!(test_dir_viewnode.is_some(), "No viewnode for test_dir datanode");
        assert!(test_file_viewnode.is_some(), "No viewnode for test_file.txt datanode");

        // There should be edges from user_root to both
        let user_root_node = datanodes.iter().find(|n| n.path() == NodePath::user_root()).expect("No user_root datanode");
        let has_edge_to_dir = edges.iter().any(|e| e.source() == &user_root_node.path() && e.target() == &test_dir_node.path());
        let has_edge_to_file = edges.iter().any(|e| e.source() == &user_root_node.path() && e.target() == &test_file_node.path());
        assert!(has_edge_to_dir, "No edge from user_root to test_dir");
        assert!(has_edge_to_file, "No edge from user_root to test_file.txt");
    }
}