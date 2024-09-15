use std::{error::Error, path::{self, PathBuf}};

use agdb::QueryBuilder;

use crate::{
    elements::nodetype::NodeType,
    graph_traits::{self, graph_core::GraphCore, graph_node::GraphNode},
};

use super::{node::Node, node_path::NodePath, nodetype::ARCHETYPES, GraphAgdb, StoragePath};

/// Implementation block for the Graph struct itself.
/// Includes constructors and utility functions.
impl GraphCore for GraphAgdb {
    fn storage_path(&self) -> graph_traits::StoragePath {
        self.storage_path.clone()
    }

    fn user_root_dirpath(&self) -> PathBuf {
        let path = self.root_path.clone();
        println!("root_path: {:?}", path);
        path
    }

    fn root_nodepath(&self) -> NodePath {
        NodePath::root()
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

    /// Constructor. Panics if the db cannot be created.
    ///
    /// Takes the desired root directory of the graph as a parameter and the name for the db.
    /// The name of the root directory will become the user_root of the graph,
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

        println!("WE ARE ABOUT TO CREATE ARCHHHHH");

        if !open_existing {
            println!("WE HAVE ENTERED THE IF");
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

        println!("Length of archetypes {}", archetypes.len());

        archetypes.iter().for_each(|at| {
            println!("{}", at);
        });

        archetypes.iter().for_each(|atype| {
            let atype_path = NodePath::atype(*atype);
            println!("Atypepath {:?}", atype_path);

            println!("Creating archetype node: {}", atype_path.alias());

            let ntype = if atype_path == NodePath::root() {
                println!("Root node in question");
                NodeType::root_type()
            } else {
                println!("Archetype node in question");
                NodeType::archetype_type()
            };

            let node: Node = Node::new(&atype_path, ntype);

            println!("alias is {}", atype_path.alias());

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
                    panic!("Failed to create archetype node: {}", err);
                    println!("Failed to create archetype node: {}", err);
                }
            }

            if atype_path != NodePath::root() {
                println!(
                    "autoparent: parent {:?} to child {:?}",
                    &NodePath::root(),
                    &atype_path
                );
                self.autoparent_nodes(&NodePath::root(), &atype_path);
            } else {
                println!("Root node, no autoparenting");
            }
        });
    }

    /// Syncs a node in the db with the file system. Errs on archetype nodes as
    /// well as other virtual nodes. 
    fn index_single_node(&mut self, path: &NodePath) -> Result<Node, Box<dyn Error>>{
        
        let full_path = path.full(&self.root_path);
        let is_user_root = full_path == self.user_root_dirpath();

        let mut node_alias: String;

        let mut is_phys: bool;
        let mut is_dir: bool;

        // The user_root is a special case, because its directory name and its
        // alias are different. So we need to check for that.
        if is_user_root {
            node_alias = NodePath::user_root().alias();
            is_phys = full_path.exists();
            is_dir = full_path.is_dir();
            assert!(is_phys && is_dir, "User root directory must exist and be a directory");
        } else {
            if path.is_atype() {
                return Err("Archetype nodes cannot be indexed".into())
            }
            node_alias = path.alias();
            is_phys = full_path.exists();
            is_dir = full_path.is_dir();
        }

        println!("Indexing node: {}", node_alias);
        println!("Is phys: {} is dir: {}", is_phys, is_dir);

        // Handle the case where the node is already in the db
        let node = self.db.exec(&QueryBuilder::select().ids(node_alias.clone()).query());
        if node.is_ok() {
            println!("Node already exists");
            return Err("Node already exists".into())
        }

        if is_phys {
            println!("Indexing node: {}", node_alias);
            if is_dir {
                return self.create_node_by_path(path, Some(NodeType::dir()))
            } else {
                return self.create_node_by_path(path, Some(NodeType::file()))
            } 
        } else {
            return Err("Cannot index virtual node".into())
        }
        return Err("Indexing of path failed".into())
    }

    /// Syncs the node's and its relationships in the db with the file system.
    fn index_node_context(&mut self, path: &NodePath) {
        let full_path = path.full(&self.root_path);
        let mut node_alias: String;

        let is_user_root = full_path == self.user_root_dirpath();
        let mut is_phys: bool;
        let mut is_dir: bool;

        // The user_root is a special case, because its directory name and its
        // alias are different. So we need to check for that.
        if is_user_root {
            node_alias = NodePath::user_root().alias();
            is_phys = full_path.exists();
            is_dir = full_path.is_dir();
            assert!(is_phys && is_dir, "User root directory must exist and be a directory");
        } else {
            node_alias = path.alias();
            is_phys = full_path.exists();
            is_dir = full_path.is_dir();
        }

        // Only the user_root nodes is guaranteed to not have a valid parent
        // in the current vault. Other parents should be indexed. 
        if !is_user_root {
            let parent = path.parent();
            if parent.is_some(){
                self.index_single_node(&parent.unwrap());
            }
        }

        // If the path is a directory, we must check for its contents in the 
        // file system and index them.
        if is_dir {
            let children = full_path.read_dir().unwrap();
            children.into_iter().for_each(|child| {
                match child {
                    Ok(child) => {
                        let path = child.path();
                        let child_path = NodePath::from_dir_path(&self.user_root_dirpath(), &path);
                        println!("Indexing child: {:?}", child_path);

                        self.index_single_node(&child_path);
                    },
                    Err(err) => {
                        println!("Error reading directory: {}", err);
                    }
                }
            });
        }

        // TODO: Check an existing node's relationships to find nodes that need updating 

        // More code here pls
        // More pls
        // Pls?
    }

    /// Delete all dead nodes from the graph.
    fn cleanup_dead_nodes(&mut self) {
        todo!()
    }

    /// Set whether the library should maintain readable files for the nodes in the graph.
    fn maintain_readable_files(&mut self, maintain: bool) {
        self.maintain_readable_files = maintain;
    }
}
