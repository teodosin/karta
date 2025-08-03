use std::error::Error;

use uuid::Uuid;

use super::{edge::Edge};


pub trait GraphEdge {
    fn get_edge_strict(
        &self,
        from: &Uuid,
        to: &Uuid
    ) -> Result<Edge, Box<dyn Error>>;

    fn insert_edges(&mut self, edges: Vec<Edge>);

    fn get_edges_between_nodes(&self, nodes: &[Uuid]) -> Result<Vec<Edge>, Box<dyn Error>>;

    fn delete_edges(&mut self, edges: &[(Uuid, Uuid)]) -> Result<(), Box<dyn Error>>;

    fn reconnect_edge(
        &mut self,
        old_from: &Uuid,
        old_to: &Uuid,
        new_from: &Uuid,
        new_to: &Uuid,
    ) -> Result<Edge, Box<dyn Error>>;

    /// Reparent a node by moving its contains edge from old parent to new parent
    /// This is specifically for moving nodes in the hierarchy and bypasses the
    /// normal restrictions on contains edge manipulation
    fn reparent_node(
        &mut self,
        node_uuid: &Uuid,
        old_parent_uuid: &Uuid,
        new_parent_uuid: &Uuid,
    ) -> Result<(), Box<dyn Error>>;
}

#[cfg(test)]
mod tests {
    use crate::{
        elements::{node::DataNode, node_path::NodePath, edge::Edge, nodetype::NodeTypeId},
        graph_agdb::GraphAgdb,
        graph_traits::{graph_edge::GraphEdge, graph_node::GraphNodes},
        utils::utils::KartaServiceTestContext,
    };

    #[test]
    fn test_delete_edge() {
        let mut ctx = KartaServiceTestContext::new("test_delete_edge");
        let node1 = DataNode::new(&NodePath::from("node1"), NodeTypeId::new("core/text"));
        let node2 = DataNode::new(&NodePath::from("node2"), NodeTypeId::new("core/text"));
        let edge = Edge::new(node1.uuid(), node2.uuid());

        ctx.with_service_mut(|s| {
            s.data_mut().insert_nodes(vec![node1.clone(), node2.clone()]);
            s.data_mut().insert_edges(vec![edge.clone()]);
        });

        let edge_result = ctx.with_service(|s| s.data().get_edge_strict(&node1.uuid(), &node2.uuid()));
        assert!(edge_result.is_ok());

        let result = ctx.with_service_mut(|s| s.data_mut().delete_edges(&[(node1.uuid(), node2.uuid())]));
        assert!(result.is_ok());

        let edge_result = ctx.with_service(|s| s.data().get_edge_strict(&node1.uuid(), &node2.uuid()));
        assert!(edge_result.is_err());
    }

    #[test]
    fn test_reconnect_edge() {
        let mut ctx = KartaServiceTestContext::new("test_reconnect_edge");
        let node1 = DataNode::new(&NodePath::from("node1"), NodeTypeId::new("core/text"));
        let node2 = DataNode::new(&NodePath::from("node2"), NodeTypeId::new("core/text"));
        let node3 = DataNode::new(&NodePath::from("node3"), NodeTypeId::new("core/text"));
        let edge = Edge::new(node1.uuid(), node2.uuid());

        ctx.with_service_mut(|s| {
            s.data_mut().insert_nodes(vec![node1.clone(), node2.clone(), node3.clone()]);
            s.data_mut().insert_edges(vec![edge.clone()]);
        });

        let initial_edge = 
            ctx.with_service(|s| s.data().get_edge_strict(&node1.uuid(), &node2.uuid()));
        assert!(initial_edge.is_ok());

        let result = ctx.with_service_mut(|s| {
            s.data_mut()
                .reconnect_edge(&node1.uuid(), &node2.uuid(), &node1.uuid(), &node3.uuid())
        });

        assert!(result.is_ok());

        let old_edge =
            ctx.with_service(|s| s.data().get_edge_strict(&node1.uuid(), &node2.uuid()));
        assert!(old_edge.is_err());

        let new_edge =
            ctx.with_service(|s| s.data().get_edge_strict(&node1.uuid(), &node3.uuid()));
        assert!(new_edge.is_ok());
    }

    #[test]
    fn test_reconnect_edge_to_root() {
        // Test reconnecting the "to" end to the root
        let mut ctx_to = KartaServiceTestContext::new("test_reconnect_edge_to_root_to");
        let root_node_to = ctx_to.with_service(|s| s.data().open_node(&crate::elements::node_path::NodeHandle::Path(NodePath::root()))).unwrap();
        let node1_to = DataNode::new(&NodePath::from("node1"), NodeTypeId::new("core/text"));
        let node2_to = DataNode::new(&NodePath::from("node2"), NodeTypeId::new("core/text"));
        let edge_to = Edge::new(node1_to.uuid(), node2_to.uuid());

        ctx_to.with_service_mut(|s| {
            s.data_mut().insert_nodes(vec![node1_to.clone(), node2_to.clone()]);
            s.data_mut().insert_edges(vec![edge_to.clone()]);
        });

        let result_to = ctx_to.with_service_mut(|s| {
            s.data_mut()
                .reconnect_edge(&node1_to.uuid(), &node2_to.uuid(), &node1_to.uuid(), &root_node_to.uuid())
        });

        assert!(result_to.is_ok());
        assert!(ctx_to.with_service(|s| s.data().get_edge_strict(&node1_to.uuid(), &node2_to.uuid())).is_err());
        assert!(ctx_to.with_service(|s| s.data().get_edge_strict(&node1_to.uuid(), &root_node_to.uuid())).is_ok());

        // Test reconnecting the "from" end to the root
        let mut ctx_from = KartaServiceTestContext::new("test_reconnect_edge_to_root_from");
        let root_node_from = ctx_from.with_service(|s| s.data().open_node(&crate::elements::node_path::NodeHandle::Path(NodePath::root()))).unwrap();
        let node1_from = DataNode::new(&NodePath::from("node1"), NodeTypeId::new("core/text"));
        let node2_from = DataNode::new(&NodePath::from("node2"), NodeTypeId::new("core/text"));
        let edge_from = Edge::new(node1_from.uuid(), node2_from.uuid());

        ctx_from.with_service_mut(|s| {
            s.data_mut().insert_nodes(vec![node1_from.clone(), node2_from.clone()]);
            s.data_mut().insert_edges(vec![edge_from.clone()]);
        });

        let result_from = ctx_from.with_service_mut(|s| {
            s.data_mut()
                .reconnect_edge(&node1_from.uuid(), &node2_from.uuid(), &root_node_from.uuid(), &node2_from.uuid())
        });

        assert!(result_from.is_ok());
        assert!(ctx_from.with_service(|s| s.data().get_edge_strict(&node1_from.uuid(), &node2_from.uuid())).is_err());
        assert!(ctx_from.with_service(|s| s.data().get_edge_strict(&root_node_from.uuid(), &node2_from.uuid())).is_ok());
    }
}
