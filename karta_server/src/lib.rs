use std::path::PathBuf;

use agdb::{CountComparison, DbError, DbUserValue, QueryBuilder, QueryError};
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

    /// For physical nodes. Syncs the node's relationships in the db with the file system.
    fn index_node_connections(&self, path: PathBuf){
        let full_path = self.root_path.join(&path);

        if !full_path.exists() {
            return;
        }

        let alias = buf_to_str(&path);

        // 

        todo!()
    }

    /// Retrieves a particular node's data from the database.
    /// The path is relative to the root of the graph.
    pub fn open_node(&self, path: PathBuf) -> Node {
        todo!()
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

        let as_str = buf_to_str(&full_path);

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

    /// Creates a node from the given path. Inserts it into the graph. 
    /// Insert the relative path from the root. 
    pub fn insert_node(&self, path: PathBuf) -> Result<(), agdb::DbError> {
        let full_path = self.root_path.join(&path);


        let alias = buf_to_str(&path);


        Ok(())
    }

    /// Changes the parent directory of a node. If the node is physical, it will be moved in the file system.
    /// If the node is virtual, the parent will be changed in the db.
    /// Note that due to the implementation, all children of the node will have to be reindexed. 
    pub fn change_node_parent(&self, node_path: PathBuf, new_parent_path: PathBuf) -> Result<(), agdb::DbError> {
        // Check if node is in database at all
        let alias = buf_to_str(&node_path);
        let existing = self.db.exec(&QueryBuilder::select().ids(alias).query());
        match existing {
            QueryError => {
                return Err(DbError::from("Node does not exist in the database"));
            }
            QueryResult => {},
        }
        Ok(())
    }

    pub fn insert_edge(&self, edge: Edge) -> Result<(), agdb::DbError> {
        Ok(())
    }

    pub fn delete_node(&self, path: PathBuf) -> Result<(), agdb::DbError> {
        Ok(())
    }

    /// Delete an edge from the graph. Edges with the attribute "contains" refer to the parent-child relationship
    /// between nodes and will be ignored. All other attributes will be cleared from them instead. 
    pub fn delete_edge(&self, edge: Edge) -> Result<(), agdb::DbError> {
        Ok(())
    }

    /// Insert attributes to a node. Ignore reserved attribute names. 
    pub fn insert_node_attr(&self, path: PathBuf, attr: Vec<Attribute>) -> Result<(), agdb::DbError> {
        Ok(())
    }

    /// Delete attributes from a node. Ignore reserved attribute names.
    pub fn delete_node_attr(&self, path: PathBuf, attr: Vec<Attribute>) -> Result<(), agdb::DbError> {
        Ok(())
    }

    /// Insert attributes to an edge. Ignore reserved attribute names.
    pub fn insert_edge_attr(&self, edge: Edge, attr: Vec<Attribute>) -> Result<(), agdb::DbError> {
        Ok(())
    }

    /// Delete attributes from an edge. Ignore reserved attribute names.
    pub fn delete_edge_attr(&self, edge: Edge, attr: Vec<Attribute>) -> Result<(), agdb::DbError> {
        Ok(())
    }


}

