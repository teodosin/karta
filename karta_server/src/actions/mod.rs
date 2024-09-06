use std::{error::Error, path::PathBuf};
use agdb::QueryBuilder;

use crate::prelude::*;



pub struct ActionManager {
    actions: Vec<Box<dyn ActionAgdb>>,
    current_index: usize,
}

pub trait ActionAgdb {
    fn apply(&self, graph: &mut GraphAgdb) -> Result<(), Box<dyn Error>>;

    fn undo(&self, graph: &mut GraphAgdb) -> Result<(), Box<dyn Error>>;

    fn redo(&self, graph: &mut GraphAgdb) -> Result<(), Box<dyn Error>>;
}

/// Action for creating a new node. 
/// Note that this action may create multiple nodes if its
/// ancestor nodes are missing. Therefore the undo of this action 
/// must also undo the creation of all the ancestor nodes.
pub struct CreateNodeAction {
    node_path: NodePath,
    node_type: Option<NodeType>,
}

impl ActionAgdb for CreateNodeAction {
    fn apply(&self, graph: &mut GraphAgdb) -> Result<(), Box<dyn Error>> {
        return Ok(()); 

        let path = self.node_path.clone();

        let full_path = path.full(&graph.user_root_dirpath());
        let alias = path.alias();

        // Check if the node already exists in the db.
        // If it does, don't insert it, and return an error.
        // Possibly redundant, unless used for updating an existing node.
        let existing = graph
            .db()
            .exec(&QueryBuilder::select().ids(alias.clone()).query());

        match existing {
            Ok(_) => {
                return Err("Node already exists".into());
            }
            Err(_e) => {
                // Node doesn't exist, proceed to insertion
            }
        }

        // Determine type of node. If not specified, it's an Other node.
        let mut ntype = match &self.node_type {
            Some(ntype) => ntype,
            None => &NodeType::other(),
        };

        // Check if the node is physical in the file system.
        // If it is, check if it exists in the db.
        let is_file = full_path.exists() && !full_path.is_dir();
        let is_dir = full_path.is_dir();

        if is_file {
            ntype = &NodeType::new("File".to_string());
        } else if is_dir {
            ntype = &NodeType::new("Directory".to_string());
        }

        let node = Node::new(&path.clone(), NodeType::other());

        let nodeqr = graph.db_mut().exec_mut(
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
                        if parent_path.parent().is_some() {
                            println!("About to insert parent node: {:?}", parent_path);

                            let create_parent_action: CreateNodeAction = CreateNodeAction {
                                node_path: parent_path,
                                node_type: None,
                            };

                            let n = create_parent_action.apply(graph);

                            match n {
                                Ok(n) => {
                                    // let parent_path = n.path();
                                    graph.autoparent_nodes(&parent_path, &path);
                                }
                                Err(e) => {
                                    println!("Failed to insert parent node: {}", e);
                                }
                            }
                        }
                        Ok(())
                    }
                    None => {
                        // If the parent is root, parent them and move along.
                        graph.autoparent_nodes(&NodePath::new(PathBuf::from("")), &path);
                        Ok(())
                    }
                }

            }
            Err(e) => {
                println!("Failed to insert node: {}", e);
                Err(e.into())
            }
        }
    }
    fn undo(&self, graph: &mut GraphAgdb) -> Result<(), Box<dyn Error>> {
        todo!();
    }
    fn redo(&self, graph: &mut GraphAgdb) -> Result<(), Box<dyn Error>> {
        todo!();
    }
}

pub struct InsertNodeAttributeAction {
    node_path: NodePath,
    old_values: Vec<Attribute>,
    new_values: Vec<Attribute>,
}

impl ActionAgdb for InsertNodeAttributeAction {
    fn apply(&self, graph: &mut GraphAgdb) -> Result<(), Box<dyn Error>> {
        todo!();
    }
    fn undo(&self, graph: &mut GraphAgdb) -> Result<(), Box<dyn Error>> {
        todo!();
    }
    fn redo(&self, graph: &mut GraphAgdb) -> Result<(), Box<dyn Error>> {
        todo!();
    }
}