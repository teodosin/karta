use std::{error::Error, path::PathBuf};

use agdb::{DbElement, QueryBuilder};

use crate::{elements, graph_traits::graph_edge::GraphEdge, prelude::{Edge, NodePath}};

use super::GraphAgdb;


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
        let keys: Vec<String> = Vec::new();
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

    fn insert_edge(&mut self, edge: Edge) -> Result<(), Box<dyn Error>> {
        let parent = edge.source();
        let child = edge.target();

        let edge = self.db.exec_mut(
            &QueryBuilder::insert()
                .edges()
                .from(parent.alias())
                .to(child.alias())
                .values_uniform(edge)
                .query(),
        ); // For whatever reason this does not insert the attribute into the edge.

        let eid = edge.unwrap().ids();
        let eid = eid.first().unwrap();
        // println!("Id of the edge: {:#?}", eid);

        let edge = self
            .db
            .exec(&QueryBuilder::select().keys().ids(*eid).query());

        match edge {
            Ok(edge) => {
                // Insert the attribute to the edge
                // // println!("Edge inserted: {:#?}", edge.elements);
                Ok(())
            }
            Err(e) => Err(e.into()),
        }
    }

    /// Delete an edge from the graph. Edges with the attribute "contains" refer to the parent-child relationship
    /// between nodes and will be ignored. All other attributes will be cleared from them instead.
    fn delete_edge(&mut self, edge: Edge) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}