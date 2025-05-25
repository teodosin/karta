// Karta Server - Context API Endpoints

use axum::{
    extract::{Path as AxumPath, State}, // Added State
    response::{IntoResponse, Response},
    Json, Router, routing::get, // Removed Extension
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
    let relative_path_str = path_segments.trim_start_matches('/');
    
    let joined_path = vault_path.join(relative_path_str);

    // Simplified security check:
    if !joined_path.starts_with(&vault_path) {
        if relative_path_str.contains("..") {
             return Err(box_error_to_response("Path traversal attempt with '..' detected.".into()));
        }
        return Err(box_error_to_response("Path appears to be outside the vault.".into()));
    }
    // TODO: Implement robust path canonicalization and security checks.

    let node_path_to_open = NodePath::from(relative_path_str);

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
