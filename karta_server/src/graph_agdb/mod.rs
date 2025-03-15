use std::{error::Error, path::PathBuf};

use agdb::{CountComparison, DbElement, DbError, DbId, DbUserValue, QueryBuilder, QueryError};
use crate::graph_traits::{Graph, StoragePath};

pub (crate) mod graph_core;
pub (crate) mod graph_node;
pub (crate) mod graph_edge;

/// The main graph structure to be interacted with.
///
/// bevy_karta_client will instantiate this as a Resource through a newtype.
pub struct GraphAgdb {
    /// The name of the application using this library.
    name: String,

    /// AGDB database.
    db: agdb::Db,

    /// Path to the root directory of the graph.
    /// All paths are relative to this root.
    root_path: std::path::PathBuf,

    /// Path to the where the db is stored in the file system.
    /// Either default for the operating system (as determined by the directories crate) or custom.
    /// Includes the name of the directory.  
    storage_path: StoragePath,
}


/// Agdb has multiple implementations. If the size of the database is small enough, it can be stored in memory.
/// If the database is too large, it can be stored in a file.
/// TODO: Not in use currently.
enum GraphDb {
    Mem(agdb::Db),
    File(agdb::DbFile),
}

impl Graph for GraphAgdb {}

impl GraphAgdb {
    /// Direct getter for the db. Not recommended to use. If possible, 
    /// use the other implemented functions. They are the intended way
    /// of interacting with the db.
    pub fn db(&self) -> &agdb::Db {
        &self.db
    }

    /// Direct mutable getter for the db. Not recommended to use. If possible,
    /// use the other implemented functions. They are the intended way
    /// of interacting with the db.
    pub fn db_mut(&mut self) -> &mut agdb::Db {
        &mut self.db
    }
}

