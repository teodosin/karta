use std::{error::Error, path::PathBuf, time::SystemTime, vec};

use agdb::{DbElement, DbId, DbUserValue, QueryBuilder};
use uuid::Uuid;

use crate::{
    elements::{self, edge::Edge, node, node_path::NodeHandle},
    graph_traits::graph_node::GraphNodes,
    prelude::{DataNode, GraphCore, GraphEdge, NodePath, NodeTypeId},
};

use super::GraphAgdb;

impl GraphNodes for GraphAgdb {
    fn open_node(&self, handle: &NodeHandle) -> Result<DataNode, Box<dyn Error>> {
        let query_result = match handle {
            NodeHandle::Path(path) => self.db.exec(
                &QueryBuilder::select()
                    .search()
                    .index("path")
                    .value(path.alias())
                    .query(),
            ),
            NodeHandle::Uuid(id) => self.db.exec(
                &QueryBuilder::select()
                    .ids(id.to_string())
                    .query(),
            ),
        }?;

        let db_element = query_result
            .elements
            .first()
            .ok_or_else(|| format!("Node with handle {:?} not found in DB", handle))?;

        DataNode::try_from(db_element.clone()).map_err(|e| e.into())
    }

    fn open_node_connections(&self, path: &NodePath) -> Vec<(DataNode, Edge)> {
        let focal_node = match self.open_node(&NodeHandle::Path(path.clone())) {
            Ok(node) => node,
            Err(_) => return vec![], // If the focal node doesn't exist, no connections can be found.
        };
        let focal_uuid_str = focal_node.uuid().to_string();

        let mut node_ids: Vec<DbId> = Vec::new();
        let mut edge_ids: Vec<DbId> = Vec::new();

        // Search for outgoing and incoming edges from the focal node's UUID alias
        let from_query = QueryBuilder::search().from(focal_uuid_str.clone()).query();
        let to_query = QueryBuilder::search().to(focal_uuid_str).query();

        if let Ok(search_result) = self.db.exec(&from_query) {
            for elem in search_result.elements.iter() {
                if elem.id.0 < 0 {
                    edge_ids.push(elem.id);
                } else {
                    node_ids.push(elem.id);
                }
            }
        }

        if let Ok(search_result) = self.db.exec(&to_query) {
            for elem in search_result.elements.iter() {
                if elem.id.0 < 0 {
                    edge_ids.push(elem.id);
                } else {
                    node_ids.push(elem.id);
                }
            }
        }

        let full_nodes_result = self.db.exec(&QueryBuilder::select().ids(node_ids).query());
        let full_edges_result = self.db.exec(&QueryBuilder::select().ids(edge_ids).query());

        let full_nodes = full_nodes_result.map_or(vec![], |r| r.elements);
        let full_edges = full_edges_result.map_or(vec![], |r| r.elements);

        let mut connections: Vec<(DataNode, Edge)> = Vec::new();
        let mut processed_nodes = std::collections::HashSet::new();

        for db_edge in &full_edges {
            let edge = match Edge::try_from(db_edge.clone()) {
                Ok(e) => e,
                Err(_) => continue,
            };

            let other_node_uuid = if *edge.source() == focal_node.uuid() {
                *edge.target()
            } else {
                *edge.source()
            };

            if other_node_uuid == focal_node.uuid() {
                continue;
            }

            if let Some(db_node) = full_nodes.iter().find(|n| {
                if let Ok(data_node) = DataNode::try_from((*n).clone()) {
                    data_node.uuid() == other_node_uuid
                } else {
                    false
                }
            }) {
                if let Ok(data_node) = DataNode::try_from(db_node.clone()) {
                    if processed_nodes.insert(data_node.uuid()) {
                        connections.push((data_node, edge));
                    }
                }
            }
        }

        connections
    }

    /// Inserts a Node.
    fn insert_nodes(&mut self, nodes: Vec<DataNode>) {
        for node in nodes {
            let node_uuid = node.uuid();
            let npath = node.path();

            // Check if a node with this UUID already exists.
            if self
                .db
                .exec(&QueryBuilder::select().ids(node_uuid.to_string()).query())
                .is_ok()
            {
                // Node with UUID exists, so update it instead of skipping.
                self.db
                    .exec_mut(
                        &QueryBuilder::insert()
                            .values_uniform(node.clone().to_db_values())
                            .ids(node_uuid.to_string())
                            .query(),
                    )
                    .unwrap();
                continue;
            }

            // Insert the new node using its UUID as the alias.
            if self
                .db
                .exec_mut(
                    &QueryBuilder::insert()
                        .nodes()
                        .aliases(node_uuid.to_string())
                        .values(node.clone())
                        .query(),
                )
                .is_err()
            {
                // Failed to insert, continue to next node.
                continue;
            }

            // Handle parent connection
            if let Some(parent_path) = npath.parent() {
                // Find the parent node in the DB by its path to get its UUID.
                if let Ok(parent_node) = self.open_node(&NodeHandle::Path(parent_path.clone())) {
                    let parent_uuid = parent_node.uuid();
                    let edge = Edge::new(parent_uuid, node_uuid);
                    self.insert_edges(vec![edge]);
                } else {
                    // If parent doesn't exist, we might need to create it.
                    // This maintains the behavior of creating parent directories.
                    let parent_node = DataNode::new(&parent_path, NodeTypeId::dir_type());
                    let parent_uuid = parent_node.uuid();
                    self.insert_nodes(vec![parent_node]); // Recursive call
                    let edge = Edge::new(parent_uuid, node_uuid);
                    self.insert_edges(vec![edge]);
                }
            }
        }
    }
}
