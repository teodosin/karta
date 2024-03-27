use std::path::PathBuf;

use agdb::{DbError, DbId, DbValue, QueryBuilder, UserValue};
use path_ser::{buf_to_str, str_to_buf};

mod path_ser;

/// The main graph structure to be interacted with.
struct Graph {
    /// The name of the application using this library.
    name: String,

    /// AGDB database
    db: agdb::Db,

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
    fn new(root_path: PathBuf, name: &str) -> Self {
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
            Node {
                db_id: None,
                path: NodePath("root".into()),
                ntype: NodeType::Directory,
            },
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
    fn new_custom_storage(root_path: PathBuf, name: &str, storage_path: PathBuf) -> Self {
        // Create the path if it doesn't exist
        if !storage_path.exists() {
            std::fs::create_dir_all(&storage_path).expect("Failed to create storage path");
        }

        let db = agdb::Db::new(storage_path.join(name).to_str().unwrap());

        let mut db = db.expect("Failed to create db");

        // Create the root node
        let root: Vec<Node> = vec![
            Node {
                db_id: None,
                path: NodePath("root".into()),
                ntype: NodeType::Directory,
            },
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

    /// Set whether the library should maintain readable files for the nodes in the graph.
    fn maintain_readable_files(&mut self, maintain: bool) {
        self.maintain_readable_files = maintain;
    }

    /// Opens the connections of a particular node.
    /// Takes in the path to the node relative to the root of the graph.
    fn open_node(&self, path: PathBuf) {
        // Resolve the full path to the node
        let full_path = self.root_path.join(path);

        // Check if the node is physical
        if full_path.exists() {
            ()
        } else {
            ()
        }
    }

    fn insert_node(&self, node: Node) -> Result<(), agdb::DbError> {
        Ok(())
    }

    fn insert_edge(&self, edge: Edge) -> Result<(), agdb::DbError> {
        Ok(())
    }
}

/// The universal node type. 
#[derive(Debug, UserValue)]
struct Node {
    /// The id of the node in the database.
    db_id: Option<DbId>,
    /// The path of the node relative to the root of the graph.
    /// The path is stored as a string in the database, but is converted to a PathBuf when
    /// the node is loaded.
    path: NodePath,
    ntype: NodeType,
    //attributes: Vec<Attribute>,
}

/// Newtype wrapper for the node path. 
#[derive(Debug, Clone)]
pub struct NodePath(PathBuf);

impl TryFrom<DbValue> for NodePath {
    type Error = DbError;

    fn try_from(value: DbValue) -> Result<Self, Self::Error> {
        Ok(NodePath(str_to_buf(&value.to_string())))
    }
}

impl From<NodePath> for DbValue {
    fn from(path: NodePath) -> Self {
        buf_to_str(path.0).into()
    }
}

#[derive(Debug, Clone)]
enum NodeType {
    Directory,
    File,
}

// TODO: Could a macro be created for this?
impl TryFrom<DbValue> for NodeType {
    type Error = DbError;

    fn try_from(value: DbValue) -> Result<Self, Self::Error> {
        match value.to_string().as_str() {
            "Directory" => Ok(NodeType::Directory),
            "File" => Ok(NodeType::File),
            _ => Err(DbError::from("Invalid NodeType")),
        }
    }
}

impl From<NodeType> for DbValue {
    fn from(ntype: NodeType) -> Self {
        match ntype {
            NodeType::Directory => "Directory".into(),
            NodeType::File => "File".into(),
        }
    }
}

struct Edge {
    attributes: Vec<Attribute>,
}

#[derive(Clone)]
struct Attribute {
    name: String,
    value: f32,
}

/// A list of reserved node attribute names that cannot be used by the user.
const RESERVED_NODE_ATTRS: [&str; 2] = [
    "name", // The full path of the node
    "ntype", // The type of the node
];
/// A list of reserved edge attribute names that cannot be used by the user.
const RESERVED_EDGE_ATTRS: [&str; 1] = [
    "contains", // For directories
];

// Tests
// -----------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use agdb::QueryResult;
    use directories::ProjectDirs;

    use super::*;

    /// Setup function for tests. Always stores the db in the data_dir.
    fn setup(test_name: &str) -> Graph {
        let name = format!("fs_graph_test_{}", test_name);
        let root = ProjectDirs::from("com", "fs_graph", &name)
            .unwrap()
            .data_dir()
            .to_path_buf();
        let full_path = root.join(&name);

        let graph = Graph::new(root.clone().into(), &name);

        assert_eq!(
            full_path.exists(),
            true,
            "Test directory has not been created"
        );

        graph
    }

    /// Cleanup function for tests. Removes the root directory from the data_dir.
    fn cleanup(test_name: &str) {
        // Uncomment this return only if you need to temporarily look at the contents
        // return;

        let name = format!("fs_graph_test_{}", test_name);
        let root = ProjectDirs::from("com", "fs_graph", &name)
            .unwrap()
            .data_dir()
            .to_path_buf();

        std::fs::remove_dir_all(root).expect("Failed to remove root directory");
    }

    #[test]
    fn test_new_graph() {
        let func_name = "test_new_graph";

        let name = format!("fs_graph_test_{}", func_name);
        let root = ProjectDirs::from("com", "fs_graph", &name)
            .unwrap()
            .data_dir()
            .to_path_buf();

        println!("Expected full path: {:?}", root);

        let graph = Graph::new(root.clone().into(), &name);

        println!("Size of graph: {:?} bytes", graph.db.size());

        assert_eq!(root.exists(), true, "Root directory does not exist");

        // Check that there exists a root node
        let root_node_result = graph.db.exec(&QueryBuilder::select().ids("root").query());

        match root_node_result {
            Ok(root_node) => {
                assert_eq!(root_node.result /* expected value */, 1);
            }
            Err(e) => {
                println!("Failed to execute query: {}", e);
            }
        }

        cleanup(func_name);
    }

    #[test]
    fn existing_db_in_directory() {
        // We add a node to the db, then create a new graph with the same name.
        // The new graph should be able to access the node.
        let func_name = "existing_db_in_directory";
        let mut first = setup(func_name);

        let _ = first
            .db
            .exec_mut(&QueryBuilder::insert().nodes().aliases("testalias").query());

        let second = setup(func_name);

        let root_node_result = second
            .db
            .exec(&QueryBuilder::select().ids("testalias").query());

        match root_node_result {
            Ok(root_node) => {
                assert_eq!(root_node.result /* expected value */, 1);
            }
            Err(e) => {
                println!("Failed to execute query: {}", e);
            }
        }

        assert_eq!(true, true);

        cleanup(func_name);
    }

    #[test]
    fn new_custom_storage_directory(){
        let func_name = "new_custom_storage_directory";
        let name = format!("fs_graph_test_{}", func_name);
        let root = ProjectDirs::from("com", "fs_graph", &name)
            .unwrap()
            .config_dir()
            .to_path_buf();
        let storage = root.join("storage");

        let graph = Graph::new_custom_storage(root.clone().into(), &name, storage.clone());

        assert_eq!(
            storage.exists(),
            true,
            "Storage directory has not been created"
        );

        // Clean up the custom storage directory
        std::fs::remove_dir_all(storage).expect("Failed to remove storage directory");
    }
}
