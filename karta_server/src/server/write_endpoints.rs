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

#[derive(serde::Serialize, serde::Deserialize)]
pub struct MoveNodesResponse {
    moved_nodes: Vec<DataNode>,
    errors: Vec<MoveError>,
}

#[derive(serde::Serialize, serde::Deserialize)]
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
    let mut final_path = parent_path.join(&name);
    let mut counter = 2;

    // Loop until we find a unique path
    while service.data().open_node(&NodeHandle::Path(final_path.clone())).is_ok() {
        name = format!("{}_{}", payload.name.clone(), counter);
        final_path = parent_path.join(&name);
        counter += 1;
    }

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

        // Validate target parent exists and is a directory
        match service.data().open_node(&NodeHandle::Path(target_parent_path.clone())) {
            Ok(target_node) => {
                if !target_node.is_dir() {
                    errors.push(MoveError {
                        source_path: move_op.source_path.clone(),
                        error: "Target parent must be a directory".to_string(),
                    });
                    continue;
                }
            }
            Err(_) => {
                errors.push(MoveError {
                    source_path: move_op.source_path.clone(),
                    error: "Target parent path not found".to_string(),
                });
                continue;
            }
        }

        // Prevent moving to self or child (circular move)
        if target_parent_path.alias().starts_with(&source_path.alias()) {
            errors.push(MoveError {
                source_path: move_op.source_path.clone(),
                error: "Cannot move node to itself or its child".to_string(),
            });
            continue;
        }

        // Perform the move operation
        match service.move_node(&source_path, &target_parent_path) {
            Ok(_) => {
                // Get the moved node at its new location
                let new_path = target_parent_path.join(source_path.name().as_str());
                if let Ok(moved_node) = service.data().open_node(&NodeHandle::Path(new_path)) {
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
}