use std::{collections::{HashMap, HashSet}, error::Error, path::PathBuf, sync::Arc};
use uuid::Uuid;

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

            let context = self.view.generate_context(focal_node.uuid(), None, datanodes_for_context.clone());
            return Ok((datanodes_for_context, edges_for_context, context));
        }

        // --- Existing logic for vault and other FS-related paths ---
        let mut additional_nodes_to_include: Vec<DataNode> = Vec::new();
        let mut additional_edges_to_include: Vec<Edge> = Vec::new();
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
                    fs_edges.push(Edge::new(focal_node_unwrapped.uuid(), child_node.uuid()));
                }
            }
        } else if absolute_path.is_file() {
            focal_fs_datanode = fs_nodes_from_destructure.into_iter().find(|n| n.path() == path);
            if let Some(focal_file_node_unwrapped) = &focal_fs_datanode {
                if let Some(parent_path) = path.parent() {
                    // We need the parent's UUID. We can create a transient parent node to get it.
                    let parent_node = DataNode::new(&parent_path, NodeTypeId::dir_type());
                    fs_edges.push(Edge::new(parent_node.uuid(), focal_file_node_unwrapped.uuid()));
                }
            }
        }

        let fs_derived_focal_node = focal_fs_datanode.ok_or_else(|| {
            format!("Focal node for path {:?} could not be determined from filesystem.", path)
        })?;

        let db_focal_datanode_optional = self.data().open_node(&NodeHandle::Path(path.clone())).ok();
        let db_child_connections = self.data().open_node_connections(&path);

        let mut db_child_datanodes_map: HashMap<Uuid, DataNode> = HashMap::new();
        for (node, _) in db_child_connections {
            db_child_datanodes_map.insert(node.uuid(), node);
        }

        let mut final_datanodes_map: HashMap<Uuid, DataNode> = HashMap::new();
        let mut final_edges_set: HashSet<(Uuid, Uuid)> = HashSet::new();
        let mut reconciled_edges: Vec<Edge> = Vec::new();

        let definitive_focal_node = match db_focal_datanode_optional {
            Some(db_node) => db_node,
            None => fs_derived_focal_node.clone(),
        };

        if let Some(parent_path) = definitive_focal_node.path().parent() {
            if parent_path == NodePath::vault() {
                if let Ok(vault_node) = self.data().open_node(&NodeHandle::Path(NodePath::vault())) {
                    let vault_to_focal_edge = Edge::new(vault_node.uuid(), definitive_focal_node.uuid());
                    additional_nodes_to_include.push(vault_node);
                    additional_edges_to_include.push(vault_to_focal_edge);
                } else {
                    eprintln!("Critical error: Vault node not found in DB.");
                }
            }
        }

        final_datanodes_map.insert(definitive_focal_node.uuid(), definitive_focal_node.clone());

        let mut parent_uuid: Option<Uuid> = None;
        if let Some(parent_path) = definitive_focal_node.path().parent() {
            let parent_node = self.data().open_node(&NodeHandle::Path(parent_path.clone()))
                .unwrap_or_else(|_| DataNode::new(&parent_path, NodeTypeId::dir_type()));
            parent_uuid = Some(parent_node.uuid());
            final_datanodes_map.entry(parent_node.uuid()).or_insert(parent_node);
        }

        for fs_child_node in &child_fs_datanodes {
            let child_uuid = fs_child_node.uuid();
            let definitive_child = db_child_datanodes_map.get(&child_uuid)
                .cloned()
                .unwrap_or_else(|| fs_child_node.clone());
            final_datanodes_map.insert(child_uuid, definitive_child);
        }

        for (db_node_uuid, db_node_data) in db_child_datanodes_map.iter() {
            final_datanodes_map.entry(*db_node_uuid).or_insert_with(|| db_node_data.clone());
        }

        for node_to_add in &additional_nodes_to_include {
            final_datanodes_map.entry(node_to_add.uuid()).or_insert_with(|| node_to_add.clone());
        }

        let mut all_edges_to_process = fs_edges;
        all_edges_to_process.extend(db_child_datanodes_map.values().flat_map(|node| {
            self.data().open_node_connections(&node.path()).into_iter().map(|(_, edge)| edge)
        }));
        all_edges_to_process.extend(additional_edges_to_include);

        for edge in all_edges_to_process {
            if final_datanodes_map.contains_key(edge.source()) && final_datanodes_map.contains_key(edge.target()) {
                let edge_key = (*edge.source(), *edge.target());
                if final_edges_set.insert(edge_key) {
                    reconciled_edges.push(edge);
                }
            }
        }

        let collected_final_datanodes: Vec<DataNode> = final_datanodes_map.values().cloned().collect();
        let context_focal_uuid = definitive_focal_node.uuid();

        let context = self.view.generate_context(
            context_focal_uuid,
            parent_uuid, // Pass the parent's UUID
            collected_final_datanodes.clone(),
        );

        Ok((collected_final_datanodes, reconciled_edges, context))
    }
}

#[cfg(test)]
mod tests {
    use crate::{prelude::*, utils::utils::KartaServiceTestContext, elements::{node_path::NodeHandle, attribute::{Attribute, AttrValue}, view_node::ViewNode}, graph_traits::{graph_node::GraphNodes, graph_edge::GraphEdge}, context::context::Context};

    #[test]
    fn opening_directory_spawns_viewnodes_without_indexing() {
        let func_name = "opening_directory_spawns_viewnodes_without_indexing";
        let ctx = KartaServiceTestContext::new(func_name);
        let root_path = ctx.get_vault_root();

        let dir_path_fs = root_path.join("test_dir");
        let nested_dir_path_fs = dir_path_fs.join("nested_dir");
        let file_path_fs = root_path.join("test_file.txt");
        
        std::fs::create_dir_all(&nested_dir_path_fs).unwrap();
        std::fs::File::create(&file_path_fs).unwrap();

        // --- Part 1: Test opening the vault context ---
        let (datanodes, edges, _) = ctx.with_service(|s| s.open_context_from_path(NodePath::vault())).unwrap();

        let vault_node = datanodes.iter().find(|n| n.path() == NodePath::vault()).expect("Vault node not found");
        let root_node = datanodes.iter().find(|n| n.path() == NodePath::root()).expect("Root node not found");
        let test_dir_node = datanodes.iter().find(|n| n.path() == NodePath::vault().join("test_dir")).expect("test_dir not found");

        assert_eq!(datanodes.len(), 4, "Should contain root, vault, test_dir, and test_file.txt");
        assert!(edges.iter().any(|e| *e.source() == root_node.uuid() && *e.target() == vault_node.uuid()), "Missing edge from root to vault");
        assert!(edges.iter().any(|e| *e.source() == vault_node.uuid() && *e.target() == test_dir_node.uuid()), "Missing edge from vault to test_dir");

        // --- Part 2: Test opening a deeper context to check for grandparent bug ---
        let (datanodes_deeper, _, _) = ctx.with_service(|s| s.open_context_from_path(NodePath::vault().join("test_dir"))).unwrap();

        assert!(datanodes_deeper.iter().any(|n| n.path() == NodePath::vault().join("test_dir")), "Focal node test_dir missing");
        assert!(datanodes_deeper.iter().any(|n| n.path() == NodePath::vault()), "Parent node vault missing");
        assert!(datanodes_deeper.iter().any(|n| n.path() == NodePath::vault().join("test_dir").join("nested_dir")), "Child node nested_dir missing");
        assert!(!datanodes_deeper.iter().any(|n| n.path() == NodePath::root()), "Grandparent root node should NOT be present");
        assert_eq!(datanodes_deeper.len(), 3, "Should only contain focal, parent, and child");
    }

    #[test]
    fn test_load_filesystem_context_with_db_entries() {
        let func_name = "test_load_filesystem_context_with_db_entries";
        let ctx = KartaServiceTestContext::new(func_name);
        let root_path = ctx.get_vault_root();
        let dir_path = root_path.join("another_dir");
        let file_path = dir_path.join("another_file.txt");
        std::fs::create_dir_all(&dir_path).unwrap();
        std::fs::File::create(&file_path).unwrap();

        let file_node_path = NodePath::vault().join("another_dir").join("another_file.txt");
        ctx.with_service_mut(|s| {
            let mut file_node = DataNode::new(&file_node_path, NodeTypeId::file_type());
            file_node.set_attributes(vec![Attribute::new_string("custom_attr".to_string(), "db_value".to_string())]);
            s.data_mut().insert_nodes(vec![file_node]);
        });

        let (datanodes, _, _) = ctx.with_service(|s| s.open_context_from_path(NodePath::vault().join("another_dir"))).unwrap();
        
        let fetched_file_node = datanodes.iter().find(|n| n.path() == file_node_path).expect("File node not found in context");
        let binding = fetched_file_node.attributes();
        let attr = binding.iter().find(|a| a.name == "custom_attr").expect("Custom attribute not found");
        assert_eq!(attr.value, AttrValue::String("db_value".to_string()));
        assert_eq!(datanodes.len(), 3, "Should contain focal, parent, and child");
    }

    #[test]
    fn test_load_virtual_node_context() {
        let func_name = "test_load_virtual_node_context";
        let ctx = KartaServiceTestContext::new(func_name);
        let root_path = ctx.get_vault_root();
        let parent_dir_path_fs = root_path.join("parent_dir");
        std::fs::create_dir_all(&parent_dir_path_fs).unwrap();

        let parent_node_path = NodePath::vault().join("parent_dir");
        let virtual_node_path = parent_node_path.join("virtual_text_node");

        ctx.with_service_mut(|s| {
            let parent_node = DataNode::new(&parent_node_path, NodeTypeId::dir_type());
            let virtual_node = DataNode::new(&virtual_node_path, NodeTypeId::new("core/text"));
            s.data_mut().insert_nodes(vec![parent_node.clone(), virtual_node.clone()]);
        });

        let (datanodes, _, _) = ctx.with_service(|s| s.open_context_from_path(virtual_node_path.clone())).unwrap();

        assert!(datanodes.iter().any(|n| n.path() == virtual_node_path), "Focal virtual node not found");
        assert!(datanodes.iter().any(|n| n.path() == parent_node_path), "Parent node not found");
        assert_eq!(datanodes.len(), 2, "Should only contain focal and parent");
    }

    #[test]
    fn test_load_context_with_unconnected_node_in_ctx_file() {
        let func_name = "test_load_context_with_unconnected_node_in_ctx_file";
        let ctx = KartaServiceTestContext::new(func_name);
        let root_path = ctx.get_vault_root();
        let focal_dir_path_fs = root_path.join("focal_dir");
        std::fs::create_dir_all(&focal_dir_path_fs).unwrap();

        let focal_path = NodePath::vault().join("focal_dir");
        let unrelated_path = NodePath::vault().join("unrelated_node");

        let (focal_node, unrelated_node) = ctx.with_service_mut(|s| {
            let focal = DataNode::new(&focal_path, NodeTypeId::dir_type());
            let unrelated = DataNode::new(&unrelated_path, NodeTypeId::new("core/text"));
            s.data_mut().insert_nodes(vec![focal.clone(), unrelated.clone()]);
            (focal, unrelated)
        });

        let mut context_file = Context::new(focal_node.uuid());
        context_file.add_node(ViewNode::from_data_node(unrelated_node.clone()));
        ctx.with_service_mut(|s| {
            s.view_mut().save_context(&context_file).unwrap();
        });

        let (datanodes, _, _) = ctx.with_service(|s| s.open_context_from_path(focal_path.clone())).unwrap();

        assert!(datanodes.iter().any(|n| n.path() == focal_path));
        assert!(datanodes.iter().any(|n| n.path() == NodePath::vault()));
        assert!(datanodes.iter().any(|n| n.path() == unrelated_path));
        assert_eq!(datanodes.len(), 3);
    }

    #[test]
    fn test_load_context_with_non_child_connected_node() {
        let func_name = "test_load_context_with_non_child_connected_node";
        let ctx = KartaServiceTestContext::new(func_name);
        let root_path = ctx.get_vault_root();
        std::fs::create_dir_all(root_path.join("dir_A")).unwrap();
        std::fs::create_dir_all(root_path.join("dir_B")).unwrap();

        let path_a = NodePath::vault().join("dir_A");
        let path_b = NodePath::vault().join("dir_B");

        let (node_a, node_b) = ctx.with_service_mut(|s| {
            let a = DataNode::new(&path_a, NodeTypeId::dir_type());
            let b = DataNode::new(&path_b, NodeTypeId::dir_type());
            s.data_mut().insert_nodes(vec![a.clone(), b.clone()]);
            let a_to_b = Edge::new(a.uuid(), b.uuid());
            s.data_mut().insert_edges(vec![a_to_b]);
            (a, b)
        });

        let (datanodes, edges, _) = ctx.with_service(|s| s.open_context_from_path(path_a.clone())).unwrap();

        assert!(datanodes.iter().any(|n| n.path() == path_a));
        assert!(datanodes.iter().any(|n| n.path() == path_b));
        assert!(datanodes.iter().any(|n| n.path() == NodePath::vault()));
        assert!(edges.iter().any(|e| *e.source() == node_a.uuid() && *e.target() == node_b.uuid()));
        assert_eq!(datanodes.len(), 3);
    }

    #[test]
    fn test_load_root_context_shows_only_direct_children() {
        let func_name = "test_load_root_context_shows_only_direct_children";
        let ctx = KartaServiceTestContext::new(func_name);
        
        let virtual_node_path = NodePath::new("/root_virtual_node".into());

        ctx.with_service_mut(|s| {
            let root_node = s.data().open_node(&NodeHandle::Path(NodePath::root())).unwrap();
            let virtual_node = DataNode::new(&virtual_node_path, NodeTypeId::new("core/text"));
            s.data_mut().insert_nodes(vec![virtual_node.clone()]);
        });

        let (datanodes, _, _) = ctx.with_service(|s| s.open_context_from_path(NodePath::root())).unwrap();

        assert!(datanodes.iter().any(|n| n.path() == NodePath::root()));
        assert!(datanodes.iter().any(|n| n.path() == NodePath::vault()));
        assert!(datanodes.iter().any(|n| n.path() == virtual_node_path));
        assert!(!datanodes.iter().any(|n| n.path().buf().to_string_lossy().starts_with("/vault/")));
        assert_eq!(datanodes.len(), 3);
    }
}