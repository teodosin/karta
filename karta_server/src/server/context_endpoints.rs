// Karta Server - Context API Endpoints

use axum::http::StatusCode;
use axum::{
    extract::{Path as AxumPath, State},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use dunce;
use std::error::Error as StdError;
use std::path::PathBuf;
use std::sync::Arc; // Keep Arc for KartaService within AppState // Added use for dunce crate

use crate::context::context::Context;
use crate::elements::edge::Edge;
use crate::elements::node::DataNode;
use crate::elements::node_path::NodePath;
use crate::server::karta_service::KartaService;
use crate::server::AppState; // Import AppState

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

    let vault_path = karta_service.vault_fs_path().clone();
    let processed_api_path_param = path_segments.trim_start_matches('/'); // Keep as &str for direct comparison

    let node_path_to_open: NodePath;
    let mut path_for_fs_check_str: String; // Will hold the string for FS path joining

    if processed_api_path_param == "root" {
        node_path_to_open = NodePath::root();
        // No direct FS path to check for the virtual root in this manner
        path_for_fs_check_str = "".to_string(); // Or handle differently, FS checks might be skipped
    } else {
        if processed_api_path_param == "." || processed_api_path_param.is_empty() {
            node_path_to_open = NodePath::vault();
            path_for_fs_check_str = "".to_string(); // Represents the vault root for FS checks
        } else {
            node_path_to_open = NodePath::from(processed_api_path_param.to_string());
            // If NodePath::from correctly makes it relative to vault (e.g. "foo" -> "vault/foo")
            // then path_for_fs_check_str should be the part after "vault/"
            // Or, if NodePath::from makes it "foo", then path_for_fs_check_str is "foo"
            // The current NodePath::from prepends "vault/" if not "vault" or empty.
            // So, if node_path_to_open is "vault/foo", we need "foo" for fs check.
            // If node_path_to_open is "vault", we need "" for fs check.
            match node_path_to_open.strip_vault_prefix() {
                Some(relative_to_vault) => path_for_fs_check_str = relative_to_vault,
                None => {
                    // This case should ideally not be hit if processed_api_path_param is not "root", ".", or empty,
                    // as NodePath::from should produce something under "vault".
                    // If it's an absolute path or something unexpected, error out.
                    return Err(box_error_to_response(
                        format!(
                            "Invalid path structure for FS checks: {}",
                            processed_api_path_param
                        )
                        .into(),
                    ));
                }
            }
        }

        // Security check for paths that relate to the filesystem vault
        let joined_path = vault_path.join(&path_for_fs_check_str); // path_for_fs_check_str is relative to vault root

        if processed_api_path_param.contains("..") {
            // Check original input for ".."
            return Err(box_error_to_response(
                "Path traversal attempt with '..' detected.".into(),
            ));
        }

        match dunce::canonicalize(&joined_path) {
            Ok(canonical_joined_path) => {
                if !canonical_joined_path.starts_with(
                    dunce::canonicalize(&vault_path).unwrap_or_else(|_| vault_path.clone()),
                ) {
                    return Err(box_error_to_response(
                        format!(
                            "Path '{}' resolves outside the vault.",
                            processed_api_path_param
                        )
                        .into(),
                    ));
                }
            }
            Err(_) => {
                // If canonicalization fails, it might be a path to a virtual node that doesn't exist on FS.
                // Check non-canonical path only if it's not the virtual root itself.
                // The virtual root won't exist on FS and its joined_path would be vault_path if path_for_fs_check_str is empty.
                if !node_path_to_open.is_root() && !joined_path.starts_with(&vault_path) {
                    return Err(box_error_to_response(
                        format!(
                            "Path '{}' appears to be outside the vault (non-canonical check).",
                            processed_api_path_param
                        )
                        .into(),
                    ));
                }
            }
        }
    }

    // Call the synchronous KartaService method directly.
    // Drop the read lock before calling a potentially blocking operation if KartaService methods were to become async
    // and required `&mut self`. For a read operation with `&self`, holding the read lock is fine.
    // If KartaService methods were long & synchronous, spawn_blocking would be better.
    // For now, direct call:
    match karta_service.open_context_from_path(node_path_to_open) {
        Ok(context_data) => {
            // println!("Context data: {:#?}", context_data);
            let cdata = Json(context_data);
            // println!("cdata: {:#?}", cdata);
            Ok(cdata)
        }
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
    use serde_json::Value;
    use std::sync::{Arc, RwLock};
    use tower::ServiceExt;

    // Helper to build a router for testing this specific endpoint
    fn test_router_for_context_endpoints(app_state: AppState) -> Router {
        Router::new()
            .route("/ctx/{*path_segments}", get(open_context_from_fs_path))
            .with_state(app_state)
    }

    #[tokio::test]
    async fn open_vault_context_from_api() {
        let test_ctx = KartaServiceTestContext::new("api_open_vault_ctx");
        test_ctx
            .create_file_in_vault("fileA.txt", b"content of file A")
            .unwrap();
        test_ctx.create_dir_in_vault("dir1").unwrap();
        test_ctx
            .create_file_in_vault("dir1/fileB.txt", b"content of file B")
            .unwrap();

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
                    .uri("/ctx/vault/")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_json: Value =
            serde_json::from_slice(&body).expect("Failed to parse JSON response");
        println!(
            "Test: open_vault_context_from_api - Response JSON: {:#?}",
            body_json
        );

        // Overall structure: Tuple of (Vec<DataNode>, Vec<Edge>, Context)
        assert!(
            body_json.is_array() && body_json.as_array().unwrap().len() == 3,
            "Response should be a 3-element array: [nodes, edges, context]"
        );

        // Nodes checks
        let nodes_array = body_json.as_array().unwrap()[0]
            .as_array()
            .expect("Nodes element should be an array");
        assert_eq!(
            nodes_array.len(),
            4,
            "Expected 4 nodes in vault context: root, vault, fileA.txt, dir1"
        );

        let has_node_with_path = |nodes: &Vec<Value>, target_path: &str| -> bool {
            nodes
                .iter()
                .any(|n| n.get("path").and_then(|p| p.as_str()) == Some(target_path))
        };

        assert!(
            has_node_with_path(nodes_array, NodePath::root().buf().to_str().unwrap()),
            "Node 'root' (path: \"\") not found"
        );
        assert!(
            has_node_with_path(nodes_array, "vault/fileA.txt"),
            "Node 'vault/fileA.txt' not found"
        );
        assert!(
            has_node_with_path(nodes_array, "vault/dir1"),
            "Node 'vault/dir1' not found"
        );
        assert!(
            has_node_with_path(nodes_array, NodePath::vault().buf().to_str().unwrap()),
            "Node 'vault' (focal) not found"
        );

        // Edge checks (optional, but good for completeness)
        let edges_array = body_json.as_array().unwrap()[1]
            .as_array()
            .expect("Edges element should be an array");
        assert_eq!(
            edges_array.len(),
            3,
            "Expected 3 edges: root->vault, vault->fileA.txt, vault->dir1"
        );

        let has_edge = |edges: &Vec<Value>, src: &str, tgt: &str| -> bool {
            edges.iter().any(|e| {
                e.get("source").and_then(|s| s.as_str()) == Some(src)
                    && e.get("target").and_then(|t| t.as_str()) == Some(tgt)
            })
        };
        assert!(
            has_edge(
                edges_array,
                NodePath::root().buf().to_str().unwrap(),
                NodePath::vault().buf().to_str().unwrap()
            ),
            "Missing edge: root -> vault"
        );
        assert!(
            has_edge(edges_array, "vault", "vault/fileA.txt"),
            "Missing edge: vault -> vault/fileA.txt"
        );
        assert!(
            has_edge(edges_array, "vault", "vault/dir1"),
            "Missing edge: vault -> vault/dir1"
        );
    }

    #[tokio::test]
    async fn test_open_virtual_root_api() {
        let test_ctx = KartaServiceTestContext::new("api_open_virtual_root");
        // No specific FS setup needed beyond what KartaServiceTestContext does,
        // as NodePath::root() and NodePath::vault() are archetypes.

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

        assert_eq!(
            response.status(),
            StatusCode::OK,
            "API call to /ctx/root should be OK"
        );

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_json: Value =
            serde_json::from_slice(&body).expect("Failed to parse JSON response for /ctx/root");
        println!(
            "Test: test_open_virtual_root_api - Response JSON: {:#?}",
            body_json
        );

        // Expected structure: [DataNode[], Edge[], Context]
        assert!(
            body_json.is_array() && body_json.as_array().unwrap().len() == 3,
            "Response for /ctx/root should be a 3-element array"
        );

        let nodes_array = body_json.as_array().unwrap()[0]
            .as_array()
            .expect("Nodes element should be an array for /ctx/root");

        // For the virtual root context, we expect two nodes:
        // 1. The virtual root itself (NodePath::root(), serialized path usually "/")
        // 2. Its child, the vault (NodePath::vault(), serialized path usually "/vault")
        assert_eq!(
            nodes_array.len(),
            2,
            "Expected 2 nodes in virtual root context: root and vault"
        );

        let has_node_with_serialized_path =
            |nodes: &Vec<Value>, target_path: &str| -> Option<String> {
                nodes.iter().find_map(|n| {
                    if n.get("path").and_then(|p| p.as_str()) == Some(target_path) {
                        n.get("uuid").and_then(|u| u.as_str()).map(String::from)
                    } else {
                        None
                    }
                })
            };

        // NodePath::root().alias() is "/"
        // NodePath::vault().alias() is "/vault"
        // However, the DataNode's "path" field in JSON is the direct NodePath string, not its alias()
        // NodePath::root() -> path_str: ""
        // NodePath::vault() -> path_str: "vault"
        // The KartaService::open_context_from_path returns DataNodes whose paths are NodePath instances.
        // When these NodePath instances are serialized as part of DataNode, they seem to use their internal `path_str`.
        // Let's re-check the previous test output for NodePath("vault") serialization.
        // Previous output for vault node: "path": String("vault")
        // So, NodePath::root() should serialize to "path": String("") if it's consistent.

        let virtual_root_uuid =
            has_node_with_serialized_path(nodes_array, NodePath::root().buf().to_str().unwrap())
                .expect("Virtual root node (path: \"\") not found");
        let vault_uuid =
            has_node_with_serialized_path(nodes_array, NodePath::vault().buf().to_str().unwrap())
                .expect("User root node (path: \"vault\") not found");

        let edges_array = body_json.as_array().unwrap()[1]
            .as_array()
            .expect("Edges element should be an array for /ctx/root");
        assert_eq!(
            edges_array.len(),
            1,
            "Expected 1 edge in virtual root context (root -> vault)"
        );

        let has_edge = |edges: &Vec<Value>, src_path: &str, tgt_path: &str| -> bool {
            edges.iter().any(|e| {
                e.get("source").and_then(|s| s.as_str()) == Some(src_path)
                    && e.get("target").and_then(|t| t.as_str()) == Some(tgt_path)
            })
        };
        // Edge source/target in JSON are also direct NodePath string representations
        assert!(
            has_edge(
                edges_array,
                NodePath::root().buf().to_str().unwrap(),
                NodePath::vault().buf().to_str().unwrap()
            ),
            "Missing edge: root -> vault"
        );

        let context_json = &body_json.as_array().unwrap()[2];
        let actual_focal_uuid = context_json
            .get("focal")
            .and_then(|f| f.as_str())
            .expect("Context JSON for /ctx/root missing 'focal' field or it's not a string");

        assert_eq!(
            actual_focal_uuid, virtual_root_uuid,
            "Context's focal UUID for /ctx/root should match the UUID of the virtual root node"
        );
    }

    #[tokio::test]
    async fn go_to_file_context() {
        let test_ctx = KartaServiceTestContext::new("api_open_file_ctx");
        test_ctx
            .create_file_in_vault("fileA.txt", b"content of file A")
            .unwrap();
        test_ctx.create_dir_in_vault("dir1").unwrap();
        test_ctx
            .create_file_in_vault("dir1/fileB.txt", b"content of file B")
            .unwrap();

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
                    .uri("/ctx/vault/dir1/fileB.txt") // Requesting a specific file context
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_json: Value =
            serde_json::from_slice(&body).expect("Failed to parse JSON response");
        println!(
            "Test: open_vault_context_from_api - Response JSON: {:#?}",
            body_json
        );

        // Overall structure: Tuple of (Vec<DataNode>, Vec<Edge>, Context)
        let nodes_array = body_json.as_array().unwrap()[0]
            .as_array()
            .expect("Nodes element should be an array");
        let edges_array = body_json.as_array().unwrap()[1]
            .as_array()
            .expect("Edges element should be an array");
        let context_json = &body_json.as_array().unwrap()[2];

        assert!(nodes_array.iter().any(|node| node.get("path").and_then(|v| v.as_str()) == Some("vault/dir1")), "Parent directory 'vault/dir1' not found");
        assert!(nodes_array.iter().any(|node| node.get("path").and_then(|v| v.as_str()) == Some("vault/dir1/fileB.txt")), "File 'vault/dir1/fileB.txt' not found");
        assert_eq!(
            nodes_array.len(),
            2,
            "Expected 2 nodes in virtual root context: dir1 and fileB.txt"
        );


        assert_eq!(
            edges_array.len(),
            1,
            "Expected 1 edge in virtual root context: dir1 -> fileB.txt"
        );
    }

        #[tokio::test]
    async fn going_to_vault_child_context__includes_vault() {
        let test_ctx = KartaServiceTestContext::new("api_open_file_ctx");
        test_ctx
            .create_file_in_vault("fileA.txt", b"content of file A")
            .unwrap();

        let app_state_service = test_ctx.service_arc.clone();

        let app_state = AppState {
            service: app_state_service,
            tx: tokio::sync::broadcast::channel(1).0,
        };

        let router = test_router_for_context_endpoints(app_state.clone());

        let response = router
            .oneshot(
                Request::builder()
                    .uri("/ctx/vault/fileA.txt") 
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_json: Value =
            serde_json::from_slice(&body).expect("Failed to parse JSON response");
        println!(
            "Test: open_vault_context_from_api - Response JSON: {:#?}",
            body_json
        );

        // Overall structure: Tuple of (Vec<DataNode>, Vec<Edge>, Context)
        let nodes_array = body_json.as_array().unwrap()[0]
            .as_array()
            .expect("Nodes element should be an array");
        let edges_array = body_json.as_array().unwrap()[1]
            .as_array()
            .expect("Edges element should be an array");
        let context_json = &body_json.as_array().unwrap()[2];

        println!("Nodes array: {:#?}", nodes_array);

        assert!(nodes_array.iter().any(|node| node.get("path").and_then(|v| v.as_str()) == Some("vault/fileA.txt")), "File 'vault/fileA.txt' not found");
        assert!(nodes_array.iter().any(|node| node.get("path").and_then(|v| v.as_str()) == Some("vault")), "Parent directory 'vault' not found");
        assert_eq!(
            nodes_array.len(),
            2,
            "Expected 2 nodes in virtual root context: vault and fileA.txt"
        );

        assert_eq!(
            edges_array.len(),
            1,
            "Expected 1 edge in virtual root context: vault -> fileA.txt"
        );
    }
}
