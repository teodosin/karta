use std::{
    collections::{HashMap, HashSet},
    error::Error,
    path::PathBuf,
    str::FromStr,
    sync::Arc,
};
use tracing::info;
use uuid::Uuid;

use tokio::sync::RwLock;

use crate::{
    context::{context::Context, context_db::ContextDb},
    elements::{attribute::AttrValue, node_path::NodeHandle},
    fs_reader,
    prelude::*,
};

use super::edge_endpoints::{CreateEdgePayload, DeleteEdgePayload};

pub struct KartaService {
    vault_fs_path: PathBuf,
    storage_dir: PathBuf,
    data: GraphAgdb,
    view: ContextDb,
}

impl KartaService {
    pub fn new(name: &str, vault_fs_path: PathBuf, storage_dir: PathBuf) -> Self {
        // Check if the storage dir is called .karta.
        // If not, create it.
        // This might be a bit crude, but it will do for now.
        let mut storage_dir = storage_dir;
        if storage_dir.file_name().unwrap() != ".karta" {
            storage_dir = storage_dir.join(".karta");
            std::fs::create_dir_all(&storage_dir).unwrap();
        }

        let data = GraphAgdb::new(name, vault_fs_path.clone(), storage_dir.clone());
        let view = ContextDb::new(name.to_owned(), vault_fs_path.clone(), storage_dir.clone());

        Self {
            vault_fs_path,
            storage_dir,
            data,
            view,
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

    pub fn create_edges(
        &mut self,
        payload: Vec<CreateEdgePayload>,
    ) -> Result<Vec<CreateEdgePayload>, String> {
        let mut created_edges_payload = Vec::new();
        let mut edges_to_insert = Vec::new();

        for edge_payload in payload {
            let source_path = NodePath::from_alias(&edge_payload.source_path);
            let target_path = NodePath::from_alias(&edge_payload.target_path);

            let source_node = match self
                .data()
                .open_node(&NodeHandle::Path(source_path.clone()))
            {
                Ok(node) => node,
                Err(_) => {
                    let node =
                        fs_reader::destructure_single_path(self.vault_fs_path(), &source_path)
                            .unwrap();
                    self.data_mut().insert_nodes(vec![node.clone()]);
                    node
                }
            };

            let target_node = match self
                .data()
                .open_node(&NodeHandle::Path(target_path.clone()))
            {
                Ok(node) => node,
                Err(_) => {
                    let node =
                        fs_reader::destructure_single_path(self.vault_fs_path(), &target_path)
                            .unwrap();
                    self.data_mut().insert_nodes(vec![node.clone()]);
                    node
                }
            };

            let mut new_edge = Edge::new(source_node.uuid(), target_node.uuid());

            let attributes: Vec<Attribute> = (&edge_payload.attributes)
                .into_iter()
                .map(|(key, value)| match value {
                    serde_json::Value::Number(n) => {
                        if let Some(f) = n.as_f64() {
                            Attribute::new_float(key.clone(), f as f32)
                        } else if let Some(u) = n.as_u64() {
                            Attribute::new_uint(key.clone(), u as u32)
                        } else {
                            Attribute::new_string(key.clone(), n.to_string())
                        }
                    }
                    serde_json::Value::String(s) => Attribute::new_string(key.clone(), s.clone()),
                    serde_json::Value::Bool(b) => Attribute::new_uint(key.clone(), *b as u32),
                    _ => Attribute::new_string(key.clone(), value.to_string()),
                })
                .collect();

            new_edge.set_attributes(attributes);
            edges_to_insert.push(new_edge);
            created_edges_payload.push(edge_payload);
        }

        dbg!(&edges_to_insert);
        self.data.insert_edges(edges_to_insert);

        Ok(created_edges_payload)
    }

    pub fn delete_edges(&mut self, payload: Vec<DeleteEdgePayload>) -> Result<(), Box<dyn Error>> {
        let edges_to_delete: Vec<(Uuid, Uuid)> =
            payload.into_iter().map(|p| (p.source, p.target)).collect();
        self.data.delete_edges(&edges_to_delete)
    }

    pub fn reconnect_edge(
        &mut self,
        old_from: &Uuid,
        old_to: &Uuid,
        new_from: &Uuid,
        new_to: &Uuid,
        new_from_path: &str,
        new_to_path: &str,
    ) -> Result<Edge, Box<dyn Error>> {
        let new_from_path = NodePath::from_alias(new_from_path);
        let new_to_path = NodePath::from_alias(new_to_path);

        // Ensure the new nodes are indexed before attempting to reconnect.
        if self.data.open_node(&NodeHandle::Uuid(*new_from)).is_err() {
            let node = fs_reader::destructure_single_path(self.vault_fs_path(), &new_from_path)?;
            self.data_mut().insert_nodes(vec![node]);
        }

        if self.data.open_node(&NodeHandle::Uuid(*new_to)).is_err() {
            let node = fs_reader::destructure_single_path(self.vault_fs_path(), &new_to_path)?;
            self.data_mut().insert_nodes(vec![node]);
        }

        self.data.reconnect_edge(old_from, old_to, new_from, new_to)
    }

    pub fn get_paths(&self, only_indexed: bool) -> Result<Vec<String>, Box<dyn Error>> {
        if only_indexed {
            self.data.get_all_indexed_paths()
        } else {
            let physical_paths = fs_reader::get_all_paths(self.vault_fs_path())?;
            let indexed_paths = self.data.get_all_indexed_paths()?;

            let mut all_paths = HashSet::new();
            all_paths.extend(physical_paths);
            all_paths.extend(indexed_paths);

            Ok(all_paths.into_iter().collect())
        }
    }

    /// Opens a single node by its handle, reconciling filesystem and database information.
    ///
    /// This is the definitive method for retrieving a `DataNode`.
    /// - If the handle is a `Path`, it checks the filesystem first. If an entry exists,
    ///   it creates a provisional `DataNode` and then attempts to augment it with
    ///   data from the database (e.g., UUID, attributes).
    /// - If the handle is a `UUID`, it fetches the node directly from the database.
    pub fn open_node(&self, handle: &NodeHandle) -> Result<DataNode, Box<dyn Error>> {
        match handle {
            NodeHandle::Uuid(uuid) => self.data().open_node(&NodeHandle::Uuid(*uuid)),
            NodeHandle::Path(path) => {
                let absolute_path = path.full(self.vault_fs_path());
                let fs_exists = absolute_path.exists();

                // Try to get the node from the database first.
                let db_node_result = self.data().open_node(&NodeHandle::Path(path.clone()));

                if fs_exists {
                    // Filesystem takes precedence for path/type/existence, but preserve UUID from DB if it exists.
                    if let Ok(db_node) = db_node_result {
                        // Node exists in both FS and DB - prefer DB node which has the correct UUID
                        // The database should already have the correct path/type from previous indexing
                        Ok(db_node)
                    } else {
                        // Node exists only on FS - create new node with new UUID
                        let node = fs_reader::destructure_single_path(self.vault_fs_path(), &path)?;
                        Ok(node)
                    }
                } else {
                    // If it doesn't exist on the filesystem, it must exist in the DB.
                    db_node_result
                }
            }
        }
    }

    /// Opens a context's Data and View.
    /// This is the main function for opening a context.
    /// Reconciles indexed data from the database with physical data from the filesystem.
    /// Filesystem state (existence, name, type) takes precedence for the returned view.
    /// Karta-specific attributes and UUIDs are sourced from the database if an entry exists.
    /// This function is read-only regarding database writes.
    pub fn open_context_from_path(
        &self,
        path: NodePath,
    ) -> Result<(Vec<DataNode>, Vec<Edge>, Context), Box<dyn Error>> {
        let absolute_path = path.full(self.vault_fs_path());
        let is_fs_node = absolute_path.exists();
        let is_db_node = self
            .data()
            .open_node(&NodeHandle::Path(path.clone()))
            .is_ok();

        println!(
            "[open_context_from_path] Routing path: {:?}. is_fs_node: {}, is_db_node: {}",
            path, is_fs_node, is_db_node
        );

        if path == NodePath::root() {
            println!("[open_context_from_path] -> Routing to open_root_context");
            self.open_root_context()
        } else if is_db_node && !is_fs_node {
            println!("[open_context_from_path] -> Routing to open_virtual_context");
            self.open_virtual_context(&path)
        } else {
            println!("[open_context_from_path] -> Routing to open_physical_context");
            self.open_physical_context(&path)
        }
    }

    /// Opens the root context. This is a special case as it has no parent and its children are determined differently.
    fn open_root_context(&self) -> Result<(Vec<DataNode>, Vec<Edge>, Context), Box<dyn Error>> {
        println!("[open_root_context] Opening root context.");
        let mut nodes: HashMap<Uuid, DataNode> = HashMap::new();
        let mut direct_edges: Vec<Edge> = Vec::new();

        let focal_node = self.data().open_node(&NodeHandle::Path(NodePath::root()))?;
        println!(
            "[open_root_context] -> Focal node (root) found: {}",
            focal_node.uuid()
        );
        nodes.insert(focal_node.uuid(), focal_node.clone());

        for (child_node, edge) in self.data().open_node_connections(&NodePath::root()) {
            println!(
                "[open_root_context] -> Found connected node: path='{}', uuid='{}'",
                child_node.path().alias(),
                child_node.uuid()
            );
            nodes.insert(child_node.uuid(), child_node);
            direct_edges.push(edge);
        }

        println!(
            "[open_root_context] -> Total nodes for finalization: {}",
            nodes.len()
        );
        println!(
            "[open_root_context] -> Total edges for finalization: {}",
            direct_edges.len()
        );
        self._finalize_context(focal_node, nodes, direct_edges)
    }

    /// Opens a context for a "virtual" node (exists in DB, but not on the filesystem).
    fn open_virtual_context(
        &self,
        path: &NodePath,
    ) -> Result<(Vec<DataNode>, Vec<Edge>, Context), Box<dyn Error>> {
        let mut nodes: HashMap<Uuid, DataNode> = HashMap::new();
        let mut direct_edges: Vec<Edge> = Vec::new();

        let focal_node = self.data().open_node(&NodeHandle::Path(path.clone()))?;
        nodes.insert(focal_node.uuid(), focal_node.clone());

        // Add parent if it exists.
        if let Some(parent_path) = path.parent() {
            if let Ok(parent_node) = self.data().open_node(&NodeHandle::Path(parent_path)) {
                let edge = self
                    .data()
                    .get_edge_strict(&parent_node.uuid(), &focal_node.uuid())
                    .unwrap_or_else(|_| Edge::new_cont(parent_node.uuid(), focal_node.uuid()));
                direct_edges.push(edge);
                nodes.insert(parent_node.uuid(), parent_node);
            }
        }

        // Add DB connections (children and others).
        for (child_node, edge) in self.data().open_node_connections(path) {
            nodes.insert(child_node.uuid(), child_node);
            direct_edges.push(edge);
        }

        self._finalize_context(focal_node, nodes, direct_edges)
    }

    /// Opens a context for a "physical" node (exists on the filesystem).
    fn open_physical_context(
        &self,
        path: &NodePath,
    ) -> Result<(Vec<DataNode>, Vec<Edge>, Context), Box<dyn Error>> {
        let mut nodes: HashMap<Uuid, DataNode> = HashMap::new();
        let mut direct_edges: Vec<Edge> = Vec::new();
        let absolute_path = path.full(self.vault_fs_path());

        // Get the focal node, which must exist on the filesystem.
        let focal_node = self.open_node(&NodeHandle::Path(path.clone()))?;
        nodes.insert(focal_node.uuid(), focal_node.clone());

        // Add parent if it exists.
        if let Some(parent_path) = path.parent() {
            let parent_node = self.open_node(&NodeHandle::Path(parent_path.clone()))?;
            let edge = self
                .data()
                .get_edge_strict(&parent_node.uuid(), &focal_node.uuid())
                .unwrap_or_else(|_| Edge::new_cont(parent_node.uuid(), focal_node.uuid()));
            direct_edges.push(edge);
            nodes.insert(parent_node.uuid(), parent_node);
        }

        // Add/update nodes from the filesystem if it's a directory.
        if absolute_path.is_dir() {
            // Instead of using fs_reader::destructure_file_path which creates new nodes with new UUIDs,
            // we need to read the directory and use open_node for each child to preserve existing UUIDs
            if let Ok(dir_entries) = std::fs::read_dir(&absolute_path) {
                for entry in dir_entries {
                    if let Ok(entry) = entry {
                        let entry_path = entry.path();
                        
                        // Skip .karta directory
                        if entry_path.file_name().map_or(false, |name| name == ".karta") {
                            continue;
                        }
                        
                        // Create NodePath for this child
                        let child_node_path = NodePath::from_dir_path(self.vault_fs_path(), &entry_path);
                        
                        // Use open_node to get the child, which will preserve UUIDs if they exist
                        match self.open_node(&NodeHandle::Path(child_node_path)) {
                            Ok(child_node) => {
                                direct_edges.push(Edge::new_cont(focal_node.uuid(), child_node.uuid()));
                                nodes.entry(child_node.uuid()).or_insert(child_node);
                            }
                            Err(e) => {
                                println!("Failed to open child node {}: {}", entry_path.display(), e);
                            }
                        }
                    }
                }
            }
        }

        // Add any additional connections from the database.
        for (child_node, edge) in self.data().open_node_connections(path) {
            nodes.insert(child_node.uuid(), child_node);
            direct_edges.push(edge);
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
        println!(
            "[_finalize_context] Finalizing context for focal node: '{}'",
            focal_node.path().alias()
        );
        println!("[_finalize_context] -> Initial node count: {}", nodes.len());
        println!(
            "[_finalize_context] -> Initial edge count: {}",
            direct_edges.len()
        );

        if let Ok(saved_context) = self.view.get_context_file(focal_node.uuid()) {
            println!("[_finalize_context] -> Found saved context file. Augmenting nodes.");
            let saved_node_uuids: Vec<Uuid> = saved_context
                .viewnodes()
                .iter()
                .map(|vn| vn.uuid())
                .filter(|uuid| !nodes.contains_key(uuid))
                .collect();

            if !saved_node_uuids.is_empty() {
                println!(
                    "[_finalize_context] -> Fetching {} missing nodes from DB.",
                    saved_node_uuids.len()
                );
                let missing_nodes = self.data().open_nodes_by_uuid(saved_node_uuids)?;
                for node in missing_nodes {
                    println!(
                        "[_finalize_context] -> Adding node from saved context: '{}' ({})",
                        node.path().alias(),
                        node.uuid()
                    );
                    nodes.entry(node.uuid()).or_insert(node);
                }
            }
        }

        let mut final_edges: Vec<Edge> = Vec::new();
        let mut final_edges_set: HashSet<(Uuid, Uuid)> = HashSet::new();

        for edge in direct_edges {
            if edge.source() != edge.target()
                && nodes.contains_key(edge.source())
                && nodes.contains_key(edge.target())
            {
                if final_edges_set.insert((*edge.source(), *edge.target())) {
                    final_edges.push(edge.clone());
                }
            }
        }
        println!(
            "[_finalize_context] -> Edge count after filtering direct edges: {}",
            final_edges.len()
        );

        let final_datanodes: Vec<DataNode> = nodes.values().cloned().collect();
        let final_datanode_uuids: Vec<Uuid> = final_datanodes.iter().map(|n| n.uuid()).collect();

        // Get all edges between the nodes in the context.
        if let Ok(interconnect_edges) = self.data.get_edges_between_nodes(&final_datanode_uuids) {
            println!(
                "[_finalize_context] -> Found {} interconnecting edges.",
                interconnect_edges.len()
            );
            for edge in interconnect_edges {
                if final_edges_set.insert((*edge.source(), *edge.target())) {
                    final_edges.push(edge);
                }
            }
        }
        println!(
            "[_finalize_context] -> Total final edge count: {}",
            final_edges.len()
        );

        let parent_uuid = if let Some(parent_path) = focal_node.path().parent() {
            final_datanodes
                .iter()
                .find(|n| n.path() == parent_path)
                .map(|n| n.uuid())
        } else {
            None
        };

        println!(
            "[_finalize_context] -> Calling generate_context with {} nodes.",
            final_datanodes.len()
        );
        let context =
            self.view
                .generate_context(focal_node.uuid(), parent_uuid, final_datanodes.clone());

        Ok((final_datanodes, final_edges, context))
    }

    pub fn move_node(
        &mut self,
        node_path: &NodePath,
        target_path: &NodePath,
    ) -> Result<(), Box<dyn Error>> {
        let source_fs_path = node_path.full(self.vault_fs_path());
        let is_physical_node = source_fs_path.exists();

        // 1. Ensure the target parent node is indexed before we try to use it.
        if self
            .data
            .open_node(&NodeHandle::Path(target_path.clone()))
            .is_err()
        {
            let target_node_data =
                fs_reader::destructure_single_path(self.vault_fs_path(), target_path)?;
            self.data.insert_nodes(vec![target_node_data]);
        }

        // 2. Do database updates recursively
        self.move_node_in_database(node_path, target_path)?;

        // 3. Move on filesystem (only once, at the top level)
        if is_physical_node {
            let target_fs_path = target_path
                .full(self.vault_fs_path())
                .join(node_path.name());
            std::fs::rename(&source_fs_path, &target_fs_path)?;
        } else {
            // If the node is virtual, the target parent must be a physical directory
            let target_parent_fs_path = target_path.full(self.vault_fs_path());
            if !target_parent_fs_path.is_dir() {
                return Err("Virtual nodes can only be moved to physical directories.".into());
            }
        }

        Ok(())
    }

    /// Move a node to a target location with optional renaming for collision resolution
    pub fn move_node_with_rename(
        &mut self,
        node_path: &NodePath,
        target_parent_path: &NodePath,
        new_name: Option<&str>,
    ) -> Result<NodePath, Box<dyn Error>> {
        let source_fs_path = node_path.full(self.vault_fs_path());
        let is_physical_node = source_fs_path.exists();

        // Determine the final name (either provided or original)
        let original_name = node_path.name();
        let final_name = new_name.unwrap_or(&original_name);
        let target_node_path = target_parent_path.join(final_name);

        // 1. Check if target parent is indexed, and index it if it exists on filesystem
        let target_parent_indexed = self
            .data
            .open_node(&NodeHandle::Path(target_parent_path.clone()))
            .is_ok();
            
        if !target_parent_indexed {
            // Try to index the target parent if it exists on filesystem
            let target_parent_fs_path = target_parent_path.full(self.vault_fs_path());
            if target_parent_fs_path.exists() && target_parent_fs_path.is_dir() {
                let target_node_data =
                    fs_reader::destructure_single_path(self.vault_fs_path(), target_parent_path)?;
                self.data.insert_nodes(vec![target_node_data]);
            }
            // Note: If target doesn't exist on filesystem, the move will fail during filesystem operation
        }

        // 2. Do database updates recursively with the final target path (only if source is indexed)
        if let Ok(_) = self.data.open_node(&NodeHandle::Path(node_path.clone())) {
            self.move_node_in_database_with_target(node_path, &target_node_path)?;
        }

        // 3. Move on filesystem (only once, at the top level)
        if is_physical_node {
            let target_fs_path = target_parent_path
                .full(self.vault_fs_path())
                .join(final_name);
            std::fs::rename(&source_fs_path, &target_fs_path)?;
        } else {
            // If the node is virtual, the target parent must be a physical directory
            let target_parent_fs_path = target_parent_path.full(self.vault_fs_path());
            if !target_parent_fs_path.is_dir() {
                return Err("Virtual nodes can only be moved to physical directories.".into());
            }
        }

        Ok(target_node_path)
    }

    /// Rename a node in place, handling both filesystem and database updates
    pub fn rename_node(
        &mut self,
        node_path: &NodePath,
        new_name: &str,
    ) -> Result<NodePath, Box<dyn Error>> {
        // Get the parent path
        let parent_path = node_path.parent()
            .ok_or("Cannot rename root node")?;
        
        // Check for name collision and generate unique name if needed
        let unique_name = self.generate_unique_name(&parent_path, new_name);
        
        // Use move_node_with_rename to the same parent with the unique name
        self.move_node_with_rename(node_path, &parent_path, Some(&unique_name))
    }

    /// Generate a unique name by appending a counter if the original name already exists
    fn generate_unique_name(&self, parent_path: &NodePath, original_name: &str) -> String {
        let mut name = original_name.to_string();
        let mut final_path = parent_path.join(&name);
        let mut counter = 2;

        // Loop until we find a unique path
        while self.data.open_node(&NodeHandle::Path(final_path.clone())).is_ok() {
            // Handle file extensions properly
            if let Some(dot_pos) = original_name.rfind('.') {
                // Split name and extension
                let base_name = &original_name[..dot_pos];
                let extension = &original_name[dot_pos..];
                name = format!("{}_{}{}", base_name, counter, extension);
            } else {
                // No extension, just append counter
                name = format!("{}_{}", original_name, counter);
            }
            final_path = parent_path.join(&name);
            counter += 1;
        }

        name
    }

    /// Helper function to recursively update database paths without touching the filesystem
    fn move_node_in_database(
        &mut self,
        node_path: &NodePath,
        target_path: &NodePath,
    ) -> Result<(), Box<dyn Error>> {
        // If the node is indexed in the DB, handle database updates recursively
        if let Ok(node) = self.data.open_node(&NodeHandle::Path(node_path.clone())) {
            // Get current parent UUID by looking at edges before reparenting
            let mut old_parent_uuid = None;
            let connections = self.data().open_node_connections(&node_path);
            for (connected_node, edge) in connections {
                if edge.is_contains() && *edge.target() == node.uuid() {
                    old_parent_uuid = Some(*edge.source());
                    break;
                }
            }
            
            let old_parent_uuid = old_parent_uuid.ok_or_else(|| "Could not find old parent UUID")?;
            let new_parent_node = self
                .data
                .open_node(&NodeHandle::Path(target_path.clone()))?;

            // Update this node's path attribute in the database FIRST
            let new_path = target_path.join(node_path.name().as_str());
            let new_path_attribute =
                Attribute::new_string("path".to_string(), new_path.alias().to_string());
            self.data
                .update_node_attributes(node.uuid(), vec![new_path_attribute])?;

            // Reconnect the 'contains' edge from the old parent to the new parent
            // Use the specialized reparent_node method for contains edges
            self.data.reparent_node(&node.uuid(), &old_parent_uuid, &new_parent_node.uuid())?;

            // Handle children AFTER updating this node's path
            if node.is_dir() {
                // Get direct children using the updated path
                let connections = self.data().open_node_connections(&new_path);
                
                for (child_node, edge) in connections {
                    // Only process "contains" edges (parent-child relationships) where this node is the parent
                    if edge.is_contains() && *edge.source() == node.uuid() {
                        let child_path = child_node.path();
                        let new_child_target = new_path.clone(); // Use the updated path as the target
                        
                        // Recursively update the child in database only
                        self.move_node_in_database(&child_path, &new_child_target)?;
                    }
                }
            }
        } else {
            // Node not found in database - this is OK for unindexed nodes
        }

        Ok(())
    }

    /// Helper function to recursively update database paths to a specific target path (for renaming moves)
    fn move_node_in_database_with_target(
        &mut self,
        node_path: &NodePath,
        target_path: &NodePath,
    ) -> Result<(), Box<dyn Error>> {
        // If the node is indexed in the DB, handle database updates recursively
        if let Ok(node) = self.data.open_node(&NodeHandle::Path(node_path.clone())) {
            // Get current parent UUID by looking at edges before reparenting
            let mut old_parent_uuid = None;
            let connections = self.data().open_node_connections(&node_path);
            for (connected_node, edge) in connections {
                if edge.is_contains() && *edge.target() == node.uuid() {
                    old_parent_uuid = Some(*edge.source());
                    break;
                }
            }
            
            let old_parent_uuid = old_parent_uuid.ok_or_else(|| "Could not find old parent UUID")?;
            let new_parent_path = target_path.parent().ok_or_else(|| "Target path has no parent")?;
            let new_parent_node = self
                .data
                .open_node(&NodeHandle::Path(new_parent_path.clone()))?;

            // Update this node's path and name attributes in the database FIRST
            let new_path_attribute =
                Attribute::new_string("path".to_string(), target_path.alias().to_string());
            let new_name_attribute =
                Attribute::new_string("name".to_string(), target_path.name());
            self.data
                .update_node_attributes(node.uuid(), vec![new_path_attribute, new_name_attribute])?;

            // Reconnect the 'contains' edge from the old parent to the new parent
            // Use the specialized reparent_node method for contains edges
            self.data.reparent_node(&node.uuid(), &old_parent_uuid, &new_parent_node.uuid())?;

            // Handle children AFTER updating this node's path
            if node.is_dir() {
                // Get direct children using the updated path
                let connections = self.data().open_node_connections(&target_path);
                
                for (child_node, edge) in connections {
                    // Only process "contains" edges (parent-child relationships) where this node is the parent
                    if edge.is_contains() && *edge.source() == node.uuid() {
                        let child_original_path = child_node.path();
                        let child_name = child_original_path.name();
                        let new_child_target = target_path.join(&child_name);
                        
                        // Recursively update the child in database with the exact target path
                        self.move_node_in_database_with_target(&child_original_path, &new_child_target)?;
                    }
                }
            }
        } else {
            // Node not found in database - this is OK for unindexed nodes
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        context::context::Context,
        elements::{
            attribute::{AttrValue, Attribute},
            node_path::NodeHandle,
            view_node::ViewNode,
        },
        fs_reader,
        graph_traits::{graph_edge::GraphEdge, graph_node::GraphNodes},
        prelude::*,
        utils::utils::KartaServiceTestContext,
    };

    #[test]
    fn opening_directory_spawns_viewnodes_without_indexing() {
        let func_name = "opening_directory_spawns_viewnodes_without_indexing";
        let ctx = KartaServiceTestContext::new(func_name);

        ctx.create_dir_in_vault("test_dir/nested_dir").unwrap();
        ctx.create_file_in_vault("test_file.txt", b"").unwrap();

        // --- Part 1: Test opening the vault context ---
        let (datanodes, edges, _) = ctx
            .with_service(|s| s.open_context_from_path(NodePath::vault()))
            .unwrap();

        assert!(
            datanodes.iter().any(|n| n.path() == NodePath::vault()),
            "Vault node not found"
        );
        assert!(
            datanodes.iter().any(|n| n.path() == NodePath::root()),
            "Root node not found"
        );
        assert!(
            datanodes
                .iter()
                .any(|n| n.path() == NodePath::vault().join("test_dir")),
            "test_dir not found"
        );
        assert!(
            datanodes
                .iter()
                .any(|n| n.path() == NodePath::vault().join("test_file.txt")),
            "test_file.txt not found"
        );

        assert_eq!(
            datanodes.len(),
            4,
            "Should contain root, vault, test_dir, and test_file.txt"
        );
        assert_eq!(
            edges.len(),
            3,
            "Should contain root->vault, vault->test_dir, and vault->test_file.txt edges"
        );

        // --- Part 2: Test opening a deeper context to check for grandparent bug ---
        let (datanodes_deeper, _, _) = ctx
            .with_service(|s| s.open_context_from_path(NodePath::vault().join("test_dir")))
            .unwrap();

        assert!(
            datanodes_deeper
                .iter()
                .any(|n| n.path() == NodePath::vault().join("test_dir")),
            "Focal node test_dir missing"
        );
        assert!(
            datanodes_deeper
                .iter()
                .any(|n| n.path() == NodePath::vault()),
            "Parent node vault missing"
        );
        assert!(
            datanodes_deeper
                .iter()
                .any(|n| n.path() == NodePath::vault().join("test_dir").join("nested_dir")),
            "Child node nested_dir missing"
        );
        assert!(
            !datanodes_deeper
                .iter()
                .any(|n| n.path() == NodePath::root()),
            "Grandparent root node should NOT be present"
        );
        assert_eq!(
            datanodes_deeper.len(),
            3,
            "Should only contain focal, parent, and child"
        );
    }

    #[test]
    fn test_load_filesystem_context_with_db_entries() {
        let func_name = "test_load_filesystem_context_with_db_entries";
        let ctx = KartaServiceTestContext::new(func_name);

        ctx.create_dir_in_vault("another_dir").unwrap();
        ctx.create_file_in_vault("another_dir/another_file.txt", b"content")
            .unwrap();

        let file_node_path = NodePath::new("vault/another_dir/another_file.txt".into());
        ctx.with_service_mut(|s| {
            let mut file_node = DataNode::new(&file_node_path, NodeTypeId::file_type());
            file_node.set_attributes(vec![Attribute::new_string(
                "custom_attr".to_string(),
                "db_value".to_string(),
            )]);
            s.data_mut().insert_nodes(vec![file_node]);
        });

        let (datanodes, _, _) = ctx
            .with_service(|s| s.open_context_from_path(NodePath::vault().join("another_dir")))
            .unwrap();

        let fetched_file_node = datanodes
            .iter()
            .find(|n| n.path() == file_node_path)
            .expect("File node not found in context");
        let binding = fetched_file_node.attributes();
        let attr = binding
            .iter()
            .find(|a| a.name == "custom_attr")
            .expect("Custom attribute not found");
        assert_eq!(attr.value, AttrValue::String("db_value".to_string()));
        assert_eq!(
            datanodes.len(),
            3,
            "Should contain focal, parent, and child"
        );
    }

    #[test]
    fn test_load_virtual_node_context() {
        let func_name = "test_load_virtual_node_context";
        let ctx = KartaServiceTestContext::new(func_name);
        ctx.create_dir_in_vault("parent_dir").unwrap();

        let parent_node_path = NodePath::new("vault/parent_dir".into());
        let virtual_node_path = parent_node_path.join("virtual_text_node");

        ctx.with_service_mut(|s| {
            let parent_node = DataNode::new(&parent_node_path, NodeTypeId::dir_type());
            let virtual_node = DataNode::new(&virtual_node_path, NodeTypeId::new("core/text"));
            s.data_mut()
                .insert_nodes(vec![parent_node.clone(), virtual_node.clone()]);
        });

        let (datanodes, _, _) = ctx
            .with_service(|s| s.open_context_from_path(virtual_node_path.clone()))
            .unwrap();

        assert!(
            datanodes.iter().any(|n| n.path() == virtual_node_path),
            "Focal virtual node not found"
        );
        assert!(
            datanodes.iter().any(|n| n.path() == parent_node_path),
            "Parent node not found"
        );
        assert_eq!(datanodes.len(), 2, "Should only contain focal and parent");
    }

    #[test]
    fn test_load_context_with_unconnected_node_in_ctx_file() {
        let func_name = "test_load_context_with_unconnected_node_in_ctx_file";
        let ctx = KartaServiceTestContext::new(func_name);
        ctx.create_dir_in_vault("focal_dir").unwrap();

        let focal_path = NodePath::new("vault/focal_dir".into());
        let unrelated_path = NodePath::vault().join("unrelated_node");

        let (focal_node, unrelated_node) = ctx.with_service_mut(|s| {
            let focal = DataNode::new(&focal_path, NodeTypeId::dir_type());
            let unrelated = DataNode::new(&unrelated_path, NodeTypeId::new("core/text"));
            s.data_mut()
                .insert_nodes(vec![focal.clone(), unrelated.clone()]);
            (focal, unrelated)
        });

        let mut context_file = Context::new(focal_node.uuid());
        context_file.add_node(ViewNode::from_data_node(unrelated_node.clone()));
        ctx.with_service_mut(|s| {
            s.view_mut().save_context(&context_file).unwrap();
        });

        let (datanodes, _, _) = ctx
            .with_service(|s| s.open_context_from_path(focal_path.clone()))
            .unwrap();

        assert!(datanodes.iter().any(|n| n.path() == focal_path));
        assert!(datanodes.iter().any(|n| n.path() == NodePath::vault()));
        assert!(datanodes.iter().any(|n| n.path() == unrelated_path));
        assert_eq!(datanodes.len(), 3);
    }

    #[test]
    fn test_load_context_with_non_child_connected_node() {
        let func_name = "test_load_context_with_non_child_connected_node";
        let ctx = KartaServiceTestContext::new(func_name);
        ctx.create_dir_in_vault("dir_A").unwrap();
        ctx.create_dir_in_vault("dir_B").unwrap();

        let path_a = NodePath::new("vault/dir_A".into());
        let path_b = NodePath::new("vault/dir_B".into());

        let (node_a, node_b) = ctx.with_service_mut(|s| {
            let a = DataNode::new(&path_a, NodeTypeId::dir_type());
            let b = DataNode::new(&path_b, NodeTypeId::dir_type());
            s.data_mut().insert_nodes(vec![a.clone(), b.clone()]);
            let a_to_b = Edge::new(a.uuid(), b.uuid());
            s.data_mut().insert_edges(vec![a_to_b]);
            (a, b)
        });

        let (datanodes, edges, _) = ctx
            .with_service(|s| s.open_context_from_path(path_a.clone()))
            .unwrap();

        assert!(datanodes.iter().any(|n| n.path() == path_a));
        assert!(datanodes.iter().any(|n| n.path() == path_b));
        assert!(datanodes.iter().any(|n| n.path() == NodePath::vault()));
        assert!(edges
            .iter()
            .any(|e| *e.source() == node_a.uuid() && *e.target() == node_b.uuid()));
        assert_eq!(datanodes.len(), 3);
    }

    #[test]
    fn test_load_root_context_shows_only_direct_children() {
        let func_name = "test_load_root_context_shows_only_direct_children";
        let ctx = KartaServiceTestContext::new(func_name);

        let virtual_node_path = NodePath::new("root_virtual_node".into());

        ctx.with_service_mut(|s| {
            let root_node = s
                .data()
                .open_node(&NodeHandle::Path(NodePath::root()))
                .unwrap();
            let virtual_node = DataNode::new(&virtual_node_path, NodeTypeId::new("core/text"));
            s.data_mut().insert_nodes(vec![virtual_node.clone()]);
            let edge = Edge::new(root_node.uuid(), virtual_node.uuid());
            s.data_mut().insert_edges(vec![edge]);
        });

        let (datanodes, _, _) = ctx
            .with_service(|s| s.open_context_from_path(NodePath::root()))
            .unwrap();

        assert!(datanodes.iter().any(|n| n.path() == NodePath::root()));
        assert!(datanodes.iter().any(|n| n.path() == NodePath::vault()));
        assert!(datanodes.iter().any(|n| n.path() == virtual_node_path));
        assert!(!datanodes
            .iter()
            .any(|n| n.path().buf().to_string_lossy().starts_with("/vault/")));
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
        let vault_node = ctx
            .with_service(|s| s.data().open_node(&NodeHandle::Path(NodePath::vault())))
            .unwrap();
        let vault_connections =
            ctx.with_service(|s| s.data().open_node_connections(&NodePath::vault()));

        println!(
            "\n--- Vault Connections (Total: {}) ---",
            vault_connections.len()
        );
        for (node, edge) in &vault_connections {
            println!(
                "  - Node: {:?}, Edge: {:?} -> {:?}",
                node.path(),
                edge.source(),
                edge.target()
            );
        }

        // The only *child* of vault should be 'dir1'.
        let vault_children: Vec<_> = vault_connections
            .iter()
            .filter(|(_, edge)| *edge.source() == vault_node.uuid())
            .collect();
        assert_eq!(
            vault_children.len(),
            1,
            "Vault should have exactly one child: 'dir1'"
        );
        assert!(
            vault_children
                .iter()
                .any(|(n, _)| n.path().name() == "dir1"),
            "Child 'dir1' not found for vault"
        );

        // CRITICAL: Check connections for 'root'
        let root_connections =
            ctx.with_service(|s| s.data().open_node_connections(&NodePath::root()));

        println!(
            "\n--- Root Connections (Total: {}) ---",
            root_connections.len()
        );
        for (node, edge) in &root_connections {
            println!(
                "  - Node: {:?}, Edge: {:?} -> {:?}",
                node.path(),
                edge.source(),
                edge.target()
            );
        }

        // The only direct children of root should be archetypes like 'vault'. 'dir1' should NOT be a direct child.
        let has_dir1_as_child_of_root = root_connections.iter().any(|(n, edge)| {
            n.path().name() == "dir1" && *edge.source() == crate::elements::node::ROOT_UUID
        });

        assert!(
            !has_dir1_as_child_of_root,
            "Root context should not contain 'dir1' as a direct child after nested insertion."
        );
    }

    #[test]
    fn test_move_indexed_physical_file_node() {
        let func_name = "test_move_indexed_physical_file_node";
        let ctx = KartaServiceTestContext::new(func_name);

        // 1. Setup initial file structure and index the node
        ctx.create_dir_in_vault("initial_dir").unwrap();
        ctx.create_dir_in_vault("target_dir").unwrap();
        ctx.create_file_in_vault("initial_dir/movable_file.txt", b"content")
            .unwrap();

        let file_node_path = NodePath::new("vault/initial_dir/movable_file.txt".into());
        let target_node_path = NodePath::new("vault/target_dir".into());

        ctx.with_service_mut(|s| {
            let node =
                fs_reader::destructure_single_path(s.vault_fs_path(), &file_node_path).unwrap();
            s.data_mut().insert_nodes(vec![node]);
        });

        // 2. Assert initial context state
        let original_node_uuid = ctx.with_service(|s| {
            s.data()
                .open_node(&NodeHandle::Path(file_node_path.clone()))
                .unwrap()
                .uuid()
        });

        let (initial_dir_nodes, _, _) = ctx.with_service(|s| {
            s.open_context_from_path(NodePath::new("vault/initial_dir".into()))
                .unwrap()
        });
        assert!(
            initial_dir_nodes
                .iter()
                .any(|n| n.uuid() == original_node_uuid),
            "Source context should contain the node before move"
        );

        // 3. Execute the move operation
        ctx.with_service_mut(|s| {
            s.move_node(&file_node_path, &target_node_path).unwrap();
        });

        // 4. Assert the results
        let initial_file_path = ctx.get_vault_root().join("initial_dir/movable_file.txt");
        let target_file_path = ctx.get_vault_root().join("target_dir/movable_file.txt");
        assert!(
            !initial_file_path.exists(),
            "Original file should not exist"
        );
        assert!(
            target_file_path.exists(),
            "File should exist in the new location"
        );

        // Assert DB state
        let new_node_path = NodePath::new("vault/target_dir/movable_file.txt".into());
        let db_node = ctx.with_service(|s| {
            s.data()
                .open_node(&NodeHandle::Path(new_node_path))
                .unwrap()
        });
        assert_eq!(db_node.path().alias(), "/vault/target_dir/movable_file.txt");
        assert_eq!(
            db_node.uuid(),
            original_node_uuid,
            "UUID should not change after move"
        );

        // Assert context state after move
        let (initial_dir_nodes_after, _, _) = ctx.with_service(|s| {
            s.open_context_from_path(NodePath::new("vault/initial_dir".into()))
                .unwrap()
        });
        assert!(
            !initial_dir_nodes_after
                .iter()
                .any(|n| n.uuid() == original_node_uuid),
            "Source context should not contain the node after move"
        );

        let (target_dir_nodes_after, _, _) = ctx.with_service(|s| {
            s.open_context_from_path(NodePath::new("vault/target_dir".into()))
                .unwrap()
        });
        assert!(
            target_dir_nodes_after
                .iter()
                .any(|n| n.uuid() == original_node_uuid),
            "Target context should contain the node after move"
        );
    }

    #[test]
    fn test_move_unindexed_physical_file_node() {
        let func_name = "test_move_unindexed_physical_file_node";
        let ctx = KartaServiceTestContext::new(func_name);

        // 1. Setup initial file structure
        ctx.create_dir_in_vault("initial_dir").unwrap();
        ctx.create_dir_in_vault("target_dir").unwrap();
        ctx.create_file_in_vault("initial_dir/movable_file.txt", b"content")
            .unwrap();

        let file_node_path = NodePath::new("vault/initial_dir/movable_file.txt".into());
        let target_node_path = NodePath::new("vault/target_dir".into());

        // 2. Assert initial context state
        let (initial_dir_nodes, _, _) = ctx.with_service(|s| {
            s.open_context_from_path(NodePath::new("vault/initial_dir".into()))
                .unwrap()
        });
        assert!(
            initial_dir_nodes.iter().any(|n| n.path() == file_node_path),
            "Source context should contain the node before move"
        );

        // 3. Execute the move operation
        ctx.with_service_mut(|s| {
            s.move_node(&file_node_path, &target_node_path).unwrap();
        });

        // 4. Assert the results
        let initial_file_path = ctx.get_vault_root().join("initial_dir/movable_file.txt");
        let target_file_path = ctx.get_vault_root().join("target_dir/movable_file.txt");
        assert!(
            !initial_file_path.exists(),
            "Original file should not exist"
        );
        assert!(
            target_file_path.exists(),
            "File should exist in the new location"
        );

        // Assert context state after move
        let (initial_dir_nodes_after, _, _) = ctx.with_service(|s| {
            s.open_context_from_path(NodePath::new("vault/initial_dir".into()))
                .unwrap()
        });
        assert!(
            !initial_dir_nodes_after
                .iter()
                .any(|n| n.path() == file_node_path),
            "Source context should not contain the node after move"
        );

        let new_node_path = NodePath::new("vault/target_dir/movable_file.txt".into());
        let (target_dir_nodes_after, _, _) = ctx.with_service(|s| {
            s.open_context_from_path(NodePath::new("vault/target_dir".into()))
                .unwrap()
        });
        assert!(
            target_dir_nodes_after
                .iter()
                .any(|n| n.path() == new_node_path),
            "Target context should contain the node after move"
        );
    }

    #[test]
    fn test_move_virtual_node() {
        let func_name = "test_move_virtual_node";
        let ctx = KartaServiceTestContext::new(func_name);

        // 1. Setup initial nodes
        ctx.create_dir_in_vault("physical_dir").unwrap();
        let virtual_node_path = NodePath::new("vault/virtual_node".into());
        let virtual_node = DataNode::new(&virtual_node_path, NodeTypeId::virtual_generic());

        let invalid_parent_path = NodePath::new("vault/invalid_virtual_parent".into());
        let invalid_parent_node =
            DataNode::new(&invalid_parent_path, NodeTypeId::virtual_generic());

        ctx.with_service_mut(|s| {
            s.data_mut()
                .insert_nodes(vec![virtual_node.clone(), invalid_parent_node]);
        });

        // 2. Move virtual node to a physical directory (should succeed)
        let target_physical_path = NodePath::new("vault/physical_dir".into());
        ctx.with_service_mut(|s| {
            s.move_node(&virtual_node_path, &target_physical_path)
                .unwrap();
        });

        // 3. Assert successful move
        let new_virtual_path = NodePath::new("vault/physical_dir/virtual_node".into());
        let moved_node = ctx.with_service(|s| {
            s.data()
                .open_node(&NodeHandle::Path(new_virtual_path))
                .unwrap()
        });
        assert_eq!(moved_node.uuid(), virtual_node.uuid());
        assert_eq!(
            moved_node.path().alias(),
            "/vault/physical_dir/virtual_node"
        );

        // 4. Attempt to move virtual node to another virtual node (should fail)
        let move_to_virtual_result =
            ctx.with_service_mut(|s| s.move_node(&moved_node.path(), &invalid_parent_path));
        assert!(
            move_to_virtual_result.is_err(),
            "Moving a virtual node to another virtual node should fail"
        );
    }

    #[test]
    fn test_move_directory_with_physical_files() {
        let func_name = "test_move_directory_with_physical_files";
        let ctx = KartaServiceTestContext::new(func_name);

        // 1. Setup initial file structure
        ctx.create_dir_in_vault("source_dir").unwrap();
        ctx.create_dir_in_vault("dest_dir").unwrap();
        ctx.create_dir_in_vault("source_dir/movable_dir").unwrap();
        ctx.create_file_in_vault("source_dir/movable_dir/file.txt", b"content")
            .unwrap();

        // 2. Index all nodes
        let nodes_to_index = vec![
            NodePath::new("vault/source_dir".into()),
            NodePath::new("vault/dest_dir".into()),
            NodePath::new("vault/source_dir/movable_dir".into()),
            NodePath::new("vault/source_dir/movable_dir/file.txt".into()),
        ];

        ctx.with_service_mut(|s| {
            for path in nodes_to_index {
                let node = fs_reader::destructure_single_path(s.vault_fs_path(), &path).unwrap();
                s.data_mut().insert_nodes(vec![node]);
            }
        });

        // 3. Execute the move operation
        let movable_dir_path = NodePath::new("vault/source_dir/movable_dir".into());
        let dest_dir_path = NodePath::new("vault/dest_dir".into());
        ctx.with_service_mut(|s| {
            s.move_node(&movable_dir_path, &dest_dir_path).unwrap();
        });

        // 4. Assert filesystem changes
        assert!(!ctx.get_vault_root().join("source_dir/movable_dir").exists());
        assert!(ctx.get_vault_root().join("dest_dir/movable_dir").exists());
        assert!(ctx
            .get_vault_root()
            .join("dest_dir/movable_dir/file.txt")
            .exists());

        // 5. Assert database path changes
        let moved_dir_new_path = NodePath::new("vault/dest_dir/movable_dir".into());
        let moved_file_new_path = NodePath::new("vault/dest_dir/movable_dir/file.txt".into());

        ctx.with_service(|s| {
            // Check the moved directory itself
            let moved_dir_node = s
                .data()
                .open_node(&NodeHandle::Path(moved_dir_new_path))
                .expect("Moved directory should exist at new path in DB");
            assert_eq!(moved_dir_node.path().alias(), "/vault/dest_dir/movable_dir");

            // Check the child file
            let moved_file_node = s
                .data()
                .open_node(&NodeHandle::Path(moved_file_new_path))
                .expect("Child file should exist at new path in DB");
            assert_eq!(
                moved_file_node.path().alias(),
                "/vault/dest_dir/movable_dir/file.txt"
            );
        });
    }

    #[test]
    fn test_move_directory_with_nested_subdirectories() {
        let func_name = "test_move_directory_with_nested_subdirectories";
        let mut ctx = KartaServiceTestContext::new(func_name);

        // 1. Create a nested directory structure
        std::fs::create_dir_all(ctx.get_vault_root().join("source_dir/movable_dir/sub_dir")).unwrap();
        std::fs::create_dir_all(ctx.get_vault_root().join("dest_dir")).unwrap();
        std::fs::write(ctx.get_vault_root().join("source_dir/movable_dir/file.txt"), "content").unwrap();
        std::fs::write(ctx.get_vault_root().join("source_dir/movable_dir/sub_dir/nested_file.txt"), "nested content").unwrap();

        // 2. Index all nodes in the database
        let nodes_to_index = vec![
            NodePath::new("vault".into()),
            NodePath::new("vault/source_dir".into()),
            NodePath::new("vault/dest_dir".into()),
            NodePath::new("vault/source_dir/movable_dir".into()),
            NodePath::new("vault/source_dir/movable_dir/file.txt".into()),
            NodePath::new("vault/source_dir/movable_dir/sub_dir".into()),
            NodePath::new("vault/source_dir/movable_dir/sub_dir/nested_file.txt".into()),
        ];

        ctx.with_service_mut(|s| {
            for path in nodes_to_index {
                let node = fs_reader::destructure_single_path(s.vault_fs_path(), &path).unwrap();
                s.data_mut().insert_nodes(vec![node]);
            }
        });

        // 3. Execute the move operation
        let movable_dir_path = NodePath::new("vault/source_dir/movable_dir".into());
        let dest_dir_path = NodePath::new("vault/dest_dir".into());
        ctx.with_service_mut(|s| {
            s.move_node(&movable_dir_path, &dest_dir_path).unwrap();
        });

        // 4. Assert filesystem changes
        assert!(!ctx.get_vault_root().join("source_dir/movable_dir").exists());
        assert!(ctx.get_vault_root().join("dest_dir/movable_dir").exists());
        assert!(ctx.get_vault_root().join("dest_dir/movable_dir/file.txt").exists());
        assert!(ctx.get_vault_root().join("dest_dir/movable_dir/sub_dir").exists());
        assert!(ctx.get_vault_root().join("dest_dir/movable_dir/sub_dir/nested_file.txt").exists());

        // 5. Assert database path changes for all nodes
        let moved_dir_new_path = NodePath::new("vault/dest_dir/movable_dir".into());
        let moved_file_new_path = NodePath::new("vault/dest_dir/movable_dir/file.txt".into());
        let moved_subdir_new_path = NodePath::new("vault/dest_dir/movable_dir/sub_dir".into());
        let moved_nested_file_new_path = NodePath::new("vault/dest_dir/movable_dir/sub_dir/nested_file.txt".into());

        ctx.with_service(|s| {
            // Check main directory
            let moved_dir = s.data().open_node(&NodeHandle::Path(moved_dir_new_path)).unwrap();
            assert_eq!(moved_dir.path().alias(), "/vault/dest_dir/movable_dir");

            // Check file in main directory
            let moved_file = s.data().open_node(&NodeHandle::Path(moved_file_new_path)).unwrap();
            assert_eq!(moved_file.path().alias(), "/vault/dest_dir/movable_dir/file.txt");

            // Check subdirectory
            let moved_subdir = s.data().open_node(&NodeHandle::Path(moved_subdir_new_path)).unwrap();
            assert_eq!(moved_subdir.path().alias(), "/vault/dest_dir/movable_dir/sub_dir");

            // Check nested file
            let moved_nested_file = s.data().open_node(&NodeHandle::Path(moved_nested_file_new_path)).unwrap();
            assert_eq!(moved_nested_file.path().alias(), "/vault/dest_dir/movable_dir/sub_dir/nested_file.txt");
        });
    }

    #[test]
    fn test_move_directory_with_virtual_nodes() {
        let func_name = "test_move_directory_with_virtual_nodes";
        let mut ctx = KartaServiceTestContext::new(func_name);

        // 1. Create directory structure and a virtual node
        std::fs::create_dir_all(ctx.get_vault_root().join("source_dir/movable_dir")).unwrap();
        std::fs::create_dir_all(ctx.get_vault_root().join("dest_dir")).unwrap();

        // 2. Index the directory structure
        let nodes_to_index = vec![
            NodePath::new("vault".into()),
            NodePath::new("vault/source_dir".into()),
            NodePath::new("vault/dest_dir".into()),
            NodePath::new("vault/source_dir/movable_dir".into()),
        ];

        ctx.with_service_mut(|s| {
            for path in nodes_to_index {
                let node = fs_reader::destructure_single_path(s.vault_fs_path(), &path).unwrap();
                s.data_mut().insert_nodes(vec![node]);
            }
        });

        // 3. Create a virtual node inside the movable directory
        let virtual_node_path = NodePath::new("vault/source_dir/movable_dir/my_virtual_node".into());
        let virtual_node = DataNode::new(&virtual_node_path, NodeTypeId::new("core/text"));
        
        ctx.with_service_mut(|s| {
            s.data_mut().insert_nodes(vec![virtual_node.clone()]);
        });

        // 4. Execute the move operation
        let movable_dir_path = NodePath::new("vault/source_dir/movable_dir".into());
        let dest_dir_path = NodePath::new("vault/dest_dir".into());
        ctx.with_service_mut(|s| {
            s.move_node(&movable_dir_path, &dest_dir_path).unwrap();
        });

        // 5. Assert filesystem changes (directory moved)
        assert!(!ctx.get_vault_root().join("source_dir/movable_dir").exists());
        assert!(ctx.get_vault_root().join("dest_dir/movable_dir").exists());

        // 6. Assert database path changes including virtual node
        let moved_dir_new_path = NodePath::new("vault/dest_dir/movable_dir".into());
        let moved_virtual_node_new_path = NodePath::new("vault/dest_dir/movable_dir/my_virtual_node".into());

        ctx.with_service(|s| {
            // Check main directory
            let moved_dir = s.data().open_node(&NodeHandle::Path(moved_dir_new_path)).unwrap();
            assert_eq!(moved_dir.path().alias(), "/vault/dest_dir/movable_dir");

            // Check virtual node path was updated
            let moved_virtual_node = s.data().open_node(&NodeHandle::Path(moved_virtual_node_new_path)).unwrap();
            assert_eq!(moved_virtual_node.path().alias(), "/vault/dest_dir/movable_dir/my_virtual_node");
        });
    }

    #[test]
    fn test_move_directory_with_mixed_content() {
        let func_name = "test_move_directory_with_mixed_content";
        let mut ctx = KartaServiceTestContext::new(func_name);

        // 1. Create complex directory structure
        std::fs::create_dir_all(ctx.get_vault_root().join("source_dir/complex_dir/sub_dir")).unwrap();
        std::fs::create_dir_all(ctx.get_vault_root().join("dest_dir")).unwrap();
        std::fs::write(ctx.get_vault_root().join("source_dir/complex_dir/physical_file.txt"), "content").unwrap();
        std::fs::write(ctx.get_vault_root().join("source_dir/complex_dir/sub_dir/nested_file.md"), "nested content").unwrap();

        // 2. Index the physical structure
        let nodes_to_index = vec![
            NodePath::new("vault".into()),
            NodePath::new("vault/source_dir".into()),
            NodePath::new("vault/dest_dir".into()),
            NodePath::new("vault/source_dir/complex_dir".into()),
            NodePath::new("vault/source_dir/complex_dir/physical_file.txt".into()),
            NodePath::new("vault/source_dir/complex_dir/sub_dir".into()),
            NodePath::new("vault/source_dir/complex_dir/sub_dir/nested_file.md".into()),
        ];

        ctx.with_service_mut(|s| {
            for path in nodes_to_index {
                let node = fs_reader::destructure_single_path(s.vault_fs_path(), &path).unwrap();
                s.data_mut().insert_nodes(vec![node]);
            }
        });

        // 3. Add virtual nodes at different levels
        let virtual_node_root = NodePath::new("vault/source_dir/complex_dir/virtual_note".into());
        let virtual_node_nested = NodePath::new("vault/source_dir/complex_dir/sub_dir/virtual_nested".into());
        let virtual_root_node = DataNode::new(&virtual_node_root, NodeTypeId::new("core/text"));
        let virtual_nested_node = DataNode::new(&virtual_node_nested, NodeTypeId::new("core/text"));
        
        ctx.with_service_mut(|s| {
            s.data_mut().insert_nodes(vec![virtual_root_node.clone(), virtual_nested_node.clone()]);
        });

        // 4. Execute the move operation
        let complex_dir_path = NodePath::new("vault/source_dir/complex_dir".into());
        let dest_dir_path = NodePath::new("vault/dest_dir".into());
        ctx.with_service_mut(|s| {
            s.move_node(&complex_dir_path, &dest_dir_path).unwrap();
        });

        // 5. Assert filesystem changes
        assert!(!ctx.get_vault_root().join("source_dir/complex_dir").exists());
        assert!(ctx.get_vault_root().join("dest_dir/complex_dir").exists());
        assert!(ctx.get_vault_root().join("dest_dir/complex_dir/physical_file.txt").exists());
        assert!(ctx.get_vault_root().join("dest_dir/complex_dir/sub_dir").exists());
        assert!(ctx.get_vault_root().join("dest_dir/complex_dir/sub_dir/nested_file.md").exists());

        // 6. Assert ALL database path changes
        ctx.with_service(|s| {
            // Main directory
            let moved_dir = s.data().open_node(&NodeHandle::Path(
                NodePath::new("vault/dest_dir/complex_dir".into())
            )).unwrap();
            assert_eq!(moved_dir.path().alias(), "/vault/dest_dir/complex_dir");

            // Physical file in root
            let moved_file = s.data().open_node(&NodeHandle::Path(
                NodePath::new("vault/dest_dir/complex_dir/physical_file.txt".into())
            )).unwrap();
            assert_eq!(moved_file.path().alias(), "/vault/dest_dir/complex_dir/physical_file.txt");

            // Subdirectory
            let moved_subdir = s.data().open_node(&NodeHandle::Path(
                NodePath::new("vault/dest_dir/complex_dir/sub_dir".into())
            )).unwrap();
            assert_eq!(moved_subdir.path().alias(), "/vault/dest_dir/complex_dir/sub_dir");

            // Physical file in subdirectory
            let moved_nested_file = s.data().open_node(&NodeHandle::Path(
                NodePath::new("vault/dest_dir/complex_dir/sub_dir/nested_file.md".into())
            )).unwrap();
            assert_eq!(moved_nested_file.path().alias(), "/vault/dest_dir/complex_dir/sub_dir/nested_file.md");

            // Virtual node in root
            let moved_virtual_root = s.data().open_node(&NodeHandle::Path(
                NodePath::new("vault/dest_dir/complex_dir/virtual_note".into())
            )).unwrap();
            assert_eq!(moved_virtual_root.path().alias(), "/vault/dest_dir/complex_dir/virtual_note");

            // Virtual node in subdirectory
            let moved_virtual_nested = s.data().open_node(&NodeHandle::Path(
                NodePath::new("vault/dest_dir/complex_dir/sub_dir/virtual_nested".into())
            )).unwrap();
            assert_eq!(moved_virtual_nested.path().alias(), "/vault/dest_dir/complex_dir/sub_dir/virtual_nested");
        });
    }

    #[test]
    fn test_rename_physical_file_node() {
        let func_name = "test_rename_physical_file_node";
        let ctx = KartaServiceTestContext::new(func_name);

        // 1. Create initial file
        ctx.create_dir_in_vault("test_dir").unwrap();
        ctx.create_file_in_vault("test_dir/original_name.txt", b"content").unwrap();

        let original_path = NodePath::new("vault/test_dir/original_name.txt".into());
        let expected_new_path = NodePath::new("vault/test_dir/new_name.txt".into());

        // 2. Index the file
        let original_uuid = ctx.with_service_mut(|s| {
            let node = fs_reader::destructure_single_path(s.vault_fs_path(), &original_path).unwrap();
            let uuid = node.uuid();
            s.data_mut().insert_nodes(vec![node]);
            uuid
        });

        // 3. Perform rename
        let result_path = ctx.with_service_mut(|s| {
            s.rename_node(&original_path, "new_name.txt").unwrap()
        });

        // 4. Assert the results
        assert_eq!(result_path, expected_new_path);

        // Check filesystem
        assert!(!ctx.get_vault_root().join("test_dir/original_name.txt").exists());
        assert!(ctx.get_vault_root().join("test_dir/new_name.txt").exists());

        // Check database
        let renamed_node = ctx.with_service(|s| {
            s.data().open_node(&NodeHandle::Path(expected_new_path)).unwrap()
        });
        assert_eq!(renamed_node.uuid(), original_uuid);
        assert_eq!(renamed_node.path().alias(), "/vault/test_dir/new_name.txt");
        assert_eq!(renamed_node.name(), "new_name.txt");
    }

    #[test]
    fn test_rename_physical_directory() {
        let func_name = "test_rename_physical_directory";
        let ctx = KartaServiceTestContext::new(func_name);

        // 1. Create directory with children
        ctx.create_dir_in_vault("parent_dir").unwrap();
        ctx.create_dir_in_vault("parent_dir/old_dir_name").unwrap();
        ctx.create_file_in_vault("parent_dir/old_dir_name/child_file.txt", b"content").unwrap();

        let original_dir_path = NodePath::new("vault/parent_dir/old_dir_name".into());
        let original_file_path = NodePath::new("vault/parent_dir/old_dir_name/child_file.txt".into());
        let expected_new_dir_path = NodePath::new("vault/parent_dir/new_dir_name".into());
        let expected_new_file_path = NodePath::new("vault/parent_dir/new_dir_name/child_file.txt".into());

        // 2. Index the nodes
        let (original_dir_uuid, original_file_uuid) = ctx.with_service_mut(|s| {
            let dir_node = fs_reader::destructure_single_path(s.vault_fs_path(), &original_dir_path).unwrap();
            let file_node = fs_reader::destructure_single_path(s.vault_fs_path(), &original_file_path).unwrap();
            let dir_uuid = dir_node.uuid();
            let file_uuid = file_node.uuid();
            s.data_mut().insert_nodes(vec![dir_node, file_node]);
            (dir_uuid, file_uuid)
        });

        // 3. Perform rename
        let result_path = ctx.with_service_mut(|s| {
            s.rename_node(&original_dir_path, "new_dir_name").unwrap()
        });

        // 4. Assert the results
        assert_eq!(result_path, expected_new_dir_path);

        // Check filesystem
        assert!(!ctx.get_vault_root().join("parent_dir/old_dir_name").exists());
        assert!(ctx.get_vault_root().join("parent_dir/new_dir_name").exists());
        assert!(ctx.get_vault_root().join("parent_dir/new_dir_name/child_file.txt").exists());

        // Check database - directory
        let renamed_dir = ctx.with_service(|s| {
            s.data().open_node(&NodeHandle::Path(expected_new_dir_path)).unwrap()
        });
        assert_eq!(renamed_dir.uuid(), original_dir_uuid);
        assert_eq!(renamed_dir.path().alias(), "/vault/parent_dir/new_dir_name");
        assert_eq!(renamed_dir.name(), "new_dir_name");

        // Check database - child file
        let renamed_file = ctx.with_service(|s| {
            s.data().open_node(&NodeHandle::Path(expected_new_file_path)).unwrap()
        });
        assert_eq!(renamed_file.uuid(), original_file_uuid);
        assert_eq!(renamed_file.path().alias(), "/vault/parent_dir/new_dir_name/child_file.txt");
    }

    #[test]
    fn test_rename_virtual_node() {
        let func_name = "test_rename_virtual_node";
        let ctx = KartaServiceTestContext::new(func_name);

        // 1. Create physical parent and virtual child
        ctx.create_dir_in_vault("parent_dir").unwrap();
        let virtual_node_path = NodePath::new("vault/parent_dir/old_virtual_name".into());
        let expected_new_path = NodePath::new("vault/parent_dir/new_virtual_name".into());

        // 2. Index the virtual node
        let original_uuid = ctx.with_service_mut(|s| {
            let virtual_node = DataNode::new(&virtual_node_path, NodeTypeId::new("core/text"));
            let uuid = virtual_node.uuid();
            s.data_mut().insert_nodes(vec![virtual_node]);
            uuid
        });

        // 3. Perform rename
        let result_path = ctx.with_service_mut(|s| {
            s.rename_node(&virtual_node_path, "new_virtual_name").unwrap()
        });

        // 4. Assert the results
        assert_eq!(result_path, expected_new_path);

        // Check database
        let renamed_node = ctx.with_service(|s| {
            s.data().open_node(&NodeHandle::Path(expected_new_path)).unwrap()
        });
        assert_eq!(renamed_node.uuid(), original_uuid);
        assert_eq!(renamed_node.path().alias(), "/vault/parent_dir/new_virtual_name");
        assert_eq!(renamed_node.name(), "new_virtual_name");

        // Ensure old path no longer exists in database
        let old_node_result = ctx.with_service(|s| {
            s.data().open_node(&NodeHandle::Path(virtual_node_path))
        });
        assert!(old_node_result.is_err());
    }

    #[test]
    fn test_rename_with_name_collision() {
        let func_name = "test_rename_with_name_collision";
        let ctx = KartaServiceTestContext::new(func_name);

        // 1. Create two files with different names
        ctx.create_dir_in_vault("test_dir").unwrap();
        ctx.create_file_in_vault("test_dir/file1.txt", b"content1").unwrap();
        ctx.create_file_in_vault("test_dir/file2.txt", b"content2").unwrap();

        let file1_path = NodePath::new("vault/test_dir/file1.txt".into());

        // 2. Index the files
        ctx.with_service_mut(|s| {
            let node1 = fs_reader::destructure_single_path(s.vault_fs_path(), &file1_path).unwrap();
            let node2 = fs_reader::destructure_single_path(s.vault_fs_path(), &NodePath::new("vault/test_dir/file2.txt".into())).unwrap();
            s.data_mut().insert_nodes(vec![node1, node2]);
        });

        // 3. Try to rename file1 to file2 (collision should be resolved automatically)
        let result_path = ctx.with_service_mut(|s| {
            s.rename_node(&file1_path, "file2.txt").unwrap()
        });

        // 4. Should have been auto-renamed to avoid collision
        assert_ne!(result_path.name(), "file2.txt"); // Should be auto-renamed
        assert!(result_path.name().starts_with("file2")); // Should start with desired name
        
        // Check filesystem - original files should exist, plus the renamed one
        assert!(ctx.get_vault_root().join("test_dir/file2.txt").exists()); // Original file2
        assert!(!ctx.get_vault_root().join("test_dir/file1.txt").exists()); // file1 should be gone
        assert!(ctx.get_vault_root().join(format!("test_dir/{}", result_path.name())).exists()); // Renamed file
    }
}
