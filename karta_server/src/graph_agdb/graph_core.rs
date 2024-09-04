use std::path::{self, PathBuf};

use agdb::QueryBuilder;

use crate::{
    graph_traits::{self, graph_core::GraphCore, graph_node::GraphNode},
    elements::nodetype::NodeType,
};

use super::{node::Node, node_path::NodePath, nodetype::ARCHETYPES, GraphAgdb, StoragePath};

/// Implementation block for the Graph struct itself.
/// Includes constructors and utility functions.
impl GraphCore for GraphAgdb {
    fn storage_path(&self) -> graph_traits::StoragePath {
        self.storage_path.clone()
    }
    
    fn userroot_path(&self) -> PathBuf {
        self.root_path.clone()
    }

    fn root_nodepath(&self) -> NodePath {
        NodePath::root()
    }

    fn userroot_nodepath(&self) -> NodePath {
        let binding = self.userroot_path();
        let name = binding.file_name().unwrap();
        let buf = PathBuf::from(name);
        NodePath::new(buf)
    }

    /// Constructor. Panics if the db cannot be created.
    ///
    /// Takes the desired root directory of the graph as a parameter and the name for the db.
    /// The name of the root directory will become the userroot of the graph,
    /// as first child of the root node. 
    /// 
    /// Creates the db at the storage_path, or initialises the db if it already exists there.
    ///
    /// TODO: Add error handling.
    fn new(name: &str, root_path: PathBuf, custom_storage_path: Option<PathBuf>) -> Self {
        let storage_enum = match custom_storage_path {
            Some(path) => graph_traits::StoragePath::Custom(path),
            None => graph_traits::StoragePath::Default,
        };
        let storage_path = match storage_enum.clone() {
            StoragePath::Custom(path) => path,
            StoragePath::Default => {
                directories::ProjectDirs::from("com", "teodosin_labs", "fs_graph")
                    .unwrap()
                    .data_dir()
                    .to_path_buf()
                }
        };

        // Create the path if it doesn't exist
        if !storage_path.exists() {
            std::fs::create_dir_all(&storage_path).expect("Failed to create storage path");
        }

        let db_path = storage_path.join(format!("{}.agdb", name));

        // Check if the database already exists
        let open_existing = db_path.exists();

        let db = agdb::Db::new(db_path.to_str().unwrap()).expect("Failed to create new db");

        let mut giraphe = GraphAgdb {
            name: name.to_string(),
            db,
            root_path: root_path.into(),
            storage_path: storage_enum,
            maintain_readable_files: false,
        };

        if !open_existing {
            giraphe.init_archetype_nodes();
        }

        return giraphe;
    }

    /// Create the initial archetype nodes for the graph. Includes
    /// the root,
    /// attributes,
    /// settings,
    /// nodetypes
    fn init_archetype_nodes(&mut self) {

        let archetypes = ARCHETYPES;
        let root_path = NodePath::root();
        
        archetypes.iter().for_each(|atype| {
            let buf = PathBuf::from(atype.to_string());
            let atype_path = NodePath::new(buf);

            let ntype = if atype_path == NodePath::root() {
                NodeType::root_type()
            } else {
                NodeType::archetype_type()
            };

            let node: Node = Node::new(&atype_path, ntype);

            let query = self.db.exec_mut(
                &QueryBuilder::insert()
                    .nodes()
                    .aliases(atype_path.alias())
                    .values(&node)
                    .query(),
            );
            
            match query {
                Ok(_) => {
                    println!("Created archetype node: {}", atype_path.alias());
                }
                Err(ref err) => {
                    println!("Failed to create archetype node: {}", err);
                }
            }

            if atype_path != NodePath::root() {
                self.autoparent_nodes(&root_path, &atype_path);
            }
        });

        // Initialise the userroot node
        // let userroot_path = self.userroot_path();

        let node: Node = Node::new(&self.userroot_nodepath(), NodeType::other());

        let query = self.db.exec_mut(
            &QueryBuilder::insert()
                .nodes()
                .aliases(self.userroot_nodepath().alias())
                .values(&node)
                .query(),
        );

        match query {
            Ok(_) => {
                println!("Created userroot node: {}", self.userroot_nodepath().alias());
            }
            Err(ref err) => {
                println!("Failed to create userroot node: {}", err);
            }
        }

        self.autoparent_nodes(&root_path, &self.userroot_nodepath());
    }

    /// Syncs a node in the db with the file system
    fn index_single_node(&mut self, path: &NodePath) {
        let full_path = path.full(&self.root_path);
        let node_alias = path.alias();

        let is_phys = full_path.exists();
        let is_dir = full_path.is_dir();

        todo!()
    }

    /// Syncs the node's relationships in the db with the file system.
    fn index_node_connections(&mut self, path: &NodePath) {
        let full_path = path.full(&self.root_path);
        let node_alias = path.alias();

        let is_phys = full_path.exists();
        let is_dir = full_path.is_dir();

        if is_phys {
            // Check if the path has a node in the db. If not, it will be created.
            let nnode = self
                .db
                .exec(&QueryBuilder::select().ids(node_alias.clone()).query());
            match nnode {
                Ok(nnode) => {
                    let mut ntype = NodeType::new("file".into());
                    if is_dir {
                        ntype = NodeType::new("folder".into());
                    }
                    if nnode.elements.len() == 0 {
                        // If the node doesn't exist, create it.
                        let node = Node::new(&path.clone(), ntype);
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
    fn cleanup_dead_nodes(&mut self) {
        todo!()
    }

    /// Set whether the library should maintain readable files for the nodes in the graph.
    fn maintain_readable_files(&mut self, maintain: bool) {
        self.maintain_readable_files = maintain;
    }

    /// Gets the name of the root directory without the full path
    fn root_name(&self) -> String {
        self.root_path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string()
    }
}
