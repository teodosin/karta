use std::{error::Error, path::PathBuf, time::SystemTime, vec};

use agdb::{DbElement, DbId, QueryBuilder};
use uuid::Uuid;

use crate::{
    elements::{self, edge::Edge, node, node_path::NodeHandle},
    graph_traits::graph_node::GraphNodes,
    prelude::{DataNode, GraphCore, GraphEdge, NodePath, NodeTypeId},
};

use super::GraphAgdb;

impl GraphNodes for GraphAgdb {
    fn open_node(&self, handle: &NodeHandle) -> Result<DataNode, Box<dyn Error>> {
        let mut node: Result<agdb::QueryResult, agdb::QueryError>;

        match handle {
            NodeHandle::Path(path) => {
                let alias = path.alias();
                node = self.db.exec(&QueryBuilder::select().ids(alias).query());
            }
            NodeHandle::Uuid(id) => {
                node = self.db.exec(&QueryBuilder::select().search().index("uuid").value(id.to_string()).query());
                // println!("Node is {:#?}", node); // Kept for potential debugging, but commented
            }
        }
        
        match node {
            Ok(node) => {
                let db_element = node.elements.first().ok_or_else(|| "DB query returned Ok but no elements found.")?;
                DataNode::try_from(db_element.clone()).map_err(|e| Box::new(e) as Box<dyn Error>)
            }
            Err(_err) => {
                match handle {
                    NodeHandle::Path(path_handle) => {
                        Err(format!("Node with path {:?} not found in DB", path_handle).into())
                    }
                    NodeHandle::Uuid(id_handle) => {
                        Err(format!("Node with UUID {} not found in DB", id_handle).into())
                    }
                }
            }
        }
    }

    fn open_node_connections(&self, path: &NodePath) -> Vec<(DataNode, Edge)> {
        // Resolve the full path to the node - This seems unused now, consider removing if not needed elsewhere.
        // let full_path = path.full(&self.root_path);
        // let is_physical = full_path.exists();
        // let is_dir = full_path.is_dir();

        // let as_str = path.alias(); // Unused

        let mut node_ids: Vec<DbId> = Vec::new();
        let mut edge_ids: Vec<DbId> = Vec::new();

        // Links from node
        // println!("Searching for links from node {}", path.alias());
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
                        // let balias = self // This balias was unused
                        //     .db
                        //     .exec(&QueryBuilder::select().aliases().ids(elem.id).query());
                        // println!("balias: {:?}", balias);
                        node_ids.push(elem.id);
                    }
                }
            }
            Err(_e) => {}
        }

        let full_nodes = match self.db.exec(&QueryBuilder::select().ids(node_ids).query()) {
            Ok(nodes) => nodes.elements,
            Err(_e) => vec![],
        };
        let full_edges = match self.db.exec(&QueryBuilder::select().ids(edge_ids).query()) {
            Ok(edges) => edges.elements,
            Err(_e) => vec![],
        };

        let connections: Vec<(DataNode, Edge)> = full_nodes
            .iter()
            .filter_map(|node| {
                let node = DataNode::try_from(node.clone()).unwrap();

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

                Some((node, edge))
            })
            .collect();

        connections
    }

    /// Inserts a Node.
    fn insert_nodes(&mut self, nodes: Vec<DataNode>) {
        for mut node in nodes {
            let npath = node.path();

            let existing = self.db.exec(
                &QueryBuilder::select()
                    .ids(node.path().alias().clone())
                    .query(),
            );

            match existing {
                Ok(_) => {
                    // Node already exists, consider if this should be an error or a silent skip/update.
                    // For now, it prints and continues, effectively skipping re-insertion of the node itself if alias matches.
                    // println!("Node with alias {} already exists in DB during insert_nodes", npath.alias());
                }
                Err(_e) => {
                    // Node doesn't exist by alias, proceed to insertion
                }
            }

            let node_query = self.db.exec_mut(
                &QueryBuilder::insert()
                    .nodes()
                    .aliases(node.path().alias())
                    .values(node)
                    .query(),
            );

            match node_query {
                Ok(nodeqr) => {
                    let node_elem = &nodeqr.elements[0];
                    let nid = node_elem.id;
                    // If parent is not root, ensure parent node exists in DB.
                    // This recursively ensures the path to the node is created.
                    let parent_path = npath.parent();
                    println!("Parent of node is: {:#?}", parent_path);
                    match parent_path {
                        Some(parent_path) => {
                            if parent_path.parent().is_some() {
                                // println!("About to ensure parent node: {:?}", parent_path);
                                // Check if parent exists before trying to insert, to avoid redundant work / errors if it's already there.
                                // This basic check might not be sufficient if parent needs specific attributes or type.
                                if self.db.exec(&QueryBuilder::select().ids(parent_path.alias()).query()).is_err() {
                                    let parent_node = DataNode::new(&parent_path, NodeTypeId::dir_type());
                                    self.insert_nodes(vec![parent_node]); // Recursive call
                                }
                                
                                let edge = Edge::new(&parent_path, &npath);
                                self.insert_edges(vec![edge]);
                            }
                        }
                        None => {
                            // Parent is root, create edge from root.
                            // Ensure root node itself exists if this is the first node ever.
                            // For simplicity, assuming root node (alias "/") is implicitly handled or pre-exists.
                            let root_edge = Edge::new(&NodePath::root(), &npath);
                            self.insert_edges(vec![root_edge]);
                        }
                    }
                }
                Err(_e) => {
                    // println!("Failed to insert node with alias {}: {}", npath.alias(), e);
                }
            }
            // println!("Processed node for insertion: {:#?}", npath.alias());
        }
    }
}
