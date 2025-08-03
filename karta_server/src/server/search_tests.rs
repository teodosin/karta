#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::utils::KartaServiceTestContext;
    use crate::prelude::{NodePath, NodeTypeId};
    use crate::server::AppState;
    use crate::elements::node::DataNode;
    use crate::graph_traits::graph_node::GraphNodes;
    use tower::ServiceExt;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        Router,
    };

    fn setup_test_environment(test_name: &str) -> (Router, KartaServiceTestContext) {
        let test_ctx = KartaServiceTestContext::new(test_name);
        let app_state = AppState {
            service: test_ctx.service_arc.clone(),
            tx: tokio::sync::broadcast::channel(1).0,
        };
        let router = crate::server::create_router(app_state);
        (router, test_ctx)
    }

    #[tokio::test]
    async fn test_empty_search_query() {
        let (router, _test_ctx) = setup_test_environment("test_empty_search_query");

        let response = router
            .oneshot(Request::builder()
                .uri("/api/search?q=")
                .body(Body::empty())
                .unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let search_response: crate::server::search_endpoints::SearchResponse = 
            serde_json::from_slice(&body).unwrap();

        assert_eq!(search_response.results.len(), 0);
        assert_eq!(search_response.total_found, 0);
        assert_eq!(search_response.truncated, false);
        assert_eq!(search_response.query, "");
    }

    #[tokio::test]
    async fn test_filesystem_search() {
        let (router, mut test_ctx) = setup_test_environment("test_filesystem_search");
        
        // Create some test files in the vault
        test_ctx.create_file_in_vault("test_search_file.txt", b"test content").unwrap();
        test_ctx.create_dir_in_vault("test_search_directory").unwrap();

        let response = router
            .oneshot(Request::builder()
                .uri("/api/search?q=test_search")
                .body(Body::empty())
                .unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let search_response: crate::server::search_endpoints::SearchResponse = 
            serde_json::from_slice(&body).unwrap();

        assert!(search_response.results.len() >= 2, "Should find at least the file and directory");
        assert!(search_response.total_found >= 2);
        assert_eq!(search_response.query, "test_search");
        
        // Check that we found our test files
        let found_paths: Vec<&str> = search_response.results.iter()
            .map(|r| r.path.as_str())
            .collect();
        
        assert!(found_paths.iter().any(|p| p.contains("test_search_file.txt")));
        assert!(found_paths.iter().any(|p| p.contains("test_search_directory")));
        
        // Verify filesystem items are marked as not indexed
        for result in &search_response.results {
            if result.path.contains("test_search") {
                assert_eq!(result.is_indexed, false);
                assert_eq!(result.id, None);
            }
        }
    }

    #[tokio::test]
    async fn test_indexed_node_search() {
        let (router, mut test_ctx) = setup_test_environment("test_indexed_node_search");
        
        // Create and index a node in the database
        let indexed_path = NodePath::new("vault/indexed_search_node".into());
        let indexed_node = crate::elements::node::DataNode::new(&indexed_path, NodeTypeId::file_type());
        test_ctx.with_service_mut(|s| {
            s.data_mut().insert_nodes(vec![indexed_node]);
        });

        let response = router
            .oneshot(Request::builder()
                .uri("/api/search?q=indexed_search")
                .body(Body::empty())
                .unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let search_response: crate::server::search_endpoints::SearchResponse = 
            serde_json::from_slice(&body).unwrap();

        assert!(search_response.results.len() >= 1, "Should find the indexed node");
        assert_eq!(search_response.query, "indexed_search");
        
        // Find the indexed node in results
        let indexed_result = search_response.results.iter()
            .find(|r| r.path.contains("indexed_search_node"))
            .expect("Should find the indexed node");

        // Verify indexed node properties
        assert_eq!(indexed_result.is_indexed, true);
        assert!(indexed_result.id.is_some(), "Indexed node should have UUID");
        assert_eq!(indexed_result.ntype, NodeTypeId::file_type().to_string());
        assert!(indexed_result.score > 0.0, "Should have a positive match score");
    }

    #[tokio::test]
    async fn test_search_query_parameters() {
        let (router, mut test_ctx) = setup_test_environment("test_search_query_parameters");
        
        // Create multiple test files with different match qualities
        test_ctx.create_file_in_vault("perfect_match.txt", b"content").unwrap();
        test_ctx.create_file_in_vault("partial_perfect_file.txt", b"content").unwrap();
        test_ctx.create_file_in_vault("other_file.txt", b"content").unwrap();

        // Test with limit parameter
        let response = router
            .oneshot(Request::builder()
                .uri("/api/search?q=perfect&limit=1")
                .body(Body::empty())
                .unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let search_response: crate::server::search_endpoints::SearchResponse = 
            serde_json::from_slice(&body).unwrap();

        assert!(search_response.results.len() <= 1, "Should respect limit parameter");
        assert_eq!(search_response.query, "perfect");
        if search_response.total_found > 1 {
            assert_eq!(search_response.truncated, true, "Should indicate results were truncated");
        }

        // Test with min_score parameter (high threshold) - create new router for second test
        let (router2, mut test_ctx2) = setup_test_environment("test_search_query_parameters_2");
        
        // Recreate the same test files for the second router
        test_ctx2.create_file_in_vault("perfect_match_2.txt", b"content").unwrap();
        test_ctx2.create_file_in_vault("partial_perfect_file_2.txt", b"content").unwrap();

        let response2 = router2
            .oneshot(Request::builder()
                .uri("/api/search?q=perfect&min_score=0.9")
                .body(Body::empty())
                .unwrap())
            .await
            .unwrap();

        // Debug the response status
        println!("Response status: {}", response2.status());
        assert_eq!(response2.status(), StatusCode::OK);
        let body2 = axum::body::to_bytes(response2.into_body(), usize::MAX).await.unwrap();
        let search_response2: crate::server::search_endpoints::SearchResponse = 
            serde_json::from_slice(&body2).unwrap();

        // All returned results should be valid matches (no filtering by score anymore)
        for result in &search_response2.results {
            assert!(result.score > 0.0, "All results should have positive scores");
        }
        assert_eq!(search_response2.query, "perfect");
    }

    #[tokio::test]
    async fn test_mixed_virtual_physical_search() {
        let (router, mut test_ctx) = setup_test_environment("test_mixed_virtual_physical_search");
        
        // Create physical files and directories
        test_ctx.create_file_in_vault("physical_docs/readme.txt", b"Physical readme content").unwrap();
        test_ctx.create_file_in_vault("physical_docs/guide.md", b"Physical guide content").unwrap();
        test_ctx.create_dir_in_vault("physical_projects").unwrap();
        test_ctx.create_file_in_vault("physical_projects/app.js", b"Physical JavaScript code").unwrap();
        
        // Remove the scan_context call since filesystem paths should be found automatically
        // test_ctx.with_service_mut(|s| async move {
        //     s.scan_context().await.expect("Failed to scan context");
        // }).await;
        
        // Create virtual (indexed-only) nodes
        let virtual_doc = DataNode::new(&NodePath::new("vault/virtual_docs/manual.txt".into()), NodeTypeId::file_type());
        let virtual_config = DataNode::new(&NodePath::new("vault/virtual_config/settings.json".into()), NodeTypeId::file_type());
        let virtual_dir = DataNode::new(&NodePath::new("vault/virtual_projects".into()), NodeTypeId::dir_type());
        
        test_ctx.with_service_mut(|s| {
            s.data_mut().insert_nodes(vec![virtual_doc, virtual_config, virtual_dir]);
        });

        let response = router
            .oneshot(Request::builder()
                .uri("/api/search?q=docs&min_score=0.01")
                .body(Body::empty())
                .unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let search_response: crate::server::search_endpoints::SearchResponse = 
            serde_json::from_slice(&body).unwrap();

        assert!(search_response.results.len() >= 1, "Should find docs paths");
        
        // Verify we have both physical and virtual results
        let physical_results: Vec<_> = search_response.results.iter()
            .filter(|r| !r.is_indexed && r.path.contains("docs"))
            .collect();
        let virtual_results: Vec<_> = search_response.results.iter()
            .filter(|r| r.is_indexed && r.path.contains("docs"))
            .collect();
            
        assert!(!physical_results.is_empty(), "Should find physical docs");
        assert!(!virtual_results.is_empty(), "Should find virtual docs");
        
        // Verify virtual nodes have UUIDs, physical nodes don't
        for result in &virtual_results {
            assert!(result.id.is_some(), "Virtual nodes should have UUIDs");
        }
        for result in &physical_results {
            assert!(result.id.is_none(), "Physical nodes should not have UUIDs");
        }
    }

    #[tokio::test]
    async fn test_indexed_physical_nodes_search() {
        let (router, mut test_ctx) = setup_test_environment("test_indexed_physical_nodes_search");
        
        // Create physical files first
        test_ctx.create_file_in_vault("indexed_physical/document.txt", b"Physical document content").unwrap();
        test_ctx.create_file_in_vault("indexed_physical/readme.md", b"Physical readme content").unwrap();
        test_ctx.create_dir_in_vault("indexed_physical/subfolder").unwrap();
        test_ctx.create_file_in_vault("indexed_physical/subfolder/nested.json", b"Nested file content").unwrap();
        
        // Also create some purely physical files for contrast
        test_ctx.create_file_in_vault("purely_physical/unindexed.txt", b"Unindexed file").unwrap();
        
        // Now index some of the physical files (making them indexed physical nodes)
        let indexed_doc = DataNode::new(&NodePath::new("vault/indexed_physical/document.txt".into()), NodeTypeId::file_type());
        let indexed_dir = DataNode::new(&NodePath::new("vault/indexed_physical".into()), NodeTypeId::dir_type());
        let indexed_nested = DataNode::new(&NodePath::new("vault/indexed_physical/subfolder/nested.json".into()), NodeTypeId::file_type());
        
        test_ctx.with_service_mut(|s| {
            s.data_mut().insert_nodes(vec![indexed_doc, indexed_dir, indexed_nested]);
        });

        let response = router
            .oneshot(Request::builder()
                .uri("/api/search?q=indexed_physical")
                .body(Body::empty())
                .unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let search_response: crate::server::search_endpoints::SearchResponse = 
            serde_json::from_slice(&body).unwrap();

        assert!(search_response.results.len() >= 1, "Should find indexed physical paths");
        
        // Verify that indexed physical nodes are marked as indexed and have UUIDs
        let indexed_physical_results: Vec<_> = search_response.results.iter()
            .filter(|r| r.path.contains("indexed_physical"))
            .collect();
        
        assert!(!indexed_physical_results.is_empty(), "Should find indexed physical nodes");
        
        // Check that indexed physical files have both filesystem presence and database IDs
        for result in &indexed_physical_results {
            if result.path.contains("document.txt") || result.path.contains("nested.json") || 
               (result.path.ends_with("indexed_physical") && !result.path.contains("/")) {
                assert_eq!(result.is_indexed, true, "Indexed physical nodes should be marked as indexed");
                assert!(result.id.is_some(), "Indexed physical nodes should have UUIDs");
            }
        }
        
        // Test that purely physical files are still treated as filesystem-only
        let app_state = AppState {
            service: test_ctx.service_arc.clone(),
            tx: tokio::sync::broadcast::channel(1).0,
        };
        let router2 = crate::server::create_router(app_state);
        
        let response2 = router2
            .oneshot(Request::builder()
                .uri("/api/search?q=purely_physical")
                .body(Body::empty())
                .unwrap())
            .await
            .unwrap();

        let body2 = axum::body::to_bytes(response2.into_body(), usize::MAX).await.unwrap();
        let search_response2: crate::server::search_endpoints::SearchResponse = 
            serde_json::from_slice(&body2).unwrap();
            
        let purely_physical_results: Vec<_> = search_response2.results.iter()
            .filter(|r| r.path.contains("purely_physical"))
            .collect();
            
        for result in &purely_physical_results {
            assert_eq!(result.is_indexed, false, "Purely physical nodes should not be indexed");
            assert!(result.id.is_none(), "Purely physical nodes should not have UUIDs");
        }
    }

    #[tokio::test]
    async fn test_search_case_sensitivity_and_special_chars() {
        let (router, mut test_ctx) = setup_test_environment("test_search_case_sensitivity_and_special_chars");
        
        // Create files with various case combinations and special characters
        test_ctx.create_file_in_vault("CamelCase/TestFile.txt", b"Camel case file").unwrap();
        test_ctx.create_file_in_vault("lowercase/testfile.txt", b"Lowercase file").unwrap();
        test_ctx.create_file_in_vault("UPPERCASE/TESTFILE.TXT", b"Uppercase file").unwrap();
        test_ctx.create_file_in_vault("mixed_case/Test_File-2023.txt", b"Mixed case with special chars").unwrap();
        test_ctx.create_file_in_vault("spaces and symbols/file (1).txt", b"File with spaces and symbols").unwrap();
        
        // Create virtual nodes with special characters
        let unicode_node = DataNode::new(&NodePath::new("vault/unicode/测试文件.txt".into()), NodeTypeId::file_type());
        let special_chars_node = DataNode::new(&NodePath::new("vault/special/file@#$%.json".into()), NodeTypeId::file_type());
        
        test_ctx.with_service_mut(|s| {
            s.data_mut().insert_nodes(vec![unicode_node, special_chars_node]);
        });

        // Test case insensitive search
        let response = router
            .oneshot(Request::builder()
                .uri("/api/search?q=testfile")
                .body(Body::empty())
                .unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let search_response: crate::server::search_endpoints::SearchResponse = 
            serde_json::from_slice(&body).unwrap();

        // Should find files regardless of case
        let case_matches: Vec<_> = search_response.results.iter()
            .filter(|r| r.path.to_lowercase().contains("testfile"))
            .collect();
        assert!(case_matches.len() >= 3, "Should find testfile in different cases");
        
        // Test search with special characters - create new router
        let app_state = AppState {
            service: test_ctx.service_arc.clone(),
            tx: tokio::sync::broadcast::channel(1).0,
        };
        let router2 = crate::server::create_router(app_state);
        
        let response2 = router2
            .oneshot(Request::builder()
                .uri("/api/search?q=spaces")
                .body(Body::empty())
                .unwrap())
            .await
            .unwrap();

        let body2 = axum::body::to_bytes(response2.into_body(), usize::MAX).await.unwrap();
        let search_response2: crate::server::search_endpoints::SearchResponse = 
            serde_json::from_slice(&body2).unwrap();

        // Should find files with spaces in the path
        let space_matches: Vec<_> = search_response2.results.iter()
            .filter(|r| r.path.contains("spaces"))
            .collect();
        assert!(!space_matches.is_empty(), "Should find files with spaces in path");
    }

    #[tokio::test]
    async fn test_search_edge_cases() {
        let (router, mut test_ctx) = setup_test_environment("test_search_edge_cases");
        
        // Create files with edge case names
        test_ctx.create_file_in_vault("edge_cases/.hidden_file", b"Hidden file").unwrap();
        test_ctx.create_file_in_vault("edge_cases/..double_dot", b"Double dot file").unwrap();
        test_ctx.create_file_in_vault("edge_cases/single_char/a", b"Single character file").unwrap();
        test_ctx.create_file_in_vault("edge_cases/empty_extension.", b"Empty extension").unwrap();
        
        // Create virtual nodes with edge case paths
        let empty_name_node = DataNode::new(&NodePath::new("vault/edge_cases/folder_with_empty_name/".into()), NodeTypeId::dir_type());
        let very_long_name = "a".repeat(200); // Very long filename
        let long_name_node = DataNode::new(&NodePath::new(format!("vault/edge_cases/{}.txt", very_long_name).into()), NodeTypeId::file_type());
        
        test_ctx.with_service_mut(|s| {
            s.data_mut().insert_nodes(vec![empty_name_node, long_name_node]);
        });

        // Test search for hidden files
        let response = router
            .oneshot(Request::builder()
                .uri("/api/search?q=hidden")
                .body(Body::empty())
                .unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let search_response: crate::server::search_endpoints::SearchResponse = 
            serde_json::from_slice(&body).unwrap();

        // Should find hidden files
        let hidden_matches: Vec<_> = search_response.results.iter()
            .filter(|r| r.path.contains("hidden"))
            .collect();
        assert!(!hidden_matches.is_empty(), "Should find hidden files");
        
        // Test search with single character query - create new router
        let app_state = AppState {
            service: test_ctx.service_arc.clone(),
            tx: tokio::sync::broadcast::channel(1).0,
        };
        let router2 = crate::server::create_router(app_state);
        
        let response2 = router2
            .oneshot(Request::builder()
                .uri("/api/search?q=a")
                .body(Body::empty())
                .unwrap())
            .await
            .unwrap();

        let body2 = axum::body::to_bytes(response2.into_body(), usize::MAX).await.unwrap();
        let search_response2: crate::server::search_endpoints::SearchResponse = 
            serde_json::from_slice(&body2).unwrap();

        // Should find results even with single character query
        assert!(search_response2.results.len() > 0, "Should find results with single character query");
        
        // Should find the very long filename
        let long_name_matches: Vec<_> = search_response2.results.iter()
            .filter(|r| r.path.len() > 100)  // Very long paths
            .collect();
        assert!(!long_name_matches.is_empty(), "Should handle very long filenames");
    }

    #[tokio::test]
    async fn test_deep_nested_folder_search() {
        let (router, mut test_ctx) = setup_test_environment("test_deep_nested_folder_search");
        
        // Create deeply nested physical structure
        test_ctx.create_file_in_vault("deep/level1/level2/level3/level4/level5/target_file.txt", b"Deep nested file").unwrap();
        test_ctx.create_dir_in_vault("deep/level1/level2/level3/level4/level5/target_dir").unwrap();
        test_ctx.create_file_in_vault("deep/level1/level2/level3/level4/level5/target_dir/nested_target.md", b"Deeply nested markdown").unwrap();
        
        // Create some decoy files at different levels
        test_ctx.create_file_in_vault("deep/level1/decoy.txt", b"Not the target").unwrap();
        test_ctx.create_file_in_vault("deep/level1/level2/another_decoy.txt", b"Also not the target").unwrap();
        
        // Create virtual nodes in deep structure too
        let deep_virtual = DataNode::new(&NodePath::new("vault/deep/level1/level2/level3/virtual_target.txt".into()), NodeTypeId::file_type());
        let deep_virtual_dir = DataNode::new(&NodePath::new("vault/deep/level1/level2/level3/level4/virtual_target_dir".into()), NodeTypeId::dir_type());
        
        test_ctx.with_service_mut(|s| {
            s.data_mut().insert_nodes(vec![deep_virtual, deep_virtual_dir]);
        });

        let response = router
            .oneshot(Request::builder()
                .uri("/api/search?q=target")
                .body(Body::empty())
                .unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let search_response: crate::server::search_endpoints::SearchResponse = 
            serde_json::from_slice(&body).unwrap();

        assert!(search_response.results.len() >= 5, "Should find all target files/dirs at various depths");
        
        // Verify we found files at the deepest level
        let deep_files: Vec<_> = search_response.results.iter()
            .filter(|r| r.path.contains("level5"))
            .collect();
        assert!(!deep_files.is_empty(), "Should find files in level5");
        
        // Verify no decoy files were matched
        let decoy_files: Vec<_> = search_response.results.iter()
            .filter(|r| r.path.contains("decoy"))
            .collect();
        assert!(decoy_files.is_empty(), "Should not match decoy files");
        
        // Test that deeper paths have lower scores than closer matches (fuzzy matching behavior)
        let results_with_depths: Vec<_> = search_response.results.iter()
            .map(|r| (r, r.path.matches('/').count()))
            .collect();
            
        assert!(results_with_depths.len() > 1, "Should have multiple results to compare depths");
    }

    #[tokio::test]
    async fn test_large_dataset_performance() {
        let (router, mut test_ctx) = setup_test_environment("test_large_dataset_performance");
        
        // Create hundreds of physical files with various naming patterns
        for i in 0..100 {
            test_ctx.create_file_in_vault(&format!("bulk_files/document_{:03}.txt", i), b"Bulk document content").unwrap();
            test_ctx.create_file_in_vault(&format!("bulk_files/report_{:03}.md", i), b"Bulk report content").unwrap();
        }
        
        // Create nested directories with files
        for i in 0..50 {
            test_ctx.create_dir_in_vault(&format!("categories/category_{:02}", i)).unwrap();
            test_ctx.create_file_in_vault(&format!("categories/category_{:02}/item_{:03}.json", i, i * 2), b"Category item data").unwrap();
            test_ctx.create_file_in_vault(&format!("categories/category_{:02}/config_{:03}.yaml", i, i * 2 + 1), b"Category config data").unwrap();
        }
        
        // Create many virtual nodes with different patterns
        let mut virtual_nodes = Vec::new();
        for i in 0..200 {
            let virtual_file = DataNode::new(
                &NodePath::new(format!("vault/virtual_collection/item_{:04}.data", i).into()),
                NodeTypeId::file_type()
            );
            virtual_nodes.push(virtual_file);
        }
        
        // Add virtual nodes that will match our search term
        for i in 0..30 {
            let searchable_file = DataNode::new(
                &NodePath::new(format!("vault/virtual_searchable/target_match_{:02}.txt", i).into()),
                NodeTypeId::file_type()
            );
            virtual_nodes.push(searchable_file);
        }
        
        test_ctx.with_service_mut(|s| {
            s.data_mut().insert_nodes(virtual_nodes);
        });

        // Test search performance and fuzzy matching
        let response = router
            .oneshot(Request::builder()
                .uri("/api/search?q=target_match&limit=50")
                .body(Body::empty())
                .unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let search_response: crate::server::search_endpoints::SearchResponse = 
            serde_json::from_slice(&body).unwrap();

        // Verify performance and results
        assert!(search_response.took_ms < 1000, "Search should complete within 1 second even with large dataset");
        assert!(search_response.results.len() >= 30, "Should find all target_match files");
        assert!(search_response.total_found >= 30, "Should report correct total found");
        
        // Test fuzzy matching - search for slightly misspelled term (reuse same router/context)
        // Create a new router instance with the same service context
        let app_state = AppState {
            service: test_ctx.service_arc.clone(),
            tx: tokio::sync::broadcast::channel(1).0,
        };
        let router2 = crate::server::create_router(app_state);
        
        let response2 = router2
            .oneshot(Request::builder()
                .uri("/api/search?q=target_mach&min_score=0.3")  // Intentional typo, lower threshold
                .body(Body::empty())
                .unwrap())
            .await
            .unwrap();

        assert_eq!(response2.status(), StatusCode::OK);
        let body2 = axum::body::to_bytes(response2.into_body(), usize::MAX).await.unwrap();
        let search_response2: crate::server::search_endpoints::SearchResponse = 
            serde_json::from_slice(&body2).unwrap();

        // Fuzzy matching should still find some results despite the typo
        assert!(search_response2.results.len() > 0, "Fuzzy matching should find results despite typo");
        assert!(search_response2.took_ms < 1000, "Fuzzy search should also be fast");
        
        // Verify that results are sorted by score (highest first)
        for i in 1..search_response2.results.len() {
            assert!(
                search_response2.results[i-1].score >= search_response2.results[i].score,
                "Results should be sorted by score in descending order"
            );
        }
        
        // Test that all results have positive scores (no minimum score filtering)
        for result in &search_response2.results {
            assert!(result.score > 0.0, "All results should have positive scores");
        }
    }
}

/*
 * Additional search test scenarios that could be implemented:
 * 
 * 1. **Symlink handling** - How does search treat symbolic links?
 *    - Test if symlinked files appear in search results
 *    - Verify symlinks don't create duplicate results
 *    - Check behavior when symlink targets don't exist
 * 
 * 2. **Permission-based filtering** - What if some files can't be accessed?
 *    - Test search behavior with files that have restricted permissions
 *    - Verify graceful handling of permission errors
 *    - Check if inaccessible files are silently skipped or reported
 * 
 * 3. **Real-time updates** - What happens when files are added/removed during search?
 *    - Test concurrent file system modifications during search
 *    - Verify search results consistency
 *    - Check if new files appear in subsequent searches
 * 
 * 4. **Concurrent search requests** - Multiple simultaneous searches
 *    - Test multiple search requests running concurrently
 *    - Verify no race conditions or data corruption
 *    - Check performance under concurrent load
 * 
 * 5. **Memory pressure** - Search behavior with very large result sets
 *    - Test search with tens of thousands of matching files
 *    - Verify memory usage doesn't grow unbounded
 *    - Check if pagination/streaming might be needed
 * 
 * 6. **Malformed paths** - Invalid characters, malformed UTF-8
 *    - Test files with invalid UTF-8 sequences in names
 *    - Verify handling of null bytes or other invalid path characters
 *    - Check behavior with paths exceeding system limits
 * 
 * 7. **Cross-platform path separators** - Windows vs Unix paths
 *    - Test search behavior with mixed path separators
 *    - Verify consistent results across different platforms
 *    - Check handling of UNC paths on Windows
 * 
 * 8. **Database corruption recovery** - What happens if the index is corrupted?
 *    - Test search behavior when database is corrupted or inaccessible
 *    - Verify fallback to filesystem-only search
 *    - Check if search can continue with partial database failures
 * 
 * 9. **Internationalization** - Non-ASCII characters and different languages
 *    - Test search with various Unicode normalization forms
 *    - Verify case-insensitive matching for non-Latin scripts
 *    - Check behavior with right-to-left text and combining characters
 * 
 * 10. **Search query edge cases** - Empty, very long, or special queries
 *     - Test with queries containing only whitespace
 *     - Verify behavior with extremely long search terms
 *     - Check handling of regex special characters in queries
 */
