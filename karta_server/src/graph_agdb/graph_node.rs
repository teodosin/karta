use std::{error::Error, path::PathBuf, vec};

use agdb::{DbElement, DbId, QueryBuilder};

use crate::{
    elements::{self, edge::Edge},
    graph_traits::graph_node::GraphNodes,
    prelude::{DataNode, GraphCore, NodePath},
};

use super::GraphAgdb;

impl GraphNodes for GraphAgdb {
    fn open_node(&self, path: &NodePath) -> Result<DataNode, Box<dyn Error>> {
        let alias = path.alias();

        let node = self.db.exec(&QueryBuilder::select().ids(alias).query());

        match node {
            Ok(node) => {
                let node = node.elements.first().unwrap().clone();
                let node = DataNode::try_from(node);

                // Dirty
                Ok(node.unwrap())
            }
            Err(_err) => {
                return Err("Could not open node".into());
            }
        }
    }

    fn open_node_connections(&self, path: &NodePath) -> Vec<(DataNode, Edge)> {
        // Step 1: Check if the node is a physical node in the file system.
        // Step 2: Check if the node exists in the db.
        // Step 3: Check if all the physical dirs and files in the node are in the db.
        // Step 4: The ones that are not, add to the db.
        // Step 5?: Delete the physical nodes in the db that are not in the file system.
        // THOUGH Automatically deleting the nodes
        // honestly seems like a bad idea. Maybe a warning should be issued instead.

        // Resolve the full path to the node
        let full_path = path.full(&self.root_path);
        let is_physical = full_path.exists();
        let is_dir = full_path.is_dir();

        let as_str = path.alias();

        let mut node_ids: Vec<DbId> = Vec::new();
        let mut edge_ids: Vec<DbId> = Vec::new();

        // Links from node
        // println!("Searching for links from node {}", as_str);
        let links = self.db.exec(
            &QueryBuilder::search()
                .from(path.alias())
                .where_()
                .distance(agdb::CountComparison::LessThanOrEqual(2))
                .query(),
        );

        match links {
            Ok(links) => {
                for elem in links.elements.iter() {
                    if elem.id.0 < 0 {
                        // Is edge
                        edge_ids.push(elem.id);
                    } else if elem.id.0 > 0 {
                        // Is node
                        // println!("Link: {:?}", elem);
                        node_ids.push(elem.id);
                    }
                }
            }
            Err(_e) => {}
        }

        // Backlinks to node
        let backlinks = self.db.exec(
            &QueryBuilder::search()
                .to(path.alias())
                .where_()
                .distance(agdb::CountComparison::LessThanOrEqual(2))
                .query(),
        );

        match backlinks {
            Ok(backlinks) => {
                for elem in backlinks.elements.iter() {
                    if elem.id.0 < 0 {
                        // Is edge
                        edge_ids.push(elem.id);
                    } else if elem.id.0 > 0 {
                        // Is node
                        // println!("Backlink: {:?}", elem);
                        let balias = self
                            .db
                            .exec(&QueryBuilder::select().aliases().ids(elem.id).query());
                        // println!("balias: {:?}", balias);
                        node_ids.push(elem.id);
                    }
                }
            }
            Err(_e) => {}
        }

        let full_nodes = match self
            .db
            .exec(&QueryBuilder::select().ids(node_ids).query())
        {
            Ok(nodes) => nodes.elements,
            Err(_e) => vec![],
        };
        let full_edges = match self
            .db
            .exec(&QueryBuilder::select().ids(edge_ids).query())
        {
            Ok(edges) => edges.elements,
            Err(_e) => vec![],
        };

        let connections: Vec<(DataNode, Edge)> = full_nodes
            .iter()
            .filter_map(|node| {
                let node = DataNode::try_from(node.clone()).unwrap();

                // println!("Returning node {:?}", node.path());
                // Ignore the original node
                if node.path() == *path {
                    return None;
                }
                let edge = full_edges
                    .iter()
                    .find(|edge| {
                        if edge.from.unwrap() == node.id().unwrap()
                            || edge.to.unwrap() == node.id().unwrap()
                        {
                            true
                        } else {
                            false
                        }
                    })
                    .unwrap();
                let edge = Edge::try_from(edge.clone()).unwrap();

                // println!("Nodes: {:?}", node.path());
                Some((node, edge))
            })
            .collect();

        connections
    }


    /// Inserts a Node.
    fn insert_nodes(&mut self, node: DataNode) -> Result<(), Box<dyn Error>> {
        todo!()
    }

}
