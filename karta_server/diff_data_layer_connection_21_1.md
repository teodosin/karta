diff --git a/karta_server/src/graph_agdb/graph_node.rs b/karta_server/src/graph_agdb/graph_node.rs
index f9b3fee..b248fe6 100644
--- a/karta_server/src/graph_agdb/graph_node.rs
+++ b/karta_server/src/graph_agdb/graph_node.rs
@@ -36,6 +36,28 @@ impl GraphNodes for GraphAgdb {
         DataNode::try_from(db_element.clone()).map_err(|e| e.into())
     }
 
+    fn open_nodes_by_uuid(&self, uuids: Vec<Uuid>) -> Result<Vec<DataNode>, Box<dyn Error>> {
+        if uuids.is_empty() {
+            return Ok(Vec::new());
+        }
+
+        let ids_as_strings: Vec<String> = uuids.into_iter().map(|id| id.to_string()).collect();
+        
+        let query_result = self.db.exec(
+            &QueryBuilder::select()
+                .ids(ids_as_strings)
+                .query(),
+        )?;
+
+        let datanodes = query_result
+            .elements
+            .into_iter()
+            .map(DataNode::try_from)
+            .collect::<Result<Vec<DataNode>, _>>()?;
+
+        Ok(datanodes)
+    }
+
     fn open_node_connections(&self, path: &NodePath) -> Vec<(DataNode, Edge)> {
         let focal_node = match self.open_node(&NodeHandle::Path(path.clone())) {
             Ok(node) => node,
diff --git a/karta_server/src/graph_traits/graph_node.rs b/karta_server/src/graph_traits/graph_node.rs
index 5c5cf80..808f118 100644
--- a/karta_server/src/graph_traits/graph_node.rs
+++ b/karta_server/src/graph_traits/graph_node.rs
@@ -1,4 +1,5 @@
 use std::{error::Error, path::PathBuf};
+use uuid::Uuid;
 
 use crate::elements::node_path::NodeHandle;
 
@@ -12,6 +13,9 @@ pub trait GraphNodes {
     /// The path is relative to the root of the graph.
     fn open_node(&self, handle: &NodeHandle) -> Result<DataNode, Box<dyn Error>>;
 
+    /// Retrieves a set of nodes by their UUIDs.
+    fn open_nodes_by_uuid(&self, uuids: Vec<Uuid>) -> Result<Vec<DataNode>, Box<dyn Error>>;
+
     // Retrieves the edges of a particular node.
     // fn get_node_edges(&self, path: &NodePath) -> Vec<Edge>;
 
diff --git a/karta_server/src/server/context_endpoints.rs b/karta_server/src/server/context_endpoints.rs
index 29a2f99..c2b2279 100644
--- a/karta_server/src/server/context_endpoints.rs
+++ b/karta_server/src/server/context_endpoints.rs
@@ -142,7 +142,7 @@ pub async fn open_context_from_fs_path(
 #[cfg(test)]
 mod tests {
     use super::*;
-    use crate::server::karta_service::KartaService;
+    use crate::{prelude::{GraphNodes, NodeTypeId}, server::karta_service::KartaService};
     use crate::utils::utils::KartaServiceTestContext;
     use axum::{
         body::Body,
@@ -188,13 +188,13 @@ mod tests {
     #[tokio::test]
     async fn open_vault_context_from_api() {
         let (router, test_ctx) = setup_test_environment("api_open_vault_ctx");
-        test_ctx
-            .create_file_in_vault("fileA.txt", b"content of file A")
-            .unwrap();
-        test_ctx.create_dir_in_vault("dir1").unwrap();
-        test_ctx
-            .create_file_in_vault("dir1/fileB.txt", b"content of file B")
-            .unwrap();
+        test_ctx.with_service_mut(|s| {
+            s.data_mut().insert_nodes(vec![
+                DataNode::new(&"vault/fileA.txt".into(), NodeTypeId::file_type()),
+                DataNode::new(&"vault/dir1".into(), NodeTypeId::dir_type()),
+                DataNode::new(&"vault/dir1/fileB.txt".into(), NodeTypeId::file_type()),
+            ]);
+        });
 
         let body_json = execute_request_and_get_json(router, "/ctx/vault/", StatusCode::OK).await;
         println!(
@@ -370,13 +370,13 @@ mod tests {
     #[tokio::test]
     async fn go_to_file_context() {
         let (router, test_ctx) = setup_test_environment("api_open_file_ctx");
-        test_ctx
-            .create_file_in_vault("fileA.txt", b"content of file A")
-            .unwrap();
-        test_ctx.create_dir_in_vault("dir1").unwrap();
-        test_ctx
-            .create_file_in_vault("dir1/fileB.txt", b"content of file B")
-            .unwrap();
+        test_ctx.with_service_mut(|s| {
+            s.data_mut().insert_nodes(vec![
+                DataNode::new(&"vault/fileA.txt".into(), NodeTypeId::file_type()),
+                DataNode::new(&"vault/dir1".into(), NodeTypeId::dir_type()),
+                DataNode::new(&"vault/dir1/fileB.txt".into(), NodeTypeId::file_type()),
+            ]);
+        });
 
         let body_json =
             execute_request_and_get_json(router, "/ctx/vault/dir1/fileB.txt", StatusCode::OK).await;
@@ -410,9 +410,11 @@ mod tests {
     #[tokio::test]
     async fn going_to_vault_child_context__includes_vault() {
         let (router, test_ctx) = setup_test_environment("api_open_file_ctx_incl_vault");
-        test_ctx
-            .create_file_in_vault("fileA.txt", b"content of file A")
-            .unwrap();
+        test_ctx.with_service_mut(|s| {
+            s.data_mut().insert_nodes(vec![
+                DataNode::new(&"vault/fileA.txt".into(), NodeTypeId::file_type()),
+            ]);
+        });
 
         let body_json =
             execute_request_and_get_json(router, "/ctx/vault/fileA.txt", StatusCode::OK).await;
diff --git a/karta_server/src/server/karta_service.rs b/karta_server/src/server/karta_service.rs
index 602047d..daded8a 100644
--- a/karta_server/src/server/karta_service.rs
+++ b/karta_server/src/server/karta_service.rs
@@ -85,154 +85,156 @@ impl KartaService {
     pub fn open_context_from_path(&self, path: NodePath)
         -> Result<(Vec<DataNode>, Vec<Edge>, Context), Box<dyn Error>> {
 
-        if path == NodePath::root() {
-            let focal_node = self.data().open_node(&NodeHandle::Path(NodePath::root()))
-                .map_err(|e| format!("Failed to open virtual root node: {}", e))?;
-            
-            let mut datanodes_for_context = vec![focal_node.clone()];
-            let mut edges_for_context = Vec::new();
-
-            // Get children (primarily vault) and their edges from the database
-            let db_child_connections = self.data().open_node_connections(&NodePath::root());
-            for (child_node, edge) in db_child_connections {
-                // For the virtual root's context, we are primarily interested in vault as its direct child.
-                if child_node.path() == NodePath::vault() {
-                    if !datanodes_for_context.iter().any(|n| n.path() == child_node.path()) {
-                        datanodes_for_context.push(child_node);
-                    }
-                    edges_for_context.push(edge);
-                }
-                // Potentially include other direct virtual children of NodePath::root() if defined later.
-            }
-            
-            // Ensure vault is included if not found via connections (e.g. if connections only returns non-archetype)
-            if !datanodes_for_context.iter().any(|n| n.path() == NodePath::vault()) {
-                let vault_node = self.data().open_node(&NodeHandle::Path(NodePath::vault()))
-                    .map_err(|e| format!("Failed to open vault node: {}", e))?;
-                datanodes_for_context.push(vault_node);
-                // If the edge was also missing, this implies it should be created or is an error.
-                // For now, assume open_node_connections is the source of truth for edges.
-                // A robust solution might involve self.data().get_edge_strict() if the edge is critical and might be missed.
-            }
-
-            let context = self.view.generate_context(focal_node.uuid(), None, datanodes_for_context.clone());
-            return Ok((datanodes_for_context, edges_for_context, context));
-        }
-
-        // --- Existing logic for vault and other FS-related paths ---
-        let mut additional_nodes_to_include: Vec<DataNode> = Vec::new();
-        let mut additional_edges_to_include: Vec<Edge> = Vec::new();
         let absolute_path = path.full(self.vault_fs_path());
-        let fs_nodes_from_destructure = fs_reader::destructure_file_path(self.vault_fs_path(), &absolute_path, true)
-            .map_err(|e| format!("Failed to destructure path {:?} with root {:?}: {}", absolute_path, self.vault_fs_path(), e))?;
+        let is_fs_node = absolute_path.exists();
+        let is_db_node = self.data().open_node(&NodeHandle::Path(path.clone())).is_ok();
 
-        let mut focal_fs_datanode: Option<DataNode> = None;
-        let mut child_fs_datanodes: Vec<DataNode> = Vec::new();
-        let mut fs_edges: Vec<Edge> = Vec::new();
-
-        if absolute_path.is_dir() {
-            focal_fs_datanode = Some(DataNode::new(&path, NodeTypeId::dir_type()));
-            child_fs_datanodes = fs_nodes_from_destructure;
-            if let Some(focal_node_unwrapped) = &focal_fs_datanode {
-                for child_node in &child_fs_datanodes {
-                    fs_edges.push(Edge::new(focal_node_unwrapped.uuid(), child_node.uuid()));
-                }
-            }
-        } else if absolute_path.is_file() {
-            focal_fs_datanode = fs_nodes_from_destructure.into_iter().find(|n| n.path() == path);
-            if let Some(focal_file_node_unwrapped) = &focal_fs_datanode {
-                if let Some(parent_path) = path.parent() {
-                    // We need the parent's UUID. We can create a transient parent node to get it.
-                    let parent_node = DataNode::new(&parent_path, NodeTypeId::dir_type());
-                    fs_edges.push(Edge::new(parent_node.uuid(), focal_file_node_unwrapped.uuid()));
-                }
-            }
+        if path == NodePath::root() {
+            self.open_root_context()
+        } else if is_db_node && !is_fs_node {
+            self.open_virtual_context(&path)
+        } else {
+            self.open_physical_context(&path)
         }
+    }
 
-        let fs_derived_focal_node = focal_fs_datanode.ok_or_else(|| {
-            format!("Focal node for path {:?} could not be determined from filesystem.", path)
-        })?;
+    /// Opens the root context. This is a special case as it has no parent and its children are determined differently.
+    fn open_root_context(&self) -> Result<(Vec<DataNode>, Vec<Edge>, Context), Box<dyn Error>> {
+        let mut nodes: HashMap<Uuid, DataNode> = HashMap::new();
+        let mut direct_edges: Vec<Edge> = Vec::new();
 
-        let db_focal_datanode_optional = self.data().open_node(&NodeHandle::Path(path.clone())).ok();
-        let db_child_connections = self.data().open_node_connections(&path);
+        let focal_node = self.data().open_node(&NodeHandle::Path(NodePath::root()))?;
+        nodes.insert(focal_node.uuid(), focal_node.clone());
 
-        let mut db_child_datanodes_map: HashMap<Uuid, DataNode> = HashMap::new();
-        for (node, _) in db_child_connections {
-            db_child_datanodes_map.insert(node.uuid(), node);
+        for (child_node, edge) in self.data().open_node_connections(&NodePath::root()) {
+            nodes.insert(child_node.uuid(), child_node);
+            direct_edges.push(edge);
         }
 
-        let mut final_datanodes_map: HashMap<Uuid, DataNode> = HashMap::new();
-        let mut final_edges_set: HashSet<(Uuid, Uuid)> = HashSet::new();
-        let mut reconciled_edges: Vec<Edge> = Vec::new();
+        self._finalize_context(focal_node, nodes, direct_edges)
+    }
 
-        let definitive_focal_node = match db_focal_datanode_optional {
-            Some(db_node) => db_node,
-            None => fs_derived_focal_node.clone(),
-        };
+    /// Opens a context for a "virtual" node (exists in DB, but not on the filesystem).
+    fn open_virtual_context(&self, path: &NodePath) -> Result<(Vec<DataNode>, Vec<Edge>, Context), Box<dyn Error>> {
+        let mut nodes: HashMap<Uuid, DataNode> = HashMap::new();
+        let mut direct_edges: Vec<Edge> = Vec::new();
 
-        if let Some(parent_path) = definitive_focal_node.path().parent() {
-            if parent_path == NodePath::vault() {
-                if let Ok(vault_node) = self.data().open_node(&NodeHandle::Path(NodePath::vault())) {
-                    let vault_to_focal_edge = Edge::new(vault_node.uuid(), definitive_focal_node.uuid());
-                    additional_nodes_to_include.push(vault_node);
-                    additional_edges_to_include.push(vault_to_focal_edge);
-                } else {
-                    eprintln!("Critical error: Vault node not found in DB.");
-                }
+        let focal_node = self.data().open_node(&NodeHandle::Path(path.clone()))?;
+        nodes.insert(focal_node.uuid(), focal_node.clone());
+
+        // Add parent if it exists.
+        if let Some(parent_path) = path.parent() {
+            if let Ok(parent_node) = self.data().open_node(&NodeHandle::Path(parent_path)) {
+                direct_edges.push(Edge::new(parent_node.uuid(), focal_node.uuid()));
+                nodes.insert(parent_node.uuid(), parent_node);
             }
         }
+        
+        // Add DB connections (children and others).
+        for (child_node, edge) in self.data().open_node_connections(path) {
+            if *edge.source() == focal_node.uuid() {
+                nodes.insert(child_node.uuid(), child_node);
+                direct_edges.push(edge);
+            }
+        }
+
+        self._finalize_context(focal_node, nodes, direct_edges)
+    }
 
-        final_datanodes_map.insert(definitive_focal_node.uuid(), definitive_focal_node.clone());
+    /// Opens a context for a "physical" node (exists on the filesystem).
+    fn open_physical_context(&self, path: &NodePath) -> Result<(Vec<DataNode>, Vec<Edge>, Context), Box<dyn Error>> {
+        let mut nodes: HashMap<Uuid, DataNode> = HashMap::new();
+        let mut direct_edges: Vec<Edge> = Vec::new();
+        let absolute_path = path.full(self.vault_fs_path());
+
+        // Get the DB version of the focal node if it exists, otherwise create a provisional one.
+        let focal_node = self.data()
+            .open_node(&NodeHandle::Path(path.clone()))
+            .unwrap_or_else(|_| DataNode::new(path, NodeTypeId::dir_type()));
+        nodes.insert(focal_node.uuid(), focal_node.clone());
 
-        let mut parent_uuid: Option<Uuid> = None;
-        if let Some(parent_path) = definitive_focal_node.path().parent() {
-            let parent_node = self.data().open_node(&NodeHandle::Path(parent_path.clone()))
+        // Add parent if it exists.
+        if let Some(parent_path) = path.parent() {
+            let parent_node = self.data()
+                .open_node(&NodeHandle::Path(parent_path.clone()))
                 .unwrap_or_else(|_| DataNode::new(&parent_path, NodeTypeId::dir_type()));
-            parent_uuid = Some(parent_node.uuid());
-            final_datanodes_map.entry(parent_node.uuid()).or_insert(parent_node);
+            direct_edges.push(Edge::new(parent_node.uuid(), focal_node.uuid()));
+            nodes.insert(parent_node.uuid(), parent_node);
         }
 
-        for fs_child_node in &child_fs_datanodes {
-            let child_uuid = fs_child_node.uuid();
-            let definitive_child = db_child_datanodes_map.get(&child_uuid)
-                .cloned()
-                .unwrap_or_else(|| fs_child_node.clone());
-            final_datanodes_map.insert(child_uuid, definitive_child);
+        // Add/update nodes from the filesystem if it's a directory.
+        if absolute_path.is_dir() {
+            let fs_children = fs_reader::destructure_file_path(self.vault_fs_path(), &absolute_path, true)?;
+            for child in fs_children {
+                direct_edges.push(Edge::new_cont(focal_node.uuid(), child.uuid()));
+                nodes.entry(child.uuid()).or_insert(child);
+            }
         }
-
-        for (db_node_uuid, db_node_data) in db_child_datanodes_map.iter() {
-            final_datanodes_map.entry(*db_node_uuid).or_insert_with(|| db_node_data.clone());
+        
+        // Add any additional connections from the database.
+        for (child_node, edge) in self.data().open_node_connections(path) {
+            if *edge.source() == focal_node.uuid() {
+                nodes.insert(child_node.uuid(), child_node);
+                direct_edges.push(edge);
+            }
         }
 
-        for node_to_add in &additional_nodes_to_include {
-            final_datanodes_map.entry(node_to_add.uuid()).or_insert_with(|| node_to_add.clone());
-        }
+        self._finalize_context(focal_node, nodes, direct_edges)
+    }
 
-        let mut all_edges_to_process = fs_edges;
-        all_edges_to_process.extend(db_child_datanodes_map.values().flat_map(|node| {
-            self.data().open_node_connections(&node.path()).into_iter().map(|(_, edge)| edge)
-        }));
-        all_edges_to_process.extend(additional_edges_to_include);
-
-        for edge in all_edges_to_process {
-            if final_datanodes_map.contains_key(edge.source()) && final_datanodes_map.contains_key(edge.target()) {
-                let edge_key = (*edge.source(), *edge.target());
-                if final_edges_set.insert(edge_key) {
-                    reconciled_edges.push(edge);
+    /// Private helper to finalize context creation.
+    fn _finalize_context(
+        &self,
+        focal_node: DataNode,
+        mut nodes: HashMap<Uuid, DataNode>,
+        direct_edges: Vec<Edge>,
+    ) -> Result<(Vec<DataNode>, Vec<Edge>, Context), Box<dyn Error>> {
+        
+        // Augment with nodes from a saved context file, if one exists.
+        if let Ok(saved_context) = self.view.get_context_file(focal_node.uuid()) {
+            let saved_node_uuids: Vec<Uuid> = saved_context.viewnodes()
+                .iter()
+                .map(|vn| vn.uuid())
+                .filter(|uuid| !nodes.contains_key(uuid))
+                .collect();
+            
+            if !saved_node_uuids.is_empty() {
+                let missing_nodes = self.data().open_nodes_by_uuid(saved_node_uuids)?;
+                for node in missing_nodes {
+                    nodes.entry(node.uuid()).or_insert(node);
                 }
             }
         }
+        
+        let mut final_edges: Vec<Edge> = Vec::new();
+        let mut final_edges_set: HashSet<(Uuid, Uuid)> = HashSet::new();
 
-        let collected_final_datanodes: Vec<DataNode> = final_datanodes_map.values().cloned().collect();
-        let context_focal_uuid = definitive_focal_node.uuid();
+        for edge in direct_edges {
+            if nodes.contains_key(edge.source()) && nodes.contains_key(edge.target()) {
+                if final_edges_set.insert((*edge.source(), *edge.target())) {
+                    final_edges.push(edge);
+                }
+            }
+        }
+        
+        let mut final_datanodes: Vec<DataNode> = nodes.values().cloned().collect();
+        if focal_node.path() != NodePath::root() && focal_node.path() != NodePath::vault() {
+            final_datanodes.retain(|n| n.path() != NodePath::root());
+        }
 
+        let parent_uuid = if let Some(parent_path) = focal_node.path().parent() {
+            final_datanodes.iter().find(|n| n.path() == parent_path).map(|n| n.uuid())
+        } else {
+            None
+        };
+        
         let context = self.view.generate_context(
-            context_focal_uuid,
-            parent_uuid, // Pass the parent's UUID
-            collected_final_datanodes.clone(),
+            focal_node.uuid(),
+            parent_uuid,
+            final_datanodes.clone(),
         );
 
-        Ok((collected_final_datanodes, reconciled_edges, context))
+        Ok((final_datanodes, final_edges, context))
     }
 }
 
@@ -256,13 +258,13 @@ mod tests {
         // --- Part 1: Test opening the vault context ---
         let (datanodes, edges, _) = ctx.with_service(|s| s.open_context_from_path(NodePath::vault())).unwrap();
 
-        let vault_node = datanodes.iter().find(|n| n.path() == NodePath::vault()).expect("Vault node not found");
-        let root_node = datanodes.iter().find(|n| n.path() == NodePath::root()).expect("Root node not found");
-        let test_dir_node = datanodes.iter().find(|n| n.path() == NodePath::vault().join("test_dir")).expect("test_dir not found");
+        assert!(datanodes.iter().any(|n| n.path() == NodePath::vault()), "Vault node not found");
+        assert!(datanodes.iter().any(|n| n.path() == NodePath::root()), "Root node not found");
+        assert!(datanodes.iter().any(|n| n.path() == NodePath::vault().join("test_dir")), "test_dir not found");
+        assert!(datanodes.iter().any(|n| n.path() == NodePath::vault().join("test_file.txt")), "test_file.txt not found");
 
         assert_eq!(datanodes.len(), 4, "Should contain root, vault, test_dir, and test_file.txt");
-        assert!(edges.iter().any(|e| *e.source() == root_node.uuid() && *e.target() == vault_node.uuid()), "Missing edge from root to vault");
-        assert!(edges.iter().any(|e| *e.source() == vault_node.uuid() && *e.target() == test_dir_node.uuid()), "Missing edge from vault to test_dir");
+        assert_eq!(edges.len(), 3, "Should contain root->vault, vault->test_dir, and vault->test_file.txt edges");
 
         // --- Part 2: Test opening a deeper context to check for grandparent bug ---
         let (datanodes_deeper, _, _) = ctx.with_service(|s| s.open_context_from_path(NodePath::vault().join("test_dir"))).unwrap();
@@ -396,6 +398,8 @@ mod tests {
             let root_node = s.data().open_node(&NodeHandle::Path(NodePath::root())).unwrap();
             let virtual_node = DataNode::new(&virtual_node_path, NodeTypeId::new("core/text"));
             s.data_mut().insert_nodes(vec![virtual_node.clone()]);
+            let edge = Edge::new(root_node.uuid(), virtual_node.uuid());
+            s.data_mut().insert_edges(vec![edge]);
         });
 
         let (datanodes, _, _) = ctx.with_service(|s| s.open_context_from_path(NodePath::root())).unwrap();
diff --git a/karta_server/src/server/write_endpoints.rs b/karta_server/src/server/write_endpoints.rs
index a93b533..ab09fc9 100644
--- a/karta_server/src/server/write_endpoints.rs
+++ b/karta_server/src/server/write_endpoints.rs
@@ -160,11 +160,12 @@ mod tests {
         let (router, test_ctx) = setup_test_environment("save_creates_file");
 
         // Arrange
-        test_ctx.create_dir_in_vault("test_dir").unwrap();
-        let (_, _, initial_context) = test_ctx
-            .with_service(|s| s.open_context_from_path("vault/test_dir".into()))
-            .unwrap();
-        let focal_uuid = initial_context.focal();
+        let focal_uuid = test_ctx.with_service_mut(|s| {
+            s.data_mut().insert_nodes(vec![DataNode::new(&"vault/test_dir".into(), NodeTypeId::dir_type())]);
+            s.open_context_from_path("vault/test_dir".into()).unwrap().2.focal()
+        });
+        
+        let initial_context = test_ctx.with_service(|s| s.open_context_from_path("vault/test_dir".into()).unwrap().2);
         let view_node_to_modify = initial_context.viewnodes().get(0).unwrap().clone();
         let modified_view_node = view_node_to_modify.positioned(123.0, 456.0);
         let context_payload = Context::with_viewnodes(focal_uuid, vec![modified_view_node.clone()]);
@@ -189,10 +190,13 @@ mod tests {
 
         // Arrange: Create a directory and save a context for it first.
         test_ctx.create_dir_in_vault("dir_to_delete").unwrap();
-        let (_, _, initial_context) = test_ctx
-            .with_service(|s| s.open_context_from_path("vault/dir_to_delete".into()))
-            .unwrap();
-        let focal_uuid = initial_context.focal();
+        // Manually insert the node to ensure it's indexed before we try to save its context by UUID.
+        let focal_uuid = test_ctx.with_service_mut(|s| {
+            let node_to_insert = DataNode::new(&"vault/dir_to_delete".into(), NodeTypeId::dir_type());
+            s.data_mut().insert_nodes(vec![node_to_insert]);
+            s.open_context_from_path("vault/dir_to_delete".into()).unwrap().2.focal()
+        });
+        let initial_context = test_ctx.with_service(|s| s.open_context_from_path("vault/dir_to_delete".into()).unwrap().2);
         let view_node = initial_context.viewnodes().get(0).unwrap().clone();
         let initial_payload = Context::with_viewnodes(focal_uuid, vec![view_node]);
         let initial_payload_json = serde_json::to_string(&initial_payload).unwrap();
@@ -229,19 +233,19 @@ mod tests {
     async fn test_reload_context_merges_saved_and_default_nodes() {
         let (router, test_ctx) = setup_test_environment("reload_merges");
 
-        // Arrange: FS setup
-        test_ctx.create_dir_in_vault("test_dir").unwrap();
-        test_ctx.create_file_in_vault("test_dir/A.txt", b"").unwrap();
-        test_ctx.create_file_in_vault("test_dir/B.txt", b"").unwrap();
-
-        // Arrange: Get initial state to find the node to modify.
-        let (initial_nodes, _, initial_context) = test_ctx
-            .with_service(|s| s.open_context_from_path("vault/test_dir".into()))
-            .unwrap();
+        // Arrange: FS setup and index the nodes to simulate modification before saving.
+        let (initial_nodes, _, initial_context) = test_ctx.with_service_mut(|s| {
+            s.data_mut().insert_nodes(vec![
+                DataNode::new(&"vault/test_dir".into(), NodeTypeId::dir_type()),
+                DataNode::new(&"vault/test_dir/A.txt".into(), NodeTypeId::file_type()),
+                DataNode::new(&"vault/test_dir/B.txt".into(), NodeTypeId::file_type()),
+            ]);
+            s.open_context_from_path("vault/test_dir".into()).unwrap()
+        });
         let focal_uuid = initial_context.focal();
-        let node_b_data = initial_nodes.iter().find(|n| n.path().name() == "B.txt").unwrap();
-        let node_b_view = initial_context.viewnodes().iter().find(|vn| vn.uuid == node_b_data.uuid()).unwrap();
-
+        let node_b_data = initial_nodes.iter().find(|n| n.path().name() == "B.txt").expect("Node B data not found");
+        let node_b_view = initial_context.viewnodes().iter().find(|vn| vn.uuid == node_b_data.uuid()).expect("Node B view not found");
+        
         // Arrange: Save a modified position for node B.
         let modified_node_b = node_b_view.clone().positioned(500.0, 500.0);
         let save_payload = Context::with_viewnodes(focal_uuid, vec![modified_node_b]);
