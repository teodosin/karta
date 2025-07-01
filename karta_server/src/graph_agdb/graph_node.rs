use std::{collections::VecDeque, error::Error, path::PathBuf, time::SystemTime, vec};

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

    fn open_nodes_by_uuid(&self, uuids: Vec<Uuid>) -> Result<Vec<DataNode>, Box<dyn Error>> {
        if uuids.is_empty() {
            return Ok(Vec::new());
        }

        let ids_as_strings: Vec<String> = uuids.into_iter().map(|id| id.to_string()).collect();
        
        let query_result = self.db.exec(
            &QueryBuilder::select()
                .ids(ids_as_strings)
                .query(),
        )?;

        let datanodes = query_result
            .elements
            .into_iter()
            .map(DataNode::try_from)
            .collect::<Result<Vec<DataNode>, _>>()?;

        Ok(datanodes)
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

    fn insert_nodes(&mut self, nodes: Vec<DataNode>) {
        let mut queue: VecDeque<DataNode> = nodes.into();
        let mut processed_paths = std::collections::HashSet::new();

        while let Some(node) = queue.pop_front() {
            let node_path = node.path();
            if processed_paths.contains(&node_path) {
                continue;
            }

            if let Some(parent_path) = node_path.parent() {
                if self.open_node(&NodeHandle::Path(parent_path.clone())).is_err() {
                    queue.push_front(node);
                    queue.push_front(DataNode::new(&parent_path, NodeTypeId::dir_type()));
                    continue;
                }
            }
            
            let node_uuid = node.uuid();

            let existing_node_query = self.db.exec(&QueryBuilder::select().ids(node_uuid.to_string()).query());
            if let Ok(query_result) = existing_node_query {
                if !query_result.elements.is_empty() {
                    self.db.exec_mut(&QueryBuilder::insert().values_uniform(node.clone().to_db_values()).ids(node_uuid.to_string()).query()).unwrap();
                } else {
                     self.db.exec_mut(&QueryBuilder::insert().nodes().aliases(node_uuid.to_string()).values(node.clone()).query()).unwrap();
                }
            } else {
                 self.db.exec_mut(&QueryBuilder::insert().nodes().aliases(node_uuid.to_string()).values(node.clone()).query()).unwrap();
            }

            if let Some(parent_path) = node.path().parent() {
                if let Ok(parent_node) = self.open_node(&NodeHandle::Path(parent_path)) {
                     let edge = Edge::new_cont(parent_node.uuid(), node_uuid);
                     self.insert_edges(vec![edge]);
                }
            }

            processed_paths.insert(node_path.clone());
        }
    }
}
