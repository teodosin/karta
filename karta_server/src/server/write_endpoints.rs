use axum::{
    extract::{Path as AxumPath, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use std::io::Write;
use uuid::Uuid;

use crate::{
    context::context::Context,
    elements::{
        attribute::{Attribute, AttrValue},
        node::DataNode,
        node_path::{NodeHandle, NodePath},
        nodetype::NodeTypeId,
    },
    graph_traits::graph_node::GraphNodes,
    server::AppState,
};

/// Generate a unique name by appending a counter if the original name already exists
fn generate_unique_name(service: &crate::server::karta_service::KartaService, parent_path: &NodePath, original_name: &str) -> String {
    let mut name = original_name.to_string();
    let mut final_path = parent_path.join(&name);
    let mut counter = 2;

    // Loop until we find a unique path
    while service.data().open_node(&NodeHandle::Path(final_path.clone())).is_ok() {
        // Handle file extensions properly
        if let Some(dot_pos) = original_name.rfind('.') {
            // Split name and extension
            let base_name = &original_name[..dot_pos];
            let extension = &original_name[dot_pos..];
            name = format!("{}_{}{}", base_name, counter, extension);
        } else {
            // No extension, just append counter
            name = format!("{}_{}", original_name, counter);
        }
        final_path = parent_path.join(&name);
        counter += 1;
    }

    name
}

/// Recursively collect all descendants of a given path
fn collect_all_descendants(service: &crate::server::karta_service::KartaService, path: &NodePath) -> Vec<DataNode> {
    let mut descendants = Vec::new();
    
    // Get all connections from this path
    let connections = service.data().open_node_connections(path);
    
    // Get the parent node to verify edge relationships
    if let Ok(parent_node) = service.data().open_node(&NodeHandle::Path(path.clone())) {
        println!("[collect_all_descendants] Collecting descendants for: '{}'", path.alias());
        
        for (connected_node, edge) in connections {
            // Check if this is a child (contains edge where we are the source)
            if edge.is_contains() && *edge.source() == parent_node.uuid() {
                println!("[collect_all_descendants] Found child: '{}'", connected_node.path().alias());
                descendants.push(connected_node.clone());
                
                // If this child is a directory, recursively collect its descendants
                if connected_node.is_dir() {
                    let mut child_descendants = collect_all_descendants(service, &connected_node.path());
                    descendants.append(&mut child_descendants);
                }
            }
        }
        
        println!("[collect_all_descendants] Total descendants found: {}", descendants.len());
    } else {
        println!("[collect_all_descendants] WARNING: Could not find parent node for path '{}'", path.alias());
    }
    
    descendants
}

#[derive(Deserialize, serde::Serialize)]
pub struct CreateNodePayload {
    name: String,
    ntype: NodeTypeId,
    parent_path: String,
    attributes: Vec<Attribute>,
}

#[derive(Deserialize, serde::Serialize)]
pub struct UpdateNodePayload {
    attributes: Vec<Attribute>,
}

#[derive(Deserialize, serde::Serialize)]
pub struct RenameNodeByPathPayload {
    path: String,
    new_name: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct RenameNodeResponse {
    renamed_nodes: Vec<MovedNodeInfo>,
}

#[derive(Deserialize, serde::Serialize)]
pub struct MoveNodesPayload {
    moves: Vec<MoveOperation>,
}

#[derive(Deserialize, serde::Serialize)]
pub struct MoveOperation {
    source_path: String,
    target_parent_path: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct MovedNodeInfo {
    uuid: Uuid,
    path: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct MoveNodesResponse {
    moved_nodes: Vec<MovedNodeInfo>,
    errors: Vec<MoveError>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct MoveError {
    source_path: String,
    error: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct UpdateNodeResponse {
    updated_node: DataNode,
    affected_nodes: Vec<DataNode>, // For rename operations that affect descendants
}

#[derive(Deserialize, serde::Serialize, Debug)]
pub struct DeleteNodesPayload {
    pub node_ids: Vec<String>, // UUIDs
    pub context_id: Option<String>, // Context where deletion is happening
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct DeleteNodesResponse {
    pub deleted_nodes: Vec<DeletedNodeInfo>,
    pub failed_deletions: Vec<FailedDeletion>,
    pub operation_id: String, // For future undo support
    pub warnings: Vec<String>, // e.g., "Deleting directory with X descendants"
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct DeletedNodeInfo {
    pub node_id: String,
    pub node_path: String,
    pub node_type: NodeTypeId,
    pub was_physical: bool,
    pub descendants_deleted: Vec<String>, // UUIDs of recursively deleted children
    // Full snapshots for undo (future)
    pub node_snapshot: DataNode,
    pub edge_snapshots: Vec<crate::elements::edge::Edge>,
    pub context_removals: Vec<String>, // Context IDs where node was removed
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct FailedDeletion {
    pub node_id: String,
    pub error: String,
}

// Trash metadata structure
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct TrashEntry {
    pub operation_id: String,
    pub timestamp: i64,
    pub deleted_nodes: Vec<DeletedNodeInfo>,
}

pub async fn create_node(
    State(app_state): State<AppState>,
    Json(payload): Json<CreateNodePayload>,
) -> Result<Json<DataNode>, StatusCode> {
    let mut service = app_state.service.write().unwrap();
    let parent_path = NodePath::from_alias(&payload.parent_path);

    if !parent_path.alias().starts_with("/vault") {
        return Err(StatusCode::BAD_REQUEST);
    }

    let mut name = payload.name.clone();
    let final_path = parent_path.join(&name);

    // Check if the path already exists and generate a unique name if needed
    if service.data().open_node(&NodeHandle::Path(final_path.clone())).is_ok() {
        name = generate_unique_name(&service, &parent_path, &payload.name);
    }

    let final_path = parent_path.join(&name);

    let mut new_node = DataNode::new(&final_path, payload.ntype);
    
    // Set attributes from payload
    new_node.set_attributes(payload.attributes);

    service.data_mut().insert_nodes(vec![new_node.clone()]);

    Ok(Json(new_node))
}

pub async fn update_node(
    State(app_state): State<AppState>,
    AxumPath(id): AxumPath<Uuid>,
    Json(payload): Json<UpdateNodePayload>,
) -> Result<Json<UpdateNodeResponse>, StatusCode> {
    println!("[update_node] Starting update for node ID: {}", id);
    println!("[update_node] Payload attributes: {:?}", payload.attributes);
    
    let mut service = app_state.service.write().unwrap();

    let mut node = match service.data().open_node(&NodeHandle::Uuid(id)) {
        Ok(node) => {
            println!("[update_node] Found node: '{}' at path '{}'", node.name(), node.path().alias());
            node
        },
        Err(e) => {
            println!("[update_node] Node not found by UUID: {}", e);
            
            // For unindexed physical files, we need to check if this is a rename operation
            // and handle it differently by using path-based lookup
            let is_rename_attempt = payload.attributes.iter()
                .any(|attr| attr.name == "name");
            
            if is_rename_attempt {
                println!("[update_node] This appears to be a rename operation for an unindexed node");
                println!("[update_node] Unindexed nodes should be renamed via direct filesystem operations");
                println!("[update_node] The frontend should handle this case differently");
            }
            
            return Err(StatusCode::NOT_FOUND);
        },
    };

    // Check if this is a rename operation (name attribute change)
    let is_rename = payload.attributes.iter()
        .any(|attr| attr.name == "name" && attr.value != AttrValue::String(node.name().to_string()));
    
    println!("[update_node] Is rename operation: {}", is_rename);

    if is_rename {
        // Extract the new name from attributes
        let new_name = payload.attributes.iter()
            .find(|attr| attr.name == "name")
            .and_then(|attr| match &attr.value {
                AttrValue::String(name) => Some(name.as_str()),
                _ => None,
            })
            .ok_or(StatusCode::BAD_REQUEST)?;

        println!("[update_node] New name for rename: '{}'", new_name);

        // Validate the new name
        if new_name.trim().is_empty() {
            println!("[update_node] ERROR: Empty name provided");
            return Err(StatusCode::BAD_REQUEST);
        }

        // Prevent renaming system nodes (root, vault)
        if node.path() == NodePath::root() {
            println!("[update_node] ERROR: Cannot rename root node: it is a system node");
            return Err(StatusCode::BAD_REQUEST);
        }
        
        if node.path() == NodePath::vault() {
            println!("[update_node] ERROR: Cannot rename vault node: it is a system node");
            return Err(StatusCode::BAD_REQUEST);
        }

        // COLLECT ALL DESCENDANTS BEFORE RENAME (if this is a directory)
        let affected_descendants: Vec<DataNode> = if node.is_dir() {
            println!("[update_node] Collecting descendants before rename (directory)");
            collect_all_descendants(&service, &node.path())
        } else {
            println!("[update_node] No descendants to collect (not a directory)");
            Vec::new()
        };

        // Perform the rename using move_node_with_rename
        println!("[update_node] Calling move_node_with_rename for in-place rename");
        let parent_path = node.path().parent().ok_or(StatusCode::BAD_REQUEST)?;
        let unique_name = generate_unique_name(&service, &parent_path, new_name.trim());
        match service.move_node_with_rename(&node.path(), &parent_path, Some(&unique_name)) {
            Ok(new_path) => {
                println!("[update_node] Rename successful, new path: '{}'", new_path.alias());
                // Reload the node from the new path to get the updated state
                match service.data().open_node(&NodeHandle::Path(new_path)) {
                    Ok(updated_node) => {
                        println!("[update_node] Reloaded node successfully");
                        // Update any other attributes besides name
                        let mut other_attributes: Vec<Attribute> = payload.attributes.into_iter()
                            .filter(|attr| attr.name != "name")
                            .collect();

                        let final_node = if !other_attributes.is_empty() {
                            println!("[update_node] Applying additional attributes");
                            let mut node_with_attrs = updated_node;
                            node_with_attrs.set_attributes(other_attributes);
                            node_with_attrs.update_modified_time();
                            service.data_mut().insert_nodes(vec![node_with_attrs.clone()]);
                            node_with_attrs
                        } else {
                            updated_node
                        };

                        // COLLECT ALL UPDATED DESCENDANTS AFTER RENAME
                        let updated_descendants: Vec<DataNode> = if final_node.is_dir() {
                            println!("[update_node] Collecting descendants after rename");
                            collect_all_descendants(&service, &final_node.path())
                        } else {
                            Vec::new()
                        };

                        println!("[update_node] SUCCESS: Returning response with {} affected nodes", updated_descendants.len());
                        return Ok(Json(UpdateNodeResponse {
                            updated_node: final_node,
                            affected_nodes: updated_descendants,
                        }));
                    }
                    Err(e) => {
                        println!("[update_node] ERROR: Failed to reload node after rename: {}", e);
                        return Err(StatusCode::INTERNAL_SERVER_ERROR);
                    }
                }
            }
            Err(e) => {
                println!("[update_node] ERROR: Failed to rename node: {}", e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    } else {
        // No rename, just update attributes normally
        println!("[update_node] Non-rename update, applying attributes");
        node.set_attributes(payload.attributes);
        node.update_modified_time();
        service.data_mut().insert_nodes(vec![node.clone()]);
        println!("[update_node] SUCCESS: Non-rename update completed");
        return Ok(Json(UpdateNodeResponse {
            updated_node: node,
            affected_nodes: Vec::new(), // No affected nodes for non-rename updates
        }));
    }
}

/// Recursively collect all nodes that will be affected by a move operation
/// This is called BEFORE the move to capture the full tree of descendants
fn collect_nodes_before_move(
    service: &crate::server::karta_service::KartaService,
    source_path: &NodePath,
    target_path: &NodePath,
) -> Vec<(uuid::Uuid, String)> {
    let mut affected_nodes = Vec::new();
    
    // First, try to get the node being moved
    if let Ok(source_node) = service.data().open_node(&NodeHandle::Path(source_path.clone())) {
        // Add the main node
        affected_nodes.push((source_node.uuid(), target_path.alias()));
        
        // If this is a directory, recursively collect all its children
        if source_node.is_dir() {
            // Use connections to find children
            let connections = service.data().open_node_connections(source_path);
            
            // Find children (connections whose paths start with our path + "/")
            let parent_prefix = format!("{}/", source_path.alias());
            let children: Vec<&DataNode> = connections.iter()
                .map(|(node, _edge)| node)
                .filter(|node| node.path().alias().starts_with(&parent_prefix))
                .collect();
            
            // Collect all descendants recursively
            let mut stack = children.into_iter().cloned().collect::<Vec<DataNode>>();
            let mut visited = std::collections::HashSet::new();
            
            while let Some(child) = stack.pop() {
                if visited.insert(child.uuid()) {
                    // Calculate the child's new path
                    let child_path_alias = child.path().alias();
                    let source_prefix = format!("{}/", source_path.alias());
                    let relative_path = child_path_alias.strip_prefix(&source_prefix)
                        .unwrap_or(&child_path_alias);
                    let child_target_path = format!("{}/{}", target_path.alias(), relative_path);
                    
                    affected_nodes.push((child.uuid(), child_target_path));
                    
                    // Add children of this child to stack if it's a directory
                    if child.is_dir() {
                        let child_connections = service.data().open_node_connections(&child.path());
                        let child_prefix = format!("{}/", child.path().alias());
                        for (grandchild, _) in child_connections {
                            if grandchild.path().alias().starts_with(&child_prefix) && !visited.contains(&grandchild.uuid()) {
                                stack.push(grandchild);
                            }
                        }
                    }
                }
            }
        }
    }
    
    affected_nodes
}

fn collect_nodes_after_move(
    service: &crate::server::karta_service::KartaService,
    original_source_path: &NodePath,
    final_path: &NodePath,
) -> Vec<MovedNodeInfo> {
    let mut affected_nodes = Vec::new();
    
    // Always include the main renamed node, regardless of whether it's indexed
    // Try to get it from database first, but create a synthetic response if not found
    if let Ok(moved_node) = service.data().open_node(&NodeHandle::Path(final_path.clone())) {
        // Node is indexed - add it normally
        affected_nodes.push(MovedNodeInfo {
            uuid: moved_node.uuid(),
            path: final_path.alias(),
        });
        
        // If this is a directory, collect all its children at their new locations
        if moved_node.is_dir() {
            let connections = service.data().open_node_connections(final_path);
            
            // Find children (connections whose paths start with our new path + "/")
            let parent_prefix = format!("{}/", final_path.alias());
            let children: Vec<&DataNode> = connections.iter()
                .map(|(node, _edge)| node)
                .filter(|node| node.path().alias().starts_with(&parent_prefix))
                .collect();
            
            // Collect all descendants recursively
            let mut stack = children.into_iter().cloned().collect::<Vec<DataNode>>();
            let mut visited = std::collections::HashSet::new();
            visited.insert(moved_node.uuid()); // Don't revisit the main node
            
            while let Some(child) = stack.pop() {
                if visited.insert(child.uuid()) {
                    affected_nodes.push(MovedNodeInfo {
                        uuid: child.uuid(),
                        path: child.path().alias(),
                    });
                    
                    // Add children of this child to stack if it's a directory
                    if child.is_dir() {
                        let child_connections = service.data().open_node_connections(&child.path());
                        let child_prefix = format!("{}/", child.path().alias());
                        for (grandchild, _) in child_connections {
                            if grandchild.path().alias().starts_with(&child_prefix) && !visited.contains(&grandchild.uuid()) {
                                stack.push(grandchild);
                            }
                        }
                    }
                }
            }
        }
    } else {
        // Node is not indexed - try to get it from the original source location
        if let Ok(original_node) = service.data().open_node(&NodeHandle::Path(original_source_path.clone())) {
            // Use the original node's UUID with the new path
            affected_nodes.push(MovedNodeInfo {
                uuid: original_node.uuid(),
                path: final_path.alias(),
            });
        } else {
            // Neither source nor target are indexed - create a deterministic UUID based on path
            // This handles physical-only files that were never indexed
            let path_hash = final_path.alias();
            let synthetic_uuid = Uuid::new_v5(&Uuid::NAMESPACE_URL, path_hash.as_bytes());
            affected_nodes.push(MovedNodeInfo {
                uuid: synthetic_uuid,
                path: final_path.alias(),
            });
        }
    }
    
    affected_nodes
}

pub async fn move_nodes(
    State(app_state): State<AppState>,
    Json(payload): Json<MoveNodesPayload>,
) -> Result<Json<MoveNodesResponse>, StatusCode> {
    let mut service = app_state.service.write().unwrap();
    let mut moved_nodes = Vec::new();
    let mut errors = Vec::new();

    println!("[MOVE_NODES] Processing {} move operations", payload.moves.len());

    for move_op in payload.moves {
        println!("[MOVE_NODES] Moving: {} -> {}", move_op.source_path, move_op.target_parent_path);
        
        let source_path = NodePath::from_alias(&move_op.source_path);
        let target_parent_path = NodePath::from_alias(&move_op.target_parent_path);

        // Validate both paths are within vault
        if !source_path.alias().starts_with("/vault") || !target_parent_path.alias().starts_with("/vault") {
            errors.push(MoveError {
                source_path: move_op.source_path.clone(),
                error: "Paths must be within vault".to_string(),
            });
            continue;
        }

        // Validate target parent exists (either in database or on filesystem)
        let target_parent_indexed = match service.data().open_node(&NodeHandle::Path(target_parent_path.clone())) {
            Ok(target_node) => {
                if !target_node.is_dir() {
                    errors.push(MoveError {
                        source_path: move_op.source_path.clone(),
                        error: "Target parent must be a directory".to_string(),
                    });
                    continue;
                }
                true // Target is indexed in database
            }
            Err(_) => {
                // Target not in database, check if it exists on filesystem
                let target_fs_path = target_parent_path.full(service.vault_fs_path());
                if !target_fs_path.exists() {
                    errors.push(MoveError {
                        source_path: move_op.source_path.clone(),
                        error: "Target parent path not found".to_string(),
                    });
                    continue;
                } else if !target_fs_path.is_dir() {
                    errors.push(MoveError {
                        source_path: move_op.source_path.clone(),
                        error: "Target parent must be a directory".to_string(),
                    });
                    continue;
                }
                false // Target exists on filesystem but not indexed
            }
        };

        // Prevent moving to self or child (circular move)
        if target_parent_path.alias().starts_with(&source_path.alias()) {
            errors.push(MoveError {
                source_path: move_op.source_path.clone(),
                error: "Cannot move node to itself or its child".to_string(),
            });
            continue;
        }

        // Check for name collision at target location and generate unique name if needed
        let original_name = source_path.name();
        let unique_name = generate_unique_name(&service, &target_parent_path, &original_name);
        let target_node_path = target_parent_path.join(&unique_name);

        // Perform the move operation with auto-renaming
        let final_name = if unique_name != original_name {
            Some(unique_name.as_str())
        } else {
            None
        };

        // Before the move, collect all nodes that will be affected
        let affected_before_move = collect_nodes_before_move(&service, &source_path, &target_node_path);
        
        match service.move_node_with_rename(&source_path, &target_parent_path, final_name) {
            Ok(final_path) => {
                println!("[MOVE_NODES] Move successful: {}", final_path.alias());
                
                // Convert to our response format, mapping the pre-calculated new paths
                let affected_nodes: Vec<MovedNodeInfo> = affected_before_move
                    .into_iter()
                    .map(|(uuid, path)| MovedNodeInfo { uuid, path })
                    .collect();
                
                println!("[MOVE_NODES] Total affected nodes: {}", affected_nodes.len());
                
                // Add all affected nodes to the response
                moved_nodes.extend(affected_nodes);
            }
            Err(e) => {
                println!("[MOVE_NODES] Move failed: {}", e);
                errors.push(MoveError {
                    source_path: move_op.source_path.clone(),
                    error: format!("Move operation failed: {}", e),
                });
            }
        }
    }

    println!("[MOVE_NODES] Completed: {} moved, {} errors", moved_nodes.len(), errors.len());
    Ok(Json(MoveNodesResponse {
        moved_nodes,
        errors,
    }))
}

pub async fn rename_node(
    State(app_state): State<AppState>,
    Json(payload): Json<RenameNodeByPathPayload>,
) -> Result<Json<RenameNodeResponse>, StatusCode> {
    let mut service = app_state.service.write().unwrap();
    
    let source_path = NodePath::from_alias(&payload.path);
    
    // Validate path is within vault
    if !source_path.alias().starts_with("/vault") {
        eprintln!("[RENAME_NODE] Path must be within vault: {}", payload.path);
        return Err(StatusCode::BAD_REQUEST);
    }
    
    // Validate new name is not empty
    if payload.new_name.trim().is_empty() {
        eprintln!("[RENAME_NODE] New name cannot be empty");
        return Err(StatusCode::BAD_REQUEST);
    }
    
    // Get the parent path for the rename operation
    let parent_path = match source_path.parent() {
        Some(parent) => parent,
        None => {
            eprintln!("[RENAME_NODE] Cannot rename root-level nodes");
            return Err(StatusCode::BAD_REQUEST);
        }
    };
    
    // Prevent renaming system nodes (root, vault)
    if source_path.alias() == "/vault" || source_path.alias() == "/" {
        eprintln!("[RENAME_NODE] Cannot rename system nodes");
        return Err(StatusCode::FORBIDDEN);
    }
    
    println!("[RENAME_NODE] Renaming '{}' to '{}'", source_path.alias(), payload.new_name);

    // Perform the rename using move_node_with_rename
    match service.move_node_with_rename(&source_path, &parent_path, Some(&payload.new_name)) {
        Ok(final_path) => {
            println!("[RENAME_NODE] Rename successful: {}", final_path.alias());
            
            // Collect affected nodes AFTER the operation to get actual final paths
            let affected_after_move = collect_nodes_after_move(&service, &source_path, &final_path);
            
            println!("[RENAME_NODE] Total affected nodes: {}", affected_after_move.len());
            
            Ok(Json(RenameNodeResponse {
                renamed_nodes: affected_after_move,
            }))
        }
        Err(e) => {
            eprintln!("[RENAME_NODE] Rename failed: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn save_context(
    State(app_state): State<AppState>,
    AxumPath(id): AxumPath<Uuid>,
    Json(payload): Json<Context>,
) -> StatusCode {
    // The `id` from the path should match the `focal` id in the payload.
    if id != payload.focal() {
        eprintln!("[SAVE_CONTEXT] Mismatch between path ID ({}) and payload focal ID ({}).", id, payload.focal());
        return StatusCode::BAD_REQUEST;
    }

    let service = app_state.service.read().unwrap();
    match service.view().save_context(&payload) {
        Ok(_) => StatusCode::OK,
        Err(e) => {
            eprintln!("[SAVE_CONTEXT] Error saving context {}: {:?}", id, e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

pub async fn rename_node_by_path(
    State(app_state): State<AppState>,
    Json(payload): Json<RenameNodeByPathPayload>,
) -> Result<Json<UpdateNodeResponse>, StatusCode> {
    println!("[rename_node_by_path] Starting rename for path: '{}' to name: '{}'", payload.path, payload.new_name);
    
    let mut service = app_state.service.write().unwrap();
    let node_path = NodePath::from_alias(&payload.path);
    
    // Validate the new name
    if payload.new_name.trim().is_empty() {
        println!("[rename_node_by_path] ERROR: Empty name provided");
        return Err(StatusCode::BAD_REQUEST);
    }

    // Prevent renaming system nodes (root, vault)
    if node_path == NodePath::root() {
        println!("[rename_node_by_path] ERROR: Cannot rename root node: it is a system node");
        return Err(StatusCode::BAD_REQUEST);
    }
    
    if node_path == NodePath::vault() {
        println!("[rename_node_by_path] ERROR: Cannot rename vault node: it is a system node");
        return Err(StatusCode::BAD_REQUEST);
    }

    // COLLECT ALL DESCENDANTS BEFORE RENAME (if this is a directory)
    let affected_descendants: Vec<DataNode> = if node_path.full(service.vault_fs_path()).is_dir() {
        println!("[rename_node_by_path] Collecting descendants before rename (directory)");
        collect_all_descendants(&service, &node_path)
    } else {
        println!("[rename_node_by_path] No descendants to collect (not a directory)");
        Vec::new()
    };

    // Perform the rename using move_node_with_rename
    println!("[rename_node_by_path] Calling move_node_with_rename for in-place rename");
    let parent_path = node_path.parent().ok_or(StatusCode::BAD_REQUEST)?;
    let unique_name = generate_unique_name(&service, &parent_path, payload.new_name.trim());
    match service.move_node_with_rename(&node_path, &parent_path, Some(&unique_name)) {
        Ok(new_path) => {
            println!("[rename_node_by_path] Rename successful, new path: '{}'", new_path.alias());
            
            // Get the renamed node (either from DB if indexed, or create from filesystem)
            let updated_node = match service.open_node(&NodeHandle::Path(new_path.clone())) {
                Ok(node) => {
                    println!("[rename_node_by_path] Retrieved updated node from database/filesystem");
                    node
                }
                Err(e) => {
                    println!("[rename_node_by_path] ERROR: Failed to retrieve renamed node: {}", e);
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
            };

            // COLLECT ALL UPDATED DESCENDANTS AFTER RENAME
            let updated_descendants: Vec<DataNode> = if updated_node.is_dir() {
                println!("[rename_node_by_path] Collecting descendants after rename");
                collect_all_descendants(&service, &updated_node.path())
            } else {
                Vec::new()
            };

            println!("[rename_node_by_path] SUCCESS: Returning response with {} affected nodes", updated_descendants.len());
            Ok(Json(UpdateNodeResponse {
                updated_node,
                affected_nodes: updated_descendants,
            }))
        }
        Err(e) => {
            println!("[rename_node_by_path] ERROR: Failed to rename node: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn delete_nodes(
    State(app_state): State<AppState>,
    Json(payload): Json<DeleteNodesPayload>,
) -> Result<Json<DeleteNodesResponse>, (StatusCode, String)> {
    let mut service = app_state.service.write().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to acquire service lock: {}", e))
    })?;

    match service.delete_nodes(payload) {
        Ok(response) => Ok(Json(response)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to delete nodes: {}", e))),
    }
}

#[cfg(test)]
#[path = "write_endpoints_tests/mod.rs"]
mod tests;


