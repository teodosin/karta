use super::{node_path::NodePath, StoragePath};
use std::path::PathBuf;

pub(crate) trait GraphCore {
    fn storage_path(&self) -> StoragePath;

    fn userroot_path(&self) -> PathBuf;

    fn root_nodepath(&self) -> NodePath;

    fn userroot_nodepath(&self) -> NodePath;

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

    fn new(name: &str, root_path: PathBuf, custom_storage_path: Option<PathBuf>) -> Self;

    /// Create the initial archetype nodes for the graph. Includes
    /// the root,
    /// attributes,
    /// settings,
    /// nodetypes
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

mod tests {
    #![allow(warnings)]

    use std::path::PathBuf;

    use directories::ProjectDirs;

    use crate::{
        elements::node_path::NodePath,
        graph_agdb::GraphAgdb,
        graph_traits::{graph_core::GraphCore, graph_edge::GraphEdge, graph_node::GraphNode, StoragePath},
        utils::TestContext,
    };

    /// Add a node to the db, then create a new graph with the same name.
    /// The new graph should be able to access the node.
    #[test]
    fn graph_with_same_name_exists__use_the_existing_and_dont_create_new() {
        let func_name = "graph_with_same_name_exists__use_the_existing_and_dont_create_new";

        let mut first = TestContext::new(func_name);

        let node_path = NodePath::new(PathBuf::from("test"));

        first.graph.create_node_by_path(&node_path, None);

        let second = TestContext::new(func_name);

        let root_node_result = second.graph.open_node(&node_path);

        println!("Root node result: {:#?}", root_node_result);

        assert_eq!(root_node_result.is_ok(), true);
    }

    #[test]
    fn create_graph_db_file_in_custom_storage_directory() {
        let func_name = "create_graph_db_file_in_custom_storage_directory";
        let ctx = TestContext::custom_storage(func_name);

        let storage = ctx.graph.storage_path().strg_path();

        assert_eq!(storage.is_some(), true, "Storage path must be set");
        let storage = storage.unwrap();
        assert_eq!(storage.exists(), true);

        assert_eq!(
            storage.exists(),
            true,
            "Storage directory has not been created"
        );

        assert_eq!(
            ctx.graph.userroot_path().exists(),
            true,
            "Graph was not created in storage directory"
        );

        assert_eq!(
            ctx.graph.storage_path(),
            crate::graph_traits::StoragePath::Custom(storage.clone())
        );

        let root_node_result = ctx.graph.open_node(&NodePath::root());

        assert_eq!(root_node_result.is_ok(), true);

        // Clean up the custom storage directory
        std::fs::remove_dir_all(storage).expect("Failed to remove storage directory");
    }

    #[test]
    fn creating_new_graph__creates_userroot_node() {
        let func_name = "creating_new_graph__creates_userroot_node";
        let ctx = TestContext::new(func_name);

        let root_path = NodePath::root();

        let userroot_path = ctx.graph.userroot_nodepath();

        assert_eq!(userroot_path.parent().unwrap(), root_path);

    }

    #[test]
    /// Test whether the db creates attributes/settings/etc. nodes when the db is first created.
    fn creating_new_graph_creates_archetype_nodes() {
        let func_name = "creating_new_graph_creates_archetype_nodes";
        let ctx = TestContext::new(func_name);

        let root_path = NodePath::root();
        let root_node = ctx.graph.open_node(&root_path);

        assert_eq!(root_node.is_ok(), true, "Root node not found");



        let atr_path = NodePath::new("attributes".into());
        let atr_node = ctx.graph.open_node(&atr_path);

        assert_eq!(atr_node.is_ok(), true, "Attributes node not found");

        let edge = ctx.graph.get_edge(&root_path, &atr_path);
        assert_eq!(edge.is_ok(), true, "Edge not found");

        

        let settings_path = NodePath::new("settings".into());
        let settings_node = ctx.graph.open_node(&settings_path);

        assert_eq!(
            settings_node.is_ok(),
            true,
            "Settings node not found"
        );

        let edge = ctx.graph.get_edge(&root_path, &settings_path);
        assert_eq!(edge.is_ok(), true, "Edge not found");



        let nodetypes_path = NodePath::new("nodetypes".into());
        let nodetypes_node = ctx.graph.open_node(&nodetypes_path);

        assert_eq!(
            nodetypes_node.is_ok(),
            true,
            "Node types node not found"
        );

        let edge = ctx.graph.get_edge(&root_path, &nodetypes_path);
        assert_eq!(edge.is_ok(), true, "Edge not found");
    }

    // /// Test for whether a file gets properly indexed into the db after it is
    // /// added to the file system.
    // #[test]
    // fn index_single_node() {
    //     let func_name = "index_single_node";
    //     let mut graph = setup_graph(func_name);
    //     let root_path = graph.root_path();

    //     let dummy = NodePath::new("dummy.txt".into());

    //     // Dummy file does not exist yet.
    //     let dum = graph
    //         .db()
    //         .exec(&QueryBuilder::select().ids(dummy.alias()).query());
    //     assert_eq!(dum.is_ok(), false);

    //     // Create dummy file.
    //     let mut dummy_file = std::fs::File::create(dummy.full(&root_path)).unwrap();
    //     graph.index_single_node(&dummy);

    //     // Now exists.
    //     let dumtoo = graph
    //         .db()
    //         .exec(&QueryBuilder::select().ids(dummy.alias()).query());
    //     assert_eq!(dumtoo.is_ok(), true);

    //     // Is the correct type.
    //     let dumtype = graph
    //         .db()
    //         .exec(&QueryBuilder::select().ids(dummy.alias()).query());

    //     cleanup_graph(func_name);
    // }

    // #[test]
    // fn index_node_connections_from_root() {
    //     let func_name = "index_node_connections";
    //     let mut graph = setup_graph(func_name);
    //     let root_path = graph.root_path();

    //     let paths: Vec<NodePath> = vec![
    //         NodePath::new("dummy.txt".into()),
    //         NodePath::new("dummy2.txt".into()),
    //         NodePath::new("dummy3.txt".into()),
    //     ];

    //     paths.iter().for_each(|p| {
    //         let mut dum = std::fs::File::create(p.full(&root_path)).unwrap();
    //     });

    //     let nodes = graph.open_node_connections(NodePath::new("".into()));

    //     let aliases = nodes.iter().map(|n| n.path()).collect::<Vec<_>>();

    //     /// Check if paths includes aliases
    //     let all_included = paths.iter().all(|p| aliases.contains(&p));

    //     assert_eq!(all_included, true, "Not all paths included in aliases");

    //     todo!();

    //     cleanup_graph(&func_name);
    // }

    // #[test]
    // fn index_connections_upon_opening_node() {
    //     let func_name = "index_connections_upon_opening_node";
    //     let mut graph = setup_graph(func_name);
    //     let root_path = graph.root_path();

    //     todo!();

    //     cleanup_graph(&func_name);
    // }

    // Loading an old db with a new root directory!
    // Should this be allowed or prevented? For usability it would be nice if you could just
    // change the root directory to beyond or within the previous one.
    //
    // This has to be handled carefully though. If each node stores its path relative to the root,
    // then the path of the node will be incorrect if the root directory is changed. Every node in the
    // entire db would have to be updated. On the other hand, if the path is stored as an absolute path,
    // then moving the root folder would break those. And if each node only stores its own name, then finding
    // the path of a node would be a slower operation. Also it seems like agdb doesn't support having the
    // same aliases for multiple nodes, so only storing the names wouldn't be feasible anyway.
    //
    // One has to consider also that a large portion of the db could be rearranged without changing the root
    // directory, meaning that there would still be a lot of updates needed.

    // Test for what happens when a db is moved to a different directory, but the root directory is the same.
}
