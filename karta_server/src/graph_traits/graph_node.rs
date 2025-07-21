use std::{error::Error, path::PathBuf};
use uuid::Uuid;

use crate::elements::node_path::NodeHandle;

use super::{attribute::{Attribute, RelativePosition}, edge::Edge, node::DataNode, node_path::NodePath, nodetype::NodeTypeId};

pub trait GraphNodes {
    // -------------------------------------------------------------------
    // Nodes

    /// Retrieves a particular node's data from the database.
    /// The path is relative to the root of the graph.
    fn open_node(&self, handle: &NodeHandle) -> Result<DataNode, Box<dyn Error>>;

    /// Retrieves a set of nodes by their UUIDs.
    fn open_nodes_by_uuid(&self, uuids: Vec<Uuid>) -> Result<Vec<DataNode>, Box<dyn Error>>;

    // Retrieves the edges of a particular node.
    // fn get_node_edges(&self, path: &NodePath) -> Vec<Edge>;

    /// Opens the connections of a particular node.
    /// Takes in the path to the node relative to the root of the graph.
    /// Doesn't include the node for the input path.
    ///
    /// TODO: Add filter argument when Filter is implemented.
    /// Note that possibly Filter could have a condition that nodes
    /// would have to be connected to some node, which would just turn
    /// this into a generic "open_nodes" function, or "search_nodes".
    /// Then filters could just be wrappers around agdb's QueryConditions...
    fn open_node_connections(&self, path: &NodePath) -> Vec<(DataNode, Edge)>;

    /// Inserts a Node.
    fn insert_nodes(&mut self, nodes: Vec<DataNode>);

    /// Updates the attributes of a node.
    fn update_node_attributes(&mut self, uuid: Uuid, attributes: Vec<Attribute>) -> Result<(), Box<dyn Error>>;

    /// Retrieves all indexed paths from the database.
    fn get_all_indexed_paths(&self) -> Result<Vec<String>, Box<dyn Error>>;
    /// Retrieves all descendant nodes of a given path by traversing 'contains' edges.
    fn get_all_descendants(&self, path: &NodePath) -> Result<Vec<DataNode>, Box<dyn Error>>;
}

// --------------------------------------------------------------------

#[cfg(test)]
mod tests {
    #![allow(warnings)]

    use crate::{
        elements::{
            self, attribute::{Attribute, RESERVED_NODE_ATTRS}, node, node_path::{NodeHandle, NodePath}, nodetype::ARCHETYPES
        }, graph_agdb::GraphAgdb, graph_traits::graph_edge::GraphEdge, prelude::{DataNode, NodeTypeId}, utils::utils::KartaServiceTestContext
    };
    use agdb::QueryBuilder;

    use std::{
        fs::{create_dir, File},
        path::PathBuf,
        vec,
    };

    use crate::graph_traits::{graph_core::GraphCore, graph_node::GraphNodes};

    #[test]
    fn opening_root_node_connections__returns_vector_of_nodes() {
        let func_name = "opening_root_node_connections__returns_vector_of_nodes";
        let ctx = KartaServiceTestContext::new(func_name); // No mut needed if only reading

        let root_path = NodePath::root();

        let connections = ctx.with_graph_db(|db| db.open_node_connections(&root_path));

        // Archetypes includes the root, but the source node is excluded from the function so
        // so the length should be one less than the number of archetypes
        let archetypes: Vec<&&str> = ARCHETYPES.iter().filter(|ar| **ar != "").collect();

        assert_eq!(
            connections.len(),
            archetypes.len(),
            "Root path should have its archetype connections: {}",
            archetypes.len()
        );

        // Check that the tuples contain matching edges
        for (i, connection) in connections.iter().enumerate() {
            assert!(
                *connection.1.source() == connection.0.uuid()
                    || *connection.1.target() == connection.0.uuid(),
                "Edge should be connected to Node"
            )
        }
    }

    #[test]
    fn inserting_node__node_is_inserted() {
        let func_name = "inserting_node__node_is_inserted";
        let mut ctx = KartaServiceTestContext::new(func_name);

        let path = NodePath::new(PathBuf::from("test"));

        let node = DataNode::new(&path, NodeTypeId::virtual_generic());

        ctx.with_graph_db_mut(|db_mut| db_mut.insert_nodes(vec![node]));

        ctx.with_graph_db(|db| db.open_node(&NodeHandle::Path(path))).expect("Node should exist");
    }

    #[test]
    fn inserting_modified_same_node__updates_node() {
        let func_name = "inserting_modified_same_node__updates_node";
        let mut ctx = KartaServiceTestContext::new(func_name);

        let path = NodePath::new(PathBuf::from("test"));

        let node = DataNode::new(&path, NodeTypeId::virtual_generic());

        ctx.with_graph_db_mut(|db_mut| db_mut.insert_nodes(vec![node.clone()]));

        let root_path = NodePath::vault();
        let root_connections = ctx.with_graph_db(|db| db.open_node_connections(&root_path));

        let mut mod_node = node.clone();
        mod_node.set_name("testerer");
        
        ctx.with_graph_db_mut(|db_mut| db_mut.insert_nodes(vec![mod_node]));
        let second_connections = ctx.with_graph_db(|db| db.open_node_connections(&root_path));

        assert_eq!(root_connections.len(), second_connections.len());

        // Check the name too
        let mod_node = ctx.with_graph_db(|db| db.open_node(&NodeHandle::Path(path))).unwrap();
        assert_eq!(mod_node.name(), "testerer");
    }

    #[test]
    fn inserting_long_path__creates_intermediate_nodes() {
        let func_name = "inserting_long_path__creates_intermediate_nodes";
        let mut ctx = KartaServiceTestContext::new(func_name);

        let first = NodePath::new(PathBuf::from("one"));
        let second = NodePath::new(PathBuf::from("one/two"));
        let third = NodePath::new(PathBuf::from("one/two/three"));

        let third_node = DataNode::new(&third, NodeTypeId::virtual_generic());

        ctx.with_graph_db_mut(|db_mut| db_mut.insert_nodes(vec![third_node]));

        let second_node = ctx.with_graph_db(|db| db.open_node(&NodeHandle::Path(second)));
        assert!(second_node.is_ok());

        let first_node = ctx.with_graph_db(|db| db.open_node(&NodeHandle::Path(first)));
        assert!(first_node.is_ok());

        println!("{:#?}", ctx.with_graph_db(|db| db.get_all_aliases()));
    }

    #[test]
    fn uuid__can_be_used_to_find_node() {
        let func_name = "uuid__can_be_used_to_find_node";
        let mut ctx = KartaServiceTestContext::new(func_name);

        let path = NodePath::new(PathBuf::from("test"));

        let node = DataNode::new(&path, NodeTypeId::virtual_generic());
        
        ctx.with_graph_db_mut(|db_mut| db_mut.insert_nodes(vec![node]));
        
        let found_by_path = ctx.with_graph_db(|db| db.open_node(&NodeHandle::Path(path))).expect("Node should exist");
        let uuid = found_by_path.uuid();

        let found_by_uuid = ctx.with_graph_db(|db| db.open_node(&NodeHandle::Uuid(uuid))).expect("Node should exist");
        
        assert_eq!(found_by_path, found_by_uuid);
    }

    #[test]
    fn updating_node_attributes__updates_attributes() {
        let func_name = "updating_node_attributes__updates_attributes";
        let mut ctx = KartaServiceTestContext::new(func_name);

        let path = NodePath::new(PathBuf::from("test"));
        let node = DataNode::new(&path, NodeTypeId::virtual_generic());
        
        ctx.with_graph_db_mut(|db_mut| db_mut.insert_nodes(vec![node.clone()]));

        let initial_node = ctx.with_graph_db(|db| db.open_node(&NodeHandle::Uuid(node.uuid()))).unwrap();
        assert_eq!(initial_node.name(), "test");

        let new_attributes = vec![
            Attribute::new_string("name".to_string(), "new_name".to_string()),
            Attribute::new_string("new_attr".to_string(), "new_value".to_string()),
        ];

        ctx.with_graph_db_mut(|db_mut| {
            db_mut.update_node_attributes(node.uuid(), new_attributes).unwrap();
        });

        let updated_node = ctx.with_graph_db(|db| db.open_node(&NodeHandle::Uuid(node.uuid()))).unwrap();

        assert_eq!(updated_node.name(), "new_name");
        let attributes = updated_node.attributes();
        let new_attr = attributes.iter().find(|a| a.name == "new_attr").unwrap();
        assert_eq!(new_attr.value, crate::elements::attribute::AttrValue::String("new_value".to_string()));
    }
}
