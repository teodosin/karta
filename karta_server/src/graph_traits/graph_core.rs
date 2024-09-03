use std::path::PathBuf;
use super::NodePath;

pub(crate) trait GraphCore {
    fn root_path(&self) -> PathBuf;

    fn root_nodepath(&self) -> NodePath;
    
    /// Constructor. Panics if the db cannot be created.
    ///
    /// Takes the desired root of the graph as a parameter and the name for the db.
    ///
    /// Creates the db at the storage_path, or initialises the db if it already exists there.
    ///
    /// Note that it uses PathBuf instead of NodePath, because of course
    /// it's not dealing with nodes yet. 
    /// 
    /// TODO: Add error handling.
    
    fn new(root_path: PathBuf, name: &str) -> Self;

    /// Alternate constructor. Use this if you want to set a custom storage path for the db.
    /// Panics if the db cannot be created
    fn new_custom_storage(root_path: PathBuf, name: &str, storage_path: PathBuf) -> Self;

    /// Create the initial archetype nodes for the graph. Includes 
    /// the root, 
    /// attributes,
    /// settings,
    /// nodecategories
    fn init_archetype_nodes(&mut self);

    /// Syncs a node in the db with the file system
    fn index_single_node(&mut self, path: &NodePath);

    /// Syncs the node's relationships in the db with the file system.
    fn index_node_connections(&mut self, path: &NodePath);

    /// Delete all dead nodes from the graph. 
    fn cleanup_dead_nodes(&mut self);

    /// Set whether the library should maintain readable files for the nodes in the graph.
    fn maintain_readable_files(&mut self, maintain: bool);

    /// Gets the name of the root directory without the full path
    fn root_name(&self) -> String;

}