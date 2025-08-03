use super::*;
use super::test_helpers::*;

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
