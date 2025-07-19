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
}
