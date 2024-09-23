use std::{error::Error, path::PathBuf};

use crate::elements;
use elements::*;

use graph_core::GraphCore;
use graph_edge::GraphEdge;
use graph_node::GraphNode;
use graph_ntype::GraphNtype;

pub(crate) mod graph_core;
pub(crate) mod graph_ntype;
pub(crate) mod graph_node;
pub(crate) mod graph_edge;

#[derive(Clone, PartialEq, Debug)]
pub enum StoragePath {
    Default,
    Custom(PathBuf),
}

impl StoragePath {
    pub fn new(path: PathBuf) -> Self {
        Self::Custom(path)
    }

    pub fn strg_path(&self) -> Option<PathBuf> {
        match self {
            Self::Default => None,
            Self::Custom(path) => Some(path.clone()),
        }
    }
}

/// The main graph trait.
pub(crate) trait Graph: GraphCore + GraphNtype + GraphNode + GraphEdge {}

