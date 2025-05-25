// Utility functions for the library tests.

#![allow(warnings)]
#[cfg(test)]
pub mod utils {

    use std::{
        fs::{create_dir_all, File}, // Consolidated fs imports
        io::{Error as IoError, Read, Seek, SeekFrom, Write}, // Added IoError, Seek, SeekFrom
        path::{Path, PathBuf},
        time::SystemTime,
    };

    use directories::ProjectDirs;

    use ron::ser::{to_string_pretty, PrettyConfig};
    use serde::{Deserialize, Serialize};

    use crate::{
        context::context_db::ContextDb, // For KartaServiceTestContext helpers
        graph_agdb::GraphAgdb,
        graph_traits::{graph_core::GraphCore, Graph},
        server::karta_service::KartaService
    };
    use std::sync::{Arc, RwLock}; // Added Arc and RwLock

    #[derive(Serialize, Deserialize, Debug, Clone)]
    struct PerfReport {
        commit: String,
        elapsed_ms: u64,
        db_size_bytes: u64,
        timestamp: String,
    }

    // KartaServiceTestContext for testing KartaService instances
    pub struct KartaServiceTestContext {
        pub test_name: String,
        pub service_arc: Arc<RwLock<KartaService>>, // Changed field name and type
        pub vault_root_path: PathBuf, // The root of the temporary vault for this service instance
        measure_perf: bool,
        start_time: std::time::Instant,
    }

    impl KartaServiceTestContext {
        pub fn new(test_name_suffix: &str) -> Self {
            let name = format!("karta_service_test_{}", test_name_suffix);
            let app_name_for_project_dirs = "karta_service_tests"; // Distinct base for these tests

            let vault_root_path =
                ProjectDirs::from("com", "karta_server", app_name_for_project_dirs)
                    .expect("Failed to get project dirs for KartaServiceTestContext")
                    .data_dir()
                    .join(&name); // Unique directory for this specific test's vault

            if !vault_root_path.exists() {
                create_dir_all(&vault_root_path)
                    .expect("Failed to create vault_root_path for KartaServiceTestContext");
            }

            // KartaService::new expects storage_dir to be the .karta directory.
            // It will create it if it doesn't exist within the vault_root_path.
            let karta_internal_storage_dir = vault_root_path.join(".karta");

            let service_instance = KartaService::new(
                &name,                      // Name for the agdb database file, etc.
                vault_root_path.clone(),    // This is the root of the user's vault.
                karta_internal_storage_dir, // This is where .karta internal files go.
            );
            let service_arc = Arc::new(RwLock::new(service_instance));

            Self {
                test_name: name,
                service_arc, // Updated field
                vault_root_path,
                measure_perf: false,
                start_time: std::time::Instant::now(),
            }
        }

        pub fn custom_storage(test_name_suffix: &str) -> Self {
            let name = format!("karta_service_test_{}", test_name_suffix);
            let app_name_for_project_dirs = "karta_service_tests";

            let base_test_dir = ProjectDirs::from("com", "karta_server", app_name_for_project_dirs)
                .expect("Failed to get project dirs for KartaServiceTestContext custom_storage")
                .data_dir()
                .join(&name); // Unique directory for this test instance

            // The vault itself will be in a 'storage' subdirectory within the test's unique temporary folder.
            let vault_root_path = base_test_dir.join("storage");

            if !vault_root_path.exists() {
                create_dir_all(&vault_root_path).expect(
                    "Failed to create vault_root_path for KartaServiceTestContext custom_storage",
                );
            }

            let karta_internal_storage_dir = vault_root_path.join(".karta");

            let service_instance =
                KartaService::new(&name, vault_root_path.clone(), karta_internal_storage_dir);
            let service_arc = Arc::new(RwLock::new(service_instance));

            Self {
                test_name: name,
                service_arc, // Updated field
                vault_root_path,
                measure_perf: false,
                start_time: std::time::Instant::now(),
            }
        }

        pub fn measure_perf(mut self) -> Self {
            self.measure_perf = true;
            self
        }

        pub fn get_vault_root(&self) -> &PathBuf {
            &self.vault_root_path
        }

        // New closure-based accessors
        pub fn with_service<F, R>(&self, operation: F) -> R
        where
            F: FnOnce(&KartaService) -> R,
        {
            let service_guard = self.service_arc.read().unwrap_or_else(|e| panic!("Failed to acquire read lock on KartaService: {}", e));
            operation(&*service_guard)
        }

        pub fn with_service_mut<F, R>(&self, operation: F) -> R
        where
            F: FnOnce(&mut KartaService) -> R,
        {
            let mut service_guard = self.service_arc.write().unwrap_or_else(|e| panic!("Failed to acquire write lock on KartaService: {}", e));
            operation(&mut *service_guard)
        }

        pub fn with_graph_db<F, R>(&self, operation: F) -> R
        where
            F: FnOnce(&GraphAgdb) -> R,
        {
            self.with_service(|s| operation(s.data()))
        }
        
        pub fn with_graph_db_mut<F, R>(&self, operation: F) -> R
        where
            F: FnOnce(&mut GraphAgdb) -> R,
        {
            self.with_service_mut(|s| operation(s.data_mut()))
        }

        pub fn with_context_db<F, R>(&self, operation: F) -> R
        where
            F: FnOnce(&ContextDb) -> R,
        {
            self.with_service(|s| operation(s.view()))
        }

        pub fn with_context_db_mut<F, R>(&self, operation: F) -> R
        where
            F: FnOnce(&mut ContextDb) -> R,
        {
            self.with_service_mut(|s| operation(s.view_mut()))
        }

        pub fn create_file_in_vault(
            &self,
            relative_path_str: &str,
            content: &[u8],
        ) -> Result<PathBuf, IoError> {
            let full_path = self.vault_root_path.join(relative_path_str);
            if let Some(parent) = full_path.parent() {
                if !parent.exists() {
                    create_dir_all(parent)?;
                }
            }
            let mut file = File::create(&full_path)?;
            file.write_all(content)?;
            Ok(full_path)
        }

        pub fn create_dir_in_vault(&self, relative_path_str: &str) -> Result<PathBuf, IoError> {
            let full_path = self.vault_root_path.join(relative_path_str);
            create_dir_all(&full_path)?;
            Ok(full_path)
        }
    }

    impl Drop for KartaServiceTestContext {
        fn drop(&mut self) {
            use git2::Repository;

            if self.measure_perf {
                let elapsed = self.start_time.elapsed().as_millis();
                let db_size = self.with_service(|s| s.data().db().size()); // Updated access
                let commit = match Repository::open(".") {
                    Ok(repo) => repo
                        .head()
                        .ok()
                        .and_then(|head| head.peel_to_commit().ok())
                        .map_or_else(
                            || "unknown_commit".to_string(),
                            |commit_obj| commit_obj.id().to_string(),
                        ),
                    Err(_) => "unknown_commit".to_string(),
                };

                let report = PerfReport {
                    commit,
                    elapsed_ms: elapsed as u64,
                    db_size_bytes: db_size,
                    timestamp: SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap()
                        .as_secs()
                        .to_string(),
                };

                let perf_report_dir = PathBuf::from("docs/perf_reports");
                create_dir_all(&perf_report_dir).unwrap();
                let perf_file_path = perf_report_dir.join(format!("{}.ron", self.test_name));

                let mut reports: Vec<PerfReport> = if perf_file_path.exists() {
                    File::open(&perf_file_path).ok().and_then(|mut file| {
                        let mut contents = String::new();
                        file.read_to_string(&mut contents).ok()?;
                        if contents.is_empty() { Some(Vec::new()) } else { ron::from_str(&contents).ok() }
                    }).unwrap_or_else(|| {
                        eprintln!("Could not read or parse existing perf report: {:?}. Starting fresh.", perf_file_path);
                        Vec::new()
                    })
                } else {
                    Vec::new()
                };
                reports.push(report);
                let pretty = PrettyConfig::new()
                    .separate_tuple_members(true)
                    .enumerate_arrays(true);
                let ser = to_string_pretty(&reports, pretty).unwrap();
                File::create(perf_file_path)
                    .ok()
                    .and_then(|mut file| file.write_all(ser.as_bytes()).ok());
            }

            // Remove the entire temporary vault directory for this KartaServiceTestContext instance
            if let Err(e) = std::fs::remove_dir_all(&self.vault_root_path) {
                eprintln!(
                    "Failed to remove KartaServiceTestContext directory {:?}: {}",
                    self.vault_root_path, e
                );
            }
        }
    }
}
