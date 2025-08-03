diff --git a/karta_server/src/context/mod.rs b/karta_server/src/context/mod.rs
index c39ad4b..6a8fc70 100644
--- a/karta_server/src/context/mod.rs
+++ b/karta_server/src/context/mod.rs
@@ -35,12 +35,12 @@ mod tests {
         let context_db = test_ctx_db(func_name);
     }
 
-    #[test]
-    fn context_file_name_is_uuid_with_ctx_extension() {
-        let func_name = "context_file_name_is_uuid_with_ctx_extension";
-        let context_db = test_ctx_db(func_name);
-
-        // What are we testing here?
-        todo!();
-    }
+    // #[test]
+    // fn context_file_name_is_uuid_with_ctx_extension() {
+    //     let func_name = "context_file_name_is_uuid_with_ctx_extension";
+    //     let context_db = test_ctx_db(func_name);
+
+    //     // What are we testing here?
+    //     todo!();
+    // }
 }
\ No newline at end of file
diff --git a/karta_server/src/elements/edge.rs b/karta_server/src/elements/edge.rs
index 0b72918..3849edc 100644
--- a/karta_server/src/elements/edge.rs
+++ b/karta_server/src/elements/edge.rs
@@ -11,8 +11,8 @@ use super::{attribute::Attribute, node_path::NodePath, SysTime};
 pub struct Edge {
     uuid: Uuid,
     db_id: Option<DbId>,
-    source: NodePath,
-    target: NodePath,
+    source: Uuid,
+    target: Uuid,
     contains: bool,
     created_time: SysTime,
     modified_time: SysTime,
@@ -20,9 +20,9 @@ pub struct Edge {
 }
 
 impl Edge {
-    pub fn new(source: &NodePath, target: &NodePath) -> Self {
+    pub fn new(source: &Uuid, target: &Uuid) -> Self {
         let now = SysTime(SystemTime::now());
-        let name_to_hash = format!("{}:{}:{}:{}", source.buf().to_string_lossy(), target.buf().to_string_lossy(), now.0.duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().as_millis(), "edge");
+        let name_to_hash = format!("{}:{}:{}:{}", source.to_string(), target.to_string(), now.0.duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().as_millis(), "edge");
         let new_uuid = Uuid::new_v5(&Uuid::NAMESPACE_URL, name_to_hash.as_bytes());
         Self {
             uuid: new_uuid,
@@ -36,13 +36,13 @@ impl Edge {
         }
     }
 
-    pub fn new_cont(source: &NodePath, target: &NodePath) -> Self {
+    pub fn new_cont(source: &Uuid, target: &Uuid) -> Self {
         let attrs: Vec<Attribute> = vec![
             Attribute::new_contains()
         ];
         let now = SysTime(SystemTime::now());
         // Placeholder for V5 UUID generation
-        let name_to_hash = format!("{}:{}:{}:{}", source.buf().to_string_lossy(), target.buf().to_string_lossy(), now.0.duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().as_millis(), "edge_cont");
+        let name_to_hash = format!("{}:{}:{}:{}", source.to_string(), target.to_string(), now.0.duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().as_millis(), "edge_cont");
         let new_uuid = Uuid::new_v5(&Uuid::NAMESPACE_URL, name_to_hash.as_bytes());
         Self {
             uuid: new_uuid,
@@ -60,11 +60,11 @@ impl Edge {
         self.db_id
     }
 
-    pub fn source(&self) -> &NodePath {
+    pub fn source(&self) -> &Uuid {
         &self.source
     }
 
-    pub fn target(&self) -> &NodePath {
+    pub fn target(&self) -> &Uuid {
         &self.target
     }
 
@@ -115,8 +115,8 @@ impl DbUserValue for Edge {
     fn to_db_values(&self) -> Vec<DbKeyValue> {
         let mut values = Vec::new();
         values.push(DbKeyValue::from(("uuid", self.uuid.to_string()))); // Added uuid
-        values.push(DbKeyValue::from(("source", self.source.clone())));
-        values.push(DbKeyValue::from(("target", self.target.clone())));
+        values.push(DbKeyValue::from(("source", self.source.to_string())));
+        values.push(DbKeyValue::from(("target", self.target.to_string())));
         values.push(DbKeyValue::from(("created_time", self.created_time.clone())));
         values.push(DbKeyValue::from(("modified_time", self.modified_time.clone())));
         // Note: 'contains' is implicitly handled by attributes if it's stored as one.
@@ -174,8 +174,10 @@ impl TryFrom<DbElement> for Edge {
         let edge = Edge {
             db_id: Some(db_id),
             uuid,
-            source: NodePath::try_from(source_val)?,
-            target: NodePath::try_from(target_val)?,
+            source: Uuid::from_str(&source_val.to_string())
+                .map_err(|e| DbError::from(format!("Failed to parse source UUID: {}", e)))?,
+            target: Uuid::from_str(&target_val.to_string())
+                .map_err(|e| DbError::from(format!("Failed to parse target UUID: {}", e)))?,
             contains: contains.is_some(), // 'contains' is optional, derived from attribute presence
             created_time: SysTime::try_from(created_time_val)?,
             modified_time: SysTime::try_from(modified_time_val)?,
diff --git a/karta_server/src/elements/node.rs b/karta_server/src/elements/node.rs
index 9822395..d176933 100644
--- a/karta_server/src/elements/node.rs
+++ b/karta_server/src/elements/node.rs
@@ -110,9 +110,9 @@ impl DataNode {
         let mut uuid = Uuid::new_v5(&Uuid::NAMESPACE_URL, hash.as_bytes());
 
         // Root node has a special uuid
-        // if *path == NodePath::root() {
-        //     uuid = ROOT_UUID;
-        // }
+        if *path == NodePath::root() {
+            uuid = ROOT_UUID;
+        }
 
         DataNode {
             db_id: None,
diff --git a/karta_server/src/elements/node_path.rs b/karta_server/src/elements/node_path.rs
index ac14ccb..7135e0a 100644
--- a/karta_server/src/elements/node_path.rs
+++ b/karta_server/src/elements/node_path.rs
@@ -1,4 +1,4 @@
-use std::path::PathBuf;
+use std::{fmt, path::PathBuf};
 
 use agdb::{DbError, DbValue};
 use uuid::Uuid;
@@ -185,6 +185,12 @@ impl NodePath {
     }
 }
 
+impl fmt::Display for NodePath {
+    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
+        write!(f, "{}", self.0.to_string_lossy())
+    }
+}
+
 impl From<String> for NodePath {
     fn from(path_str: String) -> Self {
         if path_str == "vault" {
@@ -209,13 +215,13 @@ impl TryFrom<DbValue> for NodePath {
     type Error = DbError;
 
     fn try_from(value: DbValue) -> Result<Self, Self::Error> {
-        Ok(NodePath::from_alias(&value.to_string()))
+        Ok(NodePath::new(PathBuf::from(value.to_string())))
     }
 }
 
 impl From<NodePath> for DbValue {
     fn from(path: NodePath) -> Self {
-        path.alias().into()
+        path.to_string().into()
     }
 }
 
diff --git a/karta_server/src/graph_agdb/graph_core.rs b/karta_server/src/graph_agdb/graph_core.rs
index ebc98a4..a6302eb 100644
--- a/karta_server/src/graph_agdb/graph_core.rs
+++ b/karta_server/src/graph_agdb/graph_core.rs
@@ -3,16 +3,58 @@ use std::{
     path::{self, PathBuf},
 };
 
-use agdb::QueryBuilder;
+use agdb::{DbUserValue, QueryBuilder};
 
 use crate::{
-    elements::nodetype::ARCHETYPES,
+    elements::{node::ROOT_UUID, nodetype::ARCHETYPES},
     graph_traits::{self, graph_core::GraphCore, graph_node::GraphNodes},
     prelude::{DataNode, GraphEdge, NodePath, NodeTypeId, StoragePath},
 };
 
 use super::GraphAgdb;
 
+impl GraphAgdb {
+    fn _initialize_db(&mut self) {
+        let archetypes = ARCHETYPES;
+
+        self.db.transaction_mut(|transaction| {
+            // Create indexes first to ensure they exist before any data is queried.
+            transaction.exec_mut(&QueryBuilder::insert().index("uuid").query())?;
+            transaction.exec_mut(&QueryBuilder::insert().index("ntype").query())?;
+            transaction.exec_mut(&QueryBuilder::insert().index("path").query())?;
+
+            for atype in archetypes.iter() {
+                let atype_path = NodePath::atype(atype);
+                let ntype = if atype_path == NodePath::root() {
+                    NodeTypeId::root_type()
+                } else if atype_path == NodePath::vault() {
+                    NodeTypeId::dir_type()
+                } else {
+                    NodeTypeId::archetype_type()
+                };
+
+                let node = DataNode::new(&atype_path, ntype);
+                let nodeuuid = node.uuid();
+
+                transaction.exec_mut(&QueryBuilder::insert().nodes().aliases(node.uuid().to_string()).values(node).query())?;
+
+                if atype_path != NodePath::root() {
+                    let root_to_atype_edge = crate::prelude::Edge::new_cont(&ROOT_UUID, &nodeuuid);
+                    transaction.exec_mut(
+                        &QueryBuilder::insert()
+                            .edges()
+                            .from(root_to_atype_edge.source().to_string())
+                            .to(root_to_atype_edge.target().to_string())
+                            .values_uniform(root_to_atype_edge.to_db_values()) // Use values_uniform for single edge insertion
+                            .query(),
+                    )?;
+                }
+            }
+            Ok::<(), agdb::QueryError>(())
+        }).unwrap();
+    }
+}
+
 /// Implementation block for the Graph struct itself.
 /// Includes constructors and utility functions.
 impl GraphCore for GraphAgdb {
@@ -68,68 +110,9 @@ impl GraphCore for GraphAgdb {
         };
 
         if !open_existing {
-            let archetypes = ARCHETYPES;
-
-            // println!("Length of archetypes {}", archetypes.len());
-
-            archetypes.iter().for_each(|at| {
-                // println!("{}", at);
-            });
-
-            archetypes.iter().for_each(|atype| {
-                let atype_path = NodePath::atype(*atype);
-                // println!("Atypepath {:?}", atype_path);
-
-                // println!("Creating archetype node: {}", atype_path.alias());
-
-                let ntype = if atype_path == NodePath::root() {
-                    // println!("Root node in question");
-                    NodeTypeId::root_type()
-                } else if atype_path == NodePath::vault() { // Check for vault specifically
-                    // println!("Vault node in question");
-                    NodeTypeId::dir_type() // Assign core/dir to vault
-                } else {
-                    // println!("Other archetype node in question");
-                    NodeTypeId::archetype_type() // Other archetypes get core/archetype
-                };
-
-                let node: DataNode = DataNode::new(&atype_path, ntype);
-
-                // println!("alias is {}", atype_path.alias());
-
-                let query = giraphe.db.exec_mut(
-                    &QueryBuilder::insert()
-                        .nodes()
-                        .aliases(atype_path.alias())
-                        .values(node)
-                        .query(),
-                );
-
-                match query {
-                    Ok(_) => {
-                        // println!("Created archetype node: {}", atype_path.alias());
-                    }
-                    Err(ref err) => {
-                        panic!("Failed to create archetype node: {}", err);
-                        // println!("Failed to create archetype node: {}", err);
-                    }
-                }
-
-                if atype_path != NodePath::root() {
-                    let root_to_atype_edge =
-                        crate::prelude::Edge::new_cont(&NodePath::root(), &atype_path);
-
-                    giraphe.insert_edges(vec![root_to_atype_edge]);
-                } else {
-                    // println!("Root node, no autoparenting");
-                }
-            });
+            giraphe._initialize_db();
         }
 
-        // Indexes for faster lookup based on attributes
-        giraphe.db.exec_mut(QueryBuilder::insert().index("uuid").query());
-        giraphe.db.exec_mut(QueryBuilder::insert().index("ntype").query());
-
         return giraphe;
     }
 
diff --git a/karta_server/src/graph_agdb/graph_edge.rs b/karta_server/src/graph_agdb/graph_edge.rs
index 40d28ac..049661c 100644
--- a/karta_server/src/graph_agdb/graph_edge.rs
+++ b/karta_server/src/graph_agdb/graph_edge.rs
@@ -76,9 +76,9 @@ impl GraphEdge for GraphAgdb {
             let edge = self.db.exec_mut(
                 &QueryBuilder::insert()
                     .edges()
-                    .from(parent.alias())
-                    .to(child.alias())
-                    .values_uniform(edge)
+                    .from(parent.to_string())
+                    .to(child.to_string())
+                    .values(edge)
                     .query(),
             ); // For whatever reason this does not insert the attribute into the edge.
 
diff --git a/karta_server/src/graph_agdb/graph_node.rs b/karta_server/src/graph_agdb/graph_node.rs
index 26e6002..a946a61 100644
--- a/karta_server/src/graph_agdb/graph_node.rs
+++ b/karta_server/src/graph_agdb/graph_node.rs
@@ -13,25 +13,25 @@ use super::GraphAgdb;
 
 impl GraphNodes for GraphAgdb {
     fn open_node(&self, handle: &NodeHandle) -> Result<DataNode, Box<dyn Error>> {
-        let mut node: Result<agdb::QueryResult, agdb::QueryError>;
-
-        match handle {
+        let query_result = match handle {
             NodeHandle::Path(path) => {
-                let alias = path.alias();
-                node = self.db.exec(&QueryBuilder::select().ids(alias).query());
+                // The search query finds the ID(s) based on the indexed "path" property.
+                let search_query = QueryBuilder::search().index("path").value(path.to_string()).query();
+                // The select query then uses those ID(s) to retrieve the full elements with all their properties.
+                self.db.exec(&QueryBuilder::select().ids(search_query).query())
             }
             NodeHandle::Uuid(id) => {
-                node = self.db.exec(&QueryBuilder::select().search().index("uuid").value(id.to_string()).query());
-                // println!("Node is {:#?}", node); // Kept for potential debugging, but commented
+                // The Uuid is the primary alias, so we can select by it directly.
+                self.db.exec(&QueryBuilder::select().ids(id.to_string()).query())
             }
-        }
-        
-        match node {
-            Ok(node) => {
-                let db_element = node.elements.first().ok_or_else(|| "DB query returned Ok but no elements found.")?;
+        };
+
+        match query_result {
+            Ok(result) => {
+                let db_element = result.elements.first().ok_or_else(|| "DB query returned Ok but no elements found.")?;
                 DataNode::try_from(db_element.clone()).map_err(|e| Box::new(e) as Box<dyn Error>)
             }
-            Err(_err) => {
+            Err(err) => {
                 match handle {
                     NodeHandle::Path(path_handle) => {
                         Err(format!("Node with path {:?} not found in DB", path_handle).into())
@@ -45,175 +45,106 @@ impl GraphNodes for GraphAgdb {
     }
 
     fn open_node_connections(&self, path: &NodePath) -> Vec<(DataNode, Edge)> {
-        // Resolve the full path to the node - This seems unused now, consider removing if not needed elsewhere.
-        // let full_path = path.full(&self.root_path);
-        // let is_physical = full_path.exists();
-        // let is_dir = full_path.is_dir();
-
-        // let as_str = path.alias(); // Unused
-
-        let mut node_ids: Vec<DbId> = Vec::new();
-        let mut edge_ids: Vec<DbId> = Vec::new();
-
-        // Links from node
-        // println!("Searching for links from node {}", path.alias());
-        let links = self.db.exec(
-            &QueryBuilder::search()
-                .from(path.alias())
-                .where_()
-                .distance(agdb::CountComparison::LessThanOrEqual(2))
-                .query(),
-        );
-
-        match links {
-            Ok(links) => {
-                for elem in links.elements.iter() {
-                    if elem.id.0 < 0 {
-                        // Is edge
-                        edge_ids.push(elem.id);
-                    } else if elem.id.0 > 0 {
-                        // Is node
-                        node_ids.push(elem.id);
-                    }
-                }
+        let start_node = match self.open_node(&NodeHandle::Path(path.clone())) {
+            Ok(node) => node,
+            Err(_) => return vec![],
+        };
+        let start_node_uuid = start_node.uuid().to_string();
+
+        // 1. Search for the IDs of all elements connected to the start_node.
+        let mut all_element_ids = std::collections::HashSet::new();
+        if let Ok(links) = self.db.exec(&QueryBuilder::search().from(start_node_uuid.clone()).where_().distance(agdb::CountComparison::LessThanOrEqual(2)).query()) {
+            for elem in links.elements {
+                all_element_ids.insert(elem.id);
             }
-            Err(_e) => {}
         }
-
-        // Backlinks to node
-        let backlinks = self.db.exec(
-            &QueryBuilder::search()
-                .to(path.alias())
-                .where_()
-                .distance(agdb::CountComparison::LessThanOrEqual(2))
-                .query(),
-        );
-
-        match backlinks {
-            Ok(backlinks) => {
-                for elem in backlinks.elements.iter() {
-                    if elem.id.0 < 0 {
-                        // Is edge
-                        edge_ids.push(elem.id);
-                    } else if elem.id.0 > 0 {
-                        // Is node
-                        // let balias = self // This balias was unused
-                        //     .db
-                        //     .exec(&QueryBuilder::select().aliases().ids(elem.id).query());
-                        // println!("balias: {:?}", balias);
-                        node_ids.push(elem.id);
-                    }
-                }
+        if let Ok(backlinks) = self.db.exec(&QueryBuilder::search().to(start_node_uuid).where_().distance(agdb::CountComparison::LessThanOrEqual(2)).query()) {
+            for elem in backlinks.elements {
+                all_element_ids.insert(elem.id);
             }
-            Err(_e) => {}
         }
 
-        let full_nodes = match self.db.exec(&QueryBuilder::select().ids(node_ids).query()) {
-            Ok(nodes) => nodes.elements,
-            Err(_e) => vec![],
+        // 2. Select the full elements for all found IDs.
+        let db_elements = match self.db.exec(&QueryBuilder::select().ids(all_element_ids.into_iter().collect::<Vec<DbId>>()).query()) {
+            Ok(res) => res.elements,
+            Err(_) => return vec![],
         };
-        let full_edges = match self.db.exec(&QueryBuilder::select().ids(edge_ids).query()) {
-            Ok(edges) => edges.elements,
-            Err(_e) => vec![],
-        };
-
-        let connections: Vec<(DataNode, Edge)> = full_nodes
-            .iter()
-            .filter_map(|node| {
-                let node = DataNode::try_from(node.clone()).unwrap();
 
-                // Ignore the original node
-                if node.path() == *path {
-                    return None;
+        // 3. Separate the full elements into nodes and edges.
+        let mut nodes = vec![];
+        let mut edges = vec![];
+        for elem in db_elements {
+            if elem.id.0 > 0 { // Nodes have positive IDs
+                if let Ok(node) = DataNode::try_from(elem) {
+                    // Exclude the starting node itself from the results.
+                    if node.uuid() != start_node.uuid() {
+                        nodes.push(node);
+                    }
                 }
-                let edge = full_edges
-                    .iter()
-                    .find(|edge| {
-                        if edge.from.unwrap() == node.id().unwrap()
-                            || edge.to.unwrap() == node.id().unwrap()
-                        {
-                            true
-                        } else {
-                            false
-                        }
-                    })
-                    .unwrap();
-                let edge = Edge::try_from(edge.clone()).unwrap();
-
-                Some((node, edge))
-            })
-            .collect();
+            } else { // Edges have negative IDs
+                if let Ok(edge) = Edge::try_from(elem) {
+                    edges.push(edge);
+                }
+            }
+        }
+
+        // 4. Pair up the nodes and edges.
+        let mut connections = vec![];
+        for node in nodes {
+            if let Some(edge) = edges.iter().find(|e| {
+                (e.source() == &start_node.uuid() && e.target() == &node.uuid()) ||
+                (e.target() == &start_node.uuid() && e.source() == &node.uuid())
+            }) {
+                connections.push((node, edge.clone()));
+            }
+        }
 
         connections
     }
 
     /// Inserts a Node.
     fn insert_nodes(&mut self, nodes: Vec<DataNode>) {
-        for mut node in nodes {
+        for node in nodes {
             let npath = node.path();
+            let node_uuid = node.uuid().clone();
 
-            let existing = self.db.exec(
-                &QueryBuilder::select()
-                    .ids(node.path().alias().clone())
-                    .query(),
-            );
-
-            match existing {
-                Ok(_) => {
-                    // Node already exists, consider if this should be an error or a silent skip/update.
-                    // For now, it prints and continues, effectively skipping re-insertion of the node itself if alias matches.
-                    // println!("Node with alias {} already exists in DB during insert_nodes", npath.alias());
-                }
-                Err(_e) => {
-                    // Node doesn't exist by alias, proceed to insertion
-                }
-            }
+            // Check if the node exists *before* the insert/update operation.
+            let is_new_node = self.db.exec(&QueryBuilder::select().ids(node_uuid.to_string()).query()).is_err();
 
+            // Let agdb handle the insert-or-update logic based on the alias (UUID).
             let node_query = self.db.exec_mut(
                 &QueryBuilder::insert()
                     .nodes()
-                    .aliases(node.path().alias())
+                    .aliases(node_uuid.to_string())
                     .values(node)
                     .query(),
             );
 
-            match node_query {
-                Ok(nodeqr) => {
-                    let node_elem = &nodeqr.elements[0];
-                    let nid = node_elem.id;
-                    // If parent is not root, ensure parent node exists in DB.
-                    // This recursively ensures the path to the node is created.
-                    let parent_path = npath.parent();
-                    println!("Parent of node is: {:#?}", parent_path);
-                    match parent_path {
-                        Some(parent_path) => {
-                            if parent_path.parent().is_some() {
-                                // println!("About to ensure parent node: {:?}", parent_path);
-                                // Check if parent exists before trying to insert, to avoid redundant work / errors if it's already there.
-                                // This basic check might not be sufficient if parent needs specific attributes or type.
-                                if self.db.exec(&QueryBuilder::select().ids(parent_path.alias()).query()).is_err() {
-                                    let parent_node = DataNode::new(&parent_path, NodeTypeId::dir_type());
-                                    self.insert_nodes(vec![parent_node]); // Recursive call
-                                }
-                                
-                                let edge = Edge::new(&parent_path, &npath);
-                                self.insert_edges(vec![edge]);
+            // Only create parent links if the node was newly inserted and the operation was successful.
+            if is_new_node && node_query.is_ok() {
+                if let Some(parent_path) = npath.parent() {
+                    // The _initialize_db function handles creating edges from the root to the archetypes.
+                    // This recursive logic should only handle creating the directory structure *within* those archetypes.
+                    if parent_path != NodePath::root() {
+                        let parent_node_result = self.open_node(&NodeHandle::Path(parent_path.clone()));
+                        
+                        let parent_node = match parent_node_result {
+                            Ok(p_node) => p_node,
+                            Err(_) => {
+                                // If parent doesn't exist, create it recursively.
+                                let parent_node_data = DataNode::new(&parent_path, NodeTypeId::dir_type());
+                                self.insert_nodes(vec![parent_node_data]);
+                                // Now, it must exist.
+                                self.open_node(&NodeHandle::Path(parent_path.clone()))
+                                    .expect("Parent node should exist after recursive insertion.")
                             }
-                        }
-                        None => {
-                            // Parent is root, create edge from root.
-                            // Ensure root node itself exists if this is the first node ever.
-                            // For simplicity, assuming root node (alias "/") is implicitly handled or pre-exists.
-                            let root_edge = Edge::new(&NodePath::root(), &npath);
-                            self.insert_edges(vec![root_edge]);
-                        }
+                        };
+
+                        let edge = Edge::new_cont(&parent_node.uuid(), &node_uuid);
+                        self.insert_edges(vec![edge]);
                     }
                 }
-                Err(_e) => {
-                    // println!("Failed to insert node with alias {}: {}", npath.alias(), e);
-                }
             }
-            // println!("Processed node for insertion: {:#?}", npath.alias());
         }
     }
 }
diff --git a/karta_server/src/graph_traits/graph_core.rs b/karta_server/src/graph_traits/graph_core.rs
index 7bff5cd..fc39ecb 100644
--- a/karta_server/src/graph_traits/graph_core.rs
+++ b/karta_server/src/graph_traits/graph_core.rs
@@ -122,8 +122,9 @@ mod tests {
                 let parent_node = ctx.with_graph_db(|db| db.open_node(&NodeHandle::Path(parent_path.clone())));
                 assert_eq!(parent_node.is_ok(), true, "Parent of node {} not found", path.alias());
 
-                let edge = ctx.with_graph_db(|db| db.get_edge_strict(&parent_path, &path));
-                assert_eq!(edge.is_ok(), true, "Edge not found");
+                let connections = ctx.with_graph_db(|db| db.open_node_connections(&parent_path));
+                let edge_exists = connections.iter().any(|(node, _)| node.path() == path);
+                assert!(edge_exists, "Edge from root to {} not found", path.alias());
             }
         });
     }
diff --git a/karta_server/src/graph_traits/graph_node.rs b/karta_server/src/graph_traits/graph_node.rs
index 80b0af8..5c5cf80 100644
--- a/karta_server/src/graph_traits/graph_node.rs
+++ b/karta_server/src/graph_traits/graph_node.rs
@@ -74,8 +74,8 @@ mod tests {
         // Check that the tuples contain matching edges
         for (i, connection) in connections.iter().enumerate() {
             assert!(
-                *connection.1.source() == connection.0.path()
-                    || *connection.1.target() == connection.0.path(),
+                *connection.1.source() == connection.0.uuid()
+                    || *connection.1.target() == connection.0.uuid(),
                 "Edge should be connected to Node"
             )
         }
diff --git a/karta_server/src/server/karta_service.rs b/karta_server/src/server/karta_service.rs
index 8b87ecf..3ac21e3 100644
--- a/karta_server/src/server/karta_service.rs
+++ b/karta_server/src/server/karta_service.rs
@@ -1,7 +1,7 @@
 use std::{collections::{HashMap, HashSet}, error::Error, path::PathBuf, sync::Arc};
-use uuid::Uuid;
 
 use tokio::sync::RwLock;
+use uuid::Uuid;
 
 use crate::{context::{context::Context, context_db::ContextDb}, elements::node_path::NodeHandle, fs_reader, prelude::*};
 
@@ -135,7 +135,7 @@ impl KartaService {
             child_fs_datanodes = fs_nodes_from_destructure;
             if let Some(focal_node_unwrapped) = &focal_fs_datanode {
                 for child_node in &child_fs_datanodes {
-                    fs_edges.push(Edge::new(&focal_node_unwrapped.path(), &child_node.path()));
+                    fs_edges.push(Edge::new_cont(&focal_node_unwrapped.uuid(), &child_node.uuid()));
                 }
             }
         } else if absolute_path.is_file() {
@@ -145,7 +145,8 @@ impl KartaService {
                 if let Some(parent_path) = path.parent() {
                     if let Some(focal_file_node_unwrapped) = &focal_fs_datanode {
                         // We can still infer the edge from the filesystem path.
-                        fs_edges.push(Edge::new(&parent_path, &focal_file_node_unwrapped.path()));
+                        let parent_node = self.data().open_node(&NodeHandle::Path(parent_path.clone())).unwrap();
+                        fs_edges.push(Edge::new_cont(&parent_node.uuid(), &focal_file_node_unwrapped.uuid()));
                     }
                 }
             }
@@ -158,15 +159,15 @@ impl KartaService {
         let db_focal_datanode_optional = self.data().open_node(&NodeHandle::Path(path.clone())).ok();
         let db_child_connections = self.data().open_node_connections(&path);
         
-        let mut db_child_datanodes_map: HashMap<NodePath, DataNode> = HashMap::new();
+        let mut db_child_datanodes_map: HashMap<Uuid, DataNode> = HashMap::new();
         let mut db_edges_vec: Vec<Edge> = Vec::new();
         for (node, edge) in db_child_connections {
-            db_child_datanodes_map.insert(node.path().clone(), node);
+            db_child_datanodes_map.insert(node.uuid().clone(), node);
             db_edges_vec.push(edge);
         }
 
-        let mut final_datanodes_map: HashMap<NodePath, DataNode> = HashMap::new();
-        let mut final_edges_set: HashSet<(NodePath, NodePath)> = HashSet::new();
+        let mut final_datanodes_map: HashMap<Uuid, DataNode> = HashMap::new();
+        let mut final_edges_set: HashSet<(Uuid, Uuid)> = HashSet::new();
         let mut reconciled_edges: Vec<Edge> = Vec::new();
 
         let definitive_focal_node = match db_focal_datanode_optional {
@@ -182,7 +183,8 @@ impl KartaService {
                     Ok(vault_node) => {
                         additional_nodes_to_include.push(vault_node);
                         // Create and add the edge from vault to the focal node
-                        let vault_to_focal_edge = Edge::new(&NodePath::vault(), &definitive_focal_node.path());
+                        let vault_node = self.data().open_node(&NodeHandle::Path(NodePath::vault())).unwrap();
+                        let vault_to_focal_edge = Edge::new_cont(&vault_node.uuid(), &definitive_focal_node.uuid());
                         additional_edges_to_include.push(vault_to_focal_edge);
                     }
                     Err(e) => {
@@ -194,7 +196,7 @@ impl KartaService {
             }
         }
 
-        final_datanodes_map.insert(definitive_focal_node.path().clone(), definitive_focal_node.clone());
+        final_datanodes_map.insert(definitive_focal_node.uuid().clone(), definitive_focal_node.clone());
 
         // --- Parent Node Handling (Moved Up) ---
         let mut parent_uuid: Option<Uuid> = None;
@@ -208,30 +210,30 @@ impl KartaService {
             
             parent_uuid = Some(parent_node.uuid());
             // Ensure parent is in the map so its edges can be reconciled.
-            final_datanodes_map.entry(parent_path).or_insert(parent_node);
+            final_datanodes_map.entry(parent_node.uuid()).or_insert(parent_node);
         }
         // --- End Parent Node Handling ---
 
         for fs_child_node in &child_fs_datanodes {
-            match db_child_datanodes_map.get(&fs_child_node.path()) {
+            match db_child_datanodes_map.get(&fs_child_node.uuid()) {
                 Some(db_child_node) => {
-                    final_datanodes_map.insert(db_child_node.path().clone(), db_child_node.clone());
+                    final_datanodes_map.insert(db_child_node.uuid().clone(), db_child_node.clone());
                 }
                 None => {
-                    final_datanodes_map.insert(fs_child_node.path().clone(), fs_child_node.clone());
+                    final_datanodes_map.insert(fs_child_node.uuid().clone(), fs_child_node.clone());
                 }
             }
         }
         // Include other DB-connected nodes not present in FS (e.g., parents, other virtual links)
         for (db_node_path, db_node_data) in db_child_datanodes_map.iter() {
-            if !final_datanodes_map.contains_key(db_node_path) {
-                final_datanodes_map.insert(db_node_path.clone(), db_node_data.clone());
+            if !final_datanodes_map.contains_key(&db_node_data.uuid()) {
+                final_datanodes_map.insert(db_node_data.uuid().clone(), db_node_data.clone());
             }
         }
 
         // Add any additional nodes (like the vault node)
         for node_to_add in &additional_nodes_to_include {
-            final_datanodes_map.entry(node_to_add.path().clone()).or_insert_with(|| node_to_add.clone());
+            final_datanodes_map.entry(node_to_add.uuid().clone()).or_insert_with(|| node_to_add.clone());
         }
         
         for fs_edge in fs_edges {
@@ -340,11 +342,11 @@ assert!(datanodes.iter().any(|n| n.path() == NodePath::root()), "NodePath::root(
         let vault_node = datanodes.iter().find(|n| n.path() == NodePath::vault()).expect("User root DataNode not found");
         
         assert!(
-            edges.iter().any(|e| e.source() == &vault_node.path() && e.target() == &test_dir_node.path()),
+            edges.iter().any(|e| e.source() == &vault_node.uuid() && e.target() == &test_dir_node.uuid()),
             "Missing edge from vault to test_dir"
         );
         assert!(
-            edges.iter().any(|e| e.source() == &vault_node.path() && e.target() == &test_file_node.path()),
+            edges.iter().any(|e| e.source() == &vault_node.uuid() && e.target() == &test_file_node.uuid()),
             "Missing edge from vault to test_file.txt"
         );
     }
diff --git a/karta_server/src/utils.rs b/karta_server/src/utils.rs
index fbc8989..9fae350 100644
--- a/karta_server/src/utils.rs
+++ b/karta_server/src/utils.rs
@@ -52,10 +52,11 @@ pub mod utils {
                     .data_dir()
                     .join(&name); // Unique directory for this specific test's vault
 
-            if !vault_root_path.exists() {
-                create_dir_all(&vault_root_path)
-                    .expect("Failed to create vault_root_path for KartaServiceTestContext");
+            if vault_root_path.exists() {
+                std::fs::remove_dir_all(&vault_root_path).expect("Failed to remove old test directory");
             }
+            create_dir_all(&vault_root_path)
+                .expect("Failed to create vault_root_path for KartaServiceTestContext");
 
             // KartaService::new expects storage_dir to be the .karta directory.
             // It will create it if it doesn't exist within the vault_root_path.
