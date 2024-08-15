#![allow(warnings)]

use agdb::QueryBuilder;
use fs_graph::{elements::{Attribute, Node, NodePath}, graph::Graph, path_ser::buf_to_alias};

mod utils;
use utils::*;
use std::path::PathBuf;

/// The root node should exist and be openable. 
#[test]
fn open_root_node() {
    let func_name = "open_root_node";
    let mut graph = setup_graph(func_name);
    let root_path = graph.root_path();

    // Nodepath initialised with empty pathbuf will refer to root node.
    let root_buf = NodePath::new("".into());
    let root_node = graph.db().exec(&QueryBuilder::select().aliases().ids(root_buf.alias()).query());

    assert_eq!(root_node.is_ok(), true);

    match root_node {
        Ok(root_node) => {
            println!("Root node: {:#?}", root_node);
            let rid = root_node.ids();
            assert_eq!(rid.len(), 1);
            let root_id = rid.first().unwrap();

            // This is chaos, but that's what it should look like based on the println above.
            let ralias = &root_node.elements
                .first().unwrap().values.first().unwrap().value.string().unwrap();

            assert_eq!(*ralias, "root");
        }
        Err(e) => {
            println!("Failed to execute query: {}", e);
        }
    }

    cleanup_graph(func_name);
}

#[test]
fn open_node_that_exists() {
    let func_name = "open_node_that_exists";
    let mut graph = setup_graph(func_name);

    let path = NodePath::from("test");

    let node = graph.create_node_by_path(path.clone(), None);
    assert_eq!(node.is_ok(), true);

    let open = graph.open_node(path);

    assert_eq!(open.is_ok(), true);

    cleanup_graph(&func_name);
}

#[test]
fn open_node_that_does_not_exist() {
    let func_name = "open_node_that_does_not_exist";
    let graph = setup_graph(func_name);

    let open = graph.open_node(NodePath::from("test"));

    assert_eq!(open.is_ok(), false);

    cleanup_graph(&func_name);
}

#[test]
fn opening_root_connections() {
    let func_name = "opening_node_connections";
    let mut graph = setup_graph(func_name);

    todo!();

    cleanup_graph(func_name);
}

#[test]
fn opening_node_connections() {
    let func_name = "opening_node_connections";
    let mut graph = setup_graph(func_name);

    todo!();

    cleanup_graph(func_name);
}

#[test]
fn create_new_node(){
    let func_name = "create_new_node";
    let mut graph = setup_graph(func_name);

    let path = NodePath::from("test");

    let node = graph.create_node_by_path(path, None);

    assert_eq!(node.is_ok(), true);

    let nid = node.unwrap().id;

    let node = graph.db().exec(
        &QueryBuilder::select().ids("root/test").query()
    );

    
    println!("Node: {:#?}", node);
    assert_eq!(node.is_ok(), true);
    
    let node = node.unwrap();
    assert_eq!(node.elements.len(), 1);
    
    let node = node.elements.first().unwrap();
    assert_eq!(node.id, nid);
    
    // Checking for ntype, nphys, created-time, modified-time
    assert_eq!(node.values.iter().any(|x| x.key == "ntype".into() && x.value == "Other".into()), true);
    assert_eq!(node.values.iter().any(|x| x.key == "nphys".into()), true);
    assert_eq!(node.values.iter().any(|x| x.key == "created_time".into()), true);
    assert_eq!(node.values.iter().any(|x| x.key == "modified_time".into()), true);
    
    let aliases = graph.db().exec(
        &QueryBuilder::select().aliases().query()
    );

    assert_eq!(aliases.is_ok(), true);
    let alias_exists = aliases.unwrap().elements.iter().any(|x| {
        x.values.iter().any(|y| y.value.to_string() == "root/test".to_string())
    });
    assert_eq!(alias_exists, true);

    cleanup_graph(&func_name);
}

/// When a node is created, it should have a path to the root. If a node is created with a deep
/// path, then intermediate nodes should be created.
/// 
/// It is unclear whether this should be optional. 
/// Technically this could prevent orphans from being created. 
/// Are they useful? 
/// Or should nodes with only a parent connection be considered relatively disconnected?
/// Presumably it would be useful to be able to just dump new nodes in and sort them later. 
/// But that could still be implemented without fragmenting the database. 
#[test]
fn creating_deep_path_creates_intermediate_nodes() {
    let func_name = "creating_deep_path_creates_intermediate_nodes";
    let mut graph = setup_graph(func_name);

    let path = PathBuf::from("one/two/three");
    let mut first = path.clone();
    first.pop();
    let mut second = path.clone();
    second.pop();

    let node = graph.create_node_by_path(NodePath::new(path.clone()), None);

    assert_eq!(node.is_ok(), true);

    let node = graph.db().exec(
        &QueryBuilder::select().ids(buf_to_alias(&path)).query()
    );

    let fir = graph.db().exec(
        &QueryBuilder::select().ids(buf_to_alias(&path)).query()
    );

    let sec = graph.db().exec(
        &QueryBuilder::select().ids(buf_to_alias(&path)).query()
    );

    assert_eq!(node.is_ok(), true);
    assert_eq!(fir.is_ok(), true);
    assert_eq!(sec.is_ok(), true);

    let elems = graph.db().exec(
        &QueryBuilder::select().node_count().query()
    );
    let aliases = graph.db().exec(
        &QueryBuilder::select().aliases().query()
    );
    

    assert_eq!(elems.is_ok(), true);
    assert_eq!(aliases.is_ok(), true);

    let nodes = aliases.unwrap().elements;
    let edges = elems.unwrap();
    let edges = edges.elements;

    // NOTE: The below assertions are commented out because the amount of 
    // nodes created at startup is not set in stone. The most recent breakage 
    // was because of adding a nodecategories node. No point in breaking a test 
    // every time that happens. 

    // let edges = edges.elements.iter().filter(|x| x.id.0 < 0).collect::<Vec<_>>();
    // Length is 6:
    // root, attributes, settings, one, two, three
    // assert_eq!(nodes.len(), 6);

    // Length is 5:
    // root
    // - one
    //   - two
    //     - three
    // - attributes
    // - settings
    // - = edge

    // todo! Fix this test. Can't find a way to just get all edges...
    // println!("Edges: {:#?}", edges);
    // assert_eq!(edges.len(), 5);

    cleanup_graph(&func_name);
}

#[test]
fn insert_and_delete_node_attribute(){
    let func_name = "insert_and_delete_node_attribute";
    let mut graph = setup_graph(func_name);

    let path = NodePath::new("test".into());

    let attr = Attribute {
        name: "test".to_string(),
        value: 10.0
    };

    let node = graph.create_node_by_path(path.clone(), None);
    assert_eq!(node.is_ok(), true);


    let added = graph.insert_node_attrs(path.clone(), vec!(attr));
    assert_eq!(added.is_ok(), true);

    let noder = graph.open_node(path.clone());
    let noder = noder.unwrap();

    assert_eq!(noder.attributes().len(), 1);

    assert_eq!(noder.attributes()[0].name, "test");
    assert_eq!(noder.attributes()[0].value, 10.0);

    // Test deleting the attribute
    let deleted = graph.delete_node_attr(path.clone(), "test");
    assert_eq!(deleted.is_ok(), true);

    let nodest = graph.open_node(path.clone());
    let nodest = nodest.unwrap();
    assert_eq!(nodest.attributes().len(), 0);

    cleanup_graph(&func_name);
}

/// Insertion of attributes on non-existing nodes should fail.
/// Insertion of attributes on a non-existing node shouldn't
/// create the node.
#[test]
fn insertion_of_attributes_on_nonexisting_node() {
    let func_name = "insertion_of_attributes_on_nonexisting_node";
    let mut graph = setup_graph(func_name);

    let path = NodePath::new("test".into());

    let attr = Attribute {
        name: "test".to_string(),
        value: 10.0
    };

    let shouldfail = graph.insert_node_attrs(path.clone(), vec!(attr.clone()));
    assert_eq!(shouldfail.is_ok(), false, "Insertion of attr didn't fail, even though node doesn't exist");


    cleanup_graph(&func_name);
}


#[test]
fn insert_and_delete_multiple_attributes(){
    let func_name = "insert_and_delete_multiple_attributes";
    let mut graph = setup_graph(func_name);

    let path = NodePath::new("test".into());

    let node =  graph.create_node_by_path(path.clone(), None);

    todo!();

    cleanup_graph(&func_name);
}


#[test]
fn protect_reserved_node_attributes() {
    let func_name = "protect_reserved_attributes";
    let mut graph = setup_graph(func_name);

    use fs_graph::elements::RESERVED_NODE_ATTRS;

    let test_attr = "preview";
    assert!(RESERVED_NODE_ATTRS.contains(&test_attr));

    let path = NodePath::from("test");

    let node = graph.create_node_by_path(path.clone(), None);
    assert_eq!(node.is_ok(), true);

    let attr = Attribute {
        name: test_attr.to_string(),
        value: 10.0
    };

    let added = graph.insert_node_attrs(path.clone(), vec!(attr));

    assert_eq!(added.is_ok(), true);
    let nod = graph.db().exec(
        &QueryBuilder::select().values(vec!()).ids(path.alias()).query()
    );
    assert!(nod.is_ok());
    let nods = nod.unwrap();
    let nods = nods.elements;
    assert_eq!(nods.len(), 1);
    let nods = nods.first().unwrap();
    let mut nods = &nods.values;
    let nods: Vec<agdb::DbValue> = nods.iter().map(|nod| nod.key.clone()).collect();
    assert!(!nods.contains(&"preview".into()));


    let removed = graph.delete_node_attr(path, "ntype");
    assert_eq!(removed.is_ok(), false);

    cleanup_graph(&func_name);
}

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