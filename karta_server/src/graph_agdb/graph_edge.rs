use std::error::Error;

use agdb::{DbElement, QueryBuilder};
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

            let _edge_query_result = self.db.exec_mut(
                &QueryBuilder::insert()
                    .edges()
                    .from(source_uuid.to_string())
                    .to(target_uuid.to_string())
                    .values_uniform(edge)
                    .query(),
            );
        }
    }

    /// Delete an edge from the graph. Edges with the attribute "contains" refer to the parent-child relationship
    /// between nodes and will be ignored. All other attributes will be cleared from them instead.
    fn delete_edge(&mut self, edge: Edge) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}
