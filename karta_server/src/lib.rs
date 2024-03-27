use std::path::PathBuf;

use agdb::{CountComparison, DbUserValue, QueryBuilder};
use elements::*;
use path_ser::buf_to_str;

mod elements;
mod path_ser;

/// The main graph structure to be interacted with.
pub struct Graph {
    /// The name of the application using this library.
    name: String,

    /// AGDB database.
    /// Set to public, though direct access to the db is discouraged.
    pub db: agdb::Db,

    /// Path to the root directory of the graph.
    /// All paths are relative to this root.
    root_path: std::path::PathBuf,

    /// Path to the where the db is stored in the file system.
    /// Either default for the operating system (as determined by the directories crate) or custom.
    /// Includes the name of the directory.  
    storage_path: StoragePath,

    /// Whether the library should maintain readable files for the nodes
    /// in the graph.
    ///
    /// If true, there will be a directory at the storage path which
    /// mirrors the directory structure starting from the root path.
    /// TODO: Should this be behind a feature flag?
    maintain_readable_files: bool,
}

enum StoragePath {
    Default,
    Custom(PathBuf),
}

/// Agdb has multiple implementations. If the size of the database is small enough, it can be stored in memory.
/// If the database is too large, it can be stored in a file.
/// TODO: Not in use currently.
enum GraphDb {
    Mem(agdb::Db),
    File(agdb::DbFile),
}

impl Graph {
    /// Constructor. Panics if the db cannot be created.
    ///
    /// Takes the desired root of the graph as a parameter and the name for the db.
    ///
    /// Creates the db at the storage_path, or initialises the db if it already exists there.
    ///
    /// TODO: Add error handling.
    pub fn new(root_path: PathBuf, name: &str) -> Self {
        let storage_path = directories::ProjectDirs::from("com", "fs_graph", name)
            .unwrap()
            .data_dir()
            .to_path_buf();

        // Create the path if it doesn't exist
        if !storage_path.exists() {
            std::fs::create_dir_all(&storage_path).expect("Failed to create storage path");
        }

        let db = agdb::Db::new(storage_path.join(name).to_str().unwrap());

        let mut db = db.expect("Failed to create db");

        // Create the root node
        let root: Vec<Node> = vec![
            Node::new(NodePath("root".into()), NodeType::Directory),
        ];

        let _ = db.exec_mut(&QueryBuilder::insert().nodes().aliases("root").values(&root).query());

        Graph {
            name: name.to_string(),
            db,
            root_path: root_path.into(),
            storage_path: StoragePath::Default,
            maintain_readable_files: false,
        }
    }

    /// Alternate constructor. Use this if you want to set a custom storage path for the db. Panics if the db cannot be created
    pub fn new_custom_storage(root_path: PathBuf, name: &str, storage_path: PathBuf) -> Self {
        // Create the path if it doesn't exist
        if !storage_path.exists() {
            std::fs::create_dir_all(&storage_path).expect("Failed to create storage path");
        }

        let db = agdb::Db::new(storage_path.join(name).to_str().unwrap());

        let mut db = db.expect("Failed to create db");

        // Create the root node
        let root: Vec<Node> = vec![
            Node:: new(NodePath("root".into()), NodeType::Directory),
        ];

        let _ = db.exec_mut(&QueryBuilder::insert().nodes().aliases("root").values(&root).query());

        Graph {
            name: name.to_string(),
            db,
            root_path: root_path.into(),
            storage_path: StoragePath::Custom(storage_path),
            maintain_readable_files: false,
        }
    }

    /// Set whether the library should maintain readable files for the nodes in the graph.
    pub fn maintain_readable_files(&mut self, maintain: bool) {
        self.maintain_readable_files = maintain;
    }

    /// Retrieves a particular node's data from the database.
    /// The path is relative to the root of the graph.
    pub fn open_node(&self, path: PathBuf){

    }

    /// Opens the connections of a particular node.
    /// Takes in the path to the node relative to the root of the graph.
    /// TODO: Add filter argument
    pub fn open_node_connections(&self, path: PathBuf) -> Vec<Node> {
        // Step 1: Check if the node is a physical node in the file system.
        // Step 2: Check if the node exists in the db.
        // Step 3: Check if all the physical dirs and files in the node are in the db.
        // Step 4: The ones that are not, add to the db.
        // Step 5?: Delete the physical nodes in the db that are not in the file system. 
        // THOUGH Automatically deleting the nodes 
        // honestly seems like a bad idea. Maybe a warning should be issued instead. 


        // Resolve the full path to the node
        let full_path = self.root_path.join(path);

        let is_physical = full_path.exists();

        let as_str = buf_to_str(full_path);

        let mut nodes: Vec<Node> = Vec::new();

        // Query the db for the node
        let result = 
            self.db.exec(
                &QueryBuilder::select()
                    .values(Node::db_keys())
                    .ids(
                        QueryBuilder::search()
                            .depth_first()
                            .from("as_str")
                            .where_()
                            .node()
                            .and()
                            .distance(CountComparison::GreaterThan(1))
                            .query(),
                    )
                    .query(),
            );

        match result {
            Ok(node) => {
                let db_nodes: Vec<Node> = node.try_into().unwrap();
            }
            Err(e) => {
                println!("Failed to execute query: {}", e);
                // If the node is not a physical node in the file system, nor a virtual node in the db, it doesn't exist.
                if !is_physical {
                    return nodes;
                }
            }
        }

        nodes

    }

    pub fn insert_node(&self, node: Node) -> Result<(), agdb::DbError> {
        Ok(())
    }

    pub fn insert_edge(&self, edge: Edge) -> Result<(), agdb::DbError> {
        Ok(())
    }
}

