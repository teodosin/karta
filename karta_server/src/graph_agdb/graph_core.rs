
use std::path::PathBuf;

use agdb::QueryBuilder;

use crate::{graph_traits::graph_core::GraphCore, nodetype::TypeName};

use super::{GraphAgdb, Node, NodePath, StoragePath};

/// Implementation block for the Graph struct itself.
/// Includes constructors and utility functions.
impl GraphCore for GraphAgdb {
    fn root_path(&self) -> PathBuf {
        self.root_path.clone()
    }
    
    fn root_nodepath(&self) -> NodePath {
        NodePath::new(self.root_path.clone())
    }

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
        
        let mut giraphe = GraphAgdb {
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
    fn new_custom_storage(root_path: PathBuf, name: &str, storage_path: PathBuf) -> Self {
        // Create the path if it doesn't exist
        if !storage_path.exists() {
            std::fs::create_dir_all(&storage_path).expect("Failed to create storage path");
        }

        let db = agdb::Db::new(storage_path.join(name).to_str().unwrap());

        let mut db = db.expect("Failed to create db");

        let mut giraphe = GraphAgdb {
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
    fn init_archetype_nodes(&mut self) {
        
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
        GraphAgdb::parent_nodes_by_dbids(&mut self.db, rt_id, atr_node.unwrap().ids().first().unwrap());


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
        GraphAgdb::parent_nodes_by_dbids(&mut self.db, rt_id, set_node.unwrap().ids().first().unwrap());


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
        GraphAgdb::parent_nodes_by_dbids(&mut self.db, rt_id, nca_node.unwrap().ids().first().unwrap());
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
                        let node = Node::new(path.clone(), ntype);
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