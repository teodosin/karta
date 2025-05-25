use std::{collections::{HashMap, HashSet}, error::Error, path::PathBuf, sync::Arc};

use tokio::sync::RwLock;

use crate::{context::{context::Context, context_db::ContextDb}, elements::node_path::NodeHandle, fs_reader, prelude::*};


pub struct KartaService {
    vault_fs_path: PathBuf,
    storage_dir: PathBuf,
    data: GraphAgdb,
    view: ContextDb,
}

impl KartaService {
    pub fn new(
        name: &str,
        vault_fs_path: PathBuf,
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
            vault_fs_path.clone(),
            storage_dir.clone(),
        );
        let view = ContextDb::new(
                name.to_owned(),
                vault_fs_path.clone(),
                storage_dir.clone(),
        );

        Self {
            vault_fs_path,
            storage_dir,
            data,
            view
        }
    }

    pub fn vault_fs_path(&self) -> &PathBuf {
        &self.vault_fs_path
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

        if path == NodePath::root() {
            let focal_node = self.data().open_node(&NodeHandle::Path(NodePath::root()))
                .map_err(|e| format!("Failed to open virtual root node: {}", e))?;
            
            let mut datanodes_for_context = vec![focal_node.clone()];
            let mut edges_for_context = Vec::new();

            // Get children (primarily vault) and their edges from the database
            let db_child_connections = self.data().open_node_connections(&NodePath::root());
            for (child_node, edge) in db_child_connections {
                // For the virtual root's context, we are primarily interested in vault as its direct child.
                if child_node.path() == NodePath::vault() {
                    if !datanodes_for_context.iter().any(|n| n.path() == child_node.path()) {
                        datanodes_for_context.push(child_node);
                    }
                    edges_for_context.push(edge);
                }
                // Potentially include other direct virtual children of NodePath::root() if defined later.
            }
            
            // Ensure vault is included if not found via connections (e.g. if connections only returns non-archetype)
            if !datanodes_for_context.iter().any(|n| n.path() == NodePath::vault()) {
                let vault_node = self.data().open_node(&NodeHandle::Path(NodePath::vault()))
                    .map_err(|e| format!("Failed to open vault node: {}", e))?;
                datanodes_for_context.push(vault_node);
                // If the edge was also missing, this implies it should be created or is an error.
                // For now, assume open_node_connections is the source of truth for edges.
                // A robust solution might involve self.data().get_edge_strict() if the edge is critical and might be missed.
            }

            let context = self.view.generate_context(focal_node.uuid(), datanodes_for_context.clone());
            return Ok((datanodes_for_context, edges_for_context, context));
        }

        // --- Existing logic for vault and other FS-related paths ---
        let absolute_path = path.full(self.vault_fs_path());
        let fs_nodes_from_destructure = fs_reader::destructure_file_path(self.vault_fs_path(), &absolute_path, true)
            .map_err(|e| format!("Failed to destructure path {:?} with root {:?}: {}", absolute_path, self.vault_fs_path(), e))?;

        let mut focal_fs_datanode: Option<DataNode> = None;
        let mut child_fs_datanodes: Vec<DataNode> = Vec::new();
        let mut fs_edges: Vec<Edge> = Vec::new();

        if absolute_path.is_dir() {
            focal_fs_datanode = Some(DataNode::new(&path, NodeTypeId::dir_type()));
            child_fs_datanodes = fs_nodes_from_destructure;
            if let Some(focal_node_unwrapped) = &focal_fs_datanode {
                for child_node in &child_fs_datanodes {
                    fs_edges.push(Edge::new(&focal_node_unwrapped.path(), &child_node.path()));
                }
            }
        } else if absolute_path.is_file() {
            focal_fs_datanode = fs_nodes_from_destructure.into_iter().find(|n| n.path() == path);
        }
        
        let fs_derived_focal_node = focal_fs_datanode.ok_or_else(||
            format!("Focal node for path {:?} could not be determined from filesystem.", path)
        )?;

        let db_focal_datanode_optional = self.data().open_node(&NodeHandle::Path(path.clone())).ok();
        let db_child_connections = self.data().open_node_connections(&path);
        
        let mut db_child_datanodes_map: HashMap<NodePath, DataNode> = HashMap::new();
        let mut db_edges_vec: Vec<Edge> = Vec::new();
        for (node, edge) in db_child_connections {
            db_child_datanodes_map.insert(node.path().clone(), node);
            db_edges_vec.push(edge);
        }

        let mut final_datanodes_map: HashMap<NodePath, DataNode> = HashMap::new();
        let mut final_edges_set: HashSet<(NodePath, NodePath)> = HashSet::new();
        let mut reconciled_edges: Vec<Edge> = Vec::new();

        let definitive_focal_node = match db_focal_datanode_optional {
            Some(db_node) => db_node,
            None => fs_derived_focal_node.clone(),
        };
        final_datanodes_map.insert(definitive_focal_node.path().clone(), definitive_focal_node.clone());

        for fs_child_node in &child_fs_datanodes {
            match db_child_datanodes_map.get(&fs_child_node.path()) {
                Some(db_child_node) => {
                    final_datanodes_map.insert(db_child_node.path().clone(), db_child_node.clone());
                }
                None => {
                    final_datanodes_map.insert(fs_child_node.path().clone(), fs_child_node.clone());
                }
            }
        }
// Include other DB-connected nodes not present in FS (e.g., parents, other virtual links)
        for (db_node_path, db_node_data) in db_child_datanodes_map.iter() {
            if !final_datanodes_map.contains_key(db_node_path) {
                final_datanodes_map.insert(db_node_path.clone(), db_node_data.clone());
            }
        }
        
        for fs_edge in fs_edges {
            if final_datanodes_map.contains_key(fs_edge.source()) && final_datanodes_map.contains_key(fs_edge.target()) {
                let edge_key = (fs_edge.source().clone(), fs_edge.target().clone());
                if final_edges_set.insert(edge_key) {
                    reconciled_edges.push(fs_edge);
                }
            }
        }

        for db_edge in db_edges_vec {
            if final_datanodes_map.contains_key(db_edge.source()) && final_datanodes_map.contains_key(db_edge.target()) {
                let edge_key = (db_edge.source().clone(), db_edge.target().clone());
                if final_edges_set.insert(edge_key) {
                    reconciled_edges.push(db_edge);
                }
            }
        }

        let collected_final_datanodes: Vec<DataNode> = final_datanodes_map.values().cloned().collect();

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
        let root_path = ctx.get_vault_root();

        let dir_path_fs = root_path.join("test_dir");
        let file_path_fs = root_path.join("test_file.txt");
        let karta_dir_path_fs = root_path.join(".karta");

        let node_path_dir = NodePath::vault().join("test_dir".into());
        let node_path_file = NodePath::vault().join("test_file.txt".into());

        std::fs::create_dir_all(&dir_path_fs).unwrap();
        std::fs::File::create(&file_path_fs).unwrap();
        std::fs::create_dir_all(&karta_dir_path_fs).unwrap();

        ctx.with_graph_db(|graph_db| {
            assert!(graph_db.open_node(&NodeHandle::Path(node_path_dir.clone())).is_err(), "test_dir should not be in DB before open_context");
            assert!(graph_db.open_node(&NodeHandle::Path(node_path_file.clone())).is_err(), "test_file.txt should not be in DB before open_context");
        });

        let (datanodes, edges, context) = ctx.with_service(|s| s.open_context_from_path(NodePath::vault())).unwrap();

        println!("[Test] Found Datanodes: {:?}", datanodes.iter().map(|dn| dn.path()).collect::<Vec<_>>());
        println!("[Test] Found Edges: {:?}", edges);


        let datanode_uuids: Vec<_> = datanodes.iter().map(|n| n.uuid()).collect();
        let viewnode_uuids: Vec<_> = context.viewnodes().iter().map(|vn| vn.uuid()).collect();
        assert_eq!(
            viewnode_uuids.iter().collect::<std::collections::HashSet<_>>(),
            datanode_uuids.iter().collect::<std::collections::HashSet<_>>(),
            "ViewNode UUIDs should match DataNode UUIDs"
        );

        println!("Datanodes amount: {}", datanodes.len());
        
        let expected_dir_path = NodePath::vault().join("test_dir".into());
        let expected_file_path = NodePath::vault().join("test_file.txt".into());
        let expected_karta_dir_path = NodePath::vault().join(".karta".into());

        let test_dir_node = datanodes.iter().find(|n| n.path() == expected_dir_path);
        let test_file_node = datanodes.iter().find(|n| n.path() == expected_file_path);
        let karta_dir_node = datanodes.iter().find(|n| n.path() == expected_karta_dir_path);

        assert!(test_dir_node.is_some(), "test_dir DataNode not found");
        assert!(test_file_node.is_some(), "test_file.txt DataNode not found");
        assert!(karta_dir_node.is_none(), ".karta directory should be ignored and not appear as a DataNode");

assert!(datanodes.iter().any(|n| n.path() == NodePath::root()), "NodePath::root() not found in datanodes when opening vault context");
        let test_dir_node = test_dir_node.unwrap();
        let test_file_node = test_file_node.unwrap();

        assert!(context.viewnodes().iter().any(|vn| vn.uuid() == test_dir_node.uuid()), "No ViewNode for test_dir");
        assert!(context.viewnodes().iter().any(|vn| vn.uuid() == test_file_node.uuid()), "No ViewNode for test_file.txt");

        let vault_node = datanodes.iter().find(|n| n.path() == NodePath::vault()).expect("User root DataNode not found");
        
        assert!(
            edges.iter().any(|e| e.source() == &vault_node.path() && e.target() == &test_dir_node.path()),
            "Missing edge from vault to test_dir"
        );
        assert!(
            edges.iter().any(|e| e.source() == &vault_node.path() && e.target() == &test_file_node.path()),
            "Missing edge from vault to test_file.txt"
        );
    }
}