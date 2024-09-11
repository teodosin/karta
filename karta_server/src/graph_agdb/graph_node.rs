use std::{error::Error, path::PathBuf, vec};

use agdb::{DbElement, DbId, QueryBuilder};

use crate::{elements::{self, edge::Edge, nodetype::NodeType}, graph_traits::graph_node::GraphNode, prelude::GraphCore};

use super::{attribute::{Attribute, RESERVED_NODE_ATTRS}, node::Node, node_path::NodePath, GraphAgdb, StoragePath};

impl GraphNode for GraphAgdb {
    fn open_node(&mut self, path: &NodePath) -> Result<Node, Box<dyn Error>> {
        let alias = path.alias();

        let node = self.db.exec(&QueryBuilder::select().ids(alias).query());

        match node {
            Ok(node) => {
                let node = node.elements.first().unwrap().clone();
                let node = Node::try_from(node);

                // Dirty
                Ok(node.unwrap())
            }
            Err(_err) => {
                if path.full(&self.user_root_dirpath()).exists() {
                    self.index_single_node(path)
                } else {
                    return Err("Could not open node".into());
                }
            }
        }
    }

    fn open_node_connections(&self, path: &NodePath) -> Vec<(Node, Edge)> {
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

        let as_str = path.alias();

        let mut node_ids: Vec<DbId> = Vec::new();
        let mut edge_ids: Vec<DbId> = Vec::new();

        // Links from node
        let links = self.db.exec(&QueryBuilder::search().from(path.alias()).query());

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
        let backlinks = self.db.exec(&QueryBuilder::search().to(path.alias()).query());

        match backlinks {
            Ok(backlinks) => {
                for elem in backlinks.elements.iter() {
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

        let full_nodes = match self.db.exec(&QueryBuilder::select().values(vec![]).ids(node_ids).query()) {
            Ok(nodes) => nodes.elements,
            Err(_e) => vec![]
        };
        let full_edges = match self.db.exec(&QueryBuilder::select().values(vec![]).ids(edge_ids).query()) {
            Ok(edges) => edges.elements,
            Err(_e) => vec![]
        };

        let connections: Vec<(Node, Edge)> = full_nodes.iter().filter_map(|node| {
            let node = Node::try_from(node.clone()).unwrap();

            
            // Ignore the original node
            if node.path() == *path {
                return None;
            }
            let edge = full_edges.iter().find(|edge| {
                if edge.from.unwrap() == node.id().unwrap() || edge.to.unwrap() == node.id().unwrap() {
                    true
                } else {
                    false
                }
            }).unwrap();
            let edge = Edge::try_from(edge.clone()).unwrap();
            
            println!("Nodes: {:?}", node.path());
            Some((node, edge))
        }).collect();

        connections
    }

    fn create_node_by_path(
        &mut self,
        path: &NodePath,
        ntype: Option<NodeType>,
    ) -> Result<Node, Box<dyn Error>> {

        let full_path = path.full(&self.root_path);
        let alias = path.alias();

        // Check if the node already exists in the db.
        // If it does, don't insert it, and return an error.
        // Possibly redundant, unless used for updating an existing node.
        let existing = self
            .db
            .exec(&QueryBuilder::select().ids(alias.clone()).query());

        match existing {
            Ok(_) => {
                
            }
            Err(_e) => {
                // Node doesn't exist, proceed to insertion
            }
        }

        // Determine type of node. If not specified, it's an Other node.
        let mut ntype = match ntype {
            Some(ntype) => ntype,
            None => NodeType::other(),
        };

        // Check if the node is physical in the file system.
        // If it is, check if it exists in the db.
        let is_file = full_path.exists() && !full_path.is_dir();
        let is_dir = full_path.is_dir();

        if is_file {
            ntype = NodeType::new("File".to_string());
        } else if is_dir {
            ntype = NodeType::new("Directory".to_string());
        }

        let node = Node::new(&path.clone(), ntype);

        let nodeqr = self.db.exec_mut(
            &QueryBuilder::insert()
                .nodes()
                .aliases(alias)
                .values(&node)
                .query(),
        );

        match nodeqr {
            Ok(nodeqr) => {
                let node_elem = &nodeqr.elements[0];
                let nid = node_elem.id;
                // If parent is not root, check if the parent node already exists in the db.
                // If not, call this function recursively.
                let parent_path = path.parent();
                match parent_path {
                    Some(parent_path) => {
                        if parent_path.parent().is_some() {
                            println!("About to insert parent node: {:?}", parent_path);

                            let n = self.create_node_by_path(
                                &parent_path,
                                Some(NodeType::other()),
                            );

                            match n {
                                Ok(n) => {
                                    let parent_path = n.path();
                                    self.autoparent_nodes(&parent_path, &path);
                                }
                                Err(e) => {
                                    println!("Failed to insert parent node: {}", e);
                                }
                            }
                        }
                        Ok(node)
                    }
                    None => {
                        // If the parent is root, parent them and move along.
                        self.autoparent_nodes(&NodePath::new(PathBuf::from("")), &path);
                        Ok(node)
                    }
                }

            }
            Err(e) => {
                println!("Failed to insert node: {}", e);
                Err(e.into())
            }
        }
    }

    /// Creates a node under a given parent with the given name.
    /// The path is relative to the root of the graph.
    /// Do not include the root dir name.
    fn create_node_by_name(
        &mut self,
        parent_path: Option<NodePath>,
        name: &str,
        ntype: Option<NodeType>,
    ) -> Result<Node, Box<dyn Error>> {
        let parent_path = parent_path.unwrap_or_else(|| NodePath::new("".into()));

        let rel_path = if parent_path.buf().as_os_str().is_empty() {
            NodePath::new(PathBuf::from(name))
        } else {
            NodePath::new(parent_path.buf().join(name))
        };

        self.create_node_by_path(&rel_path, ntype)
    }

    /// Inserts a Node.
    fn insert_node(&mut self, node: Node) -> Result<(), Box<dyn Error>> {
        todo!()
    }

    /// Deletes a node. Error if trying to delete root or archetype nodes. 
    ///
    /// Setting "files" and/or "dirs" to true could also delete from the file system,
    /// and recursively. Very dangerous. Though not implementing this would mean that
    /// those files would constantly be at a risk of getting reindexed, so this
    /// should probably still be implemented, unless we want to just mark nodes as deleted
    /// but never actually delete them, which seems like a smelly solution to me.
    fn delete_nodes(&mut self, paths: &Vec<NodePath>, files: bool, dirs: bool) -> Result<(), Box<dyn Error>> {
        let aliases = paths.iter().map(|path| path.alias()).collect::<Vec<String>>();

        let query = self.db.exec_mut(&QueryBuilder::remove().ids(aliases).query());

        match query {
            Ok(query) => {
                Ok(())
            }
            Err(e) => {
                Err(e.into())
            }
        }
    }

    /// Is this even needed? Does open node get all attributes?
    fn get_node_attrs(
        &self,
        path: &NodePath,
    ) -> Result<Vec<Attribute>, Box<dyn Error>> {
        let alias = path.alias();
        let keys = Vec::new();
        let attrs = self.db.exec(&QueryBuilder::select().values(keys).ids(alias).query());

        match attrs {
            Ok(attrs) => {
                let mut attrs = attrs.elements;
                assert!(attrs.len() == 1);
                let vec = attrs
                    .first()
                    .unwrap()
                    .values
                    .iter()
                    .map(| attr | {
                        let attr = attr.to_owned();
                        Attribute {
                            name: attr.key.to_string(),
                            value: attr.value.to_f64().unwrap().to_f64() as f32,
                        }
                    })
                    .collect();

                return Ok(vec);
            }
            Err(e) => {
                println!("Failed to get attributes: {}", e);
                return Err(e.to_string().into());
            }
        }
    }

    fn insert_node_attrs(
        &mut self,
        path: &NodePath,
        attrs: Vec<Attribute>,
    ) -> Result<(), Box<dyn Error>>{
        use RESERVED_NODE_ATTRS;

        // Check if the node exists. If it doesn't, errrrrrrr
        let alias = path.alias();
        let node = self.db.exec(&QueryBuilder::select().ids(alias.clone()).query());
        match node {
            Ok(node) => {}
            Err(e) => {
                return Err(e.into());
            }
        }

        // Error if attributes is empty
        if attrs.is_empty() {
            return Err("Attributes cannot be empty".into());
        }

        let filtered_attrs = attrs.iter()
            .filter(| attr | {
                let slice = attr.name.as_str();
                let is_reserved = RESERVED_NODE_ATTRS.contains(&slice);

                if is_reserved {
                    return false;
                }
                return true;
            })
            .map(| attr | {
                 let attr = (attr.name.clone(), attr.value).into();
                 return attr;
             })
            .collect::<Vec<agdb::DbKeyValue>>();

        // Error if filtered attrs is empty
        if filtered_attrs.is_empty() {
            return Err("All insertion requests were for protected attributes".into());
        }

        let added = self.db.exec_mut(
            &QueryBuilder::insert()
                .values(vec!(filtered_attrs))
                .ids(alias)
                .query(),
        );

        println!("Added: {:?}", added);

        match added {
            query_result => {
                return Ok(());
            }
            query_error => {
                return Err("Failed to insert attribute".into());
            }
        }
    }

    fn delete_node_attrs(
        &mut self,
        path: &NodePath,
        attr_names: Vec<&str>,
    ) -> Result<(), Box<dyn Error>> {
        use RESERVED_NODE_ATTRS;

        if attr_names.len() == 0 {
            return Err("No attributes to delete".into());
        }

        // Protect reserved attribute names
        let filtered_attrs: Vec<agdb::DbValue> = attr_names.iter().filter(| &&attr_name | {
            !RESERVED_NODE_ATTRS.contains(&attr_name)
        }).map(|&s| agdb::DbValue::from(s)).collect();

        if filtered_attrs.len() == 0 {
            return Err("All deletion requests were for protected attributes".into());
        }

        let node = self.db.exec_mut(
            &QueryBuilder::remove()
                .values(filtered_attrs)
                .ids(path.alias())
                .query(),
        );

        match node {
            Ok(node) => {
                Ok(())
            }
            Err(e) => {
                Err(e.into())
            }
        }
    }

    /// Merges a vector of nodes into the last one.
    fn merge_nodes(&mut self, nodes: Vec<NodePath>) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    // fn set_relative_positions

    // fn set_node_pins

    // fn set_pin_on nodes

    fn autoparent_nodes(
        &mut self, parent: &NodePath, child: &NodePath
    ) -> Result<(), Box<dyn Error>> {
        let edge = Edge::new_cont(parent, child);

        let edge = self.db.exec_mut(
            &QueryBuilder::insert()
                .edges()
                .from(parent.alias())
                .to(child.alias())
                .values_uniform(&edge)
                .query(),
        ); // For whatever reason this does not insert the attribute into the edge.

        let eid = edge.unwrap().ids();
        let eid = eid.first().unwrap();
        println!("Id of the edge: {:#?}", eid);

        let edge = self.db.exec(&QueryBuilder::select().keys().ids(*eid).query());

        match edge {
            Ok(edge) => {
                // Insert the attribute to the edge
                // println!("Edge inserted: {:#?}", edge.elements);
                Ok(())
            }
            Err(e) => {
                Err(e.into())
            }
        }
    }

}