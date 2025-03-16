use crate::prelude::*;
use axum::{
    extract::{Path, State},
    routing::{get, post},
    Extension, Json, Router,
};
use std::{io::{self, Write}, sync::RwLock};
use std::path::PathBuf;
use std::{error::Error, sync::Arc};
use tokio::sync::broadcast;

mod data_endpoints;
mod context_endpoints;

#[derive(Clone)]
pub struct AppState {
    graph_db: Arc<RwLock<GraphAgdb>>,
    context_db: Arc<RwLock<ContextDb>>,
    tx: broadcast::Sender<String>,
}

pub fn create_router(state: AppState) -> Router {
    let router = Router::new()
        .route("/", get(|| async { "You gonna get some nodes, aight?" }))

        // .route("/idx/*id", post(index_node_connections))

        // .route("/nodes", get(get_all_aliases))

        // .route("/nodes/", get(get_root_node))
        // .route("/nodes/*id", get(get_node))

        // .route("/ctx/*id", get(get_node_context))
        // .with_state(state)
        .layer(Extension(state));
    router
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
            // println!("Exiting server.");
            return;
        }

        let path = PathBuf::from(input);
        if path.is_dir() {
            break path;
        } else {
            // println!("Invalid path. Please enter a valid directory path.");
        }

    };

    let graph_agdb = Arc::new(RwLock::new(GraphAgdb::new(
        name,
        root_path.clone(),
        Some(root_path.clone()),
    )));
    let context_db = Arc::new(RwLock::new(
        ContextDb::new()
    ));
    let (tx, _rx) = broadcast::channel(100);

    let state = AppState {
        graph_db: graph_agdb,
        context_db,
        tx
    };

    let app = create_router(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    axum::serve(listener, app).await.unwrap();
}
