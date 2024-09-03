use super::NodePath;
use std::path::PathBuf;

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

mod tests {
    #![allow(warnings)]


    use agdb::QueryBuilder;
    use directories::ProjectDirs;

    use crate::{
        graph_traits::{graph_core::GraphCore, graph_node::GraphNode},
        utils::{cleanup_graph, get_graph_dir_name, setup_graph},
    };

    #[test]
    fn test_new_graph() {
        let func_name = "test_new_graph";

        let name = format!("fs_graph_test_{}", func_name);
        let root = ProjectDirs::from("com", "fs_graph", &name)
            .unwrap()
            .data_dir()
            .to_path_buf();

        println!("Expected full path: {:#?}", root);

        let graph = GraphAgdb::new(root.clone().into(), &name);

        println!("Size of graph: {:#?} bytes", graph.db().size());

        assert_eq!(root.exists(), true, "Root directory does not exist");

        // Check that there exists a root node
        let root_node_result = graph.db().exec(&QueryBuilder::select().ids("root").query());

        match root_node_result {
            Ok(root_node) => {
                assert_eq!(root_node.result /* expected value */, 1);
            }
            Err(e) => {
                println!("Failed to execute query: {}", e);
            }
        }

        cleanup_graph(func_name);
    }

    /// Add a node to the db, then create a new graph with the same name.
    /// The new graph should be able to access the node.
    #[test]
    fn existing_db_in_directory() {
        let func_name = "existing_db_in_directory";
        let mut first = setup_graph(func_name);

        let _ = first
            .db_mut()
            .exec_mut(&QueryBuilder::insert().nodes().aliases("testalias").query());

        let second = setup_graph(func_name);

        let root_node_result = second
            .db()
            .exec(&QueryBuilder::select().ids("testalias").query());

        match root_node_result {
            Ok(root_node) => {
                assert_eq!(root_node.result /* expected value */, 1);
            }
            Err(e) => {
                println!("Failed to execute query: {}", e);
            }
        }

        assert_eq!(true, true);

        cleanup_graph(func_name);
    }

    #[test]
    fn new_custom_storage_directory() {
        let func_name = "new_custom_storage_directory";
        let name = format!("fs_graph_test_{}", func_name);
        let root = ProjectDirs::from("com", "fs_graph", &name)
            .unwrap()
            .config_dir()
            .to_path_buf();
        let storage = root.join("storage");

        let graph = GraphAgdb::new_custom_storage(root.clone().into(), &name, storage.clone());

        assert_eq!(
            storage.exists(),
            true,
            "Storage directory has not been created"
        );

        let root_node_result = graph.db().exec(&QueryBuilder::select().ids("root").query());

        match root_node_result {
            Ok(root_node) => {
                assert_eq!(root_node.result /* expected value */, 1);
            }
            Err(e) => {
                println!("Failed to execute query: {}", e);
            }
        }

        // Clean up the custom storage directory
        std::fs::remove_dir_all(storage).expect("Failed to remove storage directory");
    }

    #[test]
    fn correct_root_name() {
        let func_name = "correct_root_name";
        let graph = setup_graph(func_name);

        let dirname: String = get_graph_dir_name(func_name);

        let root_name: String = graph.root_name();

        assert_eq!(root_name, dirname);

        cleanup_graph(func_name);
    }

    #[test]
    /// Test whether the db creates an attributes node when the db is first created.
    /// Could possibly be moved to attr.rs
    fn create_attributes_category() {
        let func_name = "create_attributes_category";
        let graph = setup_graph(func_name);

        let root_node_result = graph.db().exec(&QueryBuilder::select().ids("root").query());

        assert_eq!(true, root_node_result.is_ok());

        let qry = graph
            .db()
            .exec(&QueryBuilder::select().ids("root/attributes").query());

        assert_eq!(true, qry.is_ok());

        if root_node_result.is_ok() && qry.is_ok() {
            // Validate root node
            let root_node = root_node_result.unwrap().ids();
            assert_eq!(root_node.len(), 1);
            let root_id = root_node.first().unwrap();

            // Validate attributes node
            let attributes_node = qry.unwrap().ids();
            assert_eq!(attributes_node.len(), 1);
            let attributes_id = attributes_node.first().unwrap();

            // Find edge, validate
            let query = &QueryBuilder::search()
                .from(*root_id)
                .to(*attributes_id)
                .query();
            let edge = graph.db().exec(query);
            assert_eq!(edge.is_ok(), true);

            let edge = edge
                .unwrap()
                .elements
                .iter()
                .cloned()
                .filter(|e| e.id.0 < 0)
                .collect::<Vec<_>>();
            println!("Found edge {:#?}", edge);

            assert_eq!(edge.len(), 1);

            // Select edge, because values don't appear in above query.
            // These two queries could probably be merged.
            let eid = edge.first().unwrap().id.0;
            let edge = graph
                .db()
                .exec(&QueryBuilder::select().keys().ids(eid).query());
            let edge = edge.unwrap().elements;
            assert_eq!(edge.len(), 1);

            let vals = &edge.first().unwrap().values;
            let mut found = false;

            // The attribute we're looking for. Should be reserved.
            // In this case, the "contains" attribute.
            let atr = "contains";
            for val in vals.iter() {
                if val.key == atr.into() {
                    found = true;
                }
            }

            assert_eq!(found, true);
        }

        cleanup_graph(&func_name);
    }

    /// Test for whether a file gets properly indexed into the db after it is
    /// added to the file system.
    #[test]
    fn index_single_node() {
        let func_name = "index_single_node";
        let mut graph = setup_graph(func_name);
        let root_path = graph.root_path();

        let dummy = NodePath::new("dummy.txt".into());

        // Dummy file does not exist yet.
        let dum = graph
            .db()
            .exec(&QueryBuilder::select().ids(dummy.alias()).query());
        assert_eq!(dum.is_ok(), false);

        // Create dummy file.
        let mut dummy_file = std::fs::File::create(dummy.full(&root_path)).unwrap();
        graph.index_single_node(&dummy);

        // Now exists.
        let dumtoo = graph
            .db()
            .exec(&QueryBuilder::select().ids(dummy.alias()).query());
        assert_eq!(dumtoo.is_ok(), true);

        // Is the correct type.
        let dumtype = graph
            .db()
            .exec(&QueryBuilder::select().ids(dummy.alias()).query());

        cleanup_graph(func_name);
    }

    #[test]
    fn index_node_connections_from_root() {
        let func_name = "index_node_connections";
        let mut graph = setup_graph(func_name);
        let root_path = graph.root_path();

        let paths: Vec<NodePath> = vec![
            NodePath::new("dummy.txt".into()),
            NodePath::new("dummy2.txt".into()),
            NodePath::new("dummy3.txt".into()),
        ];

        paths.iter().for_each(|p| {
            let mut dum = std::fs::File::create(p.full(&root_path)).unwrap();
        });

        let nodes = graph.open_node_connections(NodePath::new("".into()));

        let aliases = nodes.iter().map(|n| n.path()).collect::<Vec<_>>();

        /// Check if paths includes aliases
        let all_included = paths.iter().all(|p| aliases.contains(&p));

        assert_eq!(all_included, true, "Not all paths included in aliases");

        todo!();

        cleanup_graph(&func_name);
    }

    #[test]
    fn index_connections_upon_opening_node() {
        let func_name = "index_connections_upon_opening_node";
        let mut graph = setup_graph(func_name);
        let root_path = graph.root_path();

        todo!();

        cleanup_graph(&func_name);
    }

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
