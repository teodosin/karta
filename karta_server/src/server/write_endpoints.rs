use axum::{
    extract::{Path as AxumPath, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    context::context::Context,
    elements::{
        attribute::{Attribute, AttrValue},
        node::DataNode,
        node_path::{NodeHandle, NodePath},
        nodetype::NodeTypeId,
    },
    graph_traits::graph_node::GraphNodes,
    server::AppState,
};

#[derive(Deserialize)]
pub struct CreateNodePayload {
    name: String,
    ntype: NodeTypeId,
    parent_path: String,
    attributes: Vec<Attribute>,
}

#[derive(Deserialize)]
pub struct UpdateNodePayload {
    attributes: Vec<Attribute>,
}

pub async fn create_node(
    State(app_state): State<AppState>,
    Json(payload): Json<CreateNodePayload>,
) -> Result<Json<DataNode>, StatusCode> {
    let mut service = app_state.service.write().unwrap();
    let parent_path = NodePath::from(payload.parent_path);
    let mut name = payload.name.clone();
    let mut final_path = parent_path.join(&name);
    let mut counter = 2;

    // Loop until we find a unique path
    while service.data().open_node(&NodeHandle::Path(final_path.clone())).is_ok() {
        name = format!("{}_{}", payload.name.clone(), counter);
        final_path = parent_path.join(&name);
        counter += 1;
    }

    let mut new_node = DataNode::new(&final_path, payload.ntype);
    
    // Update the name attribute if it was changed
    let mut attributes = payload.attributes;
    if let Some(attr) = attributes.iter_mut().find(|a| a.name == "name") {
        attr.value = AttrValue::String(name.clone());
    } else {
        attributes.push(Attribute::new_string("name".to_string(), name));
    }

    new_node.set_attributes(attributes);
    new_node.set_name(&new_node.path().name());

    service.data_mut().insert_nodes(vec![new_node.clone()]);

    Ok(Json(new_node))
}

pub async fn update_node(
    State(app_state): State<AppState>,
    AxumPath(id): AxumPath<Uuid>,
    Json(payload): Json<UpdateNodePayload>,
) -> Result<Json<DataNode>, StatusCode> {
    let mut service = app_state.service.write().unwrap();

    let mut node = match service.data().open_node(&NodeHandle::Uuid(id)) {
        Ok(node) => node,
        Err(_) => return Err(StatusCode::NOT_FOUND),
    };

    node.set_attributes(payload.attributes);
    node.update_modified_time();

    service.data_mut().insert_nodes(vec![node.clone()]);

    Ok(Json(node))
}

pub async fn save_context(
    State(app_state): State<AppState>,
    AxumPath(id): AxumPath<Uuid>,
    Json(payload): Json<Context>,
) -> StatusCode {
    // The `id` from the path should match the `focal` id in the payload.
    if id != payload.focal() {
        return StatusCode::BAD_REQUEST;
    }

    let service = app_state.service.read().unwrap();
    match service.view().save_context(&payload) {
        Ok(_) => StatusCode::OK,
        Err(e) => {
            eprintln!("Error saving context: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
    fn setup_test_environment(test_name: &str) -> (Router, KartaServiceTestContext) {
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
            .route("/api/ctx/{id}", axum::routing::put(save_context)) // Corrected syntax
            .with_state(app_state);
        (router, test_ctx)
    }

    // Helper for PUT requests
    async fn execute_put_request(router: Router, uri: &str, body: String) -> http::Response<Body> {
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

    #[tokio::test]
    async fn test_save_context_creates_file() {
        let (router, test_ctx) = setup_test_environment("save_creates_file");

        // Arrange
        test_ctx.create_dir_in_vault("test_dir").unwrap();
        let (_, _, initial_context) = test_ctx
            .with_service(|s| s.open_context_from_path("vault/test_dir".into()))
            .unwrap();
        let focal_uuid = initial_context.focal();
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
        let (_, _, initial_context) = test_ctx
            .with_service(|s| s.open_context_from_path("vault/dir_to_delete".into()))
            .unwrap();
        let focal_uuid = initial_context.focal();
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

        // Arrange: FS setup
        test_ctx.create_dir_in_vault("test_dir").unwrap();
        test_ctx.create_file_in_vault("test_dir/A.txt", b"").unwrap();
        test_ctx.create_file_in_vault("test_dir/B.txt", b"").unwrap();

        // Arrange: Get initial state to find the node to modify.
        let (initial_nodes, _, initial_context) = test_ctx
            .with_service(|s| s.open_context_from_path("vault/test_dir".into()))
            .unwrap();
        let focal_uuid = initial_context.focal();
        let node_b_data = initial_nodes.iter().find(|n| n.path().name() == "B.txt").unwrap();
        let node_b_view = initial_context.viewnodes().iter().find(|vn| vn.uuid == node_b_data.uuid()).unwrap();

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
}