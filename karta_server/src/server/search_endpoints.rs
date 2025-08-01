use axum::{extract::{State, Query}, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::AppState;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SearchResult {
    pub id: Option<Uuid>,        // UUID if indexed in database
    pub path: String,            // Full path from vault root
    pub ntype: String,          // Node type (File, Directory, etc.)
    pub is_indexed: bool,       // Whether it exists in database
    pub score: f64,             // Fuzzy match score (0.0-1.0)
}

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: String,              // Search query
    #[serde(default = "default_limit")]
    pub limit: usize,           // Max results (default 100 for infinite scroll)
    #[serde(default = "default_min_score")]
    pub min_score: f64,         // Filter low-quality matches (default 0.1)
}

#[derive(Serialize, Deserialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
    pub total_found: usize,     // How many matched (before limit)
    pub truncated: bool,        // Whether results were limited
    pub query: String,
    pub took_ms: u64,
}

fn default_limit() -> usize {
    100
}

fn default_min_score() -> f64 {
    0.1
}

pub async fn search_nodes(
    State(app_state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<SearchResponse>, StatusCode> {
    // Validate input
    if query.q.trim().is_empty() {
        return Ok(Json(SearchResponse {
            results: vec![],
            total_found: 0,
            truncated: false,
            query: query.q,
            took_ms: 0,
        }));
    }
    
    let service = app_state.service.read().unwrap();
    match service.search_nodes(&query.q, query.limit, query.min_score) {
        Ok(response) => Ok(Json(response)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
