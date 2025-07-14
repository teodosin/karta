use axum::{extract::{Path, State, Query}, http::StatusCode, Json};
use uuid::Uuid;
use serde::Deserialize;

use crate::{
    elements::node_path::NodeHandle,
    graph_traits::graph_node::GraphNodes,
    prelude::{DataNode, NodePath},
};

use super::AppState;

#[derive(Deserialize)]
pub struct PathFilter {
    #[serde(default)]
    only_indexed: bool,
}

pub async fn get_paths(
    State(app_state): State<AppState>,
    Query(filter): Query<PathFilter>,
) -> Result<Json<Vec<String>>, StatusCode> {
    let service = app_state.service.read().unwrap();
    match service.get_paths(filter.only_indexed) {
        Ok(paths) => Ok(Json(paths)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn get_node_by_uuid(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<DataNode>, StatusCode> {
    let service = app_state.service.read().unwrap();
    match service.data().open_node(&NodeHandle::Uuid(id)) {
        Ok(node) => Ok(Json(node)),
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn root() -> &'static str {
    "Welcome to Karta Server"
}

use serde_json::json;

pub async fn get_vault_info(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let service = state.service.read().unwrap();
    let vault_path = service.vault_fs_path();
    let vault_name = vault_path.file_name().unwrap().to_str().unwrap();

    Ok(Json(json!({
        "vault_name": vault_name,
    })))
}
use std::fs;
use std::collections::HashMap;

pub async fn get_available_contexts(
    State(state): State<AppState>,
) -> Result<Json<HashMap<Uuid, String>>, StatusCode> {
    let service = state.service.read().unwrap();
    let contexts_dir = service.storage_path().join(".karta").join("contexts");

    if !contexts_dir.exists() {
        return Ok(Json(HashMap::new()));
    }

    let mut contexts = HashMap::new();
    let entries = match fs::read_dir(contexts_dir) {
        Ok(entries) => entries,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    for entry in entries {
        if let Ok(entry) = entry {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("ctx") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    if let Ok(uuid) = Uuid::parse_str(stem) {
                        // We need the node path, not the file path.
                        // The file name is the UUID of the focal node.
                        // We need to get the path of that node.
                        match service.data().open_node(&NodeHandle::Uuid(uuid)) {
                            Ok(node) => {
                                contexts.insert(uuid, node.path().alias().to_string());
                            },
                            Err(_) => continue,
                        }
                    }
                }
            }
        }
    }

    Ok(Json(contexts))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        server::{karta_service::KartaService, AppState},
        utils::utils::KartaServiceTestContext,
        prelude::NodeTypeId,
    };
    use axum::{
        body::Body,
        http::{self, Request, StatusCode},
        Router,
    };
    use tower::ServiceExt;

    fn setup_test_environment(test_name: &str) -> (Router, KartaServiceTestContext) {
        let test_ctx = KartaServiceTestContext::new(test_name);
        let app_state = AppState {
            service: test_ctx.service_arc.clone(),
            tx: tokio::sync::broadcast::channel(1).0,
        };
        let router = Router::new()
            .route("/api/paths", axum::routing::get(get_paths))
            .with_state(app_state);
        (router, test_ctx)
    }

    #[tokio::test]
    async fn test_get_paths_only_indexed() {
        let (router, mut test_ctx) = setup_test_environment("get_paths_indexed");

        // Arrange: Create some indexed and non-indexed nodes
        test_ctx.create_dir_in_vault("physical_dir").unwrap();
        let indexed_path = NodePath::new("vault/indexed_node".into());
        let indexed_node = DataNode::new(&indexed_path, NodeTypeId::file_type());
        test_ctx.with_service_mut(|s| {
            s.data_mut().insert_nodes(vec![indexed_node]);
        });

        // Act
        let response = router
            .oneshot(
                Request::builder()
                    .uri("/api/paths?only_indexed=true")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let paths: Vec<String> = serde_json::from_slice(&body).unwrap();
        
        // Should contain the indexed node, and archetypes like root and vault
        assert!(paths.iter().any(|p| p == "/vault/indexed_node"));
        assert!(paths.iter().any(|p| p == "/"));
        assert!(paths.iter().any(|p| p == "/vault"));
        // Should NOT contain the purely physical directory
        assert!(!paths.iter().any(|p| p == "/vault/physical_dir"));
    }

    #[tokio::test]
    async fn test_get_paths_all() {
        let (router, test_ctx) = setup_test_environment("get_paths_all");

        // Arrange
        test_ctx.create_dir_in_vault("physical_dir").unwrap();
        let indexed_path = NodePath::new("vault/indexed_node".into());
        let indexed_node = DataNode::new(&indexed_path, NodeTypeId::file_type());
        test_ctx.with_service_mut(|s| {
            s.data_mut().insert_nodes(vec![indexed_node]);
        });

        // Act
        let response = router
            .oneshot(
                Request::builder()
                    .uri("/api/paths?only_indexed=false")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let paths: Vec<String> = serde_json::from_slice(&body).unwrap();

        println!("Paths: {:?}", paths);

        // Should contain both indexed and physical paths
        assert!(paths.iter().any(|p| p == "/vault/indexed_node"));
        assert!(paths.iter().any(|p| p == "/vault/physical_dir"));
        assert!(paths.iter().any(|p| p == "/"));
        assert!(paths.iter().any(|p| p == "/vault"));
    }

    #[tokio::test]
    async fn test_for_duplicate_indexed_paths() {
        let (router, _test_ctx) = setup_test_environment("check_duplicates");
        
        // Act
        let response = router
        .oneshot(
            Request::builder()
            .uri("/api/paths?only_indexed=true")
            .body(Body::empty())
            .unwrap(),
        )
        .await
        .unwrap();
    
    // Assert
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();

    let paths: Vec<String> = serde_json::from_slice(&body).unwrap();

    let mut seen = std::collections::HashSet::new();
    let duplicates: Vec<_> = paths.into_iter().filter(|p| !seen.insert(p.clone())).collect();

    assert!(duplicates.is_empty(), "Found duplicate indexed paths: {:?}", duplicates);
    }
}