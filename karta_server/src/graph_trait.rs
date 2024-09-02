use std::{error::Error, path::PathBuf};

use crate::{elements, nodetype::TypeName, path_ser};
use elements::*;

enum StoragePath {
    Default,
    Custom(PathBuf),
}

/// The main graph structure to be interacted with.
///
/// Bevy_fs_graph will instantiate this as a Resource through a newtype.
pub trait Graph {
    /// Constructor. Panics if the db cannot be created.
    ///
    /// Takes the desired root of the graph as a parameter and the name for the db.
    ///
    /// Creates the db at the storage_path, or initialises the db if it already exists there.
    ///
    /// TODO: Add error handling.
    fn new(&self, root_path: PathBuf, name: &str) -> Self;

    /// Alternate constructor. Use this if you want to set a custom storage path for the db.
    /// Panics if the db cannot be created
    fn new_custom_storage(&self, root_path: PathBuf, name: &str, storage_path: PathBuf) -> Self;

    /// Create the initial archetype nodes for the graph. Includes 
    /// the root, 
    /// attributes,
    /// settings,
    /// nodecategories
    fn init_archetype_nodes(&mut self);

    /// Syncs a node in the db with the file system
    fn index_single_node(&mut self, path: NodePath);

    /// Syncs the node's relationships in the db with the file system.
    fn index_node_connections(&mut self, path: NodePath);

    /// Delete all dead nodes from the graph. 
    fn cleanup_dead_nodes(&mut self);

    /// Set whether the library should maintain readable files for the nodes in the graph.
    fn maintain_readable_files(&mut self, maintain: bool);

    /// Gets the name of the root directory without the full path
    fn root_name(&self) -> String;


    // -------------------------------------------------------------------
    // Nodetypes 

    fn get_node_types(&self) -> Result<Vec<TypeName>, Box<dyn Error>>;

    fn create_nodetype(&mut self, nodetype: TypeName) -> Result<TypeName, Box<dyn Error>>;

    fn instance_nodetype(&self);

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
    ) -> Result<(), Box<dyn Error>>;

    /// Inserts a Node.
    fn insert_node(&mut self, node: Node) -> Result<(), agdb::DbError>;

    /// Deletes a node.
    ///
    /// Setting "files" and/or "dirs" to true could also delete from the file system,
    /// and recursively. Very dangerous. Though not implementing this would mean that
    /// those files would constantly be at a risk of getting reindexed, so this
    /// should probably still be implemented, unless we want to just mark nodes as deleted
    /// but never actually delete them, which seems like a smelly solution to me.
    fn delete_node(&self, path: PathBuf, files: bool, dirs: bool) -> Result<(), agdb::DbError>;

    /// Insert attributes to a node. Ignore reserved attribute names. Update attributes that already exist.
    fn insert_node_attr(
        &mut self,
        path: NodePath,
        attr: Attribute,
    ) -> Result<(), Box<dyn Error>>;

    /// Delete attributes from a node. Ignore reserved attribute names.
    fn delete_node_attr(
        &mut self,
        path: NodePath,
        attr_name: &str,
    );

    /// Merges a vector of nodes into the last one.
    fn merge_nodes(&self, nodes: Vec<NodePath>) -> Result<(), agdb::DbError>;

    // pub fn set_relative_positions

    // pub fn set_node_pins

    // pub fn set_pin_on nodes


    // -------------------------------------------------------------------
    // Edges

    fn create_edge(
        &mut self,
        source_path: &NodePath,
        target_path: &NodePath,
    ) -> Result<(), Box<dyn Error>>;

    /// Mostly used internally.
    /// Uses agdb types directly to create an exclusive parent-child connection.
    /// The attribute is "contains" and is reserved in elements.rs.
    fn parent_nodes_by_dbids(db: &mut agdb::Db, parent: &agdb::DbId, child: &agdb::DbId);

    /// Changes the parent directory of a node. If the node is physical, it will be moved in the file system.
    /// If the node is virtual, the parent will be changed in the db.
    /// Note that due to the implementation, all children of the node will have to be reindexed, recursively.
    fn reparent_node(
        &self,
        node_path: &NodePath,
        new_parent_path: &NodePath,
    ) -> Result<(), Box<dyn Error>>;

    /// Moves an edge and all its attributes to a new source and target. Parent edges can't be reconnected this way,
    /// use the reparent_node function instead.
    fn reconnect_edge(
        &self,
        edge: Edge,
        from: &NodePath,
        to: &NodePath,
    );

    fn insert_edge(&self, edge: Edge) -> Result<(), Box<dyn Error>>;

    /// Delete an edge from the graph. Edges with the attribute "contains" refer to the parent-child relationship
    /// between nodes and will be ignored. All other attributes will be cleared from them instead.
    fn delete_edge(&self, edge: Edge) -> Result<(), Box<dyn Error>>;

    /// Insert attributes to an edge. Ignore reserved attribute names. Update attributes that already exist.
    fn insert_edge_attr(&self, edge: Edge, attr: Attribute) -> Result<(), Box<dyn Error>>;

    /// Delete attributes from an edge. Ignore reserved attribute names.
    fn delete_edge_attr(&self, edge: Edge, attr: Attribute) -> Result<(), Box<dyn Error>>;
}
