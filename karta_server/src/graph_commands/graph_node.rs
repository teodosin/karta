use std::{error::Error, path::PathBuf};

use node::CreateNodeByPathCommand;

use crate::prelude::*;


impl GraphNode for GraphCommands {
    fn open_node(&mut self, path: &NodePath) -> Result<Node, Box<dyn std::error::Error>> {
        self.graph.open_node(path)
    }

    fn open_node_connections(&self, path: &NodePath) -> Vec<(Node, Edge)> {
        self.graph.open_node_connections(path)
    }

    fn create_node_by_path(
        &mut self,
        path: &NodePath,
        ntype: Option<NodeType>,
    ) -> Result<Node, Box<dyn std::error::Error>> {
        let cmd = CreateNodeByPathCommand::new(path.clone(), ntype);

        let result = match self.apply(Box::new(cmd)) {
            Ok(result) => result,
            Err(e) => {
                println!("Failed to insert node: {}", e);
                return Err(e.into());
            }
        };

        let nodes: Vec<Node> = result.into();
        let node = nodes.first().unwrap().clone();
        Ok(node)
    }

    fn create_node_by_name(
        &mut self,
        parent_path: Option<NodePath>,
        name: &str,
        ntype: Option<NodeType>,
    ) -> Result<Node, Box<dyn std::error::Error>> {
        todo!()
    }

    fn insert_node(&mut self, node: Node) -> Result<(), Box<dyn std::error::Error>> {
        todo!()
    }

    fn delete_nodes(&mut self, paths: &Vec<NodePath>, files: bool, dirs: bool) -> Result<(), Box<dyn Error>> {
        todo!()
    }

    fn get_node_attrs(&self, path: &NodePath) -> Result<Vec<Attribute>, Box<dyn std::error::Error>> {
        todo!()
    }

    fn insert_node_attrs(
        &mut self,
        path: &NodePath,
        attrs: Vec<Attribute>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        todo!()
    }

    fn delete_node_attrs(
        &mut self,
        path: &NodePath,
        attr_name: Vec<&str>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        todo!()
    }

    fn merge_nodes(&mut self, nodes: Vec<NodePath>) -> Result<(), Box<dyn Error>> {
        todo!()
    }

    fn autoparent_nodes(
        &mut self,
        parent: &NodePath,
        child: &NodePath,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.graph.autoparent_nodes(parent, child)
    }
}

mod tests {
    use crate::graph_commands::TestCommandContext;

    use super::*;

    #[test]
    fn create_node_command_returns_node_and_increases_undo_stack() {
        let mut func_name = "create_node_command_returns_node_and_increases_undo_stack";
        let mut ctx = TestCommandContext::new(&func_name);

        let undo_queue_before = ctx.graph.command_manager.get_undo_stack().len();

        let node = ctx.graph.create_node_by_path(
            &NodePath::from("test"),
            None,
        );

        let undo_queue_after = ctx.graph.command_manager.get_undo_stack().len();

        assert!(node.is_ok());
        assert_eq!(node.unwrap().path(), NodePath::from("test"));
        assert_eq!(undo_queue_after, undo_queue_before + 1, "Undo stack should have increased by 1");
    }

    #[test]
    fn create_node_command_can_be_reverted() {
        let mut func_name = "create_node_command_can_be_reverted";
        let mut ctx = TestCommandContext::new(&func_name);

        let npath = NodePath::from("test");
        let node = ctx.graph.create_node_by_path(
            &npath,
            None,
        );

        assert_eq!(node.is_ok(), true, "Node should be created");

        ctx.graph.undo();

        let node = ctx.graph.open_node(&npath);

        assert_eq!(node.is_err(), true, "Node should not be found");
    }

    #[test]
    fn reverting_create_node_deletes_created_ancestors() {
        let mut func_name = "reverting_create_node_deletes_created_ancestors";
        let mut ctx = TestCommandContext::new(&func_name);

        let npath = NodePath::from("test/test2/test3");

        let node = ctx.graph.create_node_by_path(
            &npath,
            None,
        );

        assert_eq!(node.is_ok(), true, "Node should be created");
        let parent = ctx.graph.open_node(&npath.parent().unwrap());
        assert_eq!(parent.is_ok(), true, "Parent should be created");
        let grandparent = ctx.graph.open_node(&npath.parent().unwrap().parent().unwrap());
        assert_eq!(grandparent.is_ok(), true, "Grandparent should be created");

        ctx.graph.undo();

        let node = ctx.graph.open_node(&npath);
        assert_eq!(node.is_err(), true, "Node should not be found");
        let parent = ctx.graph.open_node(&npath.parent().unwrap());
        assert_eq!(parent.is_err(), true, "Parent should not be found");
        let grandparent = ctx.graph.open_node(&npath.parent().unwrap().parent().unwrap());
        assert_eq!(grandparent.is_err(), true, "Grandparent should not be found");
    }

    #[test]
    fn create_node_command_can_be_reapplied() {
        let mut func_name = "create_node_command_can_be_reapplied";
        let mut ctx = TestCommandContext::new(&func_name);

        let npath = NodePath::from("test");
        let node = ctx.graph.create_node_by_path(
            &npath,
            None,
        );

        assert_eq!(node.is_ok(), true, "Node should be created");

        ctx.graph.undo();

        let found = ctx.graph.open_node(&npath);
        assert_eq!(found.is_err(), true, "Node should not be found");

        ctx.graph.redo();

        let found = ctx.graph.open_node(&npath);
        assert_eq!(found.is_ok(), true, "Node should be found");
    }
}
