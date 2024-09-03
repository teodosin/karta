use std::{error::Error, path::PathBuf};

use crate::nodetype::TypeName;

use super::{Attribute, Node, NodePath};

pub(crate) trait GraphNode {
    // -------------------------------------------------------------------
    // Nodes

    /// Retrieves a particular node's data from the database.
    /// The path is relative to the root of the graph.
    fn open_node(&self, path: NodePath) -> Result<Node, Box<dyn Error>>;

    /// Opens the connections of a particular node.
    /// Takes in the path to the node relative to the root of the graph.
    ///
    /// TODO: Add filter argument when Filter is implemented.
    /// Note that possibly Filter could have a condition that nodes
    /// would have to be connected to some node, which would just turn
    /// this into a generic "open_nodes" function, or "search_nodes".
    /// Then filters could just be wrappers around agdb's QueryConditions...
    ///
    /// This opens a can of worms about whether the nodes loaded up in Karta
    /// even need to be from a specific context. What if everything was just
    /// in a "soup"? But what would navigation even mean then, when you're not
    /// traveling through contexts? When are relative positions enforced?
    /// How do you determine which node has priority? Is it the one that's open?
    /// If multiple are open, how do the relative positions work?
    /// Parent takes priority over other connection?
    /// What if neither is the parent? Are the priorities configurable?
    fn open_node_connections(&self, path: NodePath) -> Vec<Node>;

    /// Creates a node from the given path. Inserts it into the graph.
    /// Insert the relative path from the root, not including the root dir.
    ///
    /// TODO: Determine whether users of the crate are meant to use this.
    /// Perhaps not. Perhaps the parent of the node should be specified.
    /// The insert_node_by_name function calls this one anyway.
    fn create_node_by_path(
        &mut self,
        path: NodePath,
        ntype: Option<TypeName>,
    );

    /// Creates a node under a given parent with the given name.
    /// The path is relative to the root of the graph.
    /// Do not include the root dir name.
    fn create_node_by_name(
        &mut self,
        parent_path: Option<NodePath>,
        name: &str,
        ntype: Option<TypeName>,
    );

    /// Inserts a Node.
    fn insert_node(&mut self, node: Node) -> Result<(), Box<dyn Error>>;

    /// Deletes a node.
    ///
    /// Setting "files" and/or "dirs" to true could also delete from the file system,
    /// and recursively. Very dangerous. Though not implementing this would mean that
    /// those files would constantly be at a risk of getting reindexed, so this
    /// should probably still be implemented, unless we want to just mark nodes as deleted
    /// but never actually delete them, which seems like a smelly solution to me.
    fn delete_node(&self, path: PathBuf, files: bool, dirs: bool) -> Result<(), agdb::DbError>;

    /// Insert attributes to a node. Ignore reserved attribute names. Update attributes that already exist.
    fn insert_node_attrs(
        &mut self,
        path: NodePath,
        attrs: Vec<Attribute>,
    ) -> Result<(), Box<dyn Error>>;

    /// Get node attributes
    fn get_node_attrs(
        &self,
        path: NodePath,
    ) -> Result<Vec<Attribute>, Box<dyn Error>>;

    /// Delete attributes from a node. Ignore reserved attribute names.
    fn delete_node_attr(
        &mut self,
        path: NodePath,
        attr_name: &str,
    ) -> Result<(), Box<dyn Error>>;

    /// Merges a vector of nodes into the last one.
    fn merge_nodes(&self, nodes: Vec<NodePath>) -> Result<(), agdb::DbError>;

    // pub fn set_relative_positions

    // pub fn set_node_pins

    // pub fn set_pin_on nodes

}