use axum::{
    extract::{Path, State}, routing::{get, post}, Extension, Json, Router
};
use std::{error::Error, sync::Arc};
use tokio::sync::broadcast;
use crate::prelude::*;
use std::path::PathBuf;
use std::io::{self, Write};

#[derive(Clone)]
pub struct AppState {
    graph_commands: Arc<GraphCommands>,
    tx: broadcast::Sender<String>,
}

pub fn create_router(state: AppState) -> Router {
    let router = Router::new()
        .route("/", get(|| async { "You gonna get some nodes, aight?" }))

        .route("/nodes", get(get_root_node))
        .route("/nodes/", get(get_root_node))
        .route("/nodes/:id", get(get_node))
        // .with_state(state)
        .layer(Extension(state))
        ;
    router
}

async fn root() -> &'static str {
    "Welcome to Karta Server"
}

async fn get_root_node(
    Extension(state): Extension<AppState>,
) -> Json<Result<Node, String>> {
    let root_path = NodePath::root();
    let result = state.graph_commands.open_node(&root_path)
        .map_err(|e| e.to_string());
    Json(result)
}

async fn get_node(
    Extension(state): Extension<AppState>,
    Path(id): Path<String>,
) -> Json<Result<Node, String>> {
    println!("Requested node with id: {}", id);
    let node_path = NodePath::from_alias(&id);
    println!("Resulting node_path: {:#?}", node_path);
    println!("Resulting alias: {}", node_path.alias());
    let result = state.graph_commands.open_node(&node_path)
        .map_err(|e| e.to_string());
    Json(result)
}

pub async fn run_server() {
    let name = "karta_server";
    let root_path = loop {
        print!("Enter the path for the server (or press Enter to exit): ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        if input.is_empty() {
            println!("Exiting server.");
            return;
        }

        let path = PathBuf::from(input);
        if path.is_dir() {
            break path;
        } else {
            println!("Invalid path. Please enter a valid directory path.");
        }
    };

    let graph_commands = Arc::new(GraphCommands::new(name, root_path.clone(), Some(root_path.clone())));
    let (tx, _rx) = broadcast::channel(100);

    let state = AppState { graph_commands, tx };

    let app = create_router(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
