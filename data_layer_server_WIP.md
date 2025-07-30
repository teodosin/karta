diff --git a/karta_server/src/elements/node.rs b/karta_server/src/elements/node.rs
index d176933..d070d9c 100644
--- a/karta_server/src/elements/node.rs
+++ b/karta_server/src/elements/node.rs
@@ -170,6 +170,10 @@ impl DataNode {
     pub fn attributes(&self) -> Vec<Attribute> {
         self.attributes.clone()
     }
+
+    pub fn set_attributes(&mut self, attributes: Vec<Attribute>) {
+        self.attributes = attributes;
+    }
 }
 
 impl TryFrom<DbElement> for DataNode {
diff --git a/karta_server/src/server/mod.rs b/karta_server/src/server/mod.rs
index e4f0ca5..a57bb60 100644
--- a/karta_server/src/server/mod.rs
+++ b/karta_server/src/server/mod.rs
@@ -56,12 +56,14 @@ pub struct AppState {
 pub fn create_router(state: AppState) -> Router<()> {
     let cors = CorsLayer::new()
         .allow_origin("http://localhost:5173".parse::<axum::http::HeaderValue>().unwrap())
-        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS]) // Specify methods
+        .allow_methods(Any) // Allow all methods
         .allow_headers(Any); // Allow any headers
 
     let router = Router::new()
         .route("/", get(|| async { "Karta Server" }))
         .route("/api/asset/{*path}", get(asset_endpoints::get_asset))
+        .route("/api/nodes", post(write_endpoints::create_node))
+        .route("/api/nodes/{id}", put(write_endpoints::update_node))
         .route("/api/ctx/{id}", put(write_endpoints::save_context))
         .route("/ctx/{*id}", get(context_endpoints::open_context_from_fs_path)) // Corrected wildcard syntax
         .layer(cors) // Apply the CORS layer
diff --git a/karta_server/src/server/write_endpoints.rs b/karta_server/src/server/write_endpoints.rs
index 4c8eb1a..a93b533 100644
--- a/karta_server/src/server/write_endpoints.rs
+++ b/karta_server/src/server/write_endpoints.rs
@@ -3,9 +3,88 @@ use axum::{
     http::StatusCode,
     Json,
 };
+use serde::Deserialize;
 use uuid::Uuid;
 
-use crate::{context::context::Context, server::AppState};
+use crate::{
+    context::context::Context,
+    elements::{
+        attribute::{Attribute, AttrValue},
+        node::DataNode,
+        node_path::{NodeHandle, NodePath},
+        nodetype::NodeTypeId,
+    },
+    graph_traits::graph_node::GraphNodes,
+    server::AppState,
+};
+
+#[derive(Deserialize)]
+pub struct CreateNodePayload {
+    name: String,
+    ntype: NodeTypeId,
+    parent_path: String,
+    attributes: Vec<Attribute>,
+}
+
+#[derive(Deserialize)]
+pub struct UpdateNodePayload {
+    attributes: Vec<Attribute>,
+}
+
+pub async fn create_node(
+    State(app_state): State<AppState>,
+    Json(payload): Json<CreateNodePayload>,
+) -> Result<Json<DataNode>, StatusCode> {
+    let mut service = app_state.service.write().unwrap();
+    let parent_path = NodePath::from(payload.parent_path);
+    let mut name = payload.name.clone();
+    let mut final_path = parent_path.join(&name);
+    let mut counter = 2;
+
+    // Loop until we find a unique path
+    while service.data().open_node(&NodeHandle::Path(final_path.clone())).is_ok() {
+        name = format!("{}_{}", payload.name.clone(), counter);
+        final_path = parent_path.join(&name);
+        counter += 1;
+    }
+
+    let mut new_node = DataNode::new(&final_path, payload.ntype);
+    
+    // Update the name attribute if it was changed
+    let mut attributes = payload.attributes;
+    if let Some(attr) = attributes.iter_mut().find(|a| a.name == "name") {
+        attr.value = AttrValue::String(name.clone());
+    } else {
+        attributes.push(Attribute::new_string("name".to_string(), name));
+    }
+
+    new_node.set_attributes(attributes);
+    new_node.set_name(&new_node.path().name());
+
+    service.data_mut().insert_nodes(vec![new_node.clone()]);
+
+    Ok(Json(new_node))
+}
+
+pub async fn update_node(
+    State(app_state): State<AppState>,
+    AxumPath(id): AxumPath<Uuid>,
+    Json(payload): Json<UpdateNodePayload>,
+) -> Result<Json<DataNode>, StatusCode> {
+    let mut service = app_state.service.write().unwrap();
+
+    let mut node = match service.data().open_node(&NodeHandle::Uuid(id)) {
+        Ok(node) => node,
+        Err(_) => return Err(StatusCode::NOT_FOUND),
+    };
+
+    node.set_attributes(payload.attributes);
+    node.update_modified_time();
+
+    service.data_mut().insert_nodes(vec![node.clone()]);
+
+    Ok(Json(node))
+}
 
 pub async fn save_context(
     State(app_state): State<AppState>,
