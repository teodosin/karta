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

        // Debug print for troubleshooting
        println!("Found indexed result: {:?}", indexed_result);

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

        // All returned results should have high scores
        for result in &search_response2.results {
            assert!(result.score >= 0.9, "All results should meet minimum score threshold");
        }
        assert_eq!(search_response2.query, "perfect");
    }
}
