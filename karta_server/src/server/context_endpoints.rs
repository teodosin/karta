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
    println!("[API DEBUG] Received path_segments: {:?}", path_segments);
    
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
    let mut relative_path_str_mut = path_segments.trim_start_matches('/');
    if relative_path_str_mut == "." {
        relative_path_str_mut = "";
    }
    let relative_path_str = relative_path_str_mut; // Re-assign to non-mut or use directly
    println!("[API DEBUG] Processed relative_path_str: {:?}", relative_path_str);
    
    let joined_path = vault_path.join(relative_path_str);

    // Simplified security check:
    if !joined_path.starts_with(&vault_path) {
        if relative_path_str.contains("..") {
             return Err(box_error_to_response("Path traversal attempt with '..' detected.".into()));
        }
        return Err(box_error_to_response("Path appears to be outside the vault.".into()));
    }
    // TODO: Implement robust path canonicalization and security checks.

    let node_path_to_open = if relative_path_str.is_empty() {
        NodePath::user_root()
    } else {
        NodePath::from(relative_path_str.to_string())
    };
    println!("[API DEBUG] Constructed node_path_to_open: {:?}", node_path_to_open);

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
        Router, // Added Router here
    };
    use tower::ServiceExt; // For `oneshot`
    use serde_json::Value;
    use std::sync::{Arc, RwLock}; // Added Arc here

    // Helper to build a router for testing this specific endpoint
    fn test_router_for_context_endpoints(app_state: AppState) -> Router { // Renamed to avoid potential conflict if super has test_router
        Router::new()
            .route("/ctx/{*path_segments}", get(open_context_from_fs_path)) // Corrected wildcard syntax
            .with_state(app_state)
    }

    #[tokio::test]
    async fn test_open_root_context_api() {
        let test_ctx = KartaServiceTestContext::new("api_open_root_ctx"); // Shortened name slightly
        test_ctx.create_file_in_vault("fileA.txt", b"content of file A").unwrap();
        test_ctx.create_dir_in_vault("dir1").unwrap();
        test_ctx.create_file_in_vault("dir1/fileB.txt", b"content of file B").unwrap();

        // KartaService from KartaServiceTestContext is not Arc<RwLock<KartaService>>
        // We need to wrap it.
        let app_state_service = test_ctx.service_arc.clone(); // Use the Arc from KartaServiceTestContext and clone it

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
        println!("[TEST DEBUG] Full response body_json: {:#?}", body_json);
        
        assert!(body_json.is_array() && body_json.as_array().unwrap().len() == 3, "Response should be a 3-element array");

        let nodes_json = &body_json.as_array().unwrap()[0];
        assert!(nodes_json.is_array(), "First element (nodes) should be an array");
        
        let nodes_array = nodes_json.as_array().unwrap();
        
        let mut found_file_a = false;
        let mut found_dir_1 = false;

        println!("[TEST DEBUG] Iterating nodes_array (count: {}):", nodes_array.len());
        for (index, node_val) in nodes_array.iter().enumerate() {
            println!("[TEST DEBUG] Node [{}]: {:#?}", index, node_val);
            let path_str = node_val.get("path").and_then(|p| p.as_str()); // Corrected: path is a direct string
            println!("[TEST DEBUG]   Extracted path_str: {:?}", path_str);
            match path_str {
                // Paths are now fully qualified from the service, e.g., "user_root/fileA.txt"
                Some("user_root/fileA.txt") => found_file_a = true,
                Some("user_root/dir1") => found_dir_1 = true,
                _ => {}
            }
        }
        assert!(found_file_a, "Node for 'fileA.txt' not found in root context response");
        assert!(found_dir_1, "Node for 'dir1' not found in root context response");

        // Find the user_root node in the nodes_array to get its UUID
        let user_root_node_json = nodes_array.iter().find(|&n| n.get("path").and_then(|p| p.as_str()) == Some("user_root"));
        assert!(user_root_node_json.is_some(), "user_root node not found in the returned nodes_array");
        let expected_focal_uuid = user_root_node_json.unwrap().get("uuid").and_then(|u| u.as_str());
        assert!(expected_focal_uuid.is_some(), "Could not extract UUID from user_root node JSON");
        
        let context_json = &body_json.as_array().unwrap()[2];
        let actual_focal_uuid = context_json.get("focal").and_then(|f| f.as_str());

        assert_eq!(
            actual_focal_uuid,
            expected_focal_uuid,
            "Context's focal UUID should match the UUID of the 'user_root' DataNode"
        );
    }
}
