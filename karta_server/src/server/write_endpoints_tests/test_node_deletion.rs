use super::*;
use super::test_helpers::*;

#[tokio::test]
async fn test_delete_single_virtual_node() {
    let (router, test_ctx) = setup_test_environment("delete_single_virtual");

    // Create a virtual node
    let virtual_node = DataNode::new(&"vault/virtual_node".into(), NodeTypeId::virtual_generic());
    let node_uuid = test_ctx.with_service_mut(|s| {
        s.data_mut().insert_nodes(vec![virtual_node.clone()]);
        virtual_node.uuid()
    });

    // Delete the node
    let delete_payload = DeleteNodesPayload {
        node_ids: vec![node_uuid.to_string()],
        context_id: None,
    };
    let payload_json = serde_json::to_string(&delete_payload).unwrap();

    let response = execute_delete_request(router, "/api/nodes", payload_json).await;
    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let response_body: DeleteNodesResponse = serde_json::from_slice(&body_bytes).unwrap();

    // Verify response
    assert_eq!(response_body.deleted_nodes.len(), 1);
    assert_eq!(response_body.failed_deletions.len(), 0);
    assert_eq!(response_body.deleted_nodes[0].node_id, node_uuid.to_string());
    assert_eq!(response_body.deleted_nodes[0].was_physical, false);
    assert_eq!(response_body.deleted_nodes[0].descendants_deleted.len(), 0);

    // Verify node is removed from graph
    test_ctx.with_service(|s| {
        assert!(s.data().open_node(&NodeHandle::Uuid(node_uuid)).is_err());
    });
}

#[tokio::test]
async fn test_delete_single_physical_file() {
    let (router, test_ctx) = setup_test_environment("delete_single_file");

    // Create a physical file
    let file_path = test_ctx.get_vault_root().join("test_file.txt");
    std::fs::write(&file_path, "test content").expect("Failed to create test file");

    // Index the file
    let file_node = crate::fs_reader::destructure_single_path(
        test_ctx.get_vault_root(),
        &NodePath::new("vault/test_file.txt".into())
    ).unwrap();
    let node_uuid = test_ctx.with_service_mut(|s| {
        s.data_mut().insert_nodes(vec![file_node.clone()]);
        file_node.uuid()
    });

    // Delete the node
    let delete_payload = DeleteNodesPayload {
        node_ids: vec![node_uuid.to_string()],
        context_id: None,
    };
    let payload_json = serde_json::to_string(&delete_payload).unwrap();

    let response = execute_delete_request(router, "/api/nodes", payload_json).await;
    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let response_body: DeleteNodesResponse = serde_json::from_slice(&body_bytes).unwrap();

    // Verify response
    assert_eq!(response_body.deleted_nodes.len(), 1);
    assert_eq!(response_body.failed_deletions.len(), 0);
    assert_eq!(response_body.deleted_nodes[0].was_physical, true);

    // Verify node is removed from graph
    test_ctx.with_service(|s| {
        assert!(s.data().open_node(&NodeHandle::Uuid(node_uuid)).is_err());
    });

    // Verify file was moved to trash (should not exist in original location)
    assert!(!file_path.exists());
}

#[tokio::test]
async fn test_delete_directory_with_descendants() {
    let (router, test_ctx) = setup_test_environment("delete_directory_descendants");

    // Create directory structure
    std::fs::create_dir_all(test_ctx.get_vault_root().join("parent_dir/child_dir")).unwrap();
    std::fs::write(test_ctx.get_vault_root().join("parent_dir/file1.txt"), "content1").unwrap();
    std::fs::write(test_ctx.get_vault_root().join("parent_dir/child_dir/file2.txt"), "content2").unwrap();

    // Index all nodes
    let paths_to_index = vec![
        NodePath::new("vault/parent_dir".into()),
        NodePath::new("vault/parent_dir/file1.txt".into()),
        NodePath::new("vault/parent_dir/child_dir".into()),
        NodePath::new("vault/parent_dir/child_dir/file2.txt".into()),
    ];

    let parent_uuid = test_ctx.with_service_mut(|s| {
        for path in paths_to_index {
            let node = crate::fs_reader::destructure_single_path(s.vault_fs_path(), &path).unwrap();
            s.data_mut().insert_nodes(vec![node]);
        }
        s.data().open_node(&NodeHandle::Path(NodePath::new("vault/parent_dir".into()))).unwrap().uuid()
    });

    // Delete the parent directory
    let delete_payload = DeleteNodesPayload {
        node_ids: vec![parent_uuid.to_string()],
        context_id: None,
    };
    let payload_json = serde_json::to_string(&delete_payload).unwrap();

    let response = execute_delete_request(router, "/api/nodes", payload_json).await;
    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let response_body: DeleteNodesResponse = serde_json::from_slice(&body_bytes).unwrap();

    // Verify response
    assert_eq!(response_body.deleted_nodes.len(), 1);
    assert_eq!(response_body.failed_deletions.len(), 0);
    assert_eq!(response_body.deleted_nodes[0].was_physical, true);
    // Should have deleted 3 descendants (file1.txt, child_dir, file2.txt)
    assert_eq!(response_body.deleted_nodes[0].descendants_deleted.len(), 3);
    assert!(response_body.warnings.len() > 0); // Should warn about descendants

    // Verify all nodes are removed from graph
    test_ctx.with_service(|s| {
        assert!(s.data().open_node(&NodeHandle::Uuid(parent_uuid)).is_err());
        // Check that descendants are also removed (they should be auto-removed when parent is deleted)
    });

    // Verify directory was moved to trash
    assert!(!test_ctx.get_vault_root().join("parent_dir").exists());
}

#[tokio::test]  
async fn test_delete_mixed_batch() {
    let (router, test_ctx) = setup_test_environment("delete_mixed_batch");

    // Create virtual node
    let virtual_node = DataNode::new(&"vault/virtual_node".into(), NodeTypeId::virtual_generic());
    let virtual_uuid = test_ctx.with_service_mut(|s| {
        s.data_mut().insert_nodes(vec![virtual_node.clone()]);
        virtual_node.uuid()
    });

    // Create physical file
    let file_path = test_ctx.get_vault_root().join("physical_file.txt");
    std::fs::write(&file_path, "test content").unwrap();
    let file_node = crate::fs_reader::destructure_single_path(
        test_ctx.get_vault_root(),
        &NodePath::new("vault/physical_file.txt".into())
    ).unwrap();
    let file_uuid = test_ctx.with_service_mut(|s| {
        s.data_mut().insert_nodes(vec![file_node.clone()]);
        file_node.uuid()
    });

    // Delete both nodes in batch
    let delete_payload = DeleteNodesPayload {
        node_ids: vec![virtual_uuid.to_string(), file_uuid.to_string()],
        context_id: None,
    };
    let payload_json = serde_json::to_string(&delete_payload).unwrap();

    let response = execute_delete_request(router, "/api/nodes", payload_json).await;
    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let response_body: DeleteNodesResponse = serde_json::from_slice(&body_bytes).unwrap();

    // Verify response
    assert_eq!(response_body.deleted_nodes.len(), 2);
    assert_eq!(response_body.failed_deletions.len(), 0);

    // Find virtual and physical nodes in response
    let virtual_deleted = response_body.deleted_nodes.iter().find(|n| n.node_id == virtual_uuid.to_string()).unwrap();
    let file_deleted = response_body.deleted_nodes.iter().find(|n| n.node_id == file_uuid.to_string()).unwrap();

    assert_eq!(virtual_deleted.was_physical, false);
    assert_eq!(file_deleted.was_physical, true);

    // Verify both nodes are removed from graph
    test_ctx.with_service(|s| {
        assert!(s.data().open_node(&NodeHandle::Uuid(virtual_uuid)).is_err());
        assert!(s.data().open_node(&NodeHandle::Uuid(file_uuid)).is_err());
    });

    // Verify physical file was moved to trash
    assert!(!file_path.exists());
}

#[tokio::test]
async fn test_delete_partial_failure() {
    let (router, test_ctx) = setup_test_environment("delete_partial_failure");

    // Create one valid node
    let valid_node = DataNode::new(&"vault/valid_node".into(), NodeTypeId::virtual_generic());
    let valid_uuid = test_ctx.with_service_mut(|s| {
        s.data_mut().insert_nodes(vec![valid_node.clone()]);
        valid_node.uuid()
    });

    // Try to delete valid node + non-existent node
    let delete_payload = DeleteNodesPayload {
        node_ids: vec![
            valid_uuid.to_string(),
            "99999999-9999-9999-9999-999999999999".to_string() // Actually non-existent UUID
        ],
        context_id: None,
    };
    let payload_json = serde_json::to_string(&delete_payload).unwrap();

    let response = execute_delete_request(router, "/api/nodes", payload_json).await;
    assert_eq!(response.status(), StatusCode::OK); // Should still return OK for partial success

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let response_body: DeleteNodesResponse = serde_json::from_slice(&body_bytes).unwrap();

    // Verify response
    assert_eq!(response_body.deleted_nodes.len(), 1); // Only valid node deleted
    assert_eq!(response_body.failed_deletions.len(), 1); // One failure
    assert_eq!(response_body.deleted_nodes[0].node_id, valid_uuid.to_string());
    assert_eq!(response_body.failed_deletions[0].node_id, "99999999-9999-9999-9999-999999999999");

    // Verify valid node is removed from graph
    test_ctx.with_service(|s| {
        assert!(s.data().open_node(&NodeHandle::Uuid(valid_uuid)).is_err());
    });
}

#[tokio::test]
async fn test_delete_with_edges() {
    let (router, test_ctx) = setup_test_environment("delete_with_edges");

    // Create two nodes and connect them
    let node1 = DataNode::new(&"vault/node1".into(), NodeTypeId::virtual_generic());
    let node2 = DataNode::new(&"vault/node2".into(), NodeTypeId::virtual_generic());
    
    let (node1_uuid, node2_uuid) = test_ctx.with_service_mut(|s| {
        s.data_mut().insert_nodes(vec![node1.clone(), node2.clone()]);
        
        // Create edge between nodes
        use crate::server::edge_endpoints::CreateEdgePayload;
        let edge_payload = vec![CreateEdgePayload {
            id: uuid::Uuid::new_v4().to_string(),
            source: node1.uuid().to_string(),
            target: node2.uuid().to_string(),
            attributes: std::collections::HashMap::new(),
            source_path: "vault/node1".to_string(),
            target_path: "vault/node2".to_string(),
        }];
        s.create_edges(edge_payload).unwrap();
        
        (node1.uuid(), node2.uuid())
    });

    // Delete node1 (which has an edge)
    let delete_payload = DeleteNodesPayload {
        node_ids: vec![node1_uuid.to_string()],
        context_id: None,
    };
    let payload_json = serde_json::to_string(&delete_payload).unwrap();

    let response = execute_delete_request(router, "/api/nodes", payload_json).await;
    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let response_body: DeleteNodesResponse = serde_json::from_slice(&body_bytes).unwrap();

    // Verify response
    assert_eq!(response_body.deleted_nodes.len(), 1);
    assert_eq!(response_body.failed_deletions.len(), 0);
    // Should have captured the edge in snapshots
    assert!(response_body.deleted_nodes[0].edge_snapshots.len() > 0);

    // Verify node1 is removed but node2 remains
    test_ctx.with_service(|s| {
        assert!(s.data().open_node(&NodeHandle::Uuid(node1_uuid)).is_err());
        assert!(s.data().open_node(&NodeHandle::Uuid(node2_uuid)).is_ok());
        
        // Verify edge is also removed (node2 should have only the vault connection, not the node1 connection)
        let connections = s.data().open_node_connections(&NodePath::new("vault/node2".into()));
        assert_eq!(connections.len(), 1);
        // Verify the remaining connection is to vault
        assert_eq!(connections[0].0.path().alias(), "/vault");
    });
}

#[tokio::test]
async fn test_trash_metadata_persistence() {
    let (router, test_ctx) = setup_test_environment("trash_metadata");

    // Create a node to delete
    let test_node = DataNode::new(&"vault/test_node".into(), NodeTypeId::virtual_generic());
    let node_uuid = test_ctx.with_service_mut(|s| {
        s.data_mut().insert_nodes(vec![test_node.clone()]);
        test_node.uuid()
    });

    // Delete the node
    let delete_payload = DeleteNodesPayload {
        node_ids: vec![node_uuid.to_string()],
        context_id: None,
    };
    let payload_json = serde_json::to_string(&delete_payload).unwrap();

    let response = execute_delete_request(router, "/api/nodes", payload_json).await;
    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let response_body: DeleteNodesResponse = serde_json::from_slice(&body_bytes).unwrap();

    // Verify trash log file was created
    let trash_log_path = test_ctx.with_service(|service| {
        service.storage_path().join("trash").join("trash_log.ron")
    });
    assert!(trash_log_path.exists());

    // Verify trash log content
    let log_content = std::fs::read_to_string(&trash_log_path).unwrap();
    assert!(log_content.contains(&response_body.operation_id));
    assert!(log_content.contains(&node_uuid.to_string()));
}
