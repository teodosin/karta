use std::{error::Error, path::PathBuf};

use crate::elements::nodetype::NodeType;

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
        &mut self,
        path: &NodePath,
        ntype: Option<NodeType>,
    ) -> Result<Node, Box<dyn Error>>;

    /// Creates a node under a given parent with the given name.
    /// The path is relative to the root of the graph.
    /// Do not include the root dir name.
    fn create_node_by_name(
        &mut self,
        parent_path: Option<NodePath>,
        name: &str,
        ntype: Option<NodeType>,
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

    /// Get node attributes
    fn get_node_attrs(&self, path: &NodePath) -> Result<Vec<Attribute>, Box<dyn Error>>;

    /// Insert attributes to a node. Ignore reserved attribute names. Update attributes that already exist.
    fn insert_node_attrs(
        &mut self,
        path: &NodePath,
        attrs: Vec<Attribute>,
    ) -> Result<(), Box<dyn Error>>;

    /// Delete attributes from a node. Ignore reserved attribute names.
    fn delete_node_attrs(
        &mut self,
        path: &NodePath,
        attr_name: Vec<&str>,
    ) -> Result<(), Box<dyn Error>>;

    /// Merges a vector of nodes into the last one.
    fn merge_nodes(&mut self, nodes: Vec<NodePath>) -> Result<(), agdb::DbError>;

    // pub fn set_relative_positions

    // pub fn set_node_pins

    // pub fn set_pin_on nodes

    /// Used internally.
    /// Uses agdb types directly to create an exclusive parent-child connection.
    /// The attribute is "contains" and is reserved in elements.rs.
    fn autoparent_nodes(
        &mut self,
        parent: &NodePath,
        child: &NodePath,
    ) -> Result<(), Box<dyn Error>>;
}

// --------------------------------------------------------------------

mod tests {
    #![allow(warnings)]

    use crate::{
        elements::{attribute::{Attribute, RESERVED_NODE_ATTRS}, node, node_path::NodePath},
        graph_agdb::GraphAgdb,
        graph_traits::graph_edge::GraphEdge,
        utils::TestContext,
    };
    use agdb::QueryBuilder;

    use std::{path::PathBuf, vec};

    use crate::graph_traits::{graph_core::GraphCore, graph_node::GraphNode};

    #[test]
    fn create_node_and_open_it() {
        let func_name = "create_node_and_open_it";
        let mut ctx = TestContext::new(func_name);

        // Creating node
        let path = NodePath::new("test".into());
        let node = ctx.graph.create_node_by_path(&path, None);

        assert_eq!(node.is_ok(), true, "Node should be created");
        let created_node = node.unwrap();

        // Check connection with user_root
        let user_root = NodePath::user_root();
        assert_eq!(path.parent().unwrap(), user_root, "Node path should be child of user_root");

        // Opening node
        let opened_node = ctx.graph.open_node(&path);

        assert_eq!(opened_node.is_ok(), true, "Node should be opened");
        let opened_node = opened_node.unwrap();

        println!("Created node: {:#?}", created_node);
        println!("Opened node: {:#?}", opened_node);



        assert_eq!(
            created_node.path(),
            opened_node.path(),
            "Node paths should be equal"
        );
        assert_eq!(
            created_node.name(),
            opened_node.name(),
            "Node names should be equal"
        );
        assert_eq!(
            created_node.ntype_name(),
            opened_node.ntype_name(),
            "Node type names should be equal"
        );
    }

    #[test]
    fn opening_node_that_does_not_exist_fails() {
        let func_name = "opening_node_that_does_not_exist_fails";
        let ctx = TestContext::new(func_name);

        let path = NodePath::from("test");

        let node = ctx.graph.open_node(&path);

        assert_eq!(node.is_ok(), false, "Node should not be found nor opened");
    }

    #[test]
    fn can_create_nodes_with_long_paths() {
        let func_name = "can_create_nodes_with_long_paths";
        let mut ctx = TestContext::new(func_name);

        // let long_path = NodePath::from("this/is/a/long/path/with/many/segments/verylongindeed/evenlonger/wow/are/we/still/here/there/must/be/something/we/can/do/about/all/this/tech/debt");
        let mut long_buf = PathBuf::from("");
        let depth = 30;
        for i in 0..depth {
            long_buf = long_buf.join(i.to_string());
        }
        let long_path = NodePath::new(long_buf);

        let node = ctx.graph.create_node_by_path(&long_path, None);

        assert_eq!(
            node.is_ok(),
            true,
            "Node should be created even with long paths"
        );
    }

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
        let mut ctx = TestContext::new(func_name);

        let path = NodePath::from("one/two/three");
        let mut parent = path.clone().parent().unwrap();
        let mut grandparent = parent.clone().parent().unwrap();

        // only create the child
        let node = ctx.graph.create_node_by_path(&path, None);
        assert_eq!(node.is_ok(), true, "Node should be created");

        // parent and grandparent should also exist now
        let parent_node = ctx.graph.open_node(&parent);
        assert_eq!(parent_node.is_ok(), true, "Parent node should exist");

        let grandparent_node = ctx.graph.open_node(&grandparent);
        assert_eq!(
            grandparent_node.is_ok(),
            true,
            "Grandparent node should exist"
        );

        // make sure they are connected by edges
        let parent_to_child_edge = ctx.graph.get_edge_strict(&parent, &path);
        assert_eq!(
            parent_to_child_edge.is_ok(),
            true,
            "Parent to child edge should exist"
        );
        let pce = parent_to_child_edge.unwrap();
        assert_eq!(pce.contains(), true, "Parent to child should be physical");
        assert_eq!(*pce.source(), parent, "Parent should be source");
        assert_eq!(*pce.target(), path, "Child should be target");

        let grandparent_to_parent_edge = ctx.graph.get_edge_strict(&grandparent, &parent);
        assert_eq!(
            grandparent_to_parent_edge.is_ok(),
            true,
            "Grandparent to parent edge should exist"
        );
        let gpe = grandparent_to_parent_edge.unwrap();
        assert_eq!(
            gpe.contains(),
            true,
            "Grandparent to parent edge should be physical"
        );
        assert_eq!(
            *gpe.source(),
            grandparent,
            "Grandparent to parent edge should have correct source path"
        );
        assert_eq!(
            *gpe.target(),
            parent,
            "Grandparent to parent edge should have correct target path"
        );
    }

    #[test]
    fn able_to_insert_and_delete_node_attributes() {
        let func_name = "able_to_insert_and_delete_node_attributes";
        let mut ctx = TestContext::new(func_name);
        
        let path = NodePath::from("test");
        let node = ctx.graph.create_node_by_path(&path, None);

        assert_eq!(node.is_ok(), true, "Node should be created");
        let node = node.unwrap();

        let attrs: Vec<Attribute> = vec![
            Attribute {
                name: "first_attr".to_string(),
                value: 10.0,
            },
            Attribute {
                name: "second_attr".to_string(),
                value: 20.0,
            },
        ];

        let added = ctx.graph.insert_node_attrs(&path, attrs.clone());
        assert_eq!(added.is_ok(), true, "Attributes should be added");

        let opened_node = ctx.graph.open_node(&path);
        assert_eq!(opened_node.is_ok(), true, "Node should be opened");
        let opened_node = opened_node.unwrap();

        let opened_attrs = opened_node.attributes();

        // All original attributes should be contained in opened_attrs
        for attr in attrs.iter() {
            assert_eq!(
                opened_attrs.contains(&attr),
                true,
                "Attribute {} should be contained in opened_attrs",
                attr.name
            )
        }

        let attr_names = attrs
            .iter()
            .map(|attr| &attr.name as &str)
            .collect::<Vec<&str>>();

        let deleted = ctx.graph.delete_node_attrs(&path, attr_names.clone());
        assert_eq!(deleted.is_ok(), true, "Attributes should be deleted");

        let opened_node_no_attrs = ctx.graph.open_node(&path);
        assert_eq!(opened_node_no_attrs.is_ok(), true, "Node should be opened");

        let opened_node_no_attrs = opened_node_no_attrs.unwrap();
        let opened_attrs_no_attrs = opened_node_no_attrs.attributes();

        // No original attributes should be contained in opened_atts_no_attrs
        for attr in attrs.iter() {
            assert_eq!(
                opened_attrs_no_attrs.contains(&attr),
                false,
                "Attribute {} should have been deleted",
                attr.name
            )
        }
    }

    #[test]
    fn insertion_of_attributes_on_nonexisting_node_should_fail() {
        let func_name = "insertion_of_attributes_on_nonexisting_node_should_fail";
        let mut ctx = TestContext::new(func_name);

        let fakepath = NodePath::from("fakepath");

        let attr = vec![Attribute {
            name: "test".to_string(),
            value: 10.0,
        }];

        let shouldfail = ctx.graph.insert_node_attrs(&fakepath, attr);

        assert_eq!(
            shouldfail.is_ok(),
            false,
            "Inserting attributes on a non-existing node should fail"
        );
    }

    #[test]
    fn reserved_node_attributes_cannot_be_inserted() {
        let func_name = "reserved_node_attributes_cannot_be_inserted";
        let mut ctx = TestContext::new(func_name);

        let path = NodePath::from("test");
        let node = ctx.graph.create_node_by_path(&path, None);
        assert_eq!(node.is_ok(), true, "Node should be created");

        let protected = RESERVED_NODE_ATTRS;

        protected.iter().for_each(|attr| {
            let attr = Attribute {
                name: attr.to_string(),
                value: 10.0,
            };

            let added = ctx.graph.insert_node_attrs(&path, vec![attr]);
            assert_eq!(added.is_ok(), false, "Attribute should not be added");
        });

    }

    #[test]
    fn reserved_node_attributes_cannot_be_deleted() {
        let func_name = "reserved_node_attributes_cannot_be_deleted";
        let mut ctx = TestContext::new(func_name);

        let path = NodePath::from("test");
        let node = ctx.graph.create_node_by_path(&path, None);
        assert_eq!(node.is_ok(), true, "Node should be created");

        let attr_names = RESERVED_NODE_ATTRS;

        attr_names.iter().for_each(|name| {
            let deleted = ctx.graph.delete_node_attrs(&path, vec![*name]);
            assert_eq!(deleted.is_ok(), false, "Reserved attribute {} should not be deleted", name);
        });
    }

    #[test]
    fn opening_root_node_connections__returns_vector_of_nodes() {
        let func_name = "opening_root_node_connections__returns_vector_of_nodes";
        let mut ctx = TestContext::new(func_name);

        let root_path = NodePath::root();

        
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
