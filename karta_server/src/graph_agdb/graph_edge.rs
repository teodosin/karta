use std::{error::Error, path::PathBuf};

use agdb::QueryBuilder;

use crate::{elements, graph_traits::graph_edge::GraphEdge, nodetype::TypeName};

use super::{Attribute, Edge, GraphAgdb, Node, NodePath, StoragePath};

impl GraphEdge for GraphAgdb {
    fn create_edge(
        &mut self,
        source_path: &NodePath,
        target_path: &NodePath,
    ) -> Result<(), Box<dyn Error>> {
        let alias = source_path.alias();
        let child_alias = target_path.alias();

        todo!()
    }

    /// Changes the parent directory of a node. If the node is physical, it will be moved in the file system.
    /// If the node is virtual, the parent will be changed in the db.
    /// Note that due to the implementation, all children of the node will have to be reindexed, recursively.
    fn reparent_node(
        &self,
        node_path: &NodePath,
        new_parent_path: &NodePath,
    ) -> Result<(), Box<dyn Error>> {
        // Check if node is in database at all
        let alias = node_path.alias();
        let existing = self.db.exec(&QueryBuilder::select().ids(alias).query());
        
        todo!()
    }

    /// Moves an edge and all its attributes to a new source and target. Parent edges can't be reconnected this way,
    /// use the reparent_node function instead.
    fn reconnect_edge(
        &self,
        edge: Edge,
        from: &NodePath,
        to: &NodePath,
    ) -> Result<(), Box<dyn Error>> {
        todo!()
    }

    fn insert_edge(&self, edge: Edge) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    /// Delete an edge from the graph. Edges with the attribute "contains" refer to the parent-child relationship
    /// between nodes and will be ignored. All other attributes will be cleared from them instead.
    fn delete_edge(&self, edge: Edge) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    /// Insert attributes to an edge. Ignore reserved attribute names. Update attributes that already exist.
    fn insert_edge_attr(&self, edge: Edge, attr: Attribute) -> Result<(), Box<dyn Error>> {
        use elements::RESERVED_EDGE_ATTRS;
        let slice = attr.name.as_str();
        let is_reserved = RESERVED_EDGE_ATTRS.contains(&slice);

        if is_reserved {
            return Err(format!(
                "Cannot delete reserved attribute name: {}",
                slice
            ).into());
        }

        Ok(())
    }

    /// Delete attributes from an edge. Ignore reserved attribute names.
    fn delete_edge_attr(&self, edge: Edge, attr: Attribute) -> Result<(), Box<dyn Error>> {
        use elements::RESERVED_EDGE_ATTRS;
        let slice = attr.name.as_str();
        let is_reserved = RESERVED_EDGE_ATTRS.contains(&slice);

        if is_reserved {
            return Err(format!(
                "Cannot insert reserved attribute name: {}",
                slice
            ).into());
        }

        Ok(())
    }
}