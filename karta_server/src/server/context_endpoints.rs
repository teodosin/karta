// Karta Server - Context API Endpoints

use axum::{
    extract::{Path as AxumPath, State},
    response::{IntoResponse, Response},
    Json, Router, routing::get,
};
use std::path::PathBuf;
use std::sync::Arc; // Keep Arc for KartaService within AppState
use axum::http::StatusCode;
use std::error::Error as StdError; 

use crate::server::karta_service::KartaService;
use crate::server::AppState; // Import AppState
use crate::context::context::Context;
use crate::elements::node::DataNode;
use crate::elements::edge::Edge;
use crate::elements::node_path::NodePath;

// Helper to convert Box<dyn StdError> to an Axum Response
fn box_error_to_response(err: Box<dyn StdError>) -> Response {
    eprintln!("API Error: {:?}", err); 
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "error": err.to_string() })),
    )
        .into_response()
}

pub async fn open_context_from_fs_path(
    State(app_state): State<AppState>, 
    AxumPath(path_segments): AxumPath<String>,
) -> Result<Json<(Vec<DataNode>, Vec<Edge>, Context)>, Response> {
    // Acquire read lock to access KartaService.
    // .unwrap() is used here for simplicity; in production, consider graceful error handling for lock poisoning.
    let karta_service = match app_state.service.read() {
        Ok(lock) => lock,
        Err(poisoned) => {
            let err_msg = format!("Failed to acquire read lock on KartaService: {}", poisoned);
            return Err(box_error_to_response(err_msg.into()));
        }
    };
    
    let vault_path = PathBuf::from(karta_service.root_path());
    let processed_api_path = path_segments.trim_start_matches('/');

    let node_path_to_open: NodePath;

    if processed_api_path == "root" {
        node_path_to_open = NodePath::root();
        // For NodePath::root(), filesystem-based security checks are not applicable in the same way.
        // It's a virtual node. We proceed directly to opening it.
    } else {
        // Handle user_root and other relative paths
        let mut path_for_fs_check = processed_api_path;
        if path_for_fs_check == "." || path_for_fs_check.is_empty() {
            path_for_fs_check = ""; // Canonical empty path for user_root for FS checks
            node_path_to_open = NodePath::user_root();
        } else {
            // For any other path, it's relative to user_root
            // NodePath::from will prepend user_root if it's not an absolute-like path
            node_path_to_open = NodePath::from(path_for_fs_check.to_string());
        }

        // Security check for paths that relate to the filesystem vault
        let joined_path = vault_path.join(path_for_fs_check);
        if !joined_path.starts_with(&vault_path) {
            if path_for_fs_check.contains("..") {
                return Err(box_error_to_response("Path traversal attempt with '..' detected.".into()));
            }
            return Err(box_error_to_response(format!("Path '{}' appears to be outside the vault.", processed_api_path).into()));
        }
        // TODO: Implement robust path canonicalization and security checks for FS paths.
    }

    // Call the synchronous KartaService method directly.
    // Drop the read lock before calling a potentially blocking operation if KartaService methods were to become async
    // and required `&mut self`. For a read operation with `&self`, holding the read lock is fine.
    // If KartaService methods were long & synchronous, spawn_blocking would be better.
    // For now, direct call:
    match karta_service.open_context_from_path(node_path_to_open) {
        Ok(context_data) => Ok(Json(context_data)),
        Err(e) => Err(box_error_to_response(e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::karta_service::KartaService;
    use crate::utils::utils::KartaServiceTestContext;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        Router, 
    };
    use tower::ServiceExt;
    use serde_json::Value;
    use std::sync::{Arc, RwLock};

    // Helper to build a router for testing this specific endpoint
    fn test_router_for_context_endpoints(app_state: AppState) -> Router {
        Router::new()
            .route("/ctx/{*path_segments}", get(open_context_from_fs_path))
            .with_state(app_state)
    }

    #[tokio::test]
    async fn test_open_root_context_api() {
        let test_ctx = KartaServiceTestContext::new("api_open_root_ctx");
        test_ctx.create_file_in_vault("fileA.txt", b"content of file A").unwrap();
        test_ctx.create_dir_in_vault("dir1").unwrap();
        test_ctx.create_file_in_vault("dir1/fileB.txt", b"content of file B").unwrap();

        // KartaService from KartaServiceTestContext is not Arc<RwLock<KartaService>>
        // We need to wrap it.
        let app_state_service = test_ctx.service_arc.clone();

        let app_state = AppState {
            service: app_state_service,
            tx: tokio::sync::broadcast::channel(1).0, 
        };

        let router = test_router_for_context_endpoints(app_state.clone());

        let response = router
            .oneshot(
                Request::builder()
                    .uri("/ctx/.") // Requesting root with "."
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body_json: Value = serde_json::from_slice(&body).expect("Failed to parse JSON response");
        
        // Overall structure: Tuple of (Vec<DataNode>, Vec<Edge>, Context)
        assert!(body_json.is_array() && body_json.as_array().unwrap().len() == 3,
            "Response should be a 3-element array: [nodes, edges, context]");

        // Nodes checks
        let nodes_array = body_json.as_array().unwrap()[0].as_array().expect("Nodes element should be an array");
        assert_eq!(nodes_array.len(), 4, "Expected 4 nodes in user_root context: root, user_root, fileA.txt, dir1");

        let has_node_with_path = |nodes: &Vec<Value>, target_path: &str| -> bool {
            nodes.iter().any(|n| n.get("path").and_then(|p| p.as_str()) == Some(target_path))
        };

assert!(has_node_with_path(nodes_array, NodePath::root().buf().to_str().unwrap()), "Node 'root' (path: \"\") not found");
        assert!(has_node_with_path(nodes_array, "user_root/fileA.txt"), "Node 'user_root/fileA.txt' not found");
        assert!(has_node_with_path(nodes_array, "user_root/dir1"), "Node 'user_root/dir1' not found");
        assert!(has_node_with_path(nodes_array, NodePath::user_root().buf().to_str().unwrap()), "Node 'user_root' (focal) not found");

        // Edge checks (optional, but good for completeness)
        let edges_array = body_json.as_array().unwrap()[1].as_array().expect("Edges element should be an array");
        assert_eq!(edges_array.len(), 3, "Expected 3 edges: root->user_root, user_root->fileA.txt, user_root->dir1");
        
        let has_edge = |edges: &Vec<Value>, src: &str, tgt: &str| -> bool {
            edges.iter().any(|e| {
                e.get("source").and_then(|s| s.as_str()) == Some(src) &&
                e.get("target").and_then(|t| t.as_str()) == Some(tgt)
            })
        };
assert!(has_edge(edges_array, NodePath::root().buf().to_str().unwrap(), NodePath::user_root().buf().to_str().unwrap()), "Missing edge: root -> user_root");
        assert!(has_edge(edges_array, "user_root", "user_root/fileA.txt"), "Missing edge: user_root -> user_root/fileA.txt");
        assert!(has_edge(edges_array, "user_root", "user_root/dir1"), "Missing edge: user_root -> user_root/dir1");
    }

    #[tokio::test]
    async fn test_open_virtual_root_api() {
        let test_ctx = KartaServiceTestContext::new("api_open_virtual_root");
        // No specific FS setup needed beyond what KartaServiceTestContext does,
        // as NodePath::root() and NodePath::user_root() are archetypes.

        let app_state_service = test_ctx.service_arc.clone();
        let app_state = AppState {
            service: app_state_service,
            tx: tokio::sync::broadcast::channel(1).0,
        };

        let router = test_router_for_context_endpoints(app_state.clone());

        let response = router
            .oneshot(
                Request::builder()
                    .uri("/ctx/root") // Requesting the virtual root
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK, "API call to /ctx/root should be OK");

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body_json: Value = serde_json::from_slice(&body).expect("Failed to parse JSON response for /ctx/root");

        // Expected structure: [DataNode[], Edge[], Context]
        assert!(body_json.is_array() && body_json.as_array().unwrap().len() == 3,
            "Response for /ctx/root should be a 3-element array");

        let nodes_array = body_json.as_array().unwrap()[0].as_array().expect("Nodes element should be an array for /ctx/root");
        
        // For the virtual root context, we expect two nodes:
        // 1. The virtual root itself (NodePath::root(), serialized path usually "/")
        // 2. Its child, the user_root (NodePath::user_root(), serialized path usually "/user_root")
        assert_eq!(nodes_array.len(), 2, "Expected 2 nodes in virtual root context: root and user_root");

        let has_node_with_serialized_path = |nodes: &Vec<Value>, target_path: &str| -> Option<String> {
            nodes.iter().find_map(|n| {
                if n.get("path").and_then(|p| p.as_str()) == Some(target_path) {
                    n.get("uuid").and_then(|u| u.as_str()).map(String::from)
                } else {
                    None
                }
            })
        };
        
        // NodePath::root().alias() is "/"
        // NodePath::user_root().alias() is "/user_root"
        // However, the DataNode's "path" field in JSON is the direct NodePath string, not its alias()
        // NodePath::root() -> path_str: ""
        // NodePath::user_root() -> path_str: "user_root"
        // The KartaService::open_context_from_path returns DataNodes whose paths are NodePath instances.
        // When these NodePath instances are serialized as part of DataNode, they seem to use their internal `path_str`.
        // Let's re-check the previous test output for NodePath("user_root") serialization.
        // Previous output for user_root node: "path": String("user_root")
        // So, NodePath::root() should serialize to "path": String("") if it's consistent.

        let virtual_root_uuid = has_node_with_serialized_path(nodes_array, NodePath::root().buf().to_str().unwrap())
            .expect("Virtual root node (path: \"\") not found");
        let user_root_uuid = has_node_with_serialized_path(nodes_array, NodePath::user_root().buf().to_str().unwrap())
            .expect("User root node (path: \"user_root\") not found");

        let edges_array = body_json.as_array().unwrap()[1].as_array().expect("Edges element should be an array for /ctx/root");
        assert_eq!(edges_array.len(), 1, "Expected 1 edge in virtual root context (root -> user_root)");

        let has_edge = |edges: &Vec<Value>, src_path: &str, tgt_path: &str| -> bool {
            edges.iter().any(|e| {
                e.get("source").and_then(|s| s.as_str()) == Some(src_path) &&
                e.get("target").and_then(|t| t.as_str()) == Some(tgt_path)
            })
        };
        // Edge source/target in JSON are also direct NodePath string representations
        assert!(has_edge(edges_array, NodePath::root().buf().to_str().unwrap(), NodePath::user_root().buf().to_str().unwrap()),
            "Missing edge: root -> user_root");

        let context_json = &body_json.as_array().unwrap()[2];
        let actual_focal_uuid = context_json.get("focal").and_then(|f| f.as_str())
            .expect("Context JSON for /ctx/root missing 'focal' field or it's not a string");
        
        assert_eq!(actual_focal_uuid, virtual_root_uuid,
            "Context's focal UUID for /ctx/root should match the UUID of the virtual root node");
    }
}
