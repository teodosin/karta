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

        let absolute_path = path.full(self.vault_fs_path());
        let is_fs_node = absolute_path.exists();
        let is_db_node = self.data().open_node(&NodeHandle::Path(path.clone())).is_ok();

        if path == NodePath::root() {
            self.open_root_context()
        } else if is_db_node && !is_fs_node {
            self.open_virtual_context(&path)
        } else {
            self.open_physical_context(&path)
        }
    }

    /// Opens the root context. This is a special case as it has no parent and its children are determined differently.
    fn open_root_context(&self) -> Result<(Vec<DataNode>, Vec<Edge>, Context), Box<dyn Error>> {
        let mut nodes: HashMap<Uuid, DataNode> = HashMap::new();
        let mut direct_edges: Vec<Edge> = Vec::new();

        let focal_node = self.data().open_node(&NodeHandle::Path(NodePath::root()))?;
        nodes.insert(focal_node.uuid(), focal_node.clone());

        for (child_node, edge) in self.data().open_node_connections(&NodePath::root()) {
            nodes.insert(child_node.uuid(), child_node);
            direct_edges.push(edge);
        }

        self._finalize_context(focal_node, nodes, direct_edges)
    }

    /// Opens a context for a "virtual" node (exists in DB, but not on the filesystem).
    fn open_virtual_context(&self, path: &NodePath) -> Result<(Vec<DataNode>, Vec<Edge>, Context), Box<dyn Error>> {
        let mut nodes: HashMap<Uuid, DataNode> = HashMap::new();
        let mut direct_edges: Vec<Edge> = Vec::new();

        let focal_node = self.data().open_node(&NodeHandle::Path(path.clone()))?;
        nodes.insert(focal_node.uuid(), focal_node.clone());

        // Add parent if it exists.
        if let Some(parent_path) = path.parent() {
            if let Ok(parent_node) = self.data().open_node(&NodeHandle::Path(parent_path)) {
                direct_edges.push(Edge::new(parent_node.uuid(), focal_node.uuid()));
                nodes.insert(parent_node.uuid(), parent_node);
            }
        }
        
        // Add DB connections (children and others).
        for (child_node, edge) in self.data().open_node_connections(path) {
            if *edge.source() == focal_node.uuid() {
                nodes.insert(child_node.uuid(), child_node);
                direct_edges.push(edge);
            }
        }

        self._finalize_context(focal_node, nodes, direct_edges)
    }

    /// Opens a context for a "physical" node (exists on the filesystem).
    fn open_physical_context(&self, path: &NodePath) -> Result<(Vec<DataNode>, Vec<Edge>, Context), Box<dyn Error>> {
        let mut nodes: HashMap<Uuid, DataNode> = HashMap::new();
        let mut direct_edges: Vec<Edge> = Vec::new();
        let absolute_path = path.full(self.vault_fs_path());

        // Get the DB version of the focal node if it exists, otherwise create a provisional one.
        let focal_node = self.data()
            .open_node(&NodeHandle::Path(path.clone()))
            .unwrap_or_else(|_| DataNode::new(path, NodeTypeId::dir_type()));
        nodes.insert(focal_node.uuid(), focal_node.clone());

        // Add parent if it exists.
        if let Some(parent_path) = path.parent() {
            let parent_node = self.data()
                .open_node(&NodeHandle::Path(parent_path.clone()))
                .unwrap_or_else(|_| DataNode::new(&parent_path, NodeTypeId::dir_type()));
            direct_edges.push(Edge::new(parent_node.uuid(), focal_node.uuid()));
            nodes.insert(parent_node.uuid(), parent_node);
        }

        // Add/update nodes from the filesystem if it's a directory.
        if absolute_path.is_dir() {
            let fs_children = fs_reader::destructure_file_path(self.vault_fs_path(), &absolute_path, true)?;
            for child in fs_children {
                direct_edges.push(Edge::new_cont(focal_node.uuid(), child.uuid()));
                nodes.entry(child.uuid()).or_insert(child);
            }
        }
        
        // Add any additional connections from the database.
        for (child_node, edge) in self.data().open_node_connections(path) {
            if *edge.source() == focal_node.uuid() {
                nodes.insert(child_node.uuid(), child_node);
                direct_edges.push(edge);
            }
        }

        self._finalize_context(focal_node, nodes, direct_edges)
    }

    /// Private helper to finalize context creation.
    fn _finalize_context(
        &self,
        focal_node: DataNode,
        mut nodes: HashMap<Uuid, DataNode>,
        direct_edges: Vec<Edge>,
    ) -> Result<(Vec<DataNode>, Vec<Edge>, Context), Box<dyn Error>> {
        
        // Augment with nodes from a saved context file, if one exists.
        if let Ok(saved_context) = self.view.get_context_file(focal_node.uuid()) {
            let saved_node_uuids: Vec<Uuid> = saved_context.viewnodes()
                .iter()
                .map(|vn| vn.uuid())
                .filter(|uuid| !nodes.contains_key(uuid))
                .collect();
            
            if !saved_node_uuids.is_empty() {
                let missing_nodes = self.data().open_nodes_by_uuid(saved_node_uuids)?;
                for node in missing_nodes {
                    nodes.entry(node.uuid()).or_insert(node);
                }
            }
        }
        
        let mut final_edges: Vec<Edge> = Vec::new();
        let mut final_edges_set: HashSet<(Uuid, Uuid)> = HashSet::new();

        for edge in direct_edges {
            if nodes.contains_key(edge.source()) && nodes.contains_key(edge.target()) {
                if final_edges_set.insert((*edge.source(), *edge.target())) {
                    final_edges.push(edge);
                }
            }
        }
        
        let mut final_datanodes: Vec<DataNode> = nodes.values().cloned().collect();
        if focal_node.path() != NodePath::root() && focal_node.path() != NodePath::vault() {
            final_datanodes.retain(|n| n.path() != NodePath::root());
        }

        let parent_uuid = if let Some(parent_path) = focal_node.path().parent() {
            final_datanodes.iter().find(|n| n.path() == parent_path).map(|n| n.uuid())
        } else {
            None
        };
        
        let context = self.view.generate_context(
            focal_node.uuid(),
            parent_uuid,
            final_datanodes.clone(),
        );

        Ok((final_datanodes, final_edges, context))
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

        assert!(datanodes.iter().any(|n| n.path() == NodePath::vault()), "Vault node not found");
        assert!(datanodes.iter().any(|n| n.path() == NodePath::root()), "Root node not found");
        assert!(datanodes.iter().any(|n| n.path() == NodePath::vault().join("test_dir")), "test_dir not found");
        assert!(datanodes.iter().any(|n| n.path() == NodePath::vault().join("test_file.txt")), "test_file.txt not found");

        assert_eq!(datanodes.len(), 4, "Should contain root, vault, test_dir, and test_file.txt");
        assert_eq!(edges.len(), 3, "Should contain root->vault, vault->test_dir, and vault->test_file.txt edges");

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
        
        let virtual_node_path = NodePath::new("root_virtual_node".into());

        ctx.with_service_mut(|s| {
            let root_node = s.data().open_node(&NodeHandle::Path(NodePath::root())).unwrap();
            let virtual_node = DataNode::new(&virtual_node_path, NodeTypeId::new("core/text"));
            s.data_mut().insert_nodes(vec![virtual_node.clone()]);
            let edge = Edge::new(root_node.uuid(), virtual_node.uuid());
            s.data_mut().insert_edges(vec![edge]);
        });

        let (datanodes, _, _) = ctx.with_service(|s| s.open_context_from_path(NodePath::root())).unwrap();

        assert!(datanodes.iter().any(|n| n.path() == NodePath::root()));
        assert!(datanodes.iter().any(|n| n.path() == NodePath::vault()));
        assert!(datanodes.iter().any(|n| n.path() == virtual_node_path));
        assert!(!datanodes.iter().any(|n| n.path().buf().to_string_lossy().starts_with("/vault/")));
        assert_eq!(datanodes.len(), 3);
    }

    #[test]
    fn inserting_nested_node_does_not_pollute_root_context() {
        let func_name = "inserting_nested_node_does_not_pollute_root_context";
        let ctx = KartaServiceTestContext::new(func_name);

        let nested_path = NodePath::new("vault/dir1/dir2".into());
        let node = DataNode::new(&nested_path, NodeTypeId::dir_type());
        
        ctx.with_service_mut(|s| s.data_mut().insert_nodes(vec![node]));

        // Check connections for 'vault'
        let vault_node = ctx.with_service(|s| s.data().open_node(&NodeHandle::Path(NodePath::vault()))).unwrap();
        let vault_connections = ctx.with_service(|s| s.data().open_node_connections(&NodePath::vault()));
        
        println!("\n--- Vault Connections (Total: {}) ---", vault_connections.len());
        for (node, edge) in &vault_connections {
            println!("  - Node: {:?}, Edge: {:?} -> {:?}", node.path(), edge.source(), edge.target());
        }

        // The only *child* of vault should be 'dir1'.
        let vault_children: Vec<_> = vault_connections.iter().filter(|(_, edge)| *edge.source() == vault_node.uuid()).collect();
        assert_eq!(vault_children.len(), 1, "Vault should have exactly one child: 'dir1'");
        assert!(vault_children.iter().any(|(n, _)| n.path().name() == "dir1"), "Child 'dir1' not found for vault");

        // CRITICAL: Check connections for 'root'
        let root_connections = ctx.with_service(|s| s.data().open_node_connections(&NodePath::root()));

        println!("\n--- Root Connections (Total: {}) ---", root_connections.len());
        for (node, edge) in &root_connections {
            println!("  - Node: {:?}, Edge: {:?} -> {:?}", node.path(), edge.source(), edge.target());
        }
        
        // The only direct children of root should be archetypes like 'vault'. 'dir1' should NOT be a direct child.
        let has_dir1_as_child_of_root = root_connections.iter().any(|(n, edge)| {
            n.path().name() == "dir1" && *edge.source() == crate::elements::node::ROOT_UUID
        });
        
        assert!(!has_dir1_as_child_of_root, "Root context should not contain 'dir1' as a direct child after nested insertion.");
    }


}