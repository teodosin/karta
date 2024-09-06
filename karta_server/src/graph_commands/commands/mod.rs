use std::{error::Error, path::PathBuf};
use agdb::QueryBuilder;

use crate::{graph_traits::Graph, prelude::*};

pub mod node;

pub struct CommandManager {
    queue: Vec<Box<dyn CommandAgdb>>,
    undo_stack: Vec<Box<dyn CommandAgdb>>,
    redo_stack: Vec<Box<dyn CommandAgdb>>,
}

impl CommandManager {
    pub fn new() -> Self {
        CommandManager {
            queue: Vec::new(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    pub fn get_undo_stack(&self) -> &Vec<Box<dyn CommandAgdb>> {
        &self.undo_stack
    }

    pub fn get_redo_stack(&self) -> &Vec<Box<dyn CommandAgdb>> {
        &self.redo_stack
    }

    pub fn apply(&mut self, graph: &mut GraphAgdb, mut command: Box<dyn CommandAgdb>) -> Result<CommandResult, Box<dyn Error>> {
        let result = command.apply(graph);
        self.undo_stack.push(command);
        result
    }

    pub fn undo(&mut self, graph: &mut GraphAgdb) -> Result<CommandResult, Box<dyn Error>> {
        if let Some(mut command) = self.undo_stack.pop() {
            let result = command.undo(graph);
            self.redo_stack.push(command);
            result
        } else {
            Err("Unable to undo".into())
        }

    }

    pub fn redo(&mut self, graph: &mut GraphAgdb) -> Result<CommandResult, Box<dyn Error>> {
        if let Some(mut command) = self.redo_stack.pop() {
            let result = command.redo(graph);
            self.undo_stack.push(command);
            result
        } else {
            Err("Unable to redo".into())
        }
    }
}

pub trait CommandAgdb {
    fn command_name(&self) -> String;

    fn apply(&mut self, graph: &mut GraphAgdb) -> Result<CommandResult, Box<dyn Error>>;

    fn undo(&mut self, graph: &mut GraphAgdb) -> Result<CommandResult, Box<dyn Error>>;

    fn redo(&mut self, graph: &mut GraphAgdb) -> Result<CommandResult, Box<dyn Error>>;
}

pub struct CommandResult {
    pub msg: String, 
    pub nodepaths: Vec<NodePath>,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub attributes: Vec<Attribute>,
}

impl Into<Vec<Node>> for CommandResult {
    fn into(self) -> Vec<Node> {
        self.nodes
    }
}   




