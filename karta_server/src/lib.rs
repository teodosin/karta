use std::{error::Error, path::PathBuf};

use agdb::{CountComparison, DbElement, DbError, DbId, DbUserValue, QueryBuilder, QueryError};
use elements::*;
use path_ser::buf_to_alias;

pub mod elements;
pub mod path_ser;

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

/// Implementation block for the Graph struct itself.
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
        let root: Vec<Node> = vec![Node::new(NodePath("root".into()), NodeType::Directory)];

        let rt_node = db.exec_mut(
            &QueryBuilder::insert()
                .nodes()
                .aliases("root")
                .values(&root)
                .query(),
        );
        match rt_node {
            Ok(_) => {
                println!("Created root node");
            }
            Err(ref err) => {
                println!("Failed to create root node: {}", err);
            }
        }
        let rt_id = rt_node.unwrap().ids();
        let rt_id = rt_id.first().unwrap();

        // Create attributes node
        // All user-defined attributes will be children of this node
        let atr: Vec<Node> = vec![Node::new(
            NodePath("root/attributes".into()),
            NodeType::Category,
        )];

        let atr_node = db.exec_mut(
            &QueryBuilder::insert()
                .nodes()
                .aliases("root/attributes")
                .values(&atr)
                .query(),
        );
        match atr_node {
            Ok(_) => {
                println!("Created attributes node");
            }
            Err(ref err) => {
                println!("Failed to create attributes node: {}", err);
            }
        }

        // Create an edge between the root and attributes nodes
        parent_node_to_node(&mut db, rt_id, atr_node.unwrap().ids().first().unwrap());

        // Create the settings node for global application settings
        let set: Vec<Node> = vec![Node::new(
            NodePath("root/settings".into()),
            NodeType::Category,
        )];

        let set_node = db.exec_mut(
            &QueryBuilder::insert()
                .nodes()
                .aliases("root/settings")
                .values(&set)
                .query(),
        );
        match set_node {
            Ok(_) => {
                println!("Created settings node");
            }
            Err(ref err) => {
                println!("Failed to create settings node: {}", err);
            }
        }

        // Create an edge between the root and settings nodes
        parent_node_to_node(&mut db, rt_id, set_node.unwrap().ids().first().unwrap());

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
        let root: Vec<Node> = vec![Node::new(NodePath("root".into()), NodeType::Directory)];

        let rt_node = db.exec_mut(
            &QueryBuilder::insert()
                .nodes()
                .aliases("root")
                .values(&root)
                .query(),
        );
        match rt_node {
            Ok(_) => {
                println!("Created root node");
            }
            Err(ref err) => {
                println!("Failed to create root node: {}", err);
            }
        }

        // Create attributes node
        // All user-defined attributes will be children of this node
        let atr: Vec<Node> = vec![Node::new(
            NodePath("root/attributes".into()),
            NodeType::Category,
        )];

        let atr_node = db.exec_mut(
            &QueryBuilder::insert()
                .nodes()
                .aliases("root/attributes")
                .values(&atr)
                .query(),
        );
        match atr_node {
            Ok(_) => {
                println!("Created attributes node");
            }
            Err(ref err) => {
                println!("Failed to create attributes node: {}", err);
            }
        }

        // Create an edge between the root and attributes nodes
        // Ugly function call, I know.
        if rt_node.is_ok() && atr_node.is_ok() {
            parent_node_to_node(
                &mut db,
                rt_node.unwrap().ids().first().unwrap(),
                atr_node.unwrap().ids().first().unwrap(),
            );
        }

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

    /// Gets the name of the root directory without the full path
    pub fn root_name(&self) -> String {
        self.root_path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string()
    }
}

impl Graph {
    /// For physical nodes. Syncs the node's relationships in the db with the file system.
    pub fn index_node_connections(&self, path: PathBuf) {
        let full_path = self.root_path.join(&path);

        if !full_path.exists() {
            return;
        }

        let alias = buf_to_alias(&path);

        //

        todo!()
    }

    /// Retrieves a particular node's data from the database.
    /// The path is relative to the root of the graph.
    pub fn open_node(&self, path: PathBuf) -> Result<Node, DbError> {
        let alias = buf_to_alias(&path);

        let node = self.db.exec(
            &QueryBuilder::select()
            .ids(alias)
            .query(),
        );

        match node {
            Ok(node) => {
                let node = node.elements.first().unwrap().clone();
                let node = Node::try_from(node);

                node
            },
            Err(_err) => {
                return Err("Could not open node".into());
            }
        }
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

        let as_str = buf_to_alias(&full_path);

        let mut nodes: Vec<Node> = Vec::new();

        // Query the db for the node
        let result = self.db.exec(
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
    /// Insert the relative path from the root, not including the root dir.
    pub fn insert_node_by_path(
        &mut self,
        path: PathBuf,
        ntype: Option<NodeType>,
    ) -> Result<DbElement, agdb::DbError> {
        let full_path = self.root_path.join(&path);
        let alias = buf_to_alias(&path);

        // Check if the node already exists in the db.
        // If it does, don't insert it, and return an error.
        // Possibly redundant, unless used for updating an existing node.
        let existing = self
            .db
            .exec(&QueryBuilder::select().ids(alias.clone()).query());

        match existing {
            Ok(_) => {
                return Err("Node already exists in the db".into());
            }
            Err(_e) => {
                // Node doesn't exist, proceed to insertion
            }
        }

        // Determine type of node. If not specified, it's an Other node.
        let mut ntype = match ntype {
            Some(ntype) => ntype,
            None => NodeType::Other,
        };

        // Check if the node is physical in the file system.
        // If it is, check if it exists in the db.
        let is_file = full_path.exists() && !full_path.is_dir();
        let is_dir = full_path.is_dir();

        if is_file {
            ntype = NodeType::File;
        } else if is_dir {
            ntype = NodeType::Directory;
        }

        let node = Node::new(NodePath(PathBuf::from(&path)), ntype);

        let nodeqr = self.db.exec_mut(
            &QueryBuilder::insert()
                .nodes()
                .aliases(alias)
                .values(&node)
                .query(),
        );

        match nodeqr {
            Ok(nodeqr) => {
                let node_elem = &nodeqr.elements[0];
                let nid = node_elem.id;
                // If parent is not root, check if the parent node already exists in the db.
                // If not, call this function recursively.
                let parent_path = path.parent();
                match parent_path {
                    Some(parent_path) => {

                        if parent_path.to_str().unwrap() != "" {

                            println!("About to insert parent node: {:?}", parent_path);
                            let n = self.insert_node_by_path(
                                parent_path.to_path_buf(),
                                Some(NodeType::Other),
                            );
                            
                            let parent_id = n.unwrap().id;
                            
                            parent_node_to_node(&mut self.db, &parent_id, &nid)
                        }
                    }
                    None => {
                        // If the parent is root, parent them and move along. 
                        parent_node_to_node(&mut self.db, &DbId(1), &nid)
                    }
                }

                return Ok(<DbElement as Clone>::clone(node_elem)
                    .try_into()
                    .unwrap());
            }
            Err(e) => {
                println!("Failed to insert node: {}", e);
                return Err(DbError::from(e.to_string()));
            }
        }
    }

    /// Creates a node under a given parent with the given name.
    /// The path is relative to the root of the graph.
    /// Do not include the root dir name.
    pub fn insert_node_by_name(
        &mut self,
        parent_path: Option<PathBuf>,
        name: &str,
        ntype: Option<NodeType>,
    ) -> Result<(), agdb::DbError> {
        let parent_path = parent_path.unwrap_or_else(|| PathBuf::from(""));

        let rel_path = if parent_path.as_os_str().is_empty() {
            PathBuf::from(name)
        } else {
            parent_path.join(name)
        };

        match self.insert_node_by_path(rel_path, ntype) {
            Ok(_) => {
                return Ok(());
            }
            Err(e) => {
                println!("Failed to insert node: {}", e);
                return Err(e);
            }
        }
    }

    /// Changes the parent directory of a node. If the node is physical, it will be moved in the file system.
    /// If the node is virtual, the parent will be changed in the db.
    /// Note that due to the implementation, all children of the node will have to be reindexed.
    pub fn reparent_node(
        &self,
        node_path: PathBuf,
        new_parent_path: PathBuf,
    ) -> Result<(), agdb::DbError> {
        // Check if node is in database at all
        let alias = buf_to_alias(&node_path);
        let existing = self.db.exec(&QueryBuilder::select().ids(alias).query());
        match existing {
            QueryError => {
                return Err(DbError::from("Node does not exist in the database"));
            }
            QueryResult => {}
        }
        Ok(())
    }

    /// Moves an edge and all its attributes to a new source and target. Parent edges can't be reconnected this way,
    /// use the reparent_node function instead.
    pub fn reconnect_edge(
        &self,
        edge: Edge,
        from: PathBuf,
        to: PathBuf,
    ) -> Result<(), agdb::DbError> {
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

    /// Insert attributes to a node. Ignore reserved attribute names. Update attributes that already exist.
    pub fn insert_node_attr(
        &mut self,
        path: PathBuf,
        attr: Attribute,
    ) -> Result<(), agdb::DbError> {

        use elements::RESERVED_NODE_ATTRS;
        let slice = attr.name.as_str();
        let is_reserved = RESERVED_NODE_ATTRS.contains(&slice);

        if is_reserved {
            return Err(DbError::from(format!(
                "Cannot insert reserved attribute name: {}",
                slice
            )));
        }

        let alias = buf_to_alias(&path);
        let added = self.db.exec_mut(
            &QueryBuilder::insert()
               .values(vec![attr.into()])
               .ids(alias)
               .query(),
        );

        println!("Added: {:?}", added);

        match added {
            QueryResult => {
                println!("Yes it's ok");
                return Ok(());
            }
            QueryError => {
                return Err(DbError::from("Failed to insert attribute"));
            }
        }
    }

    /// Delete attributes from a node. Ignore reserved attribute names.
    pub fn delete_node_attr(
        &mut self,
        path: PathBuf,
        attr_name: &str,
    ) -> Result<(), agdb::DbError> {
        use elements::RESERVED_NODE_ATTRS;
        let is_reserved = RESERVED_NODE_ATTRS.contains(&attr_name);

        if is_reserved {
            return Err(DbError::from(format!(
                "Cannot delete reserved attribute name: {}",
                attr_name
            )));
        }

        let alias = buf_to_alias(&path);

        let node = self.db.exec_mut(
            &QueryBuilder::remove()
                .values(vec![attr_name.into()])
                .ids(alias)
                .query(),
        );

        match node {
            QueryResult => {
                return Ok(());
            }
            QueryError => {
                return Err(DbError::from("Failed to delete attribute"));
            }
        }
    }

    /// Insert attributes to an edge. Ignore reserved attribute names. Update attributes that already exist.
    pub fn insert_edge_attr(&self, edge: Edge, attr: Attribute) -> Result<(), agdb::DbError> {
        use elements::RESERVED_EDGE_ATTRS;
        let slice = attr.name.as_str();
        let is_reserved = RESERVED_EDGE_ATTRS.contains(&slice);

        if is_reserved {
            return Err(DbError::from(format!(
                "Cannot delete reserved attribute name: {}",
                slice
            )));
        }

        Ok(())
    }

    /// Delete attributes from an edge. Ignore reserved attribute names.
    pub fn delete_edge_attr(&self, edge: Edge, attr: Attribute) -> Result<(), agdb::DbError> {
        use elements::RESERVED_EDGE_ATTRS;
        let slice = attr.name.as_str();
        let is_reserved = RESERVED_EDGE_ATTRS.contains(&slice);

        if is_reserved {
            return Err(DbError::from(format!(
                "Cannot insert reserved attribute name: {}",
                slice
            )));
        }
        
        Ok(())
    }

    /// Merges a vector of nodes into one node, the first.
    pub fn merge_nodes(&self, nodes: Vec<PathBuf>) -> Result<(), agdb::DbError> {
        Ok(())
    }
}

/// Internal function. Not for crate users to use directly.
/// Uses agdb types directly to create an exclusive parent-child connection.
/// The attribute is "contains" and is reserved in elements.rs.
fn parent_node_to_node(db: &mut agdb::Db, parent: &agdb::DbId, child: &agdb::DbId) {
    // Check if the child has an existing parent

    // If it does, delete the existing parent-child relationship

    // If it doesn't, create a new parent-child relationship
    let cont_attr = Attribute {
        name: "contains".into(),
        value: 0.0,
    };

    let edge = db.exec_mut(
        &QueryBuilder::insert()
            .edges()
            .from(*parent)
            .to(*child)
            .values_uniform(vec![cont_attr.clone().into()])
            .query(),
    ); // For whatever reason this does not insert the attribute into the edge.

    let eid = edge.unwrap().ids();
    let eid = eid.first().unwrap();
    println!("Id of the edge: {:#?}", eid);

    let edge = db.exec(&QueryBuilder::select().keys().ids(*eid).query());

    match edge {
        Ok(edge) => {
            // Insert the attribute to the edge
            println!("Edge inserted: {:#?}", edge.elements);
        }
        Err(e) => {
            println!("Failed to insert edge: {}", e);
        }
    }
}
