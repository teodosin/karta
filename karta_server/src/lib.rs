#![allow(warnings)]

pub(crate) mod elements;
pub(crate) mod graph_agdb;
pub(crate) mod graph_traits;

mod utils;

pub mod prelude {
    pub use crate::elements::{
        attribute::Attribute,
        edge::Edge,
        node::Node,
        node_path::NodePath,
        nodetype::NodeType,
    };

    pub use crate::graph_traits::{
        graph_core::GraphCore,
        graph_edge::GraphEdge,
        graph_node::GraphNode,
        StoragePath,
    };

    pub use crate::graph_agdb::GraphAgdb;
}