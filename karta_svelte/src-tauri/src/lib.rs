use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use tauri::{Manager, State};

// Global state for the server thread
struct ServerProcess {
    handle: Arc<Mutex<Option<std::thread::JoinHandle<()>>>>,
    shutdown_sender: Arc<Mutex<Option<std::sync::mpsc::Sender<()>>>>,
}

impl ServerProcess {
    fn new() -> Self {
        Self {
            handle: Arc::new(Mutex::new(None)),
            shutdown_sender: Arc::new(Mutex::new(None)),
        }
    }

    fn start(&self, vault_path: &str) -> Result<(), String> {
        // Stop existing server if running
        self.stop()?;
        
        let vault_path = PathBuf::from(vault_path);
        let (shutdown_tx, _shutdown_rx) = std::sync::mpsc::channel();
        
        // Store the shutdown sender
        {
            let mut sender_guard = self.shutdown_sender.lock().map_err(|e| format!("Lock error: {}", e))?;
            *sender_guard = Some(shutdown_tx);
        }
        
        // Spawn server in its own thread
        let handle = std::thread::spawn(move || {
            // Use Tauri's async runtime
            tauri::async_runtime::block_on(async {
                // Run the server without logging initialization (Tauri handles logging)
                karta_server::prelude::run_server_with_logging(vault_path, false).await;
            });
        });
        
        // Store the thread handle
        {
            let mut handle_guard = self.handle.lock().map_err(|e| format!("Lock error: {}", e))?;
            *handle_guard = Some(handle);
        }
        
        Ok(())
    }

    fn stop(&self) -> Result<(), String> {
        // Send shutdown signal
        {
            let mut sender_guard = self.shutdown_sender.lock().map_err(|e| format!("Lock error: {}", e))?;
            if let Some(sender) = sender_guard.take() {
                let _ = sender.send(());
            }
        }
        
        // Wait for thread to complete (with timeout)
        {
            let mut handle_guard = self.handle.lock().map_err(|e| format!("Lock error: {}", e))?;
            if let Some(handle) = handle_guard.take() {
                // Use a timeout to avoid hanging the app shutdown
                std::thread::spawn(move || {
                    if handle.join().is_err() {
                        println!("Warning: Server thread did not shut down cleanly");
                    }
                });
                // Don't wait for the thread, just continue with shutdown
                println!("Server shutdown initiated");
            }
        }
        
        Ok(())
    }
}

#[tauri::command]
async fn start_server(vault_path: String, server_process: State<'_, ServerProcess>) -> Result<(), String> {
    server_process.start(&vault_path)?;
    
    // Wait a moment for server to start
    tokio::time::sleep(std::time::Duration::from_millis(2000)).await;
    
    // Test if server is responding
    match reqwest::get("http://localhost:7370/").await {
        Ok(response) if response.status().is_success() => Ok(()),
        _ => Err("Server failed to start or is not responding".to_string()),
    }
}

#[tauri::command]
async fn stop_server(server_process: State<'_, ServerProcess>) -> Result<(), String> {
    server_process.stop()
}

#[tauri::command]
async fn check_server_status() -> Result<bool, String> {
    match reqwest::get("http://localhost:7370/").await {
        Ok(response) => Ok(response.status().is_success()),
        Err(_) => Ok(false),
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct VaultInfo {
    path: String,
    exists: bool,
    has_karta_dir: bool,
}

#[tauri::command]
async fn get_available_vaults() -> Result<Vec<VaultInfo>, String> {
    // Use the existing vault config functions from karta_server
    let mut vaults_config = karta_server::prelude::get_vaults_config();
    
    // Check each vault and return status
    let mut vault_infos = Vec::new();
    for vault_path in vaults_config.get() {
        let path_str = vault_path.to_string_lossy().to_string();
        let exists = vault_path.exists();
        let has_karta_dir = vault_path.join(".karta").exists();
        
        vault_infos.push(VaultInfo {
            path: path_str,
            exists,
            has_karta_dir,
        });
    }
    
    Ok(vault_infos)
}

#[tauri::command]
async fn select_vault_directory() -> Result<Option<String>, String> {
    // This command will be called from the frontend and handled by Tauri's dialog system
    // The actual dialog opening is handled by the JavaScript frontend using @tauri-apps/plugin-dialog
    Err("This command should not be called directly. Use the frontend dialog API.".to_string())
}

#[tauri::command]
async fn add_vault_to_config(vault_path: String) -> Result<(), String> {
    use std::path::PathBuf;
    
    let vault_path_buf = PathBuf::from(&vault_path);
    
    // Ensure the directory exists
    if !vault_path_buf.exists() {
        std::fs::create_dir_all(&vault_path_buf)
            .map_err(|e| format!("Failed to create vault directory: {}", e))?;
    }
    
    // Use the existing vault management functions from karta_server
    let mut vaults_config = karta_server::prelude::get_vaults_config();
    vaults_config.add_vault(&vault_path_buf);
    vaults_config.save();
    
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let server_process = ServerProcess::new();
    
    tauri::Builder::default()
        .manage(server_process)
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                // Stop server when main window closes
                let server_process = window.state::<ServerProcess>();
                let _ = server_process.stop();
            }
        })
        .invoke_handler(tauri::generate_handler![
            start_server,
            stop_server,
            check_server_status,
            get_available_vaults,
            select_vault_directory,
            add_vault_to_config
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
