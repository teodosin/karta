#![allow(warnings)]

mod elements;
mod graph_traits;
mod graph_agdb;
mod graph_commands;

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

    pub use crate::graph_commands::{
        commands::*,
        GraphCommands,
    };
}