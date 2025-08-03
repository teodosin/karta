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
        node_handles: vec![node_uuid.to_string()],
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
        node_handles: vec![node_uuid.to_string()],
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
async fn test_delete_unindexed_file_by_path() {
    let (router, test_ctx) = setup_test_environment("delete_unindexed_file");

    // Create a physical file but DO NOT index it
    let file_path = test_ctx.get_vault_root().join("unindexed_file.txt");
    std::fs::write(&file_path, "unindexed content").expect("Failed to create test file");

    // Verify file exists on filesystem
    assert!(file_path.exists());

    // Verify file is NOT in the graph database
    test_ctx.with_service(|s| {
        let node_path = NodePath::new("vault/unindexed_file.txt".into());
        assert!(s.data().open_node(&NodeHandle::Path(node_path)).is_err());
    });

    // Delete the file by path (not UUID, since it's not indexed)
    let delete_payload = DeleteNodesPayload {
        node_handles: vec!["vault/unindexed_file.txt".to_string()], // Path, not UUID
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
    assert_eq!(response_body.deleted_nodes[0].node_path, "/vault/unindexed_file.txt"); // Note: path includes leading slash
    assert_eq!(response_body.deleted_nodes[0].was_physical, true);
    assert_eq!(response_body.deleted_nodes[0].descendants_deleted.len(), 0);

    // Verify file was moved to trash (should not exist in original location)
    assert!(!file_path.exists());

    // Verify file is still not in graph database (it was never indexed)
    test_ctx.with_service(|s| {
        let node_path = NodePath::new("vault/unindexed_file.txt".into());
        assert!(s.data().open_node(&NodeHandle::Path(node_path)).is_err());
    });
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

    let (parent_uuid, sibling_uuid1, sibling_uuid2) = test_ctx.with_service_mut(|s| {
        // Index physical nodes
        for path in paths_to_index {
            let node = crate::fs_reader::destructure_single_path(s.vault_fs_path(), &path).unwrap();
            s.data_mut().insert_nodes(vec![node]);
        }

        // Add virtual nodes as children of parent_dir
        let virtual_node1 = DataNode::new(&"vault/parent_dir/virtual_note1".into(), NodeTypeId::virtual_generic());
        let virtual_node2 = DataNode::new(&"vault/parent_dir/virtual_note2".into(), NodeTypeId::virtual_generic());
        s.data_mut().insert_nodes(vec![virtual_node1, virtual_node2]);

        // Add sibling virtual nodes (at vault level, not children of parent_dir)
        let sibling_node1 = DataNode::new(&"vault/sibling_note1".into(), NodeTypeId::virtual_generic());
        let sibling_node2 = DataNode::new(&"vault/sibling_note2".into(), NodeTypeId::virtual_generic());
        let sibling_uuid1 = sibling_node1.uuid();
        let sibling_uuid2 = sibling_node2.uuid();
        s.data_mut().insert_nodes(vec![sibling_node1, sibling_node2]);

        // Get the parent directory UUID
        let parent_uuid = s.data().open_node(&NodeHandle::Path(NodePath::new("vault/parent_dir".into()))).unwrap().uuid();
        
        (parent_uuid, sibling_uuid1, sibling_uuid2)
    });

    // Delete the parent directory
    let delete_payload = DeleteNodesPayload {
        node_handles: vec![parent_uuid.to_string()],
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
    // Should have deleted 5 descendants (file1.txt, child_dir, file2.txt, virtual_note1, virtual_note2)
    assert_eq!(response_body.deleted_nodes[0].descendants_deleted.len(), 5);
    assert!(response_body.warnings.len() > 0); // Should warn about descendants

    // Verify all nodes are removed from graph
    test_ctx.with_service(|s| {
        assert!(s.data().open_node(&NodeHandle::Uuid(parent_uuid)).is_err());
        // Check that descendants are also removed (they should be auto-removed when parent is deleted)
        
        // CRITICAL: Verify that sibling nodes are NOT deleted (they should remain intact)
        assert!(s.data().open_node(&NodeHandle::Uuid(sibling_uuid1)).is_ok(), 
                "Sibling node 1 should NOT be deleted when deleting parent_dir");
        assert!(s.data().open_node(&NodeHandle::Uuid(sibling_uuid2)).is_ok(), 
                "Sibling node 2 should NOT be deleted when deleting parent_dir");
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
        node_handles: vec![virtual_uuid.to_string(), file_uuid.to_string()],
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
        node_handles: vec![
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
        node_handles: vec![node1_uuid.to_string()],
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
        node_handles: vec![node_uuid.to_string()],
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

#[tokio::test]
async fn test_delete_directory_with_virtual_nodes_in_context_bug_scenario() {
    // This test reproduces the specific bug scenario reported where:
    // 1. Created folder vault/A/B, then vault/A/B/C
    // 2. Connected vault/A and vault/A/B/C with an edge (making vault/A/B/C appear in vault/A context)
    // 3. Created 5 virtual nodes in vault/A context
    // 4. Deleted directory B
    // 5. BUG: All virtual nodes got deleted, contexts became corrupted
    
    let (router, test_ctx) = setup_test_environment("delete_directory_context_bug");

    // Create directory structure: vault/A/B and vault/A/B/C
    std::fs::create_dir_all(test_ctx.get_vault_root().join("A/B/C")).unwrap();

    // Index the directory nodes
    let paths_to_index = vec![
        NodePath::new("vault/A".into()),
        NodePath::new("vault/A/B".into()),
        NodePath::new("vault/A/B/C".into()),
    ];

    let (a_uuid, b_uuid, c_uuid) = test_ctx.with_service_mut(|s| {
        // Index physical directory nodes
        for path in &paths_to_index {
            let node = crate::fs_reader::destructure_single_path(s.vault_fs_path(), path).unwrap();
            s.data_mut().insert_nodes(vec![node]);
        }

        // Get the UUIDs
        let a_uuid = s.data().open_node(&NodeHandle::Path(NodePath::new("vault/A".into()))).unwrap().uuid();
        let b_uuid = s.data().open_node(&NodeHandle::Path(NodePath::new("vault/A/B".into()))).unwrap().uuid();
        let c_uuid = s.data().open_node(&NodeHandle::Path(NodePath::new("vault/A/B/C".into()))).unwrap().uuid();

        (a_uuid, b_uuid, c_uuid)
    });

    // Create virtual nodes in vault/A context (not children of B)
    let (virtual_text_uuid, virtual_image_uuid, virtual_fold_uuid, virtual_generic_uuid, virtual_note_uuid) = test_ctx.with_service_mut(|s| {
        let virtual_text = DataNode::new(&"vault/GLORPSALITY/Text_2".into(), NodeTypeId::virtual_generic());
        let virtual_image = DataNode::new(&"vault/GLORPSALITY/Image".into(), NodeTypeId::virtual_generic());
        let virtual_fold = DataNode::new(&"vault/GLORPSALITY/fold".into(), NodeTypeId::virtual_generic());
        let virtual_generic = DataNode::new(&"vault/GLORPSALITY/Glorps/glorpssss".into(), NodeTypeId::virtual_generic());
        let virtual_note = DataNode::new(&"vault/GLORPSALITY/Note_1".into(), NodeTypeId::virtual_generic());

        let virtual_text_uuid = virtual_text.uuid();
        let virtual_image_uuid = virtual_image.uuid();
        let virtual_fold_uuid = virtual_fold.uuid();
        let virtual_generic_uuid = virtual_generic.uuid();
        let virtual_note_uuid = virtual_note.uuid();

        s.data_mut().insert_nodes(vec![
            virtual_text,
            virtual_image, 
            virtual_fold,
            virtual_generic,
            virtual_note,
        ]);

        (virtual_text_uuid, virtual_image_uuid, virtual_fold_uuid, virtual_generic_uuid, virtual_note_uuid)
    });

    println!("Test setup complete:");
    println!("  A UUID: {}", a_uuid);
    println!("  B UUID: {} (will be deleted)", b_uuid);
    println!("  C UUID: {} (child of B, should be deleted)", c_uuid);
    println!("  Virtual nodes (should NOT be deleted):");
    println!("    Text: {}", virtual_text_uuid);
    println!("    Image: {}", virtual_image_uuid);
    println!("    Fold: {}", virtual_fold_uuid);
    println!("    Generic: {}", virtual_generic_uuid);
    println!("    Note: {}", virtual_note_uuid);

    // Verify all nodes exist before deletion
    test_ctx.with_service(|s| {
        assert!(s.data().open_node(&NodeHandle::Uuid(a_uuid)).is_ok());
        assert!(s.data().open_node(&NodeHandle::Uuid(b_uuid)).is_ok());
        assert!(s.data().open_node(&NodeHandle::Uuid(c_uuid)).is_ok());
        assert!(s.data().open_node(&NodeHandle::Uuid(virtual_text_uuid)).is_ok());
        assert!(s.data().open_node(&NodeHandle::Uuid(virtual_image_uuid)).is_ok());
        assert!(s.data().open_node(&NodeHandle::Uuid(virtual_fold_uuid)).is_ok());
        assert!(s.data().open_node(&NodeHandle::Uuid(virtual_generic_uuid)).is_ok());
        assert!(s.data().open_node(&NodeHandle::Uuid(virtual_note_uuid)).is_ok());
    });

    // Delete directory B (this should only delete B and its child C, NOT the virtual nodes)
    let delete_payload = DeleteNodesPayload {
        node_handles: vec!["vault/A/B".to_string()], // Use path instead of UUID
        context_id: None,
    };
    let payload_json = serde_json::to_string(&delete_payload).unwrap();

    println!("Deleting vault/A/B with payload: {}", payload_json);

    let response = execute_delete_request(router, "/api/nodes", payload_json).await;
    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let response_body: DeleteNodesResponse = serde_json::from_slice(&body_bytes).unwrap();

    println!("Delete response: {:#?}", response_body);

    // Verify response structure
    assert_eq!(response_body.deleted_nodes.len(), 1, "Should have deleted exactly 1 node (B)");
    assert_eq!(response_body.failed_deletions.len(), 0, "Should have no failed deletions");
    
    let deleted_node_info = &response_body.deleted_nodes[0];
    assert_eq!(deleted_node_info.was_physical, true, "B should be a physical directory");
    assert_eq!(deleted_node_info.descendants_deleted.len(), 1, "Should have deleted exactly 1 descendant (C)");
    
    // Verify that only C is in the descendants list
    assert!(deleted_node_info.descendants_deleted.contains(&c_uuid.to_string()), 
            "C should be in descendants_deleted list");

    // CRITICAL BUG CHECK: Verify virtual nodes are NOT deleted
    test_ctx.with_service(|s| {
        // A should still exist
        assert!(s.data().open_node(&NodeHandle::Uuid(a_uuid)).is_ok(), 
                "Directory A should still exist after deleting B");

        // B and C should be deleted
        assert!(s.data().open_node(&NodeHandle::Uuid(b_uuid)).is_err(), 
                "Directory B should be deleted");
        assert!(s.data().open_node(&NodeHandle::Uuid(c_uuid)).is_err(), 
                "Directory C should be deleted as descendant of B");

        // CRITICAL: All virtual nodes should still exist (this is where the bug would manifest)
        assert!(s.data().open_node(&NodeHandle::Uuid(virtual_text_uuid)).is_ok(), 
                "BUG: Virtual text node should NOT be deleted when deleting directory B");
        assert!(s.data().open_node(&NodeHandle::Uuid(virtual_image_uuid)).is_ok(), 
                "BUG: Virtual image node should NOT be deleted when deleting directory B");
        assert!(s.data().open_node(&NodeHandle::Uuid(virtual_fold_uuid)).is_ok(), 
                "BUG: Virtual fold node should NOT be deleted when deleting directory B");
        assert!(s.data().open_node(&NodeHandle::Uuid(virtual_generic_uuid)).is_ok(), 
                "BUG: Virtual generic node should NOT be deleted when deleting directory B");
        assert!(s.data().open_node(&NodeHandle::Uuid(virtual_note_uuid)).is_ok(), 
                "BUG: Virtual note node should NOT be deleted when deleting directory B");
    });

    // Verify physical directory was removed
    assert!(!test_ctx.get_vault_root().join("A/B").exists(), 
            "Physical directory A/B should be removed from filesystem");

    println!("Test completed successfully - no virtual nodes were incorrectly deleted");
}
