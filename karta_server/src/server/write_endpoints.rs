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

#[derive(Deserialize, serde::Serialize)]
pub struct CreateNodePayload {
    name: String,
    ntype: NodeTypeId,
    parent_path: String,
    attributes: Vec<Attribute>,
}

#[derive(Deserialize)]
pub struct UpdateNodePayload {
    attributes: Vec<Attribute>,
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
pub struct MoveNodesResponse {
    moved_nodes: Vec<DataNode>,
    errors: Vec<MoveError>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct MoveError {
    source_path: String,
    error: String,
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
    
    // Update the name attribute if it was changed
    let mut attributes = payload.attributes;
    if let Some(attr) = attributes.iter_mut().find(|a| a.name == "name") {
        attr.value = AttrValue::String(name.clone());
    } else {
        attributes.push(Attribute::new_string("name".to_string(), name));
    }

    new_node.set_attributes(attributes);
    new_node.set_name(&new_node.path().name());

    service.data_mut().insert_nodes(vec![new_node.clone()]);

    Ok(Json(new_node))
}

pub async fn update_node(
    State(app_state): State<AppState>,
    AxumPath(id): AxumPath<Uuid>,
    Json(payload): Json<UpdateNodePayload>,
) -> Result<Json<DataNode>, StatusCode> {
    let mut service = app_state.service.write().unwrap();

    let mut node = match service.data().open_node(&NodeHandle::Uuid(id)) {
        Ok(node) => node,
        Err(_) => return Err(StatusCode::NOT_FOUND),
    };

    node.set_attributes(payload.attributes);
    node.update_modified_time();

    service.data_mut().insert_nodes(vec![node.clone()]);

    Ok(Json(node))
}

pub async fn move_nodes(
    State(app_state): State<AppState>,
    Json(payload): Json<MoveNodesPayload>,
) -> Result<Json<MoveNodesResponse>, StatusCode> {
    let mut service = app_state.service.write().unwrap();
    let mut moved_nodes = Vec::new();
    let mut errors = Vec::new();

    for move_op in payload.moves {
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

        match service.move_node_with_rename(&source_path, &target_parent_path, final_name) {
            Ok(final_path) => {
                // Get the moved node at its new location
                if let Ok(moved_node) = service.data().open_node(&NodeHandle::Path(final_path)) {
                    moved_nodes.push(moved_node);
                }
            }
            Err(e) => {
                errors.push(MoveError {
                    source_path: move_op.source_path.clone(),
                    error: format!("Move operation failed: {}", e),
                });
            }
        }
    }

    Ok(Json(MoveNodesResponse {
        moved_nodes,
        errors,
    }))
}

pub async fn save_context(
    State(app_state): State<AppState>,
    AxumPath(id): AxumPath<Uuid>,
    Json(payload): Json<Context>,
) -> StatusCode {
    println!("[SAVE_CONTEXT] Received save request for context ID: {}", id);
    println!("[SAVE_CONTEXT] Payload: {:?}", payload);
    // The `id` from the path should match the `focal` id in the payload.
    if id != payload.focal() {
        eprintln!("[SAVE_CONTEXT] Mismatch between path ID ({}) and payload focal ID ({}).", id, payload.focal());
        return StatusCode::BAD_REQUEST;
    }

    let service = app_state.service.read().unwrap();
    match service.view().save_context(&payload) {
        Ok(_) => {
            println!("[SAVE_CONTEXT] Successfully saved context {}", id);
            StatusCode::OK
        },
        Err(e) => {
            eprintln!("[SAVE_CONTEXT] Error saving context {}: {:?}", id, e);
            StatusCode::INTERNAL_SERVER_ERROR
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
        assert_eq!(move_response.moved_nodes[0].path().alias(), "/vault/dest_dir/test_file.txt");

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
        assert_eq!(move_response.moved_nodes.len(), 1);
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
            move_response.moved_nodes[0].path().alias(),
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
}