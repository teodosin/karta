use super::*;
use crate::server::write_endpoints::{save_context, create_node, delete_nodes, update_node, rename_node, move_nodes};
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
pub fn setup_test_environment(test_name: &str) -> (Router, KartaServiceTestContext) {
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
        .route("/api/nodes", axum::routing::post(create_node).delete(delete_nodes))
        .route("/api/nodes/{id}", axum::routing::put(update_node))
        .route("/api/nodes/rename", axum::routing::post(rename_node))
        .route("/api/nodes/move", axum::routing::post(move_nodes))
        .with_state(app_state);
    (router, test_ctx)
}

// Helper for POST requests
pub async fn execute_post_request(router: Router, uri: &str, body: String) -> http::Response<Body> {
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
pub async fn execute_put_request(router: Router, uri: &str, body: String) -> http::Response<Body> {
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

// Helper for DELETE requests
pub async fn execute_delete_request(router: Router, uri: &str, body: String) -> http::Response<Body> {
    router
        .oneshot(
            Request::builder()
                .method(http::Method::DELETE)
                .uri(uri)
                .header(http::header::CONTENT_TYPE, "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap()
}
