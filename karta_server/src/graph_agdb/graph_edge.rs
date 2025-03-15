use std::{error::Error, path::PathBuf};

use agdb::{DbElement, QueryBuilder};

use crate::{elements, graph_traits::graph_edge::GraphEdge};

use super::{attribute::{Attribute, RESERVED_EDGE_ATTRS}, edge::Edge, node_path::NodePath, GraphAgdb, StoragePath};

impl GraphEdge for GraphAgdb {
    fn get_edge_strict(
        &self,
        from: &NodePath,
        to: &NodePath,
    ) -> Result<Edge, Box<dyn Error>> {
        let from_al = from.alias();
        let to_al = to.alias();

        let edge_query = self.db.exec(
            &QueryBuilder::search()
            .from(from_al)
            .to(to_al)
            .query()
        );

        if edge_query.is_err() {
            return Err("Failed to get edge".into());
        }
        let edge_query = edge_query.unwrap();

        let edge_elems: Vec<&DbElement> = edge_query
            .elements.iter().filter(|e| {
                e.id.0 < 0
            })
            .collect::<Vec<_>>();

        assert_eq!(edge_elems.len(), 1, "Expected only 1 edge, got {}", edge_elems.len());
        let edge_id = edge_elems.first().unwrap().id;

        // The search doesn't return the values, so we have to do a separate select
        // on the edge id. 
        let keys = Vec::new();
        let data_query = self.db.exec(
            &QueryBuilder::select().values(keys).ids(edge_id).query()
        );

        if data_query.is_err() {
            return Err("Failed to get edge data".into());
        }
        let data_query = data_query.unwrap();
        let data_elem = data_query.elements.first().unwrap();
        // println!("Edge element: {:#?}", data_elem);

        let edge = Edge::try_from(data_elem.clone());

        // println!("Edge: {:#?}", edge);

        match edge {
            Ok(edge) => {
                return Ok(edge);
            }
            Err(e) => {
                return Err(e.into());
            }
        }
    }

    fn insert_edge(&self, edge: Edge) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    /// Delete an edge from the graph. Edges with the attribute "contains" refer to the parent-child relationship
    /// between nodes and will be ignored. All other attributes will be cleared from them instead.
    fn delete_edge(&self, edge: Edge) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}