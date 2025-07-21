use std::error::Error;

use agdb::{DbElement, DbUserValue, QueryBuilder};
use uuid::Uuid;

use crate::{graph_traits::graph_edge::GraphEdge, prelude::Edge};

use super::GraphAgdb;

impl GraphEdge for GraphAgdb {
    fn get_edge_strict(&self, from: &Uuid, to: &Uuid) -> Result<Edge, Box<dyn Error>> {
        let edge_query = self.db.exec(
            &QueryBuilder::search()
                .from(from.to_string())
                .to(to.to_string())
                .query(),
        );

        if edge_query.is_err() {
            return Err("Failed to get edge".into());
        }
        let edge_query = edge_query.unwrap();

        let edge_elems: Vec<&DbElement> = edge_query
            .elements
            .iter()
            .filter(|e| e.id.0 < 0)
            .collect::<Vec<_>>();

        if edge_elems.len() != 1 {
            return Err(format!("Expected 1 edge, got {}", edge_elems.len()).into());
        }
        let edge_id = edge_elems.first().unwrap().id;

        // The search doesn't return the values, so we have to do a separate select
        // on the edge id.
        let data_query = self
            .db
            .exec(&QueryBuilder::select().ids(edge_id).query());

        if data_query.is_err() {
            return Err("Failed to get edge data".into());
        }
        let data_query = data_query.unwrap();
        let data_elem = data_query
            .elements
            .first()
            .ok_or("No element found for edge id")?;

        Edge::try_from(data_elem.clone()).map_err(|e| e.into())
    }

    fn insert_edges(&mut self, edges: Vec<Edge>) {
        for edge in edges {
            let source_uuid = edge.source();
            let target_uuid = edge.target();

            println!("[insert_edges] Creating edge from {} to {}", source_uuid, target_uuid);

            let edge_query_result = self.db.exec_mut(
                &QueryBuilder::insert()
                    .edges()
                    .from(source_uuid.to_string())
                    .to(target_uuid.to_string())
                    .values_uniform(edge)
                    .query(),
            );

            match edge_query_result {
                Ok(_) => println!("[insert_edges] Successfully inserted edge."),
                Err(e) => println!("[insert_edges] Error inserting edge: {:?}", e),
            }
        }
    }


    /// Delete edges by their UUIDs. If an edge has the "contains" attribute, it will be cleared instead of deleted.
    fn get_edges_between_nodes(&self, nodes: &[Uuid]) -> Result<Vec<Edge>, Box<dyn Error>> {
        if nodes.len() < 2 {
            return Ok(Vec::new());
        }

        let mut all_edges = std::collections::HashMap::new();

        for i in 0..nodes.len() {
            for j in 0..nodes.len() {
                if i == j {
                    continue;
                }

                let source_uuid = nodes[i].to_string();
                let target_uuid = nodes[j].to_string();

                let query = QueryBuilder::select().ids(
                    QueryBuilder::search()
                        .from(source_uuid)
                        .to(target_uuid)
                        .query()
                ).query();

                if let Ok(result) = self.db.exec(&query) {
                    for element in result.elements.into_iter().filter(|e| e.id.0 < 0) {
                        if let Ok(edge) = Edge::try_from(element) {
                            all_edges.insert(edge.uuid(), edge);
                        }
                    }
                }
            }
        }

        Ok(all_edges.values().cloned().collect())
    }

    fn delete_edges(&mut self, edges: &[(Uuid, Uuid)]) -> Result<(), Box<dyn Error>> {
        self.db.transaction_mut(|t| -> Result<(), agdb::QueryError> {
            for (source, target) in edges {
                // First, get the edge to determine if it's a "contains" edge.
                let get_edge_query = QueryBuilder::select().ids(
                    QueryBuilder::search()
                        .from(source.to_string())
                        .to(target.to_string())
                        .query()
                ).query();

                let query_result = t.exec(&get_edge_query)?;

                if let Some(element) = query_result.elements.iter().find(|e| e.id.0 < 0) {
                    let edge = Edge::try_from(element.clone())?;

                    if edge.is_contains() {
                        // It's a "contains" edge. This type of edge cannot be deleted via this endpoint.
                        // Return an error to abort the transaction.
                        return Err(agdb::QueryError::from("Deletion of 'contains' edges is not allowed."));
                    } else {
                        // It's a normal edge, so we can remove it entirely.
                        let remove_edge_query = QueryBuilder::remove()
                            .ids(element.id)
                            .query();
                        t.exec_mut(&remove_edge_query)?;
                    }
                }
            }
            Ok(())
        })?;

        Ok(())
    }

    fn reconnect_edge(
        &mut self,
        old_from: &Uuid,
        old_to: &Uuid,
        new_from: &Uuid,
        new_to: &Uuid,
    ) -> Result<Edge, Box<dyn Error>> {
        let new_edge = self.db.transaction_mut(|t| {
            // Replicate get_edge_strict logic inside transaction
            let edge_query = t.exec(
                &QueryBuilder::search()
                    .from(old_from.to_string())
                    .to(old_to.to_string())
                    .query(),
            )?;

            let edge_elems: Vec<&DbElement> = edge_query
                .elements
                .iter()
                .filter(|e| e.id.0 < 0)
                .collect::<Vec<_>>();

            if edge_elems.len() != 1 {
                return Err(agdb::QueryError::from(format!(
                    "Expected 1 edge, got {}",
                    edge_elems.len()
                )));
            }
            let edge_id = edge_elems.first().unwrap().id;

            let data_query = t.exec(&QueryBuilder::select().ids(edge_id).query())?;
            let data_elem = data_query
                .elements
                .first()
                .ok_or_else(|| agdb::QueryError::from("No element found for edge id"))?;

            let original_edge = Edge::try_from(data_elem.clone())?;

            if original_edge.is_contains() {
                return Err(agdb::QueryError::from(
                    "Reconnection of 'contains' edges is not allowed via this method.",
                ));
            }

            // 1. Delete old edge
            t.exec_mut(&QueryBuilder::remove().ids(edge_id).query())?;

            // 2. Create new edge
            let mut new_edge = Edge::new(*new_from, *new_to);
            new_edge.set_attributes(original_edge.attributes().clone());

            t.exec_mut(
                &QueryBuilder::insert()
                    .edges()
                    .from(new_from.to_string())
                    .to(new_to.to_string())
                    .values_uniform(new_edge.clone())
                    .query(),
            )?;

            Ok(new_edge)
        })?;

        Ok(new_edge)
    }

    fn reparent_node(
        &mut self,
        node_uuid: &Uuid,
        old_parent_uuid: &Uuid,
        new_parent_uuid: &Uuid,
    ) -> Result<(), Box<dyn Error>> {
        self.db.transaction_mut(|t| -> Result<(), agdb::QueryError> {
            // 1. Find the existing contains edge from old parent to node
            let find_edge_query = QueryBuilder::select().ids(
                QueryBuilder::search()
                    .from(old_parent_uuid.to_string())
                    .to(node_uuid.to_string())
                    .query()
            ).query();

            let query_result = t.exec(&find_edge_query)?;
            
            if let Some(element) = query_result.elements.iter().find(|e| e.id.0 < 0) {
                let edge = Edge::try_from(element.clone())?;
                
                // Verify it's actually a contains edge before proceeding
                if edge.is_contains() {
                    // 2. Delete the old contains edge (bypassing normal restrictions)
                    t.exec_mut(&QueryBuilder::remove().ids(element.id).query())?;
                    
                    // 3. Create new contains edge from new parent to node
                    let new_edge = Edge::new_cont(*new_parent_uuid, *node_uuid);
                    t.exec_mut(
                        &QueryBuilder::insert()
                            .edges()
                            .from(new_parent_uuid.to_string())
                            .to(node_uuid.to_string())
                            .values_uniform(new_edge)
                            .query(),
                    )?;
                } else {
                    return Err(agdb::QueryError::from(
                        "Edge found is not a contains edge - reparent_node can only move contains edges"
                    ));
                }
            } else {
                return Err(agdb::QueryError::from(
                    "No contains edge found between old parent and node"
                ));
            }
            
            Ok(())
        })?;

        Ok(())
    }
}
