#![allow(warnings)]

mod elements;
mod graph_traits;
mod graph_agdb;

mod fs_reader;
mod context;
mod layout;

mod server;

mod utils;

pub(crate) const SERVER_VERSION: &str = "0.1.0";

pub mod prelude {
    pub use crate::elements::{
        attribute::Attribute,
        edge::Edge,
        node::DataNode,
        view_node::ViewNode,
        node_path::NodePath,
        nodetype::NodeTypeId,
        SysTime,
    };

    pub use crate::graph_traits::{
        graph_core::GraphCore,
        graph_edge::GraphEdge,
        graph_node::GraphNodes,
        StoragePath,
    };

    pub use crate::graph_agdb::GraphAgdb;

    pub use crate::context::*;

    pub use crate::layout::*;

    pub use crate::server::*;
}