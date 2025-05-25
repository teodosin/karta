use std::{collections::{HashMap, HashSet}, error::Error, path::PathBuf, sync::Arc};

use tokio::sync::RwLock;

use crate::{context::{context::Context, context_db::ContextDb}, elements::node_path::NodeHandle, fs_reader, prelude::*};


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
    /// Reconciles indexed data from the database with physical data from the filesystem.
    /// Filesystem state (existence, name, type) takes precedence for the returned view.
    /// Karta-specific attributes and UUIDs are sourced from the database if an entry exists.
    /// This function is read-only regarding database writes.
    pub fn open_context_from_path(&self, path: NodePath)
        -> Result<(Vec<DataNode>, Vec<Edge>, Context), Box<dyn Error>> {

        // A. Get Filesystem Perspective
        let absolute_path = path.full(self.root_path());
        let fs_nodes_from_destructure = fs_reader::destructure_file_path(self.root_path(), &absolute_path, true)
            .map_err(|e| format!("Failed to destructure path {:?} with root {:?}: {}", absolute_path, self.root_path(), e))?;

        let mut focal_fs_datanode: Option<DataNode> = None;
        let mut child_fs_datanodes: Vec<DataNode> = Vec::new();
        let mut fs_edges: Vec<Edge> = Vec::new();

        if absolute_path.is_dir() {
            // For a directory, `destructure_file_path` returns its children.
            // The focal node for the directory itself needs to be created.
            focal_fs_datanode = Some(DataNode::new(&path, NodeTypeId::dir_type()));
            child_fs_datanodes = fs_nodes_from_destructure; // These already have NodePath from fs_reader
            if let Some(focal_node_unwrapped) = &focal_fs_datanode {
                for child_node in &child_fs_datanodes {
                    // Edge::new expects &NodePath
                    fs_edges.push(Edge::new(&focal_node_unwrapped.path(), &child_node.path()));
                }
            }
        } else if absolute_path.is_file() {
            focal_fs_datanode = fs_nodes_from_destructure.into_iter().find(|n| n.path() == path); // Compare NodePath with NodePath
        }
        
        let fs_derived_focal_node = focal_fs_datanode.ok_or_else(||
            format!("Focal node for path {:?} could not be determined from filesystem.", path)
        )?;

        // B. Get Database Perspective
        let db_focal_datanode_optional = self.data().open_node(&NodeHandle::Path(path.clone())).ok();
        let db_child_connections = self.data().open_node_connections(&path); // Returns Vec<(DataNode, Edge)>
        
        let mut db_child_datanodes_map: HashMap<NodePath, DataNode> = HashMap::new();
        let mut db_edges_vec: Vec<Edge> = Vec::new();
        for (node, edge) in db_child_connections {
            db_child_datanodes_map.insert(node.path().clone(), node);
            db_edges_vec.push(edge);
        }

        // C. Reconcile Data
        let mut final_datanodes_map: HashMap<NodePath, DataNode> = HashMap::new();
        let mut final_edges_set: HashSet<(NodePath, NodePath)> = HashSet::new();
        let mut reconciled_edges: Vec<Edge> = Vec::new();

        // Reconcile Focal Node
        let definitive_focal_node = match db_focal_datanode_optional {
            Some(db_node) => {
                // Prefer DB node, ensure its ntype matches FS reality if necessary (though path match implies this)
                // For now, assume db_node is mostly correct if path matches.
                // We could update db_node.ntype = fs_derived_focal_node.ntype_name() if strict FS type override is needed.
                db_node
            }
            None => fs_derived_focal_node.clone(), // Use FS derived if not in DB
        };
        final_datanodes_map.insert(definitive_focal_node.path().clone(), definitive_focal_node.clone());

        // Reconcile Child Nodes
        for fs_child_node in &child_fs_datanodes { // Iterate by reference
            match db_child_datanodes_map.get(&fs_child_node.path()) { // Pass reference to path
                Some(db_child_node) => {
                    // DB node exists for this FS path, prefer the DB version (has Karta attributes and stable UUID)
                    final_datanodes_map.insert(db_child_node.path().clone(), db_child_node.clone());
                }
                None => {
                    // FS node not in DB, use the FS version
                    final_datanodes_map.insert(fs_child_node.path().clone(), fs_child_node.clone());
                }
            }
        }
        
        // Add FS-derived edges first
        for fs_edge in fs_edges {
            if final_datanodes_map.contains_key(fs_edge.source()) && final_datanodes_map.contains_key(fs_edge.target()) {
                let edge_key = (fs_edge.source().clone(), fs_edge.target().clone());
                if final_edges_set.insert(edge_key) {
                    reconciled_edges.push(fs_edge);
                }
            }
        }

        // Add DB-derived edges if they connect nodes present in the final set and are not duplicates
        for db_edge in db_edges_vec {
            if final_datanodes_map.contains_key(db_edge.source()) && final_datanodes_map.contains_key(db_edge.target()) {
                let edge_key = (db_edge.source().clone(), db_edge.target().clone());
                if final_edges_set.insert(edge_key) {
                    reconciled_edges.push(db_edge);
                }
            }
        }

        let collected_final_datanodes: Vec<DataNode> = final_datanodes_map.values().cloned().collect();

        // D. Generate Context
        let context_focal_uuid = definitive_focal_node.uuid();
        let context = self.view.generate_context(
            context_focal_uuid,
            collected_final_datanodes.clone(),
        );

        Ok((collected_final_datanodes, reconciled_edges, context))
    }
}

#[cfg(test)]
mod tests {
    use crate::{prelude::NodePath, utils::utils::KartaServiceTestContext, elements::node_path::NodeHandle, graph_traits::graph_node::GraphNodes};

    #[test]
    fn opening_directory_spawns_viewnodes_without_indexing() {
        let func_name = "opening_directory_spawns_viewnodes_without_indexing";
        let ctx = KartaServiceTestContext::new(func_name);

        // Create a bunch of files and directories in the test vault
        let root_path = ctx.get_vault_root();

        println!("[Test] Root path for test vault: {:?}", root_path);

        let dir_path_fs = root_path.join("test_dir"); // Filesystem path
        let file_path_fs = root_path.join("test_file.txt"); // Filesystem path
        let karta_dir_path_fs = root_path.join(".karta"); // Path for .karta directory

        // Corresponding NodePaths
        let node_path_dir = NodePath::user_root().join("test_dir".into());
        let node_path_file = NodePath::user_root().join("test_file.txt".into());


        println!("[Test] Creating directory on FS: {:?}", dir_path_fs);
        println!("[Test] Creating file on FS: {:?}", file_path_fs);
        println!("[Test] Creating .karta directory on FS: {:?}", karta_dir_path_fs);
        
        std::fs::create_dir_all(&dir_path_fs).unwrap();
        std::fs::File::create(&file_path_fs).unwrap();
        std::fs::create_dir_all(&karta_dir_path_fs).unwrap(); // Create .karta dir

        // Assert that these nodes are NOT in the DB before opening context
        let graph_db = ctx.get_graph_db();
        println!("[Test] Checking DB for path: {:?}", node_path_dir);
        assert!(graph_db.open_node(&NodeHandle::Path(node_path_dir.clone())).is_err(), "test_dir should not be in DB yet");
        println!("[Test] Checking DB for path: {:?}", node_path_file);
        assert!(graph_db.open_node(&NodeHandle::Path(node_path_file.clone())).is_err(), "test_file.txt should not be in DB yet");

        println!("---------------- [Test] Calling open_context_from_path ----------------");

        let (datanodes, edges, context) = ctx.get_service().open_context_from_path(NodePath::user_root()).unwrap();

        println!("[Test] Datanodes returned by open_context_from_path:");
        for (i, dn) in datanodes.iter().enumerate() {
            println!("[Test]   {}: Name: {}, Path: {:?}", i, dn.name(), dn.path());
        }

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
        println!("[Test] Viewnodes (names derived from datanodes): {:?}", viewnode_names);

        // Print edges as Name -> Name
        let edge_strs: Vec<String> = edges.iter().map(|e| {
            let from_name = datanodes.iter().find(|n| n.path() == *e.source()).map(|n| n.name()).unwrap_or("<unknown_source_path>".to_string());
            let to_name = datanodes.iter().find(|n| n.path() == *e.target()).map(|n| n.name()).unwrap_or("<unknown_target_path>".to_string());
            format!("{} -> {}", from_name, to_name)
        }).collect();
        println!("[Test] Edges (SourcePath -> TargetPath): {:?}", edge_strs);
        println!("[Test] Edges (raw): {:?}", edges);


        // --- Additional asserts ---
        // Find datanodes for test_dir and test_file.txt
        let expected_dir_path = NodePath::user_root().join("test_dir".into()); 
        let expected_file_path = NodePath::user_root().join("test_file.txt".into()); 
        let expected_karta_dir_path = NodePath::user_root().join(".karta".into());

        println!("[Test] Expected dir_path for assertion: {:?}", expected_dir_path);
        println!("[Test] Expected file_path for assertion: {:?}", expected_file_path);
        println!("[Test] Expected .karta_dir_path for assertion: {:?}", expected_karta_dir_path);


        let test_dir_node = datanodes.iter().find(|n| {
            println!("[Test] Comparing for test_dir: {:?} == {:?}", n.path(), expected_dir_path);
            n.path() == expected_dir_path
        });
        let test_file_node = datanodes.iter().find(|n| {
            println!("[Test] Comparing for test_file: {:?} == {:?}", n.path(), expected_file_path);
            n.path() == expected_file_path
        });
        let karta_dir_node = datanodes.iter().find(|n| {
            println!("[Test] Comparing for .karta_dir: {:?} == {:?}", n.path(), expected_karta_dir_path);
            n.path() == expected_karta_dir_path
        });

        assert!(test_dir_node.is_some(), "No datanode with path for test_dir found");
        assert!(test_file_node.is_some(), "No datanode with path for test_file.txt found");
        assert!(karta_dir_node.is_none(), "A .karta directory node was found, it should be ignored.");


        let test_dir_node = test_dir_node.unwrap();
        let test_file_node = test_file_node.unwrap();

        // Each should have a corresponding viewnode
        let test_dir_viewnode = context.viewnodes().iter().find(|vn| vn.uuid() == test_dir_node.uuid());
        let test_file_viewnode = context.viewnodes().iter().find(|vn| vn.uuid() == test_file_node.uuid());
        assert!(test_dir_viewnode.is_some(), "No viewnode for test_dir datanode");
        assert!(test_file_viewnode.is_some(), "No viewnode for test_file.txt datanode");

        // There should be edges from user_root to both
        let user_root_node = datanodes.iter().find(|n| n.path() == NodePath::user_root()).expect("No user_root datanode");
        println!("[Test] User root node path for edge check: {:?}", user_root_node.path());
        println!("[Test] Test dir node path for edge check: {:?}", test_dir_node.path());
        println!("[Test] Test file node path for edge check: {:?}", test_file_node.path());

        let has_edge_to_dir = edges.iter().any(|e| {
            let check = e.source() == &user_root_node.path() && e.target() == &test_dir_node.path();
            if check { println!("[Test] Found edge to dir: {:?} -> {:?}", e.source(), e.target()); }
            check
        });
        let has_edge_to_file = edges.iter().any(|e| {
            let check = e.source() == &user_root_node.path() && e.target() == &test_file_node.path();
            if check { println!("[Test] Found edge to file: {:?} -> {:?}", e.source(), e.target()); }
            check
        });
        assert!(has_edge_to_dir, "No edge from user_root to test_dir");
        assert!(has_edge_to_file, "No edge from user_root to test_file.txt");
    }
}