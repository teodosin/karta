use std::{error::Error, path::MAIN_SEPARATOR};

use crate::prelude::*;

/// Action for creating a new node. 
/// Note that this action may create multiple nodes if its
/// ancestor nodes are missing. Therefore the undo of this action 
/// must also undo the creation of all the ancestor nodes.
pub struct CreateNodeCommand {
    created_ancestors: Vec<NodePath>,
    node_path: NodePath,
    node_type: Option<NodeType>,
}

impl CreateNodeCommand {
    pub fn new(node_path: NodePath, node_type: Option<NodeType>) -> Self {
        CreateNodeCommand {
            created_ancestors: Vec::new(),
            node_path,
            node_type,
        }
    }
}

impl CommandAgdb for CreateNodeCommand {
    fn command_name(&self) -> String {
        "Create Node".to_string()
    }

    fn apply(&mut self, graph: &mut GraphAgdb) -> Result<CommandResult, Box<dyn Error>> {
        // Find all the ancestor nodes that are missing.
        let mut missing_ancestors: Vec<NodePath> = Vec::new();
        let mut ancestor_found: bool = false;
        let mut ancestor_path = self.node_path.clone();

        while !ancestor_found {
            let ancestor_node = graph.open_node(&ancestor_path);
            match &ancestor_node {
                Ok(node) => {
                    ancestor_found = true;
                }
                Err(e) => {
                    ancestor_path = ancestor_path.parent().unwrap().clone();
                    missing_ancestors.push(ancestor_path.clone());
                }
            }
        }

        let ancestor_msg: String = if missing_ancestors.len() > 0 {
            format!("Created ancestor nodes: {:?}", missing_ancestors)
        } else {
            "".to_string()
        };

        self.created_ancestors = missing_ancestors;

        let result = graph.create_node_by_path(&self.node_path, self.node_type.clone());

        match result {
            Ok(node) => {
                return Ok(CommandResult {
                    msg: format!("Node created: {:?}. {}", node.path(), ancestor_msg),
                    nodepaths: self.created_ancestors.clone(),
                    nodes: vec![node],
                    edges: vec![],
                    attributes: vec![],
                })
            },
            Err(e) => {
                return Err(e);
            }
        }
    }

    fn undo(&mut self, graph: &mut GraphAgdb) -> Result<CommandResult, Box<dyn Error>> {
        let mut all_created_nodes: Vec<NodePath> = self.created_ancestors.clone();
        all_created_nodes.push(self.node_path.clone());
        let result = graph.delete_nodes(&all_created_nodes, false, false);

        match result {
            Ok(nodes) => {
                return Ok(CommandResult {
                    msg: format!("Nodes deleted: {:?}", nodes),
                    nodepaths: all_created_nodes,
                    nodes: vec![],
                    edges: vec![],
                    attributes: vec![],
                })
            },
            Err(e) => {
                return Err(e.into());
            }
        }
    }

    fn redo(&mut self, graph: &mut GraphAgdb) -> Result<CommandResult, Box<dyn Error>> {
        self.apply(graph)
    }
}

pub struct InsertNodeAttributeAction {
    node_path: NodePath,
    old_values: Vec<Attribute>,
    new_values: Vec<Attribute>,
}

impl CommandAgdb for InsertNodeAttributeAction {
    fn command_name(&self) -> String {
        "Insert Node Attribute".to_string()
    }
    fn apply(&mut self, graph: &mut GraphAgdb) -> Result<CommandResult, Box<dyn Error>> {
        todo!();
    }
    fn undo(&mut self, graph: &mut GraphAgdb) -> Result<CommandResult, Box<dyn Error>> {
        todo!();
    }
    fn redo(&mut self, graph: &mut GraphAgdb) -> Result<CommandResult, Box<dyn Error>> {
        todo!();
    }
}