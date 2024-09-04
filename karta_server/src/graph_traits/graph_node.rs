use std::{error::Error, path::PathBuf};

use crate::nodetype::TypeName;

use super::{attribute::Attribute, node::Node, node_path::NodePath};


pub(crate) trait GraphNode {
    // -------------------------------------------------------------------
    // Nodes

    /// Retrieves a particular node's data from the database.
    /// The path is relative to the root of the graph.
    fn open_node(&self, path: &NodePath) -> Result<Node, Box<dyn Error>>;

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
    fn open_node_connections(&self, path: &NodePath) -> Vec<Node>;

    /// Creates a node from the given path. Inserts it into the graph.
    /// Insert the relative path from the root, not including the root dir.
    ///
    /// TODO: Determine whether users of the crate are meant to use this.
    /// Perhaps not. Perhaps the parent of the node should be specified.
    /// The insert_node_by_name function calls this one anyway.
    fn create_node_by_path(
        &mut self, path: &NodePath, ntype: Option<TypeName>
    ) -> Result<Node, Box<dyn Error>>;

    /// Creates a node under a given parent with the given name.
    /// The path is relative to the root of the graph.
    /// Do not include the root dir name.
    fn create_node_by_name(
        &mut self,
        parent_path: Option<NodePath>,
        name: &str,
        ntype: Option<TypeName>,
    ) -> Result<Node, Box<dyn Error>>;

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
    fn get_node_attrs(&self, path: NodePath) -> Result<Vec<Attribute>, Box<dyn Error>>;

    /// Delete attributes from a node. Ignore reserved attribute names.
    fn delete_node_attr(&mut self, path: NodePath, attr_name: &str) -> Result<(), Box<dyn Error>>;

    /// Merges a vector of nodes into the last one.
    fn merge_nodes(&mut self, nodes: Vec<NodePath>) -> Result<(), agdb::DbError>;

    // pub fn set_relative_positions

    // pub fn set_node_pins

    // pub fn set_pin_on nodes

    /// Used internally.
    /// Uses agdb types directly to create an exclusive parent-child connection.
    /// The attribute is "contains" and is reserved in elements.rs.
    fn autoparent_nodes(
        &mut self, parent: &NodePath, child: &NodePath
    ) -> Result<(), Box<dyn Error>>;
}



// --------------------------------------------------------------------



mod tests {
    #![allow(warnings)]

    use crate::{
        elements::node_path::NodePath, graph_agdb::GraphAgdb, graph_traits::graph_edge::GraphEdge, utils::TestGraph
    };
    use agdb::QueryBuilder;

    use std::path::PathBuf;

    use crate::graph_traits::{graph_core::GraphCore, graph_node::GraphNode};

    #[test]
    fn fresh_graph_root_node_should_exist_and_be_openable() {
        let func_name = "fresh_graph_root_node_should_exist_and_be_openable";
        let file = TestGraph::new(func_name);
        let graph = file.setup();

        let root_path = NodePath::root();
        let root_node = graph.open_node(&root_path);

        assert_eq!(root_node.is_ok(), true, "Root node should exist, thought it don'teth");
    }

    #[test]
    fn create_node_and_open_it() {
        let func_name = "create_node_and_open_it";
        let file = TestGraph::new(func_name);
        let mut graph = file.setup();

        let path = NodePath::new("test".into());
        let node = graph.create_node_by_path(&path, None);

        assert_eq!(node.is_ok(), true, "Node should be created");
        let created_node = node.unwrap();

        let opened_node = graph.open_node(&path);

        assert_eq!(opened_node.is_ok(), true, "Node should be opened");
        let opened_node = opened_node.unwrap();

        println!("Created node: {:#?}", created_node);
        println!("Opened node: {:#?}", opened_node);

        assert_eq!(created_node.path(), opened_node.path(), "Node paths should be equal");
        assert_eq!(created_node.name(), opened_node.name(), "Node names should be equal");
        assert_eq!(created_node.ntype_name(), opened_node.ntype_name(), "Node type names should be equal");
    }

    #[test]
    fn opening_node_that_does_not_exist_fails() {
        let func_name = "opening_node_that_does_not_exist_fails";
        let file = TestGraph::new(func_name);
        let graph = file.setup();

        let path = NodePath::from("test");

        let node = graph.open_node(&path);

        assert_eq!(node.is_ok(), false, "Node should not be found nor opened");
    }

    #[test]
    fn can_create_nodes_with_long_paths() {
        let func_name = "can_create_nodes_with_long_paths";
        let file = TestGraph::new(func_name);
        let mut graph = file.setup();

        let long_path = NodePath::from("this/is/a/long/path/with/many/segments/verylongindeed/evenlonger/wow/are/we/still/here/there/must/be/something/we/can/do/about/all/this/tech/debt");

        let node = graph.create_node_by_path(&long_path, None);

        assert_eq!(node.is_ok(), true, "Node should be created even with long paths");
    }

    // #[test]
    // fn opening_root_connections() {
    //     let func_name = "opening_node_connections";
    //     let mut graph = setup_graph(func_name);

    //     todo!();

    //     cleanup_graph(func_name);
    // }

    // #[test]
    // fn opening_node_connections() {
    //     let func_name = "opening_node_connections";
    //     let mut graph = setup_graph(func_name);

    //     todo!();

    //     cleanup_graph(func_name);
    // }


    // /// When a node is created, it should have a path to the root. If a node is created with a deep
    // /// path, then intermediate nodes should be created.
    // ///
    // /// It is unclear whether this should be optional.
    // /// Technically this could prevent orphans from being created.
    // /// Are they useful?
    // /// Or should nodes with only a parent connection be considered relatively disconnected?
    // /// Presumably it would be useful to be able to just dump new nodes in and sort them later.
    // /// But that could still be implemented without fragmenting the database.
    #[test]
    fn creating_node_with_deep_path_creates_intermediate_nodes() {
        let func_name = "creating_node_with_deep_path_creates_intermediate_nodes";
        let file = TestGraph::new(func_name);
        let mut graph = file.setup();

        let path = NodePath::from("one/two/three");
        let mut parent = path.clone().parent().unwrap();
        let mut grandparent = parent.clone().parent().unwrap();

        // only create the child
        let node = graph.create_node_by_path(&path, None);
        assert_eq!(node.is_ok(), true, "Node should be created");

        // parent and grandparent should also exist now
        let parent_node = graph.open_node(&parent);
        assert_eq!(parent_node.is_ok(), true, "Parent node should exist");

        let grandparent_node = graph.open_node(&grandparent);
        assert_eq!(grandparent_node.is_ok(), true, "Grandparent node should exist");

        // make sure they are connected by edges
        let parent_to_child_edge = graph.get_edge(&parent, &path);
        assert_eq!(parent_to_child_edge.is_ok(), true, "Parent to child edge should exist");
        let pce = parent_to_child_edge.unwrap();
        assert_eq!(pce.contains(), true, "Parent to child should be physical");
        assert_eq!(*pce.source(), parent, "Parent should be source");
        assert_eq!(*pce.target(), path, "Child should be target");

        let grandparent_to_parent_edge = graph.get_edge(&grandparent, &parent);
        assert_eq!(grandparent_to_parent_edge.is_ok(), true, "Grandparent to parent edge should exist");
        let gpe = grandparent_to_parent_edge.unwrap();
        assert_eq!(gpe.contains(), true, "Grandparent to parent edge should be physical");
        assert_eq!(*gpe.source(), grandparent, "Grandparent to parent edge should have correct source path");
        assert_eq!(*gpe.target(), parent, "Grandparent to parent edge should have correct target path");
    }


    // #[test]
    // fn creating_deep_path_creates_intermediate_nodes() {
    //     let func_name = "creating_deep_path_creates_intermediate_nodes";
    //     let mut graph = setup_graph(func_name);

    //     let path = NodePath::from("one/two/three");
    //     let mut first = path.clone().parent().unwrap();
    //     let mut second = first.clone().parent().unwrap();

    //     let node = graph.create_node_by_path(path.clone(), None);

    //     assert_eq!(node.is_ok(), true);

    //     let node = graph
    //         .db()
    //         .exec(&QueryBuilder::select().ids(path.alias()).query());

    //     let fir = graph
    //         .db()
    //         .exec(&QueryBuilder::select().ids(first.alias()).query());

    //     let sec = graph
    //         .db()
    //         .exec(&QueryBuilder::select().ids(second.alias()).query());

    //     assert_eq!(node.is_ok(), true);
    //     assert_eq!(fir.is_ok(), true);
    //     assert_eq!(sec.is_ok(), true);

    //     let elems = graph
    //         .db()
    //         .exec(&QueryBuilder::select().node_count().query());
    //     let aliases = graph.db().exec(&QueryBuilder::select().aliases().query());

    //     assert_eq!(elems.is_ok(), true);
    //     assert_eq!(aliases.is_ok(), true);

    //     let nodes = aliases.unwrap().elements;
    //     let edges = elems.unwrap();
    //     let edges = edges.elements;

    //     // NOTE: The below assertions are commented out because the amount of
    //     // nodes created at startup is not set in stone. The most recent breakage
    //     // was because of adding a nodetypes node. No point in breaking a test
    //     // every time that happens.

    //     // let edges = edges.elements.iter().filter(|x| x.id.0 < 0).collect::<Vec<_>>();
    //     // Length is 6:
    //     // root, attributes, settings, one, two, three
    //     // assert_eq!(nodes.len(), 6);

    //     // Length is 5:
    //     // root
    //     // - one
    //     //   - two
    //     //     - three
    //     // - attributes
    //     // - settings
    //     // - = edge

    //     // todo! Fix this test. Can't find a way to just get all edges...
    //     // println!("Edges: {:#?}", edges);
    //     // assert_eq!(edges.len(), 5);

    //     cleanup_graph(&func_name);
    // }

    // #[test]
    // fn insert_and_delete_node_attribute() {
    //     let func_name = "insert_and_delete_node_attribute";
    //     let mut graph = setup_graph(func_name);

    //     let path = NodePath::new("test".into());

    //     let attr = Attribute {
    //         name: "test".to_string(),
    //         value: 10.0,
    //     };

    //     let node = graph.create_node_by_path(path.clone(), None);
    //     assert_eq!(node.is_ok(), true);

    //     let added = graph.insert_node_attrs(path.clone(), vec![attr]);
    //     assert_eq!(added.is_ok(), true);

    //     let noder = graph.open_node(path.clone());
    //     let noder = noder.unwrap();

    //     assert_eq!(noder.attributes().len(), 1);

    //     assert_eq!(noder.attributes()[0].name, "test");
    //     assert_eq!(noder.attributes()[0].value, 10.0);

    //     // Test deleting the attribute
    //     let deleted = graph.delete_node_attr(path.clone(), "test");
    //     assert_eq!(deleted.is_ok(), true);

    //     let nodest = graph.open_node(path.clone());
    //     let nodest = nodest.unwrap();
    //     assert_eq!(nodest.attributes().len(), 0);

    //     cleanup_graph(&func_name);
    // }

    // /// Insertion of attributes on non-existing nodes should fail.
    // /// Insertion of attributes on a non-existing node shouldn't
    // /// create the node.
    // #[test]
    // fn insertion_of_attributes_on_nonexisting_node() {
    //     let func_name = "insertion_of_attributes_on_nonexisting_node";
    //     let mut graph = setup_graph(func_name);

    //     let path = NodePath::new("test".into());

    //     let attr = Attribute {
    //         name: "test".to_string(),
    //         value: 10.0,
    //     };

    //     let shouldfail = graph.insert_node_attrs(path.clone(), vec![attr.clone()]);
    //     assert_eq!(
    //         shouldfail.is_ok(),
    //         false,
    //         "Insertion of attr didn't fail, even though node doesn't exist"
    //     );

    //     cleanup_graph(&func_name);
    // }

    // #[test]
    // fn insert_and_delete_multiple_attributes() {
    //     let func_name = "insert_and_delete_multiple_attributes";
    //     let mut graph = setup_graph(func_name);

    //     let path = NodePath::new("test".into());

    //     let node = graph.create_node_by_path(path.clone(), None);

    //     todo!();

    //     cleanup_graph(&func_name);
    // }

    // #[test]
    // fn protect_reserved_node_attributes() {
    //     let func_name = "protect_reserved_attributes";
    //     let mut graph = setup_graph(func_name);

    //     use fs_graph::elements::RESERVED_NODE_ATTRS;

    //     let test_attr = "preview";
    //     assert!(RESERVED_NODE_ATTRS.contains(&test_attr));

    //     let path = NodePath::from("test");

    //     let node = graph.create_node_by_path(path.clone(), None);
    //     assert_eq!(node.is_ok(), true);

    //     let attr = Attribute {
    //         name: test_attr.to_string(),
    //         value: 10.0,
    //     };

    //     let added = graph.insert_node_attrs(path.clone(), vec![attr]);

    //     assert_eq!(added.is_ok(), true);
    //     let nod = graph.db().exec(
    //         &QueryBuilder::select()
    //             .values(vec![])
    //             .ids(path.alias())
    //             .query(),
    //     );
    //     assert!(nod.is_ok());
    //     let nods = nod.unwrap();
    //     let nods = nods.elements;
    //     assert_eq!(nods.len(), 1);
    //     let nods = nods.first().unwrap();
    //     let mut nods = &nods.values;
    //     let nods: Vec<agdb::DbValue> = nods.iter().map(|nod| nod.key.clone()).collect();
    //     assert!(!nods.contains(&"preview".into()));

    //     let removed = graph.delete_node_attr(path, "ntype");
    //     assert_eq!(removed.is_ok(), false);

    //     cleanup_graph(&func_name);
    // }

    /// Test creating a Node with different NodeTypes
    /// Test inserting an existing node (should fail or update)
    /// Test deleting a node
    /// Test reparenting a physical node
    /// Test reparenting a virtual node
    /// Test inserting node attributes (normal and reserved)
    /// Test deleting node attributes (normal and reserved)
    /// Test merging two nodes
    /// Test inserting path as alias for a node
    /// Test node operations with very deep directory structures
    /// Test node operations with many sibling directories/files
    /// Test converting NodePath to and from DbValue
    /// Test converting NodePhysicality to and from DbValue
    /// Test converting NodeType to and from DbValue
    #[test]
    fn todo_tests() {
        assert_eq!(2 + 2, 4);
    }
}
