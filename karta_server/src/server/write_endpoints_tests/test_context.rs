use super::*;
use super::test_helpers::*;

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
