use super::*;
use super::test_helpers::*;

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
