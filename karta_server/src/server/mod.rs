// TODO: BUG - CLI Vault Path Handling for New Vaults:
// There's a persistent issue where if a user types a path for a *new* vault
// without a leading slash (e.g., "Users/play/MyNewVault"), intending it to be
// absolute, it can sometimes be incorrectly appended to the server's current
// working directory, resulting in an invalid vault root like
// "/current/working/dir/Users/play/MyNewVault". This seems to occur specifically
// when the path doesn't exist yet, and fs::canonicalize() fails, and the
// fallback logic to construct an absolute path isn't robustly preventing
// this relative join. Symlink resolution for existing paths is intended to work,
// but this new path creation with an assumed absolute intent is problematic.
// This needs further investigation to ensure all user-typed paths for new vaults
// are correctly and unambiguously treated as absolute from the root.
use crate::{prelude::*};
use axum::{
    extract::{Path, State},
    routing::{get, post, put},
    Extension, Json, Router, http::Method,
};
use tower_http::cors::{Any, CorsLayer}; // Added imports for CORS
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
mod write_endpoints;
mod asset_endpoints;
pub mod karta_service;

#[derive(Clone)]
pub struct AppState {
    service: Arc<RwLock<KartaService>>,
    tx: broadcast::Sender<String>,
}

pub fn create_router(state: AppState) -> Router<()> {
    let cors = CorsLayer::new()
        .allow_origin("http://localhost:5173".parse::<axum::http::HeaderValue>().unwrap())
        .allow_methods(Any) // Allow all methods
        .allow_headers(Any); // Allow any headers

    let router = Router::new()
    	.route("/", get(data_endpoints::get_vault_info))
    	.route("/api/asset/{*path}", get(asset_endpoints::get_asset))
        .route("/api/nodes", post(write_endpoints::create_node))
        .route(
            "/api/nodes/{id}",
            put(write_endpoints::update_node)
            .get(data_endpoints::get_node_by_uuid)
        )
        .route("/api/ctx/{id}", put(write_endpoints::save_context))
        .route("/ctx/{*id}", get(context_endpoints::open_context_from_fs_path)) // Corrected wildcard syntax
        .layer(cors) // Apply the CORS layer
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
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

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
            if let Ok(entries) = fs::read_dir(dir_to_scan.clone()) { // Clone dir_to_scan for use in symlink resolution
                for entry_result in entries {
                    if let Ok(entry) = entry_result {
                        let file_name_os = entry.file_name();
                        if let Some(name_str) = file_name_os.to_str() {
                            // Rule: Hide dotfiles unless the prefix itself starts with a dot.
                            if name_str.starts_with('.') && !item_prefix_to_match.starts_with('.') {
                                continue;
                            }

                            if name_str.starts_with(&item_prefix_to_match) {
                                let mut is_target_a_directory = false;
                                if let Ok(file_type) = entry.file_type() {
                                    if file_type.is_dir() {
                                        is_target_a_directory = true;
                                    } else if file_type.is_symlink() {
                                        if let Ok(target_path_buf) = fs::read_link(entry.path()) {
                                            // Resolve symlink: target_path_buf can be relative or absolute
                                            let resolved_target = if target_path_buf.is_absolute() {
                                                target_path_buf
                                            } else {
                                                // dir_to_scan is the directory containing the symlink
                                                dir_to_scan.join(target_path_buf)
                                            };
                                            // Canonicalize to resolve ".." etc. and check existence
                                            if let Ok(canonical_target) = resolved_target.canonicalize() {
                                                if canonical_target.is_dir() {
                                                    is_target_a_directory = true;
                                                }
                                            }
                                        }
                                    }
                                }

                                if is_target_a_directory {
                                    candidates.push(Pair {
                                        display: name_str.to_string() + "/",
                                        replacement: name_str.to_string() + "/",
                                    });
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
                    return Err("Exiting server.".into());
                }

                // Check if the input is just an integer (for selecting an existing vault)
                if let Ok(index) = input.parse::<usize>() {
                    if index < vaults.vaults.len() {
                        let chosen_path_from_vault_list = vaults.vaults[index].clone();
                        match fs::canonicalize(&chosen_path_from_vault_list) {
                            Ok(canonical_path) => {
                                if canonical_path.is_dir() {
                                    break canonical_path; // Successfully selected and canonicalized existing vault
                                } else {
                                    println!("Vault path '{}' (entry {}) resolved to '{}', which is not a directory. Please check.", chosen_path_from_vault_list.display(), index, canonical_path.display());
                                }
                            }
                            Err(e) => {
                                println!("Error resolving vault path '{}' (entry {}): {}. Please check.", chosen_path_from_vault_list.display(), index, e);
                            }
                        }
                        // If we reach here, the selected vault entry was problematic
                        println!("Selected vault entry {} ('{}') is not a valid directory path.", index, chosen_path_from_vault_list.display());
                        println!("");
                        continue; // Ask for input again
                    } else {
                        println!("Invalid vault number: {}. Please choose from the list or type a path.", index);
                        println!("");
                        continue; // Ask for input again
                    }
                }

                // Handle typed path (for existing or new vault)
                let path_str_for_processing = if input.starts_with('/') {
                    input.to_string()
                } else {
                    format!("/{}", input)
                };
                // path_str_for_processing now always represents an absolute path string.
                
                let path_to_check = PathBuf::from(path_str_for_processing);
                // path_to_check is now constructed from a string guaranteed to start with "/"

                // Try to canonicalize first. This resolves symlinks and checks existence.
                match fs::canonicalize(&path_to_check) { // path_to_check is absolute
                    Ok(canonical_path) => {
                        if canonical_path.is_dir() {
                            break canonical_path; // Existing, canonicalized directory
                        } else {
                            println!("Path '{}' resolved to '{}', which is not a directory.", path_to_check.display(), canonical_path.display());
                        }
                    }
                    Err(_e) => {
                        // Canonicalization failed. This could be because:
                        // 1. Path does not exist (potential new vault).
                        // 2. Path exists but is a broken symlink or other issue.
                        // 3. Permissions error.

                        // If it doesn't exist and looks like a directory path (no extension),
                        // assume it's for a new vault.
                        // path_to_check was derived from path_str_for_processing, which is absolute.
                        if !path_to_check.exists() && path_to_check.extension().is_none() {
                            // It's a new path. Try to make it absolute by canonicalizing its parent.
                            if let Some(parent) = path_to_check.parent() {
                                if let Ok(canonical_parent) = fs::canonicalize(parent) {
                                    if let Some(file_name) = path_to_check.file_name() {
                                        break canonical_parent.join(file_name);
                                    }
                                }
                            }
                            // Fallback if parent canonicalization fails or no parent/filename (should be rare for valid new paths)
                            // This break uses path_to_check which *should* be absolute from its construction.
                            println!("Warning: Could not fully canonicalize new path's parent for '{}'. Using constructed absolute path.", path_to_check.display());
                            break path_to_check;
                        }
                        // Otherwise, it's an invalid path for an existing directory or a problematic one.
                        println!("Debug: Canonicalization failed for '{}' (derived from input '{}'): {}. This path is not an existing directory and not suitable for a new vault name.", path_to_check.display(), input, _e);
                    }
                }
                // If we reach here, the typed path was not a valid existing (or canonicalizable) directory,
                // nor was it accepted as a new potential vault path.
                // Use 'input' for the error message as it's what the user typed,
                // or path_to_check.display() if you want to show the absolutized version.
                println!("Invalid path: '{}'. Please enter a valid absolute directory path for an existing or new vault.", input);
                println!("");
                // continue; // This continue is implicit as it's the end of the Ok(line) block and will loop
            } // End of Ok(line) block
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