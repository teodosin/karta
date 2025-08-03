use axum::{extract::State, http::StatusCode, response::Json};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, str::FromStr};
use uuid::Uuid;

use crate::{prelude::Edge, server::{karta_service::KartaService, AppState}};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateEdgePayload {
    pub id: String,
    pub source: String,
    pub target: String,
    pub attributes: HashMap<String, serde_json::Value>,
    pub source_path: String,
    pub target_path: String,
}

#[axum::debug_handler]
pub async fn create_edges(
    State(state): State<AppState>,
    Json(payload): Json<Vec<CreateEdgePayload>>,
) -> Result<Json<Vec<CreateEdgePayload>>, StatusCode> {

    dbg!(&payload);
    let mut service = state.service.write().unwrap();
    
    match service.create_edges(payload) {
        Ok(created_edges) => Ok(Json(created_edges)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ReconnectEdgePayload {
    pub old_from: Uuid,
    pub old_to: Uuid,
    pub new_from: Uuid,
    pub new_to: Uuid,
    pub new_from_path: String,
    pub new_to_path: String,
}

#[axum::debug_handler]
pub async fn reconnect_edge(
    State(state): State<AppState>,
    Json(payload): Json<ReconnectEdgePayload>,
) -> Result<Json<Edge>, StatusCode> {
    let mut service = state.service.write().unwrap();

    println!("[RECONNECT_EDGE] Attempting edge reconnection:");
    println!("[RECONNECT_EDGE]   Old: {} -> {}", payload.old_from, payload.old_to);
    println!("[RECONNECT_EDGE]   New: {} -> {}", payload.new_from, payload.new_to);
    println!("[RECONNECT_EDGE]   Paths: {} -> {}", payload.new_from_path, payload.new_to_path);

    match service.reconnect_edge(&payload.old_from, &payload.old_to, &payload.new_from, &payload.new_to, &payload.new_from_path, &payload.new_to_path) {
        Ok(edge) => {
            println!("[RECONNECT_EDGE] Success: {:?}", edge);
            Ok(Json(edge))
        },
        Err(e) => {
            println!("[RECONNECT_EDGE] Error: {}", e);
            if e.to_string()
                .contains("Reconnection of 'contains' edges is not allowed")
            {
                println!("[RECONNECT_EDGE] Returning 400 - Contains edge reconnection not allowed");
                Err(StatusCode::BAD_REQUEST)
            } else {
                println!("[RECONNECT_EDGE] Returning 500 - Internal server error");
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DeleteEdgePayload {
    pub source: Uuid,
    pub target: Uuid,
}

#[axum::debug_handler]
pub async fn delete_edges(
    State(state): State<AppState>,
    Json(payload): Json<Vec<DeleteEdgePayload>>,
) -> Result<StatusCode, StatusCode> {
    let mut service = state.service.write().unwrap();

    match service.delete_edges(payload) {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            if e.to_string().contains("Deletion of 'contains' edges is not allowed.") {
                Err(StatusCode::BAD_REQUEST)
            } else {
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}