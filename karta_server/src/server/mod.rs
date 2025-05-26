use crate::{prelude::*};
use axum::{
    extract::{Path, State},
    routing::{get, post},
    Extension, Json, Router,
};
use karta_service::KartaService;
use serde::Deserialize;
use std::{borrow::Cow, io::{self, Write}, sync::RwLock};
use std::path::PathBuf;
use std::{error::Error, sync::Arc};
use std::fs; // For filesystem operations in PathCompleter

// Rustyline imports
use rustyline::{Editor, Config}; // Added Config
use rustyline::Helper; // Main trait for custom helper
use rustyline::Context as RustylineContext; // Alias for rustyline::Context
use rustyline::Result as RustylineResult; // Alias for rustyline::Result
use rustyline::error::ReadlineError;     // For error handling in match
use rustyline::completion::{Completer, Pair}; // For completion, Added CompletionType
use rustyline::CompletionType;
use rustyline::highlight::{Highlighter, CmdKind};   // For syntax highlighting (even if basic), Added CmdKind
use rustyline::hint::Hinter;             // For hints (even if basic)
use rustyline::validate::{Validator, ValidationContext, ValidationResult}; // For input validation

// Remove DefaultEditor if not used elsewhere, Editor is used for custom helper
// use rustyline::DefaultEditor;

use tokio::sync::broadcast;

mod data_endpoints;
mod context_endpoints;
pub mod karta_service;

#[derive(Clone)]
pub struct AppState {
    service: Arc<RwLock<KartaService>>,
    tx: broadcast::Sender<String>,
}

pub fn create_router(state: AppState) -> Router<()> {
    let router = Router::new()
        .route("/", get(|| async { "Karta Server" }))
        .route("/ctx/*id", get(context_endpoints::open_context_from_fs_path))

        // So what routes do we want?
        // /data/
        // /ctx/

        // .route("/idx/*id", post(index_node_connections))

        // .route("/nodes", get(get_all_aliases))

        // .route("/nodes/", get(get_root_node))

        // .route("/ctx/*id", get(get_node_context))
        // .with_state(state)
        .with_state(state);

    router
}



pub async fn run_server(root_path: PathBuf) {
    let name = "karta_server";

    let storage_dir = root_path.join(".karta");

    // Create the path if it doesn't exist
    if !storage_dir.exists() {
        std::fs::create_dir_all(&storage_dir).expect("Failed to create storage path");
    }

    let service = Arc::new(RwLock::new(KartaService::new(
        name,
        root_path.clone(),
        storage_dir.clone(),
    )));

    let (tx, _rx) = broadcast::channel(100);

    let state = AppState {
        service,
        tx
    };

    let app = create_router(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:7370").await.unwrap();

    println!("Server listening on http://0.0.0.0:7370");

    axum::serve(listener, app).await.unwrap();
}



// PathCompleter focuses on the completion logic
struct PathCompleter;

impl Completer for PathCompleter {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &RustylineContext<'_>,
    ) -> RustylineResult<(usize, Vec<Self::Candidate>)> {
        let mut candidates = Vec::new();
        let input_before_cursor = &line[..pos];

        let (path_to_scan_str, item_prefix_to_match, actual_replacement_offset);

        if input_before_cursor.is_empty() {
            // Case 1: Empty input -> list root
            path_to_scan_str = "/".to_string();
            item_prefix_to_match = "".to_string();
            actual_replacement_offset = 0; // Replace from the start
        } else {
            // Construct the path as if it's absolute
            let mut temp_path_for_logic = String::new();
            if !input_before_cursor.starts_with('/') {
                temp_path_for_logic.push('/');
            }
            temp_path_for_logic.push_str(input_before_cursor);

            if temp_path_for_logic.ends_with('/') {
                // Case 2: Input ends with "/" (e.g., "/Users/" or "Users/")
                // -> list contents of this directory, match against empty prefix
                path_to_scan_str = temp_path_for_logic;
                item_prefix_to_match = "".to_string();
                actual_replacement_offset = pos; // Append after the slash
            } else {
                // Case 3: Input is a partial path (e.g., "/Us", "Users/myf", "/Users/myf")
                // Find the last slash to separate directory from item prefix
                if let Some(last_slash_idx) = temp_path_for_logic.rfind('/') {
                    path_to_scan_str = temp_path_for_logic[..last_slash_idx + 1].to_string();
                    item_prefix_to_match = temp_path_for_logic[last_slash_idx + 1..].to_string();
                    // Calculate offset based on original input_before_cursor
                    if let Some(original_last_slash_idx) = input_before_cursor.rfind('/') {
                        actual_replacement_offset = original_last_slash_idx + 1;
                    } else { // No slash in original input (e.g. "Users")
                        actual_replacement_offset = 0;
                    }
                } else {
                    // This should not be reached if temp_path_for_logic always starts with /
                    // But as a fallback for an unexpected state:
                    path_to_scan_str = "/".to_string();
                    item_prefix_to_match = temp_path_for_logic.trim_start_matches('/').to_string();
                    actual_replacement_offset = 0;
                }
            }
        }
        
        let dir_to_scan = PathBuf::from(&path_to_scan_str);

        if dir_to_scan.is_dir() {
            if let Ok(entries) = fs::read_dir(dir_to_scan) {
                for entry_result in entries {
                    if let Ok(entry) = entry_result {
                        if let Ok(file_type) = entry.file_type() {
                            if file_type.is_dir() {
                                if let Some(name_osstr) = entry.file_name().to_str() {
                                    let name = name_osstr.to_string();
                                    if name.starts_with(&item_prefix_to_match) {
                                        candidates.push(Pair {
                                            display: name.clone() + "/",
                                            replacement: name + "/",
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        candidates.sort_by(|a, b| a.display.cmp(&b.display));
        Ok((actual_replacement_offset, candidates))
    }
}

// KartaCliHelper bundles the completer and other helper traits
struct KartaCliHelper {
    completer: PathCompleter,
}

impl Completer for KartaCliHelper {
    type Candidate = Pair; // PathCompleter uses Pair

    fn complete(&self, line: &str, pos: usize, ctx: &RustylineContext<'_>) -> RustylineResult<(usize, Vec<Self::Candidate>)> {
        self.completer.complete(line, pos, ctx)
    }
    // `update` method uses default implementation from trait
}

impl Hinter for KartaCliHelper {
    type Hint = String;
    fn hint(&self, _line: &str, _pos: usize, _ctx: &RustylineContext<'_>) -> Option<Self::Hint> {
        None // No hints for now
    }
}

impl Highlighter for KartaCliHelper {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        Cow::Borrowed(line) // No custom highlighting
    }
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(&'s self, prompt: &'p str, _default: bool) -> Cow<'b, str> {
        Cow::Borrowed(prompt)
    }
    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Cow::Borrowed(hint)
    }
    fn highlight_candidate<'c>(&self, candidate: &'c str, _completion: rustyline::CompletionType) -> Cow<'c, str> {
        Cow::Borrowed(candidate)
    }
    fn highlight_char(&self, _line: &str, _pos: usize, _cmd: CmdKind) -> bool {
        false
    }
}

impl Validator for KartaCliHelper {
    fn validate(&self, _ctx: &mut ValidationContext) -> RustylineResult<ValidationResult> {
        Ok(ValidationResult::Valid(None)) // No validation for now
    }
    fn validate_while_typing(&self) -> bool {
        false
    }
}

impl Helper for KartaCliHelper {} // Bundles the above traits


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
        let helper = KartaCliHelper { completer: PathCompleter };
        let config = Config::builder()
            .completion_type(CompletionType::List)
            .build();
        let mut rl = Editor::with_config(config).expect("Failed to create rustyline editor with config");
        rl.set_helper(Some(helper));
        // Consider adding history support later if desired:
        // if rl.load_history("history.txt").is_err() {
        //     println!("No previous history.");
        // }

        let readline_result = rl.readline(">> "); // Using ">> " as a simple prompt
        match readline_result {
            Ok(line) => {
                let input = line.trim();

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
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                return Err("Exiting server due to CTRL-C.".into());
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                return Err("Exiting server due to CTRL-D.".into());
            }
            Err(err) => {
                println!("Error reading input: {:?}", err);
                return Err(Box::new(err));
            }
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
    pub fn get(&mut self) -> &Vec<PathBuf> {
        self.clean_vaults();

        &self.vaults
    }

    pub fn add_vault(&mut self, vault_path: &PathBuf) {
        self.default = vault_path.clone();

        // Check if the vault already exists in the vaults list
        if self.vaults.contains(&vault_path) { return };

        let karta_dir = vault_path.join(".karta");
        if !karta_dir.exists() {
            std::fs::create_dir_all(&karta_dir).unwrap();
        }
        self.vaults.push(vault_path.to_path_buf());
    }
    
    /// Save the current vaults config to the file.
    pub fn save(&self) {
        let file_path = vaults_config_path();

        println!("");
        println!("Saving vaults config to: {}", file_path.to_string_lossy());
        println!("");

        let mut config_file = std::fs::File::create(file_path).unwrap();
        let pretty_config = ron::ser::to_string_pretty(&self, Default::default()).unwrap();
        config_file.write_all(pretty_config.as_bytes());
    }

    /// Iterates over the vault paths and removes the ones without a .karta directory.
    fn clean_vaults(&mut self) {
        self.vaults.retain(|vault_path| {
            let karta_dir = vault_path.join(".karta");
            karta_dir.exists()
        });
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