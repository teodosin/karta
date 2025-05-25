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

        let absolute_path = path.full(self.root_path());
        let fs_nodes_from_destructure = fs_reader::destructure_file_path(self.root_path(), &absolute_path, true)
            .map_err(|e| format!("Failed to destructure path {:?} with root {:?}: {}", absolute_path, self.root_path(), e))?;

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

        let node_path_dir = NodePath::user_root().join("test_dir".into());
        let node_path_file = NodePath::user_root().join("test_file.txt".into());

        std::fs::create_dir_all(&dir_path_fs).unwrap();
        std::fs::File::create(&file_path_fs).unwrap();
        std::fs::create_dir_all(&karta_dir_path_fs).unwrap();

        let graph_db = ctx.get_graph_db();
        assert!(graph_db.open_node(&NodeHandle::Path(node_path_dir.clone())).is_err(), "test_dir should not be in DB before open_context");
        assert!(graph_db.open_node(&NodeHandle::Path(node_path_file.clone())).is_err(), "test_file.txt should not be in DB before open_context");

        let (datanodes, edges, context) = ctx.get_service().open_context_from_path(NodePath::user_root()).unwrap();

        println!("[Test] Found Datanodes: {:?}", datanodes.iter().map(|dn| dn.path()).collect::<Vec<_>>());
        println!("[Test] Found Edges: {:?}", edges);


        let datanode_uuids: Vec<_> = datanodes.iter().map(|n| n.uuid()).collect();
        let viewnode_uuids: Vec<_> = context.viewnodes().iter().map(|vn| vn.uuid()).collect();
        assert_eq!(
            viewnode_uuids.iter().collect::<std::collections::HashSet<_>>(),
            datanode_uuids.iter().collect::<std::collections::HashSet<_>>(),
            "ViewNode UUIDs should match DataNode UUIDs"
        );
        
        let expected_dir_path = NodePath::user_root().join("test_dir".into());
        let expected_file_path = NodePath::user_root().join("test_file.txt".into());
        let expected_karta_dir_path = NodePath::user_root().join(".karta".into());

        let test_dir_node = datanodes.iter().find(|n| n.path() == expected_dir_path);
        let test_file_node = datanodes.iter().find(|n| n.path() == expected_file_path);
        let karta_dir_node = datanodes.iter().find(|n| n.path() == expected_karta_dir_path);

        assert!(test_dir_node.is_some(), "test_dir DataNode not found");
        assert!(test_file_node.is_some(), "test_file.txt DataNode not found");
        assert!(karta_dir_node.is_none(), ".karta directory should be ignored and not appear as a DataNode");

        let test_dir_node = test_dir_node.unwrap();
        let test_file_node = test_file_node.unwrap();

        assert!(context.viewnodes().iter().any(|vn| vn.uuid() == test_dir_node.uuid()), "No ViewNode for test_dir");
        assert!(context.viewnodes().iter().any(|vn| vn.uuid() == test_file_node.uuid()), "No ViewNode for test_file.txt");

        let user_root_node = datanodes.iter().find(|n| n.path() == NodePath::user_root()).expect("User root DataNode not found");
        
        assert!(
            edges.iter().any(|e| e.source() == &user_root_node.path() && e.target() == &test_dir_node.path()),
            "Missing edge from user_root to test_dir"
        );
        assert!(
            edges.iter().any(|e| e.source() == &user_root_node.path() && e.target() == &test_file_node.path()),
            "Missing edge from user_root to test_file.txt"
        );
    }
}