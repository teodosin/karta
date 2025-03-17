use crate::{context::ContextDb, prelude::*};
use axum::{
    extract::{Path, State},
    routing::{get, post},
    Extension, Json, Router,
};
use serde::Deserialize;
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



pub async fn run_server(root_path: PathBuf) {
    let name = "karta_server";

    let storage_dir = root_path.join(".karta");

    // Create the path if it doesn't exist
    if !storage_dir.exists() {
        std::fs::create_dir_all(&storage_dir).expect("Failed to create storage path");
    }

    let graph_agdb = Arc::new(RwLock::new(GraphAgdb::new(
        name,
        root_path.clone(),
        storage_dir.clone(),
    )));
    let context_db = Arc::new(RwLock::new(
        ContextDb::new(
            name.to_owned(),
            root_path.clone(),
            storage_dir.clone(),
        )
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

pub fn load_or_create_vault() -> Result<PathBuf, Box<dyn Error>> {
    let root_path = loop {
        println!("Existing vaults:");
        let vaults = get_vaults_config();
        println!("{:#?}", vaults);

        println!("Enter the path for the server (or press Enter to exit): ");

        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        if input.is_empty() {
            // println!("Exiting server.");
            return Err("Exiting server.".into());
        }

        let path = PathBuf::from(input);
        if path.is_dir() {
            break path;
        } else {
            // println!("Invalid path. Please enter a valid directory path.");
        }

    };

    Ok(root_path)
}

#[derive(Debug, serde::Serialize, Deserialize, Default)]
struct Vaults {
    default: PathBuf,
    vaults: Vec<PathBuf>,
}

fn vaults_config_path() -> PathBuf {
    let file_name = "karta_vaults.ron";
    let config_path = directories::ProjectDirs::from("com", "karta_server", "karta_server")
        .unwrap()
        .config_dir()
        .to_path_buf();
    let file_path = config_path.join(file_name);
    file_path
}

fn get_vaults_config() -> Vaults {
    let file_path = vaults_config_path();

    let config_file = match std::fs::File::open(file_path) {
        Ok(file) => file,
        Err(_) => {
            return Vaults::default();
        }
    };

    let vaults = ron::de::from_reader(config_file);

    vaults.unwrap_or_default()
}