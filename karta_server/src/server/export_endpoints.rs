use crate::prelude::*;
use crate::elements::node_path::NodeHandle;
use axum::{
    extract::{Query, State},
    Json, http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

/// Request structure for getting bundle tree structure
#[derive(Debug, Deserialize)]
pub struct BundleTreeRequest {
    pub node_ids: Vec<String>,
}

/// Request structure for exporting a bundle
#[derive(Debug, Deserialize)]
pub struct ExportBundleRequest {
    pub node_ids: Vec<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub include_assets: bool,
}

/// Tree structure for bundle preview
#[derive(Debug, Serialize)]
pub struct BundleTreeNode {
    pub name: String,
    pub path: String,
    pub node_type: String,
    pub is_directory: bool,
    pub children: Option<Vec<BundleTreeNode>>,
    pub size: Option<u64>,
    pub file_count: Option<usize>,
}

/// Response structure for bundle tree
#[derive(Debug, Serialize)]
pub struct BundleTreeResponse {
    pub tree: BundleTreeNode,
    pub total_files: usize,
    pub total_size: u64,
    pub includes_assets: bool,
}

/// Response structure for export bundle
#[derive(Debug, Serialize)]
pub struct ExportBundleResponse {
    pub download_url: String,
    pub bundle_id: String,
    pub filename: String,
    pub size: u64,
    pub expires_at: String, // ISO timestamp
}

/// Get detailed tree structure for a bundle of selected nodes
/// This endpoint analyzes the selected nodes and returns a hierarchical structure
/// showing exactly what files and directories will be included in the export
pub async fn get_bundle_tree(
    State(state): State<AppState>,
    Query(params): Query<BundleTreeRequest>,
) -> Result<Json<BundleTreeResponse>, StatusCode> {
    let service = state.service.read().unwrap();
    
    // TODO: Implement detailed tree analysis
    // For now, return a simplified structure based on the selected nodes
    
    let mut total_files = 0;
    let mut total_size = 0;
    let mut children = Vec::new();
    
    for node_id in &params.node_ids {
        // Try to parse as UUID first, then as path
        let node_result = if let Ok(uuid) = Uuid::parse_str(node_id) {
            service.data().open_node(&NodeHandle::Uuid(uuid))
        } else {
            // Treat as path
            let node_path = NodePath::new(PathBuf::from(node_id.clone()));
            service.open_node(&NodeHandle::Path(node_path))
        };
        
        match node_result {
            Ok(node) => {
                let node_path = node.path().alias().to_string();
                let is_directory = node.is_dir();
                
                // For directories, we would recursively scan contents
                let (file_count, size) = if is_directory {
                    // TODO: Implement recursive directory scanning
                    // For now, return placeholder values
                    (1, 1024)
                } else {
                    // For files, get actual size if possible
                    let file_size = std::fs::metadata(&node_path)
                        .map(|m| m.len())
                        .unwrap_or(0);
                    (1, file_size)
                };
                
                total_files += file_count;
                total_size += size;
                
                let tree_node = BundleTreeNode {
                    name: node.name(),
                    path: node_path,
                    node_type: node.ntype().to_string(),
                    is_directory,
                    children: if is_directory { 
                        // TODO: Add actual children when implementing recursive scanning
                        Some(vec![])
                    } else { 
                        None 
                    },
                    size: if !is_directory { Some(size) } else { None },
                    file_count: if is_directory { Some(file_count) } else { None },
                };
                
                children.push(tree_node);
            }
            Err(_) => {
                // Error accessing node, skip
                continue;
            }
        }
    }
    
    let tree = BundleTreeNode {
        name: "Export Bundle".to_string(),
        path: "/".to_string(),
        node_type: "bundle".to_string(),
        is_directory: true,
        children: Some(children),
        size: None,
        file_count: Some(total_files),
    };
    
    let response = BundleTreeResponse {
        tree,
        total_files,
        total_size,
        includes_assets: true, // TODO: Determine based on actual content
    };
    
    Ok(Json(response))
}

/// Export selected nodes as a downloadable bundle
/// This endpoint creates a zip file containing the selected nodes and returns a download URL
pub async fn export_bundle(
    State(state): State<AppState>,
    Json(request): Json<ExportBundleRequest>,
) -> Result<Json<ExportBundleResponse>, StatusCode> {
    let _service = &state.service;
    
    // TODO: Implement actual bundle creation
    // This would involve:
    // 1. Creating a temporary directory
    // 2. Copying/exporting selected nodes to the directory
    // 3. Creating a zip file
    // 4. Moving zip to a downloads folder
    // 5. Returning download URL and cleanup info
    
    // For now, return a mock response
    let bundle_id = Uuid::new_v4().to_string();
    let filename = format!("{}.zip", 
        request.title.as_deref().unwrap_or("karta-export")
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
            .collect::<String>()
    );
    
    // Mock response - in real implementation, this would be actual file info
    let response = ExportBundleResponse {
        download_url: format!("/api/exports/download/{}", bundle_id),
        bundle_id,
        filename,
        size: 1024 * 1024, // 1MB placeholder
        expires_at: chrono::Utc::now()
            .checked_add_signed(chrono::Duration::hours(24))
            .unwrap()
            .to_rfc3339(),
    };
    
    Ok(Json(response))
}

/// Download a previously created export bundle
/// This endpoint serves the actual zip file for download
pub async fn download_bundle(
    State(_state): State<AppState>,
    axum::extract::Path(bundle_id): axum::extract::Path<String>,
) -> Result<axum::response::Response, StatusCode> {
    // TODO: Implement actual file serving
    // This would involve:
    // 1. Validating the bundle_id
    // 2. Checking if file exists and hasn't expired
    // 3. Serving the file with appropriate headers
    // 4. Optionally cleaning up expired files
    
    // For now, return not implemented
    Err(StatusCode::NOT_IMPLEMENTED)
}
