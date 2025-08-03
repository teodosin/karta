use std::{collections::VecDeque, error::Error, path::PathBuf, time::SystemTime, vec};

use agdb::{DbElement, DbId, DbUserValue, QueryBuilder, CountComparison, QueryCondition};
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

        let mut datanodes = Vec::new();
        
        // Query each UUID individually to handle missing nodes gracefully
        for uuid in uuids {
            let uuid_string = uuid.to_string();
            match self.db.exec(
                &QueryBuilder::select()
                    .ids(uuid_string)
                    .query(),
            ) {
                Ok(query_result) => {
                    // Try to convert each element that exists
                    for element in query_result.elements {
                        if let Ok(datanode) = DataNode::try_from(element) {
                            datanodes.push(datanode);
                        }
                    }
                }
                Err(_) => {
                    // Node doesn't exist, skip it
                    continue;
                }
            }
        }

        Ok(datanodes)
    }

    fn open_node_connections(&self, path: &NodePath) -> Vec<(DataNode, Edge)> {
        println!("[open_node_connections] Getting connections for path: '{}'", path.alias());
        let focal_node = match self.open_node(&NodeHandle::Path(path.clone())) {
            Ok(node) => node,
            Err(_) => {
                println!("[open_node_connections] -> Focal node not found. Returning empty vec.");
                return vec![];
            }
        };
        let focal_uuid_str = focal_node.uuid().to_string();
        println!("[open_node_connections] -> Focal node UUID: {}", focal_uuid_str);

        let mut node_ids: Vec<DbId> = Vec::new();
        let mut edge_ids: Vec<DbId> = Vec::new();

        let from_query = QueryBuilder::search().from(focal_uuid_str.clone()).where_().distance(CountComparison::LessThan(2)).query();
        let to_query = QueryBuilder::search().to(focal_uuid_str).where_().distance(CountComparison::LessThan(2)).query();

        if let Ok(search_result) = self.db.exec(&from_query) {
            println!("[open_node_connections] -> Found {} elements connected FROM focal node.", search_result.elements.len());
            for elem in search_result.elements.iter() {
                if elem.id.0 < 0 { edge_ids.push(elem.id); } else { node_ids.push(elem.id); }
            }
        }

        if let Ok(search_result) = self.db.exec(&to_query) {
            println!("[open_node_connections] -> Found {} elements connected TO focal node.", search_result.elements.len());
            for elem in search_result.elements.iter() {
                if elem.id.0 < 0 { edge_ids.push(elem.id); } else { node_ids.push(elem.id); }
            }
        }
        
        let full_edges_result = self.db.exec(&QueryBuilder::select().ids(edge_ids).query());
        let full_edges = full_edges_result.map_or(vec![], |r| r.elements);
        println!("[open_node_connections] -> Total unique edges found: {}", full_edges.len());

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

            if other_node_uuid == focal_node.uuid() { continue; }

            if let Ok(data_node) = self.open_node(&NodeHandle::Uuid(other_node_uuid)) {
                if processed_nodes.insert(data_node.uuid()) {
                    println!("[open_node_connections] -> Adding connection: '{}' ({})", data_node.path().alias(), data_node.uuid());
                    connections.push((data_node, edge));
                }
            }
        }
        
        println!("[open_node_connections] -> Returning {} connections.", connections.len());
        connections
    }

    fn insert_nodes(&mut self, nodes: Vec<DataNode>) {
        let mut queue: VecDeque<DataNode> = nodes.into();
        let mut processed_paths = std::collections::HashSet::new();

        while let Some(node) = queue.pop_front() {
            let node_path = node.path();
            println!("[insert_nodes] Processing node: {:?}", node_path);

            if processed_paths.contains(&node_path) {
                println!("[insert_nodes] -> Path already processed, skipping.");
                continue;
            }

            if let Some(parent_path) = node_path.parent() {
                println!("[insert_nodes] -> Checking parent: {:?}", parent_path);
                if parent_path != NodePath::root() && parent_path != NodePath::vault() && self.open_node(&NodeHandle::Path(parent_path.clone())).is_err() {
                    println!("[insert_nodes] -> Parent not found. Creating placeholder for {:?} and requeueing.", parent_path);
                    queue.push_front(node);
                    queue.push_front(DataNode::new(&parent_path, NodeTypeId::dir_type()));
                    continue;
                }
                 println!("[insert_nodes] -> Parent found or is archetype. Proceeding with insertion.");
            }
            
            let node_uuid = node.uuid();
            println!("[insert_nodes] -> Inserting/updating node with UUID: {}", node_uuid);

            // Use open_node to check for existence, which is more robust.
            if self.open_node(&NodeHandle::Uuid(node_uuid)).is_ok() {
                // Node exists, update its values.
                println!("[insert_nodes] -> Node {} exists, updating values.", node_uuid);
                self.db.exec_mut(
                    &QueryBuilder::insert()
                        .values_uniform(node.clone().to_db_values())
                        .ids(node_uuid.to_string())
                        .query()
                ).unwrap();
            } else {
                // Node does not exist, insert it.
                println!("[insert_nodes] -> Node {} does not exist, inserting new node.", node_uuid);
                self.db.exec_mut(
                    &QueryBuilder::insert()
                        .nodes()
                        .aliases(node_uuid.to_string())
                        .values(node.clone())
                        .query()
                ).unwrap();
            }

            if let Some(parent_path) = node.path().parent() {
                let parent_path_clone = parent_path.clone();
                println!("[insert_nodes] -> Attempting to create edge from parent {:?} to new node.", parent_path_clone);
                if let Ok(parent_node) = self.open_node(&NodeHandle::Path(parent_path)) {
                     let edge = Edge::new_cont(parent_node.uuid(), node_uuid);
                     println!("[insert_nodes] -> Edge created: {:?} -> {:?}", parent_node.uuid(), node_uuid);
                     self.insert_edges(vec![edge]);
                } else {
                    println!("[insert_nodes] -> FAILED to open parent node {:?} to create edge.", parent_path_clone);
                }
            }

            processed_paths.insert(node_path.clone());
            println!("[insert_nodes] Finished processing: {:?}", node_path);
        }
    }
    
    fn get_all_indexed_paths(&self) -> Result<Vec<String>, Box<dyn Error>> {
        let query_result = self.db.exec(
            &QueryBuilder::select()
                .ids(
                    QueryBuilder::search()
                        .elements()
                        .where_()
                        .node()
                        .query()
                )
                .query()
        )?;

        let paths = query_result
            .elements
            .into_iter()
            .filter(|element| element.id.0 > 0) // Filter for nodes only
            .filter_map(|element| {
                DataNode::try_from(element).ok().map(|node| node.path().alias().to_string())
            })
            .collect();

        Ok(paths)
    }

    fn update_node_attributes(&mut self, uuid: Uuid, attributes: Vec<crate::elements::attribute::Attribute>) -> Result<(), Box<dyn Error>> {
        let db_values: Vec<agdb::DbKeyValue> = attributes.into_iter().map(|attr| attr.into()).collect();

        self.db.exec_mut(
            &QueryBuilder::insert()
                .values_uniform(db_values)
                .ids(uuid.to_string())
                .query()
        )?;

        Ok(())
    }
    fn get_all_descendants(&self, path: &NodePath) -> Result<Vec<DataNode>, Box<dyn Error>> {
        let mut descendants = Vec::new();
        let mut stack = Vec::new();
        let mut visited = std::collections::HashSet::new();

        let start_node = self.open_node(&NodeHandle::Path(path.clone()))?;
        println!("[DEBUG] get_all_descendants: Starting with node '{}' (UUID: {})", start_node.path().alias(), start_node.uuid());
        stack.push(start_node.clone());
        visited.insert(start_node.uuid());

        while let Some(current_node) = stack.pop() {
            println!("[DEBUG] get_all_descendants: Processing node '{}' (UUID: {})", current_node.path().alias(), current_node.uuid());
            
            // Use open_node_connections which properly filters edges from this node
            let connections = self.open_node_connections(&current_node.path());
            println!("[DEBUG] get_all_descendants: Found {} connections from '{}'", connections.len(), current_node.path().alias());
            
            for (connected_node, edge) in connections {
                println!("[DEBUG] get_all_descendants: Examining edge {} -> {} (contains: {})", 
                    edge.source(), edge.target(), edge.is_contains());
                
                // Only follow contains edges where current_node is the source (parent)
                if edge.is_contains() && *edge.source() == current_node.uuid() {
                    println!("[DEBUG] get_all_descendants: Following contains edge to child '{}' (UUID: {})", 
                        connected_node.path().alias(), connected_node.uuid());
                    if visited.insert(connected_node.uuid()) {
                        stack.push(connected_node.clone());
                        descendants.push(connected_node);
                    } else {
                        println!("[DEBUG] get_all_descendants: Child already visited, skipping");
                    }
                } else if edge.is_contains() {
                    println!("[DEBUG] get_all_descendants: Skipping contains edge where this node is the child");
                } else {
                    println!("[DEBUG] get_all_descendants: Skipping non-contains edge");
                }
            }
        }

        println!("[DEBUG] get_all_descendants: Found {} total descendants of '{}'", descendants.len(), path.alias());
        for (i, desc) in descendants.iter().enumerate() {
            println!("[DEBUG] get_all_descendants:   {}: '{}' (UUID: {})", i, desc.path().alias(), desc.uuid());
        }

        Ok(descendants)
    }

    fn delete_node_and_edges(&mut self, node_id: &Uuid) -> Result<(), Box<dyn Error>> {
        // agdb automatically deletes all edges (incoming and outgoing) when a node is deleted
        let delete_node_query = QueryBuilder::remove()
            .ids(node_id.to_string())
            .query();
        self.db.exec_mut(&delete_node_query)?;
        
        Ok(())
    }
}
