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
pub(crate) enum StoragePath {
    Default,
    Custom(PathBuf),
}

/// The main graph trait.
pub(crate) trait Graph: GraphCore + GraphNtype + GraphNode + GraphEdge {}

