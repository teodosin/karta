use axum::{
    body::Body,
    extract::{Path, State},
    response::{IntoResponse, Response},
    http::{header, StatusCode},
};
use uuid::Uuid;
use crate::{prelude::GraphNodes, server::AppState};
use crate::elements::node_path::NodeHandle;
use tokio::fs;

#[axum::debug_handler]
pub async fn get_asset(
    State(state): State<AppState>,
    Path(path): Path<String>,
) -> impl IntoResponse {
    // With lazy indexing, we don't query the database. We construct the path directly.
    let absolute_path = {
        let service = state.service.read().unwrap();
        service.vault_fs_path().join(&path)
    };

    println!("[Asset Endpoint] Attempting to read asset from: {:?}", absolute_path);

    // Read the file from the filesystem
    let file_contents = match fs::read(&absolute_path).await {
        Ok(contents) => contents,
        Err(e) => {
            println!("[Asset Endpoint] Error reading file for path \"{}\": {:?}", path, e);
            let error_message = format!("Asset file not found on disk for path: {}", path);
            return (StatusCode::NOT_FOUND, error_message).into_response();
        }
    };

    // 5. Determine the MIME type from the file extension
    let mime_type = mime_guess::from_path(&absolute_path)
        .first_or_octet_stream()
        .to_string();

    // 6. Build and return the response
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, mime_type)
        .body(Body::from(file_contents))
        .unwrap()
}