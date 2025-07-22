use axum::{
    extract::{Path as AxumPath, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        context::context::Context,
        elements::{node::DataNode, view_node::ViewNode},
        graph_traits::graph_node::GraphNodes,
        server::{karta_service::KartaService, AppState},
        utils::utils::KartaServiceTestContext,
    };
    use axum::{
        body::Body,
        http::{self, Request, StatusCode},
        Router,
    };
    use tower::ServiceExt;

    // This setup is proven to work from context_endpoints.rs
    fn setup_test_environment(test_name: &str) -> (Router, KartaServiceTestContext) {
        let test_ctx = KartaServiceTestContext::new(test_name);
        let app_state = AppState {
            service: test_ctx.service_arc.clone(),
            tx: tokio::sync::broadcast::channel(1).0,
        };
        let router = Router::new()
            .route(
                "/ctx/{*id}",
                axum::routing::get(crate::server::context_endpoints::open_context_from_fs_path),
            )
            .route("/api/ctx/{id}", axum::routing::put(save_context))
            .route("/api/nodes", axum::routing::post(create_node))
            .route("/api/nodes/{id}", axum::routing::put(update_node))
            .route("/api/nodes/rename", axum::routing::post(rename_node))
            .route("/api/nodes/move", axum::routing::post(move_nodes))
            .with_state(app_state);
        (router, test_ctx)
    }

    // Helper for POST requests
    async fn execute_post_request(router: Router, uri: &str, body: String) -> http::Response<Body> {
        router
            .oneshot(
                Request::builder()
                    .method(http::Method::POST)
                    .uri(uri)
                    .header(http::header::CONTENT_TYPE, "application/json")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap()
    }

    // Helper for PUT requests
    async fn execute_put_request(router: Router, uri: &str, body: String) -> http::Response<Body> {
        router
            .oneshot(
                Request::builder()
                    .method(http::Method::PUT)
                    .uri(uri)
                    .header(http::header::CONTENT_TYPE, "application/json")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn test_save_context_creates_file() {
        let (router, test_ctx) = setup_test_environment("save_creates_file");

        // Arrange
        let focal_uuid = test_ctx.with_service_mut(|s| {
            s.data_mut().insert_nodes(vec![DataNode::new(&"vault/test_dir".into(), NodeTypeId::dir_type())]);
            s.open_context_from_path("vault/test_dir".into()).unwrap().2.focal()
        });
        
        let initial_context = test_ctx.with_service(|s| s.open_context_from_path("vault/test_dir".into()).unwrap().2);
        let view_node_to_modify = initial_context.viewnodes().get(0).unwrap().clone();
        let modified_view_node = view_node_to_modify.positioned(123.0, 456.0);
        let context_payload = Context::with_viewnodes(focal_uuid, vec![modified_view_node.clone()]);
        let payload_json = serde_json::to_string(&context_payload).unwrap();

        // Act
        let response =
            execute_put_request(router, &format!("/api/ctx/{}", focal_uuid), payload_json).await;

        // Assert
        assert_eq!(response.status(), StatusCode::OK);
        let saved_context = test_ctx
            .with_service(|s| s.view().get_context_file(focal_uuid))
            .unwrap();
        assert_eq!(saved_context.viewnodes().len(), 1);
        assert_eq!(saved_context.viewnodes()[0].relX, 123.0);
    }

    #[tokio::test]
    async fn test_save_empty_context_deletes_file() {
        let (router, test_ctx) = setup_test_environment("save_empty_deletes");

        // Arrange: Create a directory and save a context for it first.
        test_ctx.create_dir_in_vault("dir_to_delete").unwrap();
        // Manually insert the node to ensure it's indexed before we try to save its context by UUID.
        let focal_uuid = test_ctx.with_service_mut(|s| {
            let node_to_insert = DataNode::new(&"vault/dir_to_delete".into(), NodeTypeId::dir_type());
            s.data_mut().insert_nodes(vec![node_to_insert]);
            s.open_context_from_path("vault/dir_to_delete".into()).unwrap().2.focal()
        });
        let initial_context = test_ctx.with_service(|s| s.open_context_from_path("vault/dir_to_delete".into()).unwrap().2);
        let view_node = initial_context.viewnodes().get(0).unwrap().clone();
        let initial_payload = Context::with_viewnodes(focal_uuid, vec![view_node]);
        let initial_payload_json = serde_json::to_string(&initial_payload).unwrap();
        execute_put_request(
            router.clone(),
            &format!("/api/ctx/{}", focal_uuid),
            initial_payload_json,
        )
        .await;
        assert!(test_ctx
            .with_service(|s| s.view().get_context_file(focal_uuid))
            .is_ok());

        // Arrange: Create an empty payload.
        let empty_payload = Context::with_viewnodes(focal_uuid, vec![]);
        let empty_payload_json = serde_json::to_string(&empty_payload).unwrap();

        // Act
        let response = execute_put_request(
            router,
            &format!("/api/ctx/{}", focal_uuid),
            empty_payload_json,
        )
        .await;

        // Assert
        assert_eq!(response.status(), StatusCode::OK);
        assert!(test_ctx
            .with_service(|s| s.view().get_context_file(focal_uuid))
            .is_err());
    }

    #[tokio::test]
    async fn test_reload_context_merges_saved_and_default_nodes() {
        let (router, test_ctx) = setup_test_environment("reload_merges");

        // Arrange: FS setup and index the nodes to simulate modification before saving.
        let (initial_nodes, _, initial_context) = test_ctx.with_service_mut(|s| {
            s.data_mut().insert_nodes(vec![
                DataNode::new(&"vault/test_dir".into(), NodeTypeId::dir_type()),
                DataNode::new(&"vault/test_dir/A.txt".into(), NodeTypeId::file_type()),
                DataNode::new(&"vault/test_dir/B.txt".into(), NodeTypeId::file_type()),
            ]);
            s.open_context_from_path("vault/test_dir".into()).unwrap()
        });
        let focal_uuid = initial_context.focal();
        let node_b_data = initial_nodes.iter().find(|n| n.path().name() == "B.txt").expect("Node B data not found");
        let node_b_view = initial_context.viewnodes().iter().find(|vn| vn.uuid == node_b_data.uuid()).expect("Node B view not found");
        
        // Arrange: Save a modified position for node B.
        let modified_node_b = node_b_view.clone().positioned(500.0, 500.0);
        let save_payload = Context::with_viewnodes(focal_uuid, vec![modified_node_b]);
        let save_payload_json = serde_json::to_string(&save_payload).unwrap();
        execute_put_request(
            router.clone(),
            &format!("/api/ctx/{}", focal_uuid),
            save_payload_json,
        )
        .await;

        // Act: Reload the context.
        let response = router
            .oneshot(Request::builder().uri("/ctx/vault/test_dir").body(Body::empty()).unwrap())
            .await
            .unwrap();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let reloaded_bundle: (Vec<DataNode>, Vec<crate::elements::edge::Edge>, Context) =
            serde_json::from_slice(&body).unwrap();
        let reloaded_context = reloaded_bundle.2;

        // Assert
        let node_a_data = initial_nodes.iter().find(|n| n.path().name() == "A.txt").unwrap();
        let reloaded_a = reloaded_context.viewnodes().iter().find(|vn| vn.uuid == node_a_data.uuid()).unwrap();
        let reloaded_b = reloaded_context.viewnodes().iter().find(|vn| vn.uuid == node_b_data.uuid()).unwrap();

        assert_eq!(reloaded_b.relX, 500.0, "Node B should have saved X pos");
        assert_ne!(reloaded_a.relX, 0.0, "Node A should have default X pos");
        assert_ne!(reloaded_a.relX, reloaded_b.relX, "A and B should have different X positions");
    }

    #[tokio::test]
    async fn test_create_node_outside_vault_fails() {
        let (router, _test_ctx) = setup_test_environment("create_node_fails");

        // Arrange
        let payload = CreateNodePayload {
            name: "test_node".to_string(),
            ntype: NodeTypeId::file_type(),
            parent_path: "/some_other_path".to_string(),
            attributes: vec![],
        };
        let payload_json = serde_json::to_string(&payload).unwrap();

        // Act
        let response = execute_post_request(router, "/api/nodes", payload_json).await;

        // Assert
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_create_node_inside_vault_succeeds() {
        let (router, test_ctx) = setup_test_environment("create_node_succeeds");

        // Arrange
        let payload = CreateNodePayload {
            name: "test_node".to_string(),
            ntype: NodeTypeId::file_type(),
            parent_path: "/vault/some_dir".to_string(),
            attributes: vec![],
        };
        let payload_json = serde_json::to_string(&payload).unwrap();

        // Act
        let response = execute_post_request(router, "/api/nodes", payload_json).await;

        // Assert
        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let created_node: DataNode = serde_json::from_slice(&body).unwrap();
        assert_eq!(created_node.path().alias(), "/vault/some_dir/test_node");

        // Verify it was actually inserted
        let node_from_db = test_ctx.with_service(|s| {
            s.data().open_node(&NodeHandle::Path(created_node.path()))
        });
        assert!(node_from_db.is_ok());
    }

    #[tokio::test]
    async fn test_move_nodes_single_file() {
        let (router, test_ctx) = setup_test_environment("move_nodes_single_file");

        // Create test directory structure
        std::fs::create_dir_all(test_ctx.get_vault_root().join("source_dir")).unwrap();
        std::fs::create_dir_all(test_ctx.get_vault_root().join("dest_dir")).unwrap();
        std::fs::write(test_ctx.get_vault_root().join("source_dir/test_file.txt"), "content").unwrap();

        // Index nodes in database
        test_ctx.with_service_mut(|s| {
            s.data_mut().insert_nodes(vec![
                DataNode::new(&"vault".into(), NodeTypeId::dir_type()),
                DataNode::new(&"vault/source_dir".into(), NodeTypeId::dir_type()),
                DataNode::new(&"vault/dest_dir".into(), NodeTypeId::dir_type()),
                DataNode::new(&"vault/source_dir/test_file.txt".into(), NodeTypeId::file_type()),
            ]);
        });

        // Create move request
        let move_payload = MoveNodesPayload {
            moves: vec![MoveOperation {
                source_path: "/vault/source_dir/test_file.txt".to_string(),
                target_parent_path: "/vault/dest_dir".to_string(),
            }],
        };

        let request = Request::builder()
            .method(http::Method::POST)
            .uri("/api/nodes/move")
            .header(http::header::CONTENT_TYPE, "application/json")
            .body(Body::from(serde_json::to_string(&move_payload).unwrap()))
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        // Verify response
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let move_response: MoveNodesResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(move_response.moved_nodes.len(), 1);
        assert_eq!(move_response.errors.len(), 0);
        assert_eq!(move_response.moved_nodes[0].path, "/vault/dest_dir/test_file.txt");

        // Verify filesystem changes
        assert!(!test_ctx.get_vault_root().join("source_dir/test_file.txt").exists());
        assert!(test_ctx.get_vault_root().join("dest_dir/test_file.txt").exists());

        // Verify context changes - moved file should no longer be in source context
        let (source_nodes, _, _) = test_ctx.with_service(|s| {
            s.open_context_from_path("vault/source_dir".into()).unwrap()
        });
        assert!(!source_nodes.iter().any(|n| n.path().name() == "test_file.txt"), 
                "File should no longer be in source directory context");

        // Verify moved file is now in destination context  
        let (dest_nodes, _, _) = test_ctx.with_service(|s| {
            s.open_context_from_path("vault/dest_dir".into()).unwrap()
        });
        assert!(dest_nodes.iter().any(|n| n.path().name() == "test_file.txt"), 
                "File should now be in destination directory context");
    }

    #[tokio::test]
    async fn test_move_nodes_directory_with_children() {
        let (router, test_ctx) = setup_test_environment("move_nodes_directory");

        // Create nested directory structure
        std::fs::create_dir_all(test_ctx.get_vault_root().join("source_dir/movable_dir")).unwrap();
        std::fs::create_dir_all(test_ctx.get_vault_root().join("dest_dir")).unwrap();
        std::fs::write(test_ctx.get_vault_root().join("source_dir/movable_dir/file.txt"), "content").unwrap();

        // Index nodes in database
        test_ctx.with_service_mut(|s| {
            s.data_mut().insert_nodes(vec![
                DataNode::new(&"vault".into(), NodeTypeId::dir_type()),
                DataNode::new(&"vault/source_dir".into(), NodeTypeId::dir_type()),
                DataNode::new(&"vault/dest_dir".into(), NodeTypeId::dir_type()),
                DataNode::new(&"vault/source_dir/movable_dir".into(), NodeTypeId::dir_type()),
                DataNode::new(&"vault/source_dir/movable_dir/file.txt".into(), NodeTypeId::file_type()),
            ]);
        });

        // Create move request
        let move_payload = MoveNodesPayload {
            moves: vec![MoveOperation {
                source_path: "/vault/source_dir/movable_dir".to_string(),
                target_parent_path: "/vault/dest_dir".to_string(),
            }],
        };

        let request = Request::builder()
            .method(http::Method::POST)
            .uri("/api/nodes/move")
            .header(http::header::CONTENT_TYPE, "application/json")
            .body(Body::from(serde_json::to_string(&move_payload).unwrap()))
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let move_response: MoveNodesResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(move_response.moved_nodes.len(), 2); // Directory + file inside it
        assert_eq!(move_response.errors.len(), 0);

        // Verify both directory and child file paths were updated in database
        test_ctx.with_service(|s| {
            let moved_dir = s.data().open_node(&NodeHandle::Path("vault/dest_dir/movable_dir".into())).unwrap();
            assert_eq!(moved_dir.path().alias(), "/vault/dest_dir/movable_dir");

            let moved_file = s.data().open_node(&NodeHandle::Path("vault/dest_dir/movable_dir/file.txt".into())).unwrap();
            assert_eq!(moved_file.path().alias(), "/vault/dest_dir/movable_dir/file.txt");
        });

        // Verify filesystem changes
        assert!(!test_ctx.get_vault_root().join("source_dir/movable_dir").exists());
        assert!(test_ctx.get_vault_root().join("dest_dir/movable_dir").exists());
        assert!(test_ctx.get_vault_root().join("dest_dir/movable_dir/file.txt").exists());

        // Verify context changes - moved directory should no longer be in source context
        let (source_nodes, _, _) = test_ctx.with_service(|s| {
            s.open_context_from_path("vault/source_dir".into()).unwrap()
        });
        assert!(!source_nodes.iter().any(|n| n.path().name() == "movable_dir"), 
                "Directory should no longer be in source context");

        // Verify moved directory is now in destination context
        let (dest_nodes, _, _) = test_ctx.with_service(|s| {
            s.open_context_from_path("vault/dest_dir".into()).unwrap()
        });
        assert!(dest_nodes.iter().any(|n| n.path().name() == "movable_dir"), 
                "Directory should now be in destination context");
    }

    #[tokio::test]
    async fn test_move_nodes_directory_returns_all_affected_nodes() {
        let (router, test_ctx) = setup_test_environment("move_nodes_all_affected");

        // Create nested directory structure with multiple levels
        std::fs::create_dir_all(test_ctx.get_vault_root().join("source_dir/movable_dir/subdir")).unwrap();
        std::fs::create_dir_all(test_ctx.get_vault_root().join("dest_dir")).unwrap();
        std::fs::write(test_ctx.get_vault_root().join("source_dir/movable_dir/file1.txt"), "content1").unwrap();
        std::fs::write(test_ctx.get_vault_root().join("source_dir/movable_dir/subdir/file2.txt"), "content2").unwrap();

        // Index nodes in database
        test_ctx.with_service_mut(|s| {
            s.data_mut().insert_nodes(vec![
                DataNode::new(&"vault".into(), NodeTypeId::dir_type()),
                DataNode::new(&"vault/source_dir".into(), NodeTypeId::dir_type()),
                DataNode::new(&"vault/dest_dir".into(), NodeTypeId::dir_type()),
                DataNode::new(&"vault/source_dir/movable_dir".into(), NodeTypeId::dir_type()),
                DataNode::new(&"vault/source_dir/movable_dir/file1.txt".into(), NodeTypeId::file_type()),
                DataNode::new(&"vault/source_dir/movable_dir/subdir".into(), NodeTypeId::dir_type()),
                DataNode::new(&"vault/source_dir/movable_dir/subdir/file2.txt".into(), NodeTypeId::file_type()),
            ]);
        });

        // Create move request
        let move_payload = MoveNodesPayload {
            moves: vec![MoveOperation {
                source_path: "/vault/source_dir/movable_dir".to_string(),
                target_parent_path: "/vault/dest_dir".to_string(),
            }],
        };

        let request = Request::builder()
            .method(http::Method::POST)
            .uri("/api/nodes/move")
            .header(http::header::CONTENT_TYPE, "application/json")
            .body(Body::from(serde_json::to_string(&move_payload).unwrap()))
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let move_response: MoveNodesResponse = serde_json::from_slice(&body).unwrap();
        
        // Should include directory + file1.txt + subdir + file2.txt = 4 nodes
        println!("Moved nodes returned: {}", move_response.moved_nodes.len());
        for (i, node) in move_response.moved_nodes.iter().enumerate() {
            println!("  {}: {} ({})", i+1, node.path, node.uuid);
        }
        
        assert_eq!(move_response.errors.len(), 0);
        assert_eq!(move_response.moved_nodes.len(), 4, "Should return all affected nodes: directory + 2 files + 1 subdirectory");

        // Verify all nodes have correct new paths
        let paths: Vec<_> = move_response.moved_nodes.iter().map(|n| &n.path).collect();
        assert!(paths.contains(&&"/vault/dest_dir/movable_dir".to_string()));
        assert!(paths.contains(&&"/vault/dest_dir/movable_dir/file1.txt".to_string()));
        assert!(paths.contains(&&"/vault/dest_dir/movable_dir/subdir".to_string()));
        assert!(paths.contains(&&"/vault/dest_dir/movable_dir/subdir/file2.txt".to_string()));
    }

    #[tokio::test]
    async fn test_move_nodes_batch_operations() {
        let (router, test_ctx) = setup_test_environment("move_nodes_batch");

        // Create test structure
        std::fs::create_dir_all(test_ctx.get_vault_root().join("source_dir")).unwrap();
        std::fs::create_dir_all(test_ctx.get_vault_root().join("dest_dir")).unwrap();
        std::fs::write(test_ctx.get_vault_root().join("source_dir/file1.txt"), "content1").unwrap();
        std::fs::write(test_ctx.get_vault_root().join("source_dir/file2.txt"), "content2").unwrap();

        // Index nodes
        test_ctx.with_service_mut(|s| {
            s.data_mut().insert_nodes(vec![
                DataNode::new(&"vault".into(), NodeTypeId::dir_type()),
                DataNode::new(&"vault/source_dir".into(), NodeTypeId::dir_type()),
                DataNode::new(&"vault/dest_dir".into(), NodeTypeId::dir_type()),
                DataNode::new(&"vault/source_dir/file1.txt".into(), NodeTypeId::file_type()),
                DataNode::new(&"vault/source_dir/file2.txt".into(), NodeTypeId::file_type()),
            ]);
        });

        // Create batch move request
        let move_payload = MoveNodesPayload {
            moves: vec![
                MoveOperation {
                    source_path: "/vault/source_dir/file1.txt".to_string(),
                    target_parent_path: "/vault/dest_dir".to_string(),
                },
                MoveOperation {
                    source_path: "/vault/source_dir/file2.txt".to_string(),
                    target_parent_path: "/vault/dest_dir".to_string(),
                },
            ],
        };

        let request = Request::builder()
            .method(http::Method::POST)
            .uri("/api/nodes/move")
            .header(http::header::CONTENT_TYPE, "application/json")
            .body(Body::from(serde_json::to_string(&move_payload).unwrap()))
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let move_response: MoveNodesResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(move_response.moved_nodes.len(), 2);
        assert_eq!(move_response.errors.len(), 0);

        // Verify both files were moved
        assert!(test_ctx.get_vault_root().join("dest_dir/file1.txt").exists());
        assert!(test_ctx.get_vault_root().join("dest_dir/file2.txt").exists());
        assert!(!test_ctx.get_vault_root().join("source_dir/file1.txt").exists());
        assert!(!test_ctx.get_vault_root().join("source_dir/file2.txt").exists());

        // Verify context changes - both files should no longer be in source context
        let (source_nodes, _, _) = test_ctx.with_service(|s| {
            s.open_context_from_path("vault/source_dir".into()).unwrap()
        });
        assert!(!source_nodes.iter().any(|n| n.path().name() == "file1.txt"), 
                "file1.txt should no longer be in source context");
        assert!(!source_nodes.iter().any(|n| n.path().name() == "file2.txt"), 
                "file2.txt should no longer be in source context");

        // Verify both files are now in destination context
        let (dest_nodes, _, _) = test_ctx.with_service(|s| {
            s.open_context_from_path("vault/dest_dir".into()).unwrap()
        });
        assert!(dest_nodes.iter().any(|n| n.path().name() == "file1.txt"), 
                "file1.txt should now be in destination context");
        assert!(dest_nodes.iter().any(|n| n.path().name() == "file2.txt"), 
                "file2.txt should now be in destination context");
    }

    #[tokio::test]
    async fn test_move_nodes_error_handling() {
        let (router, test_ctx) = setup_test_environment("move_nodes_errors");

        // Create test structure (missing dest_dir to trigger error)
        std::fs::create_dir_all(test_ctx.get_vault_root().join("source_dir")).unwrap();
        std::fs::write(test_ctx.get_vault_root().join("source_dir/file.txt"), "content").unwrap();

        // Index source nodes only (dest_dir not indexed)
        test_ctx.with_service_mut(|s| {
            s.data_mut().insert_nodes(vec![
                DataNode::new(&"vault".into(), NodeTypeId::dir_type()),
                DataNode::new(&"vault/source_dir".into(), NodeTypeId::dir_type()),
                DataNode::new(&"vault/source_dir/file.txt".into(), NodeTypeId::file_type()),
            ]);
        });

        // Create move request with invalid target
        let move_payload = MoveNodesPayload {
            moves: vec![MoveOperation {
                source_path: "/vault/source_dir/file.txt".to_string(),
                target_parent_path: "/vault/nonexistent_dir".to_string(),
            }],
        };

        let request = Request::builder()
            .method(http::Method::POST)
            .uri("/api/nodes/move")
            .header(http::header::CONTENT_TYPE, "application/json")
            .body(Body::from(serde_json::to_string(&move_payload).unwrap()))
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK); // Should still return 200 with errors in body

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let move_response: MoveNodesResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(move_response.moved_nodes.len(), 0);
        assert_eq!(move_response.errors.len(), 1);
        assert!(move_response.errors[0].error.contains("Target parent path not found"));
    }

    #[tokio::test]
    async fn test_move_nodes_prevents_circular_moves() {
        let (router, test_ctx) = setup_test_environment("move_nodes_circular");

        // Create directory structure
        std::fs::create_dir_all(test_ctx.get_vault_root().join("parent_dir/child_dir")).unwrap();

        // Index nodes
        test_ctx.with_service_mut(|s| {
            s.data_mut().insert_nodes(vec![
                DataNode::new(&"vault".into(), NodeTypeId::dir_type()),
                DataNode::new(&"vault/parent_dir".into(), NodeTypeId::dir_type()),
                DataNode::new(&"vault/parent_dir/child_dir".into(), NodeTypeId::dir_type()),
            ]);
        });

        // Attempt to move parent into its own child (circular move)
        let move_payload = MoveNodesPayload {
            moves: vec![MoveOperation {
                source_path: "/vault/parent_dir".to_string(),
                target_parent_path: "/vault/parent_dir/child_dir".to_string(),
            }],
        };

        let request = Request::builder()
            .method(http::Method::POST)
            .uri("/api/nodes/move")
            .header(http::header::CONTENT_TYPE, "application/json")
            .body(Body::from(serde_json::to_string(&move_payload).unwrap()))
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let move_response: MoveNodesResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(move_response.moved_nodes.len(), 0);
        assert_eq!(move_response.errors.len(), 1);
        assert!(move_response.errors[0].error.contains("Cannot move node to itself or its child"));
    }

    #[tokio::test]
    async fn test_move_nodes_name_collision_auto_rename() {
        let (router, test_ctx) = setup_test_environment("move_nodes_collision");

        // Create source directory with a file
        let source_dir = test_ctx.vault_root_path.join("source_folder");
        std::fs::create_dir(&source_dir).expect("Failed to create source directory");
        let source_file = source_dir.join("test_file.txt");
        std::fs::write(&source_file, "source content").expect("Failed to create source file");

        // Create target directory with a file that has the same name
        let target_dir = test_ctx.vault_root_path.join("target_folder");
        std::fs::create_dir(&target_dir).expect("Failed to create target directory");
        let existing_file = target_dir.join("test_file.txt");
        std::fs::write(&existing_file, "existing content").expect("Failed to create existing file");

        // Index nodes in database
        test_ctx.with_service_mut(|s| {
            s.data_mut().insert_nodes(vec![
                DataNode::new(&"vault".into(), NodeTypeId::dir_type()),
                DataNode::new(&"vault/source_folder".into(), NodeTypeId::dir_type()),
                DataNode::new(&"vault/target_folder".into(), NodeTypeId::dir_type()),
                DataNode::new(&"vault/source_folder/test_file.txt".into(), NodeTypeId::file_type()),
                DataNode::new(&"vault/target_folder/test_file.txt".into(), NodeTypeId::file_type()),
            ]);
        });

        // Attempt to move source file to target directory (should auto-rename)
        let move_payload = MoveNodesPayload {
            moves: vec![MoveOperation {
                source_path: "/vault/source_folder/test_file.txt".to_string(),
                target_parent_path: "/vault/target_folder".to_string(),
            }],
        };

        let response = router
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/nodes/move")
                    .header("Content-Type", "application/json")
                    .body(Body::from(serde_json::to_string(&move_payload).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let move_response: MoveNodesResponse = serde_json::from_slice(&body).unwrap();
        
        // Should succeed with auto-renaming
        assert_eq!(move_response.errors.len(), 0);
        assert_eq!(move_response.moved_nodes.len(), 1);

        // Verify the file was auto-renamed (should be test_file_2.txt)
        let renamed_file = target_dir.join("test_file_2.txt");
        assert!(renamed_file.exists(), "Renamed file should exist");
        assert_eq!(
            std::fs::read_to_string(&renamed_file).unwrap(),
            "source content",
            "Renamed file should have source content"
        );

        // Original collision file should still exist
        assert!(existing_file.exists(), "Original file should still exist");
        assert_eq!(
            std::fs::read_to_string(&existing_file).unwrap(),
            "existing content",
            "Original file content should be unchanged"
        );

        // Source file should no longer exist
        assert!(!source_file.exists(), "Source file should be moved");
    }

    #[tokio::test]
    async fn test_move_nodes_to_unindexed_directory() {
        let (router, test_ctx) = setup_test_environment("move_nodes_unindexed");

        // Create source directory with a file
        let source_dir = test_ctx.vault_root_path.join("source_folder");
        std::fs::create_dir(&source_dir).expect("Failed to create source directory");
        let source_file = source_dir.join("test_file.txt");
        std::fs::write(&source_file, "source content").expect("Failed to create source file");

        // Create target directory but DON'T index it in the database
        let target_dir = test_ctx.vault_root_path.join("unindexed_folder");
        std::fs::create_dir(&target_dir).expect("Failed to create target directory");

        // Index only the source nodes in database (not the target)
        test_ctx.with_service_mut(|s| {
            s.data_mut().insert_nodes(vec![
                DataNode::new(&"vault".into(), NodeTypeId::dir_type()),
                DataNode::new(&"vault/source_folder".into(), NodeTypeId::dir_type()),
                DataNode::new(&"vault/source_folder/test_file.txt".into(), NodeTypeId::file_type()),
            ]);
        });

        // Attempt to move source file to unindexed target directory (should succeed)
        let move_payload = MoveNodesPayload {
            moves: vec![MoveOperation {
                source_path: "/vault/source_folder/test_file.txt".to_string(),
                target_parent_path: "/vault/unindexed_folder".to_string(),
            }],
        };

        let response = router
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/nodes/move")
                    .header("Content-Type", "application/json")
                    .body(Body::from(serde_json::to_string(&move_payload).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let move_response: MoveNodesResponse = serde_json::from_slice(&body).unwrap();
        
        // Should succeed even with unindexed target directory
        assert_eq!(move_response.errors.len(), 0);
        assert_eq!(move_response.moved_nodes.len(), 1);

        // Verify the file was moved on filesystem
        let moved_file = target_dir.join("test_file.txt");
        assert!(moved_file.exists(), "File should be moved to unindexed directory");
        assert_eq!(
            std::fs::read_to_string(&moved_file).unwrap(),
            "source content",
            "Moved file should have original content"
        );

        // Source file should no longer exist
        assert!(!source_file.exists(), "Source file should be moved");

        // Verify the moved node has the correct path
        assert_eq!(
            move_response.moved_nodes[0].path,
            "/vault/unindexed_folder/test_file.txt"
        );
    }

    #[tokio::test]
    async fn test_move_nodes_with_context_file_persistence() {
        let (router, test_ctx) = setup_test_environment("move_nodes_context_files");

        // Create test structure
        std::fs::create_dir_all(test_ctx.get_vault_root().join("source_dir")).unwrap();
        std::fs::create_dir_all(test_ctx.get_vault_root().join("dest_dir")).unwrap();
        std::fs::write(test_ctx.get_vault_root().join("source_dir/test_file.txt"), "content").unwrap();

        // Index nodes in database
        let file_uuid = test_ctx.with_service_mut(|s| {
            let file_node = DataNode::new(&"vault/source_dir/test_file.txt".into(), NodeTypeId::file_type());
            let file_uuid = file_node.uuid();
            s.data_mut().insert_nodes(vec![
                DataNode::new(&"vault".into(), NodeTypeId::dir_type()),
                DataNode::new(&"vault/source_dir".into(), NodeTypeId::dir_type()),
                DataNode::new(&"vault/dest_dir".into(), NodeTypeId::dir_type()),
                file_node,
            ]);
            file_uuid
        });

        // Get the source directory context and save it
        let (source_nodes, _, source_context) = test_ctx.with_service(|s| {
            s.open_context_from_path("vault/source_dir".into()).unwrap()
        });
        let source_focal_uuid = source_context.focal();
        
        // Save the source context file BEFORE the move
        test_ctx.with_service_mut(|s| {
            s.view_mut().save_context(&source_context).unwrap();
        });

        // Verify source context contains the file before move
        assert!(source_nodes.iter().any(|n| n.path().name() == "test_file.txt"), 
                "Source context should contain file before move");

        // Create move request
        let move_payload = MoveNodesPayload {
            moves: vec![MoveOperation {
                source_path: "/vault/source_dir/test_file.txt".to_string(),
                target_parent_path: "/vault/dest_dir".to_string(),
            }],
        };

        let request = Request::builder()
            .method(http::Method::POST)
            .uri("/api/nodes/move")
            .header(http::header::CONTENT_TYPE, "application/json")
            .body(Body::from(serde_json::to_string(&move_payload).unwrap()))
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let move_response: MoveNodesResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(move_response.moved_nodes.len(), 1);
        assert_eq!(move_response.errors.len(), 0);

        // Test 1: When loading a context that has a saved context file, 
        // it should merge saved nodes (by UUID) with current filesystem children
        let (source_nodes_with_saved, _, _) = test_ctx.with_service(|s| {
            s.open_context_from_path("vault/source_dir".into()).unwrap()
        });
        
        // The context should contain the moved file because it's in the saved context file
        assert!(source_nodes_with_saved.iter().any(|n| n.uuid() == file_uuid), 
                "Source context WITH saved file should contain moved file by UUID");
        
        // Test 2: Verify the moved file now has the updated path in the database
        let moved_file_node = test_ctx.with_service(|s| {
            s.data().open_node(&NodeHandle::Uuid(file_uuid)).unwrap()
        });
        assert_eq!(moved_file_node.path().alias(), "/vault/dest_dir/test_file.txt",
                "Moved file should have updated path in database");

        // Test 3: Generated destination context should contain the moved file
        let (dest_nodes_generated, _, _) = test_ctx.with_service(|s| {
            s.open_context_from_path("vault/dest_dir".into()).unwrap()
        });
        assert!(dest_nodes_generated.iter().any(|n| n.path().name() == "test_file.txt"), 
                "Destination context should contain moved file");

        // Test 4: Verify saved context files preserve the UUID correctly
        let saved_source_context = test_ctx.with_service(|s| {
            s.view().get_context_file(source_focal_uuid).unwrap()
        });
        assert!(saved_source_context.viewnodes().iter().any(|vn| vn.uuid == file_uuid), 
                "Saved source context should preserve file UUID even after move");

        // Test 5: CRITICAL - Verify edges reflect current database state, not saved context
        // After move, there should be NO "contains" edge between source_dir and the moved file
        let (_, source_edges_after_move, _) = test_ctx.with_service(|s| {
            s.open_context_from_path("vault/source_dir".into()).unwrap()
        });
        
        let source_dir_uuid = test_ctx.with_service(|s| {
            s.data().open_node(&NodeHandle::Path("vault/source_dir".into())).unwrap().uuid()
        });
        
        // There should be NO edge from source_dir to the moved file
        let has_contains_edge = source_edges_after_move.iter().any(|edge| {
            edge.source() == &source_dir_uuid && edge.target() == &file_uuid
        });
        assert!(!has_contains_edge, 
                "Should be NO contains edge between source_dir and moved file (edges come from DB, not saved context)");
        
        // But the destination should now have the contains edge
        let (_, dest_edges_after_move, _) = test_ctx.with_service(|s| {
            s.open_context_from_path("vault/dest_dir".into()).unwrap()
        });
        
        let dest_dir_uuid = test_ctx.with_service(|s| {
            s.data().open_node(&NodeHandle::Path("vault/dest_dir".into())).unwrap().uuid()
        });
        
        let dest_has_contains_edge = dest_edges_after_move.iter().any(|edge| {
            edge.source() == &dest_dir_uuid && edge.target() == &file_uuid
        });
        assert!(dest_has_contains_edge, 
                "Destination should now have contains edge to moved file");
    }




    #[tokio::test]
    async fn test_move_nodes_preserves_uuid_no_duplicates() {
        let (router, test_ctx) = setup_test_environment("move_nodes_uuid_preservation");

        // Create test structure
        std::fs::create_dir_all(test_ctx.get_vault_root().join("source_dir")).unwrap();
        std::fs::create_dir_all(test_ctx.get_vault_root().join("dest_dir")).unwrap();
        std::fs::write(test_ctx.get_vault_root().join("source_dir/test_file.txt"), "content").unwrap();

        // Index nodes in database and capture original UUID
        let original_file_uuid = test_ctx.with_service_mut(|s| {
            let file_node = DataNode::new(&"vault/source_dir/test_file.txt".into(), NodeTypeId::file_type());
            let file_uuid = file_node.uuid();
            s.data_mut().insert_nodes(vec![
                DataNode::new(&"vault".into(), NodeTypeId::dir_type()),
                DataNode::new(&"vault/source_dir".into(), NodeTypeId::dir_type()),
                DataNode::new(&"vault/dest_dir".into(), NodeTypeId::dir_type()),
                file_node,
            ]);
            file_uuid
        });

        println!("[TEST] Original file UUID: {}", original_file_uuid);

        // Create move request
        let move_payload = MoveNodesPayload {
            moves: vec![MoveOperation {
                source_path: "/vault/source_dir/test_file.txt".to_string(),
                target_parent_path: "/vault/dest_dir".to_string(),
            }],
        };

        let request = Request::builder()
            .method(http::Method::POST)
            .uri("/api/nodes/move")
            .header(http::header::CONTENT_TYPE, "application/json")
            .body(Body::from(serde_json::to_string(&move_payload).unwrap()))
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let move_response: MoveNodesResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(move_response.moved_nodes.len(), 1);
        assert_eq!(move_response.errors.len(), 0);

        let moved_node = &move_response.moved_nodes[0];
        println!("[TEST] Moved node UUID: {}", moved_node.uuid);
        println!("[TEST] Moved node path: {}", moved_node.path);

        // CRITICAL TEST 1: UUID should be preserved during move
        assert_eq!(moved_node.uuid, original_file_uuid, 
                "Move operation must preserve the original UUID");

        // CRITICAL TEST 2: Opening the destination context should show exactly ONE file with the original UUID
        let (dest_nodes, _, dest_context) = test_ctx.with_service(|s| {
            s.open_context_from_path("vault/dest_dir".into()).unwrap()
        });

        let files_with_same_name: Vec<_> = dest_nodes.iter()
            .filter(|n| n.path().name() == "test_file.txt")
            .collect();
        
        println!("[TEST] Files with name 'test_file.txt' in destination: {}", files_with_same_name.len());
        for (i, file) in files_with_same_name.iter().enumerate() {
            println!("[TEST]   File {}: UUID={}, Path={}", i+1, file.uuid(), file.path().alias());
        }

        assert_eq!(files_with_same_name.len(), 1, 
                "Should be exactly ONE file with the moved name in destination context");
        
        assert_eq!(files_with_same_name[0].uuid(), original_file_uuid,
                "The file in destination should have the original UUID");

        // CRITICAL TEST 3: Check ViewNodes in context for duplicates
        let viewnodes_with_original_uuid: Vec<_> = dest_context.viewnodes().iter()
            .filter(|vn| vn.uuid == original_file_uuid)
            .collect();

        println!("[TEST] ViewNodes with original UUID in destination context: {}", viewnodes_with_original_uuid.len());
        for (i, vn) in viewnodes_with_original_uuid.iter().enumerate() {
            println!("[TEST]   ViewNode {}: UUID={}", i+1, vn.uuid);
        }

        assert_eq!(viewnodes_with_original_uuid.len(), 1,
                "Should be exactly ONE ViewNode with the original UUID in destination context");

        // CRITICAL TEST 4: Check that old path no longer exists in database
        let old_path_lookup = test_ctx.with_service(|s| {
            s.data().open_node(&NodeHandle::Path("vault/source_dir/test_file.txt".into()))
        });

        assert!(old_path_lookup.is_err(),
                "Old path should no longer exist in database after move");

        // CRITICAL TEST 5: Check that the UUID now points to the new path
        let uuid_lookup = test_ctx.with_service(|s| {
            s.data().open_node(&NodeHandle::Uuid(original_file_uuid))
        });

        assert!(uuid_lookup.is_ok(), "UUID should still be valid after move");
        let node_by_uuid = uuid_lookup.unwrap();
        assert_eq!(node_by_uuid.path().alias(), "/vault/dest_dir/test_file.txt",
                "UUID should now point to the new path");

        // CRITICAL TEST 6: Filesystem should only have the file in the new location
        assert!(!test_ctx.get_vault_root().join("source_dir/test_file.txt").exists(),
                "File should no longer exist at old filesystem location");
        assert!(test_ctx.get_vault_root().join("dest_dir/test_file.txt").exists(),
                "File should exist at new filesystem location");
    }

    #[tokio::test]
    async fn test_rename_node_endpoint() {
        let (router, test_ctx) = setup_test_environment("test_rename_node_endpoint");
        
        // Create a test file
        test_ctx.create_file_in_vault("test_file.txt", b"test content").unwrap();
        
        // Index the file in the database
        let file_path = NodePath::new("vault/test_file.txt".into());
        let file_uuid = test_ctx.with_service_mut(|s| {
            let file_node = crate::fs_reader::destructure_single_path(s.vault_fs_path(), &file_path).unwrap();
            let uuid = file_node.uuid();
            s.data_mut().insert_nodes(vec![file_node]);
            uuid
        });

        // Test renaming the file using the dedicated rename endpoint
        let rename_payload = RenameNodeByPathPayload {
            path: "/vault/test_file.txt".to_string(),
            new_name: "renamed_file.txt".to_string(),
        };

        let response = execute_post_request(router, "/api/nodes/rename", serde_json::to_string(&rename_payload).unwrap()).await;
        
        assert_eq!(response.status(), StatusCode::OK);
        
        let response_body: RenameNodeResponse = serde_json::from_slice(
            &axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap()
        ).unwrap();
        
        // Should have exactly one renamed node
        assert_eq!(response_body.renamed_nodes.len(), 1);
        
        let renamed_node = &response_body.renamed_nodes[0];
        assert_eq!(renamed_node.path, "/vault/renamed_file.txt");
        assert_eq!(renamed_node.uuid, file_uuid);
        
        // Verify the file was actually renamed on filesystem
        assert!(test_ctx.get_vault_root().join("renamed_file.txt").exists());
        assert!(!test_ctx.get_vault_root().join("test_file.txt").exists());
        
        // Verify the database was updated correctly
        test_ctx.with_service(|s| {
            // Old path should not exist
            assert!(s.data().open_node(&NodeHandle::Path("vault/test_file.txt".into())).is_err());
            
            // New path should exist with same UUID
            let renamed_node = s.data().open_node(&NodeHandle::Path("vault/renamed_file.txt".into())).unwrap();
            assert_eq!(renamed_node.uuid(), file_uuid);
            assert_eq!(renamed_node.name(), "renamed_file.txt");
        });
    }

    #[tokio::test]
    async fn test_rename_node_with_collision_resolution() {
        let (router, test_ctx) = setup_test_environment("test_rename_collision");

        // Create two files with different names
        test_ctx.create_dir_in_vault("test_dir").unwrap();
        test_ctx.create_file_in_vault("test_dir/file1.txt", b"content1").unwrap();
        test_ctx.create_file_in_vault("test_dir/file2.txt", b"content2").unwrap();

        let file1_path = NodePath::new("vault/test_dir/file1.txt".into());
        test_ctx.with_service_mut(|s| {
            let node1 = crate::fs_reader::destructure_single_path(s.vault_fs_path(), &file1_path).unwrap();
            let node2 = crate::fs_reader::destructure_single_path(s.vault_fs_path(), &NodePath::new("vault/test_dir/file2.txt".into())).unwrap();
            s.data_mut().insert_nodes(vec![node1, node2]);
        });

        // Try to rename file1 to file2 (collision should be auto-resolved)
        let rename_payload = RenameNodeByPathPayload {
            path: "/vault/test_dir/file1.txt".to_string(),
            new_name: "file2.txt".to_string(),
        };

        let response = execute_post_request(router, "/api/nodes/rename", serde_json::to_string(&rename_payload).unwrap()).await;
        
        assert_eq!(response.status(), StatusCode::OK);
        
        let response_body: RenameNodeResponse = serde_json::from_slice(
            &axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap()
        ).unwrap();
        
        // Should have exactly one renamed node
        assert_eq!(response_body.renamed_nodes.len(), 1);
        
        let renamed_node = &response_body.renamed_nodes[0];
        // Should be auto-renamed to avoid collision (e.g., "file2_2.txt")
        assert_ne!(renamed_node.path, "/vault/test_dir/file2.txt");
        assert!(renamed_node.path.contains("file2"));
        assert!(renamed_node.path.contains("_"));
        
        // Verify filesystem - original files should exist, plus the renamed one
        assert!(test_ctx.get_vault_root().join("test_dir/file2.txt").exists()); // Original file2
        assert!(!test_ctx.get_vault_root().join("test_dir/file1.txt").exists()); // file1 should be gone
        
        // Extract just the filename from the full path
        let renamed_filename = renamed_node.path.split('/').last().unwrap();
        assert!(test_ctx.get_vault_root().join(format!("test_dir/{}", renamed_filename)).exists()); // Renamed file
    }

    #[tokio::test]
    async fn test_rename_virtual_node() {
        let (router, test_ctx) = setup_test_environment("test_rename_virtual_node");

        // Create a virtual node (no corresponding filesystem entry)
        let virtual_node_path = NodePath::new("vault/test_virtual_node".into());
        let node_uuid = test_ctx.with_service_mut(|s| {
            let virtual_node = DataNode::new(&virtual_node_path, NodeTypeId::file_type());
            let uuid = virtual_node.uuid();
            s.data_mut().insert_nodes(vec![virtual_node]);
            uuid
        });

        // Rename the virtual node using the dedicated rename endpoint
        let rename_payload = RenameNodeByPathPayload {
            path: "/vault/test_virtual_node".to_string(),
            new_name: "renamed_virtual_node".to_string(),
        };

        let response = execute_post_request(router, "/api/nodes/rename", serde_json::to_string(&rename_payload).unwrap()).await;
        
        assert_eq!(response.status(), StatusCode::OK);
        
        let response_body: RenameNodeResponse = serde_json::from_slice(
            &axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap()
        ).unwrap();
        
        // Should have exactly one renamed node
        assert_eq!(response_body.renamed_nodes.len(), 1);
        
        let renamed_node = &response_body.renamed_nodes[0];
        assert_eq!(renamed_node.path, "/vault/renamed_virtual_node");
        assert_eq!(renamed_node.uuid, node_uuid);

        // Verify no filesystem entries were created (since it's virtual)
        assert!(!test_ctx.get_vault_root().join("test_virtual_node").exists());
        assert!(!test_ctx.get_vault_root().join("renamed_virtual_node").exists());

        // Verify database was updated
        let new_path = NodePath::new("vault/renamed_virtual_node".into());
        test_ctx.with_service(|s| {
            let node = s.data().open_node(&NodeHandle::Path(new_path)).unwrap();
            assert_eq!(node.uuid(), node_uuid);
            assert_eq!(node.name(), "renamed_virtual_node");
            assert_eq!(node.path().alias(), "/vault/renamed_virtual_node");

            // Verify old path no longer exists
            assert!(s.data().open_node(&NodeHandle::Path(virtual_node_path)).is_err());
        });
    }

    #[tokio::test]
    async fn test_rename_directory_with_descendants() {
        let (router, test_ctx) = setup_test_environment("test_rename_directory_descendants");

        // Create a directory with children
        test_ctx.create_dir_in_vault("parent_dir").unwrap();
        test_ctx.create_dir_in_vault("parent_dir/old_dir_name").unwrap();
        test_ctx.create_file_in_vault("parent_dir/old_dir_name/child_file.txt", b"content").unwrap();
        test_ctx.create_dir_in_vault("parent_dir/old_dir_name/child_dir").unwrap();
        test_ctx.create_file_in_vault("parent_dir/old_dir_name/child_dir/nested_file.txt", b"nested").unwrap();

        // Index all nodes
        let paths_to_index = vec![
            NodePath::new("vault/parent_dir".into()),
            NodePath::new("vault/parent_dir/old_dir_name".into()),
            NodePath::new("vault/parent_dir/old_dir_name/child_file.txt".into()),
            NodePath::new("vault/parent_dir/old_dir_name/child_dir".into()),
            NodePath::new("vault/parent_dir/old_dir_name/child_dir/nested_file.txt".into()),
        ];

        let dir_uuid = test_ctx.with_service_mut(|s| {
            for path in paths_to_index {
                let node = crate::fs_reader::destructure_single_path(s.vault_fs_path(), &path).unwrap();
                s.data_mut().insert_nodes(vec![node]);
            }
            s.data().open_node(&NodeHandle::Path(NodePath::new("vault/parent_dir/old_dir_name".into()))).unwrap().uuid()
        });

        // Rename the directory using the dedicated rename endpoint
        let rename_payload = RenameNodeByPathPayload {
            path: "/vault/parent_dir/old_dir_name".to_string(),
            new_name: "new_dir_name".to_string(),
        };

        let response = execute_post_request(router, "/api/nodes/rename", serde_json::to_string(&rename_payload).unwrap()).await;
        
        assert_eq!(response.status(), StatusCode::OK);
        
        let response_body: RenameNodeResponse = serde_json::from_slice(
            &axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap()
        ).unwrap();
        
        // Should have multiple renamed nodes (directory + all descendants)
        assert!(response_body.renamed_nodes.len() >= 4, "Should rename directory and all descendants");
        
        // Find the main directory
        let renamed_dir = response_body.renamed_nodes.iter()
            .find(|n| n.uuid == dir_uuid)
            .expect("Renamed directory should be in response");
        assert_eq!(renamed_dir.path, "/vault/parent_dir/new_dir_name");
        
        // Verify all descendants have updated paths
        let child_file = response_body.renamed_nodes.iter()
            .find(|n| n.path.ends_with("child_file.txt"))
            .expect("child_file.txt should be in renamed nodes");
        assert_eq!(child_file.path, "/vault/parent_dir/new_dir_name/child_file.txt");
        
        let child_dir = response_body.renamed_nodes.iter()
            .find(|n| n.path.ends_with("child_dir") && !n.path.ends_with(".txt"))
            .expect("child_dir should be in renamed nodes");
        assert_eq!(child_dir.path, "/vault/parent_dir/new_dir_name/child_dir");
        
        let nested_file = response_body.renamed_nodes.iter()
            .find(|n| n.path.ends_with("nested_file.txt"))
            .expect("nested_file.txt should be in renamed nodes");
        assert_eq!(nested_file.path, "/vault/parent_dir/new_dir_name/child_dir/nested_file.txt");

        // Verify filesystem changes
        assert!(!test_ctx.get_vault_root().join("parent_dir/old_dir_name").exists());
        assert!(test_ctx.get_vault_root().join("parent_dir/new_dir_name").exists());
        assert!(test_ctx.get_vault_root().join("parent_dir/new_dir_name/child_file.txt").exists());
        assert!(test_ctx.get_vault_root().join("parent_dir/new_dir_name/child_dir/nested_file.txt").exists());
    }
}