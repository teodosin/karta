use std::path::PathBuf;

use agdb::{UserValue, DbId};

/// The main graph structure to be interacted with. 
struct Graph {
    /// AGDB database
    db: agdb::Db,

    /// Path to the root directory of the graph. 
    /// All paths are relative to this root.
    root_path: std::path::PathBuf,

    /// Path to the where the db is stored in the file system.
    /// Includes the name of the directory.  
    storage_path: std::path::PathBuf,

    /// Whether the library should maintain readable files for the nodes
    /// in the graph.
    /// 
    /// If true, there will be a directory at the storage path which
    /// mirrors the directory structure starting from the root path. 
    maintain_readable_files: bool,
}

/// Agdb has multiple implementations. If the size of the database is small enough, it can be stored in memory.
/// If the database is too large, it can be stored in a file. 
/// TODO: Not in use currently. 
enum graph_db {
    Mem(agdb::Db),
    File(agdb::DbFile),
}

impl Graph {

    /// Constructor. Panics if the db cannot be created.
    /// TODO: Add error handling.
    fn new(
        root_path: &str, 
        storage_path: &str, 
    ) -> Self {
        let db = agdb::Db::new(storage_path);

        Graph { 
            db: db.expect("Failed to create db"), 
            root_path: root_path.into(), 
            storage_path: storage_path.into(),
            maintain_readable_files: false,
        }
    }

    fn maintain_readable_files(&self, maintain: bool) {
        self.maintain_readable_files = maintain;
    }

    /// Opens the connections of a particular node.
    /// Insert the path to the node relative to the root of the graph.
    fn open_node(&self, node_path: &str) {

        // Resolve the full path to the node
        let full_path: PathBuf = self.root_path.join(node_path);

        // Check if the node is physical
        if full_path.exists() {

            return Ok(());
        } else {
            return Ok(());
        }
    }

    fn insert_node(&self, node: Node) -> Result<(), agdb::DbError> {
        Ok(())
    }

    fn insert_edge(&self, edge: Edge) -> Result<(), agdb::DbError> {
        Ok(())
    }
}

#[derive(UserValue)]
struct Node {
    db_id: Option<DbId>,
    name: String,
    attributes: Vec<Attribute>,
}

struct Edge {
    attributes: Vec<Attribute>,
}

#[derive(Clone)]
struct Attribute {
    name: String,
    value: String,
}

// Tests
// -----------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_graph() {
        let graph = Graph::new("root", "storage");
        assert_eq!(graph.root_path, PathBuf::from("root"));
        assert_eq!(graph.storage_path, PathBuf::from("storage"));
    }
}


