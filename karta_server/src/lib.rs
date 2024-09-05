#![allow(warnings)]

pub mod elements;
pub mod graph_agdb;
pub mod graph_traits;

mod utils;

pub mod prelude {
    pub use crate::elements::{
        attribute::Attribute,
        edge::Edge,
        node::Node,
        node_path::NodePath,
        nodetype::NodeType,
        SysTime,
    };

    pub use crate::graph_traits::{
        graph_core::GraphCore,
        graph_edge::GraphEdge,
        graph_node::GraphNode,
        StoragePath,
    };

    pub use crate::graph_agdb::GraphAgdb;
}