use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use tauri::{Manager, State};
use std::collections::HashMap;

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
        .manage(BookmarkStore::default())
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
                // Stop all bookmark access on shutdown (best-effort)
                #[cfg(target_os = "macos")]
                {
                    let store = window.state::<BookmarkStore>();
                    let _ = store.stop_all_access();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            start_server,
            stop_server,
            check_server_status,
            get_available_vaults,
            select_vault_directory,
            add_vault_to_config,
            ensure_vault_access,
            save_vault_bookmark
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// ---------------- macOS Security-Scoped Bookmarks ----------------

#[derive(Default)]
struct BookmarkStore(Arc<Mutex<HashMap<String, String>>>); // path -> base64(bookmark)

impl BookmarkStore {
    #[cfg(target_os = "macos")]
    fn save(&self, path: &str, bookmark_b64: &str) {
        if let Ok(mut m) = self.0.lock() {
            m.insert(path.to_string(), bookmark_b64.to_string());
        }
        let _ = self.persist_to_disk();
    }

    #[cfg(target_os = "macos")]
    fn get(&self, path: &str) -> Option<String> {
        if let Ok(m) = self.0.lock() {
            m.get(path).cloned()
        } else { None }
    }

    #[cfg(target_os = "macos")]
    fn stop_all_access(&self) -> Result<(), String> {
        // We don't hold NSUrl here; stopAccessing is best handled per-open URL.
        Ok(())
    }

    #[cfg(target_os = "macos")]
    fn persist_to_disk(&self) -> Result<(), String> {
        use std::fs;
        let app_dir = dirs::data_local_dir().ok_or("No data dir")?;
        let dir = app_dir.join("karta");
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let file = dir.join("bookmarks.json");
        let map = self.0.lock().map_err(|e| e.to_string())?;
        let json = serde_json::to_string(&*map).map_err(|e| e.to_string())?;
        fs::write(file, json).map_err(|e| e.to_string())?;
        Ok(())
    }

    #[cfg(target_os = "macos")]
    fn load_from_disk(&self) -> Result<(), String> {
        use std::fs;
        let app_dir = dirs::data_local_dir().ok_or("No data dir")?;
        let dir = app_dir.join("karta");
        let file = dir.join("bookmarks.json");
        if let Ok(bytes) = fs::read(&file) {
            if let Ok(map) = serde_json::from_slice::<HashMap<String, String>>(&bytes) {
                if let Ok(mut m) = self.0.lock() { *m = map; }
            }
        }
        Ok(())
    }
}

#[tauri::command]
async fn ensure_vault_access(path: String, store: State<'_, BookmarkStore>) -> Result<bool, String> {
    // Returns true if access is ensured (bookmark exists and startAccess succeeded), false if UI must reauth
    #[cfg(target_os = "macos")]
    {
        store.load_from_disk().ok();
        if let Some(bookmark_b64) = store.get(&path) {
            start_access_from_bookmark(&bookmark_b64)?;
            return Ok(true);
        }
        Ok(false)
    }
    #[cfg(not(target_os = "macos"))]
    { Ok(true) }
}

#[tauri::command]
async fn save_vault_bookmark(path: String, bookmark_b64: String, store: State<'_, BookmarkStore>) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        store.save(&path, &bookmark_b64);
        Ok(())
    }
    #[cfg(not(target_os = "macos"))]
    { Ok(()) }
}

#[cfg(target_os = "macos")]
fn start_access_from_bookmark(bookmark_b64: &str) -> Result<(), String> {
    use cocoa::base::{id, nil};
    use cocoa::foundation::{NSData, NSAutoreleasePool, NSURL};
    use objc::rc::StrongPtr;
    use objc::runtime::Object;
    use objc::{msg_send, sel, sel_impl};
    use base64::Engine as _;

    unsafe {
        let _pool = NSAutoreleasePool::new(nil);

        let engine = base64::engine::general_purpose::STANDARD;
        let bytes = engine.decode(bookmark_b64).map_err(|e| e.to_string())?;
        let data: id = NSData::dataWithBytes_length_(nil, bytes.as_ptr() as _, bytes.len() as _);

        let mut is_stale: bool = false;
        let error: *mut Object = std::ptr::null_mut();
        let url: id = NSURL::URLByResolvingBookmarkData_options_relativeToURL_bookmarkDataIsStale_error_(
            NSURL::alloc(nil),
            data,
            0,
            nil,
            &mut is_stale as *mut bool,
            error,
        );
        if url == nil {
            return Err("Failed to resolve bookmark".into());
        }
        let _url = StrongPtr::new(url);

        // Start accessing security-scoped resource
        let ok: bool = msg_send![_url.as_ptr(), startAccessingSecurityScopedResource];
        if !ok { return Err("startAccessingSecurityScopedResource failed".into()); }
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn create_bookmark_from_path(path: &str) -> Result<String, String> {
    use cocoa::base::{id, nil};
    use cocoa::foundation::{NSData, NSString, NSAutoreleasePool, NSURL};
    use objc::runtime::Object;
    use objc::{msg_send, sel, sel_impl};
    use base64::Engine as _;

    unsafe {
        let _pool = NSAutoreleasePool::new(nil);
        let ns_path = NSString::alloc(nil).init_str(path);
        let url: id = NSURL::fileURLWithPath_(nil, ns_path);
        if url.is_null() { return Err("Failed to create file URL".into()); }

        // Constant for NSURLBookmarkCreationWithSecurityScope
        let opts: u64 = 0x2000;
        let mut error: *mut Object = std::ptr::null_mut();
        let data: id = msg_send![url, bookmarkDataWithOptions: opts as u64
            includingResourceValuesForKeys: nil
            relativeToURL: nil
            error: &mut error];
        if data.is_null() { return Err("Failed to create bookmark data".into()); }

        let bytes_ptr: *const u8 = msg_send![data, bytes];
        let len: usize = msg_send![data, length];
        let slice = std::slice::from_raw_parts(bytes_ptr, len);
        let b64 = base64::engine::general_purpose::STANDARD.encode(slice);
        Ok(b64)
    }
}

#[tauri::command]
async fn save_vault_bookmark_from_path(path: String, store: State<'_, BookmarkStore>) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let b64 = create_bookmark_from_path(&path)?;
        store.save(&path, &b64);
        Ok(())
    }
    #[cfg(not(target_os = "macos"))]
    { Ok(()) }
}
