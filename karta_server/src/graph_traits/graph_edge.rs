use std::error::Error;

use super::{Attribute, Edge, NodePath};

pub(crate) trait GraphEdge {
    // -------------------------------------------------------------------
    // Edges

    fn create_edge(
        &mut self,
        source_path: &NodePath,
        target_path: &NodePath,
    ) -> Result<(), Box<dyn Error>>;

    /// Mostly used internally.
    /// Uses agdb types directly to create an exclusive parent-child connection.
    /// The attribute is "contains" and is reserved in elements.rs.
    fn parent_nodes_by_dbids(db: &mut agdb::Db, parent: &agdb::DbId, child: &agdb::DbId);

    /// Changes the parent directory of a node. If the node is physical, it will be moved in the file system.
    /// If the node is virtual, the parent will be changed in the db.
    /// Note that due to the implementation, all children of the node will have to be reindexed, recursively.
    fn reparent_node(
        &self,
        node_path: &NodePath,
        new_parent_path: &NodePath,
    ) -> Result<(), Box<dyn Error>>;

    /// Moves an edge and all its attributes to a new source and target. Parent edges can't be reconnected this way,
    /// use the reparent_node function instead.
    fn reconnect_edge(
        &self,
        edge: Edge,
        from: &NodePath,
        to: &NodePath,
    ) ->  Result<(), Box<dyn Error>>;

    fn insert_edge(&self, edge: Edge) -> Result<(), Box<dyn Error>>;

    /// Delete an edge from the graph. Edges with the attribute "contains" refer to the parent-child relationship
    /// between nodes and will be ignored. All other attributes will be cleared from them instead.
    fn delete_edge(&self, edge: Edge) -> Result<(), Box<dyn Error>>;

    /// Insert attributes to an edge. Ignore reserved attribute names. Update attributes that already exist.
    fn insert_edge_attr(&self, edge: Edge, attr: Attribute) -> Result<(), Box<dyn Error>>;

    /// Delete attributes from an edge. Ignore reserved attribute names.
    fn delete_edge_attr(&self, edge: Edge, attr: Attribute) -> Result<(), Box<dyn Error>>;
}
