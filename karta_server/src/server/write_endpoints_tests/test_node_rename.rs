use super::*;
use super::test_helpers::*;

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
