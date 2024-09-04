use std::path::{self, PathBuf};

use agdb::QueryBuilder;

use crate::{
    graph_traits::{self, graph_core::GraphCore, graph_node::GraphNode},
    elements::nodetype::TypeName,
};

use super::{node::Node, node_path::NodePath, GraphAgdb, StoragePath};

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
        todo!()
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
        // Create the root node
        let root_path = NodePath::root();
        let root: Vec<Node> = vec![Node::new(&NodePath::new("".into()), TypeName::root_type())];

        let rt_node = self.db.exec_mut(
            &QueryBuilder::insert()
                .nodes()
                .aliases(root_path.alias())
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
        let atr_path = NodePath::new("attributes".into());
        let atr: Vec<Node> = vec![Node::new(&atr_path, TypeName::archetype_type())];

        let atr_node = self.db.exec_mut(
            &QueryBuilder::insert()
                .nodes()
                .aliases(atr_path.alias())
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
        self.autoparent_nodes(&root_path, &atr_path);

        // Archetype ------------------------------------------------
        // Create the settings node for global application settings
        let set_path = NodePath::new("settings".into());
        let set: Vec<Node> = vec![Node::new(&set_path, TypeName::archetype_type())];

        let set_node = self.db.exec_mut(
            &QueryBuilder::insert()
                .nodes()
                .aliases(set_path.alias())
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
        self.autoparent_nodes(&root_path, &set_path);

        // Archetype ------------------------------------------------
        // Create the nodetypes node for global node categories.
        // Node types are then children of nodetypes or operators.
        let nca_path = NodePath::new("nodetypes".into());
        let nca: Vec<Node> = vec![Node::new(&nca_path, TypeName::archetype_type())];

        let nca_node = self.db.exec_mut(
            &QueryBuilder::insert()
                .nodes()
                .aliases(nca_path.alias())
                .values(&nca)
                .query(),
        );
        match nca_node {
            Ok(_) => {
                println!("Created nodetypes node");
            }
            Err(ref err) => {
                println!("Failed to create nodetypes node: {}", err);
            }
        }
        // Create an edge between the root and nodetypes nodes
        self.autoparent_nodes(&root_path, &nca_path);
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
                    let mut ntype = TypeName::new("file".into());
                    if is_dir {
                        ntype = TypeName::new("folder".into());
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
