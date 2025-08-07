use crate::prelude::*;
use crate::elements::node_path::NodeHandle;
use axum::{
    extract::{Query, State},
    Json, http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
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
    pub total_directories: usize,
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
    
    println!("Getting bundle tree for node IDs: {:?}", params.node_ids);
    
    let mut total_files = 0;
    let mut total_directories = 0;
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
                let node_path_str = node.path().alias().to_string();
                let node_path = Path::new(&node_path_str);
                
                println!("Processing node: {} -> path: {}", node_id, node_path_str);
                
                // Check if the path actually exists on the filesystem
                if !node_path.exists() {
                    println!("Path does not exist: {}", node_path_str);
                    // If path doesn't exist, create a simple node entry
                    let tree_node = BundleTreeNode {
                        name: node.name(),
                        path: node_path_str,
                        node_type: node.ntype().to_string(),
                        is_directory: false,
                        children: None,
                        size: Some(0),
                        file_count: None,
                    };
                    children.push(tree_node);
                    total_files += 1; // Count non-existent paths as files
                    continue;
                }
                
                // Use filesystem metadata to determine if it's a directory
                let metadata = match fs::metadata(node_path) {
                    Ok(meta) => meta,
                    Err(_) => {
                        // If we can't read metadata, treat as a regular file with 0 size
                        let tree_node = BundleTreeNode {
                            name: node.name(),
                            path: node_path_str,
                            node_type: node.ntype().to_string(),
                            is_directory: false,
                            children: None,
                            size: Some(0),
                            file_count: None,
                        };
                        children.push(tree_node);
                        total_files += 1;
                        continue;
                    }
                };
                
                let is_directory = metadata.is_dir();
                
                if is_directory {
                    // Recursively scan directory contents
                    let (dir_children, dir_file_count, dir_dir_count, dir_size) = scan_directory_tree(node_path)?;
                    
                    total_files += dir_file_count;
                    total_directories += dir_dir_count + 1; // +1 for the directory itself
                    total_size += dir_size;
                    
                    let tree_node = BundleTreeNode {
                        name: node.name(),
                        path: node_path_str,
                        node_type: node.ntype().to_string(),
                        is_directory: true,
                        children: Some(dir_children),
                        size: None,
                        file_count: Some(dir_file_count),
                    };
                    
                    children.push(tree_node);
                } else {
                    // Handle individual files
                    let file_size = metadata.len();
                    total_files += 1;
                    total_size += file_size;
                    
                    let tree_node = BundleTreeNode {
                        name: node.name(),
                        path: node_path_str,
                        node_type: node.ntype().to_string(),
                        is_directory: false,
                        children: None,
                        size: Some(file_size),
                        file_count: None,
                    };
                    
                    children.push(tree_node);
                }
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
        total_directories,
        total_size,
        includes_assets: true, // TODO: Determine based on actual content
    };
    
    println!("Returning bundle tree response: {} files, {} directories, {} bytes", 
             total_files, total_directories, total_size);
    
    Ok(Json(response))
}

/// Recursively scan a directory and build tree structure
/// Returns (children, total_file_count, total_directory_count, total_size)
fn scan_directory_tree(dir_path: &Path) -> Result<(Vec<BundleTreeNode>, usize, usize, u64), StatusCode> {
    let mut children = Vec::new();
    let mut total_files = 0;
    let mut total_directories = 0;
    let mut total_size = 0;
    
    let read_dir = fs::read_dir(dir_path).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    for entry in read_dir {
        let entry = entry.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let entry_path = entry.path();
        let metadata = entry.metadata().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        
        let name = entry_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown")
            .to_string();
        
        let path = entry_path.to_string_lossy().to_string();
        
        if metadata.is_dir() {
            // Recursively scan subdirectory
            let (subdir_children, subdir_file_count, subdir_dir_count, subdir_size) = scan_directory_tree(&entry_path)?;
            
            total_files += subdir_file_count;
            total_directories += subdir_dir_count + 1; // +1 for the subdirectory itself
            total_size += subdir_size;
            
            let tree_node = BundleTreeNode {
                name,
                path,
                node_type: "core/fs/dir".to_string(),
                is_directory: true,
                children: Some(subdir_children),
                size: None,
                file_count: Some(subdir_file_count),
            };
            
            children.push(tree_node);
        } else {
            // Handle file
            let file_size = metadata.len();
            total_files += 1;
            total_size += file_size;
            
            // Determine file type based on extension
            let file_type = entry_path
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| format!("core/fs/file/{}", ext))
                .unwrap_or_else(|| "core/fs/file".to_string());
            
            let tree_node = BundleTreeNode {
                name,
                path,
                node_type: file_type,
                is_directory: false,
                children: None,
                size: Some(file_size),
                file_count: None,
            };
            
            children.push(tree_node);
        }
    }
    
    Ok((children, total_files, total_directories, total_size))
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
