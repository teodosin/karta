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

    println!("Server listening on http://0.0.0.0:3000");

    axum::serve(listener, app).await.unwrap();
}

pub fn cli_load_or_create_vault() -> Result<PathBuf, Box<dyn Error>> {
    let mut vaults = get_vaults_config();


    println!("");
    println!("]----- Starting Karta Server -----[");
    println!("");


    if vaults.vaults.is_empty() {
        println!("No vaults found. Please create a new vault.");
        println!("Type the path for the new vault (or press Enter to exit): ");
    } else {
        println!("Existing vaults:");
        println!("");
        for (index, vault) in vaults.vaults.iter().enumerate() {
            println!("{}: {}", index, vault.to_string_lossy());
        }
        println!("");
        println!("Type the path or number for the vault. A valid path not listed above will create a new vault.");
        println!("Leave empty to exit.");
        println!("");
    }

    let root_path = loop {

        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        if input.is_empty() {
            // println!("Exiting server.");
            return Err("Exiting server.".into());
        }

        // Check if the input is just an integer
        if let Ok(index) = input.parse::<usize>() {
            if index < vaults.vaults.len() {
                break vaults.vaults[index].clone();
            }
        }

        let path = PathBuf::from(input);
        if path.is_dir() {
            break path;
        } else {
            println!("Invalid path. Please enter a valid directory path. Leave empty to exit.");
            println!("");
        }

    };

    vaults.add_vault(&root_path);
    vaults.save();

    println!("");
    println!("Starting server on root path: ");
    println!("{}", root_path.to_string_lossy());

    Ok(root_path.to_path_buf())
}

#[derive(Debug, serde::Serialize, Deserialize, Default)]
pub struct Vaults {
    default: PathBuf,
    vaults: Vec<PathBuf>,
}

impl Vaults {

    pub fn add_vault(&mut self, vault_path: &PathBuf) {
        self.default = vault_path.clone();

        // Check if the vault already exists in the vaults list
        if self.vaults.contains(&vault_path) { return };
        self.vaults.push(vault_path.to_path_buf());
    }
    
    // Save the current vaults config to the file.
    pub fn save(&self) {
        let file_path = vaults_config_path();

        println!("");
        println!("Saving vaults config to: {}", file_path.to_string_lossy());
        println!("");

        let config_file = std::fs::File::create(file_path).unwrap();
        ron::ser::to_writer_pretty(config_file, &self, Default::default()).unwrap();
    }
}
    

pub fn vaults_config_path() -> PathBuf {
    let file_name = "karta_vaults.ron";
    let config_path = directories::ProjectDirs::from("com", "karta_server", "karta_server")
        .unwrap()
        .config_dir()
        .to_path_buf();

    std::fs::create_dir_all(&config_path).unwrap();
    let file_path = config_path.join(file_name);
    file_path
}

pub fn get_vaults_config() -> Vaults {
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