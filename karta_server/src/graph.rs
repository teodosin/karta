use std::{error::Error, path::PathBuf};

use agdb::{CountComparison, DbElement, DbError, DbId, DbUserValue, QueryBuilder, QueryError};

use crate::{elements, nodetype::TypeName, path_ser};
use elements::*;

/// The main graph structure to be interacted with.
///
/// Bevy_fs_graph will instantiate this as a Resource through a newtype.
pub struct Graph {
    /// The name of the application using this library.
    name: String,

    /// AGDB database.
    /// Set to public, though direct access to the db is discouraged.
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

// ------------------------------------------------------------------
// In the event that the backend database is to be changed,
// the following implementations could be turned into traits.
// This would allow for the db to be changed without changing the library.
// Storing the database in text files could be reimplemented this way.
// ------------------------------------------------------------------

/// Implementation block for the Graph struct itself.
/// Includes constructors and utility functions.
impl Graph {
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

    pub fn root_path(&self) -> PathBuf {
        self.root_path.clone()
    }

    pub fn root_nodepath(&self) -> NodePath {
        NodePath::new(self.root_path.clone())
    }

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
        
        let mut giraphe = Graph {
            name: name.to_string(),
            db,
            root_path: root_path.into(),
            storage_path: StoragePath::Default,
            maintain_readable_files: false,
        };

        giraphe.init_archetype_nodes();

        return giraphe;
    }

    /// Alternate constructor. Use this if you want to set a custom storage path for the db.
    /// Panics if the db cannot be created
    pub fn new_custom_storage(root_path: PathBuf, name: &str, storage_path: PathBuf) -> Self {
        // Create the path if it doesn't exist
        if !storage_path.exists() {
            std::fs::create_dir_all(&storage_path).expect("Failed to create storage path");
        }

        let db = agdb::Db::new(storage_path.join(name).to_str().unwrap());

        let mut db = db.expect("Failed to create db");

        let mut giraphe = Graph {
            name: name.to_string(),
            db,
            root_path: root_path.into(),
            storage_path: StoragePath::Custom(storage_path),
            maintain_readable_files: false,
        };

        giraphe.init_archetype_nodes();

        return giraphe;
    }

    /// Create the initial archetype nodes for the graph. Includes 
    /// the root, 
    /// attributes,
    /// settings,
    /// nodecategories
    pub fn init_archetype_nodes(&mut self) {
        
        // Create the root node
        let root: Vec<Node> = vec![Node::new(NodePath::new("root".into()), TypeName::root_type())];

        let rt_node = self.db.exec_mut(
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
            NodePath::new("root/attributes".into()),
            TypeName::archetype_type(),
        )];

        let atr_node = self.db.exec_mut(
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
        Graph::parent_nodes_by_dbids(&mut self.db, rt_id, atr_node.unwrap().ids().first().unwrap());


        // Archetype ------------------------------------------------
        // Create the settings node for global application settings
        let set: Vec<Node> = vec![Node::new(
            NodePath::new("root/settings".into()),
            TypeName::archetype_type(),
        )];

        let set_node = self.db.exec_mut(
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
        Graph::parent_nodes_by_dbids(&mut self.db, rt_id, set_node.unwrap().ids().first().unwrap());


        // Archetype ------------------------------------------------
        // Create the nodecategories node for global node categories.
        // Node types are then children of nodecategories or operators. 
        let nca: Vec<Node> = vec![Node::new(
            NodePath::new("root/nodecategories".into()),
            TypeName::archetype_type(),
        )];

        let nca_node = self.db.exec_mut(
            &QueryBuilder::insert()
                .nodes()
                .aliases("root/nodecategories")
                .values(&nca)
                .query(),
        );
        match nca_node {
            Ok(_) => {
                println!("Created nodecategories node");
            }
            Err(ref err) => {
                println!("Failed to create nodecategories node: {}", err);
            }
        }
        // Create an edge between the root and nodecategories nodes
        Graph::parent_nodes_by_dbids(&mut self.db, rt_id, nca_node.unwrap().ids().first().unwrap());
    }

    /// Syncs a node in the db with the file system
    pub fn index_single_node(&mut self, path: &NodePath) {
        let full_path = path.full(&self.root_path);
        let node_alias = path.alias();

        let is_phys = full_path.exists();
        let is_dir = full_path.is_dir();

        todo!()
    }

    /// Syncs the node's relationships in the db with the file system.
    pub fn index_node_connections(&mut self, path: NodePath) {
        let full_path = path.full(&self.root_path);
        let node_alias = path.alias();

        let is_phys = full_path.exists();
        let is_dir = full_path.is_dir();

        if is_phys {
            // Check if the path has a node in the db. If not, it will be created. 
            let nnode = self.db.exec(
                &QueryBuilder::select()
                    .ids(node_alias.clone())
                    .query(),
            );
            match nnode {
                Ok(nnode) => {
                    let mut ntype = TypeName::new("file".into());
                    if is_dir {
                        ntype = TypeName::new("folder".into());
                    }
                    if nnode.elements.len() == 0 {
                        // If the node doesn't exist, create it.
                        let node = Node::new(path, ntype);
                        let node_id = self.db.exec_mut(
                            &QueryBuilder::insert()
                                .nodes()
                                .aliases(node_alias)
                                .values(&node)
                                .query(),
                        );
                        match node_id {
                            Ok(node_id) => {
                                // Create an edge between the root and the node
                                //Graph::parent_nodes_by_dbids(&mut self.db, rt_id, node_id);
                            }
                            Err(ref err) => {
                                println!("Failed to create node: {}", err);
                            }
                        }
                    }
                }
                Err(ref err) => {
                    println!("Failed to get node: {}", err);
                }
            }
        }

        if is_dir {
            // If full_path exists, its parent does too. 
        }


        //

        todo!()
    }

    /// Delete all dead nodes from the graph. 
    pub fn cleanup_dead_nodes(&mut self) {
        todo!()
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

/// Implementation block for handling node types.
impl Graph {
    pub fn get_node_types(&self) -> Result<Vec<TypeName>, DbError> {
        todo!()
    }

    pub fn create_nodetype(&mut self, nodetype: TypeName) -> Result<TypeName, DbError> {
        todo!()
    }

    pub fn instance_nodetype(&self) {}
}

/// Implementation block for handling nodes.
impl Graph {
    /// Retrieves a particular node's data from the database.
    /// The path is relative to the root of the graph.
    pub fn open_node(&self, path: NodePath) -> Result<Node, DbError> {
        let alias = path.alias();

        let node = self.db.exec(&QueryBuilder::select().ids(alias).query());

        match node {
            Ok(node) => {
                let node = node.elements.first().unwrap().clone();
                let node = Node::try_from(node);

                node
            }
            Err(_err) => {
                return Err("Could not open node".into());
            }
        }
    }

    /// Opens the connections of a particular node.
    /// Takes in the path to the node relative to the root of the graph.
    ///
    /// TODO: Add filter argument when Filter is implemented.
    /// Note that possibly Filter could have a condition that nodes
    /// would have to be connected to some node, which would just turn
    /// this into a generic "open_nodes" function, or "search_nodes".
    /// Then filters could just be wrappers around agdb's QueryConditions...
    ///
    /// This opens a can of worms about whether the nodes loaded up in Karta
    /// even need to be from a specific context. What if everything was just
    /// in a "soup"? But what would navigation even mean then, when you're not
    /// traveling through contexts? When are relative positions enforced?
    /// How do you determine which node has priority? Is it the one that's open?
    /// If multiple are open, how do the relative positions work?
    /// Parent takes priority over other connection?
    /// What if neither is the parent? Are the priorities configurable?
    ///
    ///
    pub fn open_node_connections(&self, path: NodePath) -> Vec<Node> {
        // Step 1: Check if the node is a physical node in the file system.
        // Step 2: Check if the node exists in the db.
        // Step 3: Check if all the physical dirs and files in the node are in the db.
        // Step 4: The ones that are not, add to the db.
        // Step 5?: Delete the physical nodes in the db that are not in the file system.
        // THOUGH Automatically deleting the nodes
        // honestly seems like a bad idea. Maybe a warning should be issued instead.

        // Resolve the full path to the node
        let full_path = path.full(&self.root_path);

        let is_physical = full_path.exists();

        let as_str = path.alias();

        let mut nodes: Vec<Node> = Vec::new();

        // Query the db for the node

        nodes
    }

    /// Creates a node from the given path. Inserts it into the graph.
    /// Insert the relative path from the root, not including the root dir.
    ///
    /// TODO: Determine whether users of the crate are meant to use this.
    /// Perhaps not. Perhaps the parent of the node should be specified.
    /// The insert_node_by_name function calls this one anyway.
    pub fn create_node_by_path(
        &mut self,
        path: NodePath,
        ntype: Option<TypeName>,
    ) -> Result<DbElement, agdb::DbError> {
        let full_path = path.full(&self.root_path);
        let alias = path.alias();

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
            None => TypeName::other(),
        };

        // Check if the node is physical in the file system.
        // If it is, check if it exists in the db.
        let is_file = full_path.exists() && !full_path.is_dir();
        let is_dir = full_path.is_dir();

        if is_file {
            ntype = TypeName::new("File".to_string());
        } else if is_dir {
            ntype = TypeName::new("Directory".to_string());
        }

        let node = Node::new(path.clone(), ntype);

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
                        if parent_path.buf().to_str().unwrap() != "" {
                            println!("About to insert parent node: {:?}", parent_path);
                            let n = self.create_node_by_path(
                                parent_path,
                                Some(TypeName::other()),
                            );

                            let parent_id = n.unwrap().id;

                            Graph::parent_nodes_by_dbids(&mut self.db, &parent_id, &nid)
                        }
                    }
                    None => {
                        // If the parent is root, parent them and move along.
                        Graph::parent_nodes_by_dbids(&mut self.db, &DbId(1), &nid)
                    }
                }

                return Ok(<DbElement as Clone>::clone(node_elem).try_into().unwrap());
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
    pub fn create_node_by_name(
        &mut self,
        parent_path: Option<NodePath>,
        name: &str,
        ntype: Option<TypeName>,
    ) -> Result<(), agdb::DbError> {
        let parent_path = parent_path.unwrap_or_else(|| NodePath::new("".into()));

        let rel_path = if parent_path.buf().as_os_str().is_empty() {
            NodePath::new(PathBuf::from(name))
        } else {
            NodePath::new(parent_path.buf().join(name))
        };

        match self.create_node_by_path(rel_path, ntype) {
            Ok(_) => {
                return Ok(());
            }
            Err(e) => {
                println!("Failed to insert node: {}", e);
                return Err(e);
            }
        }
    }

    /// Inserts a Node.
    pub fn insert_node(&mut self, node: Node) -> Result<(), agdb::DbError> {
        todo!()
    }

    /// Deletes a node.
    ///
    /// Setting "files" and/or "dirs" to true could also delete from the file system,
    /// and recursively. Very dangerous. Though not implementing this would mean that
    /// those files would constantly be at a risk of getting reindexed, so this
    /// should probably still be implemented, unless we want to just mark nodes as deleted
    /// but never actually delete them, which seems like a smelly solution to me.
    pub fn delete_node(&self, path: PathBuf, files: bool, dirs: bool) -> Result<(), agdb::DbError> {
        Ok(())
    }

    /// Is this even needed? Does open node get all attributes?
    pub fn get_node_attrs(
        &self,
        path: NodePath,
    ) -> Result<Vec<Attribute>, agdb::DbError> {
        let alias = path.alias();
        let keys = Vec::new();
        let attrs = self.db.exec(&QueryBuilder::select().values(keys).ids(alias).query());

        match attrs {
            Ok(attrs) => {
                let mut attrs = attrs.elements;
                assert!(attrs.len() == 1);
                let vec = attrs
                    .first()
                    .unwrap()
                    .values
                    .iter()
                    .map(| attr | {
                        let attr = attr.to_owned();
                        Attribute {
                            name: attr.key.to_string(),
                            value: attr.value.to_f64().unwrap().to_f64() as f32,
                        }
                    })
                    .collect();

                return Ok(vec);
            }
            Err(e) => {
                println!("Failed to get attributes: {}", e);
                return Err(DbError::from(e.to_string()));
            }
        }
    }

    /// Insert attributes to a node. Ignore reserved attribute names. Update attributes that already exist.
    pub fn insert_node_attrs(
        &mut self,
        path: NodePath,
        attrs: Vec<Attribute>,
    ) -> Result<(), agdb::DbError> {
        use elements::RESERVED_NODE_ATTRS;

        // Check if the node exists. If it doesn't, errrrrrrr
        let alias = path.alias();
        let node = self.db.exec(&QueryBuilder::select().ids(alias.clone()).query());
        match node {
            Ok(node) => {
                if node.elements.len() == 0 {
                    return Err(DbError::from(format!("Node not found: {}", alias)));
                }
            }
            Err(e) => {
                println!("Failed to get node: {}", e);
                return Err(DbError::from(e.to_string()));
            }
        }

        let attrs = attrs.iter()
            .filter(| attr | {
                let slice = attr.name.as_str();
                let is_reserved = RESERVED_NODE_ATTRS.contains(&slice);

                if is_reserved {
                    return false;
                }
                return true;
            })
            .map(| attr | {
                 let attr = (attr.name.clone(), attr.value).into();
                 return attr;
             })
            .collect::<Vec<agdb::DbKeyValue>>();

        let added = self.db.exec_mut(
            &QueryBuilder::insert()
                .values(vec!(attrs))
                .ids(alias)
                .query(),
        );

        println!("Added: {:?}", added);

        match added {
            query_result => {
                return Ok(());
            }
            query_error => {
                return Err(DbError::from("Failed to insert attribute"));
            }
        }
    }

    /// Delete attributes from a node. Ignore reserved attribute names.
    pub fn delete_node_attr(
        &mut self,
        path: NodePath,
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

        let alias = path.alias();

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

    /// Merges a vector of nodes into the last one.
    pub fn merge_nodes(&self, nodes: Vec<NodePath>) -> Result<(), agdb::DbError> {
        Ok(())
    }

    // pub fn set_relative_positions

    // pub fn set_node_pins

    // pub fn set_pin_on nodes
}

/// Implementation block for handling edges.
impl Graph {
    pub fn create_edge(
        &mut self,
        source_path: &NodePath,
        target_path: &NodePath,
    ) -> Result<(), agdb::DbError> {
        let alias = source_path.alias();
        let child_alias = target_path.alias();

        Err(DbError::from("Not implemented"))
    }

    /// Mostly used internally.
    /// Uses agdb types directly to create an exclusive parent-child connection.
    /// The attribute is "contains" and is reserved in elements.rs.
    pub fn parent_nodes_by_dbids(db: &mut agdb::Db, parent: &agdb::DbId, child: &agdb::DbId) {
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

    /// Changes the parent directory of a node. If the node is physical, it will be moved in the file system.
    /// If the node is virtual, the parent will be changed in the db.
    /// Note that due to the implementation, all children of the node will have to be reindexed, recursively.
    pub fn reparent_node(
        &self,
        node_path: NodePath,
        new_parent_path: NodePath,
    ) -> Result<(), agdb::DbError> {
        // Check if node is in database at all
        let alias = node_path.alias();
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

    /// Delete an edge from the graph. Edges with the attribute "contains" refer to the parent-child relationship
    /// between nodes and will be ignored. All other attributes will be cleared from them instead.
    pub fn delete_edge(&self, edge: Edge) -> Result<(), agdb::DbError> {
        Ok(())
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
}
