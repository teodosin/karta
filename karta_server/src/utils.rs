// Utility functions for the library tests.

#![allow(warnings)]
#[cfg(test)]
pub mod utils {

    use std::{
        io::{Read, Write},
        path::{Path, PathBuf},
        time::SystemTime,
    };

    use directories::ProjectDirs;

    use ron::ser::{to_string_pretty, PrettyConfig};
    use serde::{Deserialize, Serialize};

    use crate::{
        graph_agdb::GraphAgdb,
        graph_traits::{graph_core::GraphCore, Graph},
    };

    pub struct TestContext {
        pub test_name: String,
        pub graph: GraphAgdb,
        start_time: std::time::Instant,
    }

    #[derive(Serialize, Deserialize)]
    struct PerfReport {
        commit: String,
        elapsed_ms: u64,
        db_size_bytes: u64,
        timestamp: String,
    }

    impl TestContext {
        pub fn new(name: &str) -> Self {
            let name = format!("fs_graph_test_{}", name);

            Self {
                test_name: name.to_string(),
                graph: TestContext::setup(&name, None),
                start_time: std::time::Instant::now(),
            }
        }

        pub fn custom_storage(name: &str) -> Self {
            let name = format!("fs_graph_test_{}", name);

            Self {
                test_name: name.to_string(),
                graph: TestContext::setup(&name, Some(PathBuf::from("storage"))),
                start_time: std::time::Instant::now(),
            }
        }

        /// Graph setup function for tests. Always stores the db in the data_dir.
        fn setup(test_name: &str, storage: Option<PathBuf>) -> GraphAgdb {
            // let test_name = self.test_name.clone();
            let strg_name = "fs_graph";

            let root = ProjectDirs::from("com", "fs_graph", strg_name)
                .unwrap()
                .data_dir()
                .to_path_buf();

            let full_path = root.join(&test_name);
            let strg_dir = match storage {
                Some(path) => full_path.join(path),
                None => full_path.clone(),
            };

            println!("Trying to create test directory: {:#?}", full_path);

            let graph = GraphAgdb::new(&test_name, full_path.clone(), Some(strg_dir));

            assert_eq!(
                full_path.exists(),
                true,
                "Test directory has not been created"
            );

            graph
        }
    }

    impl Drop for TestContext {
        fn drop(&mut self) {
            // Uncomment this return only if you need to temporarily look at the contents
            // return;

            use git2::Repository;

            let name = &self.test_name;

            let compile_report = false;
            if compile_report {
                // Compile a performance report and append it to the tests file
                let elapsed = self.start_time.elapsed().as_millis();
                let db_size = self.graph.db().size();
                let commit = {
                    let repo = Repository::open(".").expect("Failed to open repository");
                    let head = repo.head().expect("Failed to get HEAD");
                    let commit = head.peel_to_commit().expect("Failed to peel the commit");
                    commit.id().to_string()
                };

                // Format the report as a ron object
                let report = PerfReport {
                    commit,
                    elapsed_ms: elapsed as u64,
                    db_size_bytes: db_size,
                    timestamp: std::time::SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap()
                        .as_secs()
                        .to_string(),
                };

                let perf_report_dir = PathBuf::from("docs/perf_reports");
                std::fs::create_dir_all(&perf_report_dir).unwrap();
                let perf_file_path = perf_report_dir.join(format!("{}.ron", self.test_name));

                let mut file = std::fs::OpenOptions::new()
                    .create(true)
                    .read(true)
                    .append(true)
                    .open(perf_file_path)
                    .unwrap();

                let mut contents = String::new();
                file.read_to_string(&mut contents).unwrap();

                let mut reports: Vec<PerfReport> = if contents.is_empty() {
                    Vec::new()
                } else {
                    ron::from_str(&contents).unwrap()
                };

                reports.push(report);

                let pretty = PrettyConfig::new()
                    .separate_tuple_members(true)
                    .enumerate_arrays(true);
                let ser = to_string_pretty(&reports, pretty).unwrap();

                // Append to end of file
                file.set_len(0).unwrap();
                std::io::Seek::seek(&mut file, std::io::SeekFrom::Start(0)).unwrap();
                file.write_all(ser.as_bytes()).unwrap();
            }

            // Find and remove test db
            let root = ProjectDirs::from("com", "fs_graph", "fs_graph")
                .unwrap()
                .data_dir()
                .to_path_buf();

            let full_path = root.join(name);

            let removal = std::fs::remove_dir_all(full_path);

            match removal {
                Ok(_) => {}
                Err(_err) => {
                    //println!("Failed to remove test directory: {}", err);
                }
            }
        }
    }
}
