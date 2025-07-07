use std::{error::Error, io::Write, path::PathBuf};

use uuid::Uuid;

use crate::{elements::{nodetype::NodeTypeId, view_node::ViewNode}, prelude::DataNode, SERVER_VERSION};
use std::cmp::Ordering;

use super::context::Context;


pub struct ContextDb {
    name: String,
    root_path: PathBuf,
    storage_path: PathBuf,
}

impl ContextDb {
    pub fn new(name: String, root_path: PathBuf, storage_path: PathBuf) -> Self {
        Self {
            name,
            root_path,
            storage_path,
        }
    }

    fn get_contexts_dir(&self) -> PathBuf {
        self.storage_path.join("contexts")
    }

    /// Generates the in-memory context for a given group of datanodes.
    /// The server does lazy indexing, so it always tries to generate default
    /// contexts and datanodes based on file system data. Only when the user
    /// edits data that can't be stored in the file system itself, does the server
    /// index the datanode or edge or viewnode.
    ///
    /// For example, if we open the context of a directory we've never opened before,
    /// we will get a default context with the viewnodes arranged in a grid. Unless we
    /// edit any of that data, we don't need to index those nodes in the database nor
    /// create a context file.
    pub fn generate_context(&self, focal_uuid: Uuid, parent_uuid: Option<Uuid>, data_nodes: Vec<DataNode>) -> Context {
        println!("[generate_context] Generating context for focal UUID: {}", focal_uuid);
        println!("[generate_context] -> Received {} data nodes.", data_nodes.len());
        for node in &data_nodes {
            println!("[generate_context] ->   - Node: '{}', UUID: {}", node.path().alias(), node.uuid());
        }

        let mut view_nodes: Vec<ViewNode> = Vec::new();
        const GRID_COLUMNS: usize = 5;
        const NODE_WIDTH: f32 = 100.0;
        const NODE_HEIGHT: f32 = 100.0;
        const GAPX: f32 = 20.0;
        const GAPY: f32 = GAPX * 3.2;

        // --- Find Focal, Parent, and Children Nodes ---
        let focal_node = data_nodes.iter().find(|node| node.uuid() == focal_uuid).expect("Focal node not found in dataNodes");
        let parent_node = parent_uuid.and_then(|p_uuid| data_nodes.iter().find(|node| node.uuid() == p_uuid));
        let mut children_nodes: Vec<&DataNode> = data_nodes.iter().filter(|node| {
            let is_focal = node.uuid() == focal_uuid;
            let is_parent = parent_node.map_or(false, |p| p.uuid() == node.uuid());
            !is_focal && !is_parent
        }).collect();

        // --- Sorting for Deterministic Layout ---
        children_nodes.sort_by(|a, b| {
            let a_is_dir = a.ntype_name() == NodeTypeId::dir_type();
            let b_is_dir = b.ntype_name() == NodeTypeId::dir_type();
            let a_is_file = a.ntype_name() == NodeTypeId::file_type();
            let b_is_file = b.ntype_name() == NodeTypeId::file_type();

            match (a_is_dir, b_is_dir, a_is_file, b_is_file) {
                // Both are directories or both are files, or both are neither
                (true, true, _, _) | (false, false, true, true) | (false, false, false, false) => {
                    a.path().buf().cmp(b.path().buf())
                }
                // A is a directory, B is not
                (true, false, _, _) => Ordering::Less,
                // B is a directory, A is not
                (false, true, _, _) => Ordering::Greater,
                // A is a file, B is not a directory (must be other)
                (false, false, true, false) => Ordering::Less,
                // B is a file, A is not a directory (must be other)
                (false, false, false, true) => Ordering::Greater,
            }
        });

        // --- Layout Logic ---
        // Calculate the actual width of the children grid
        let num_children = children_nodes.len();
        let actual_cols = if num_children < GRID_COLUMNS { num_children } else { GRID_COLUMNS };
        let children_grid_width = (actual_cols as f32 * (NODE_WIDTH + GAPX)) - GAPX;
        let grid_offset_x = -(children_grid_width / 2.0) + (NODE_WIDTH / 2.0);

        // 1. Position Parent Node (if it exists)
        if let Some(p_node) = parent_node {
            let parent_x = 0.0; // Directly above focal
            let parent_y = -(NODE_HEIGHT + GAPY); // Above focal
            let parent_view_node = ViewNode::from_data_node(p_node.clone())
                .sized(NODE_WIDTH, NODE_HEIGHT)
                .positioned(parent_x, parent_y);
            view_nodes.push(parent_view_node);
        }

        // 2. Position Focal Node
        let focal_view_node = ViewNode::from_data_node(focal_node.clone())
            .sized(NODE_WIDTH, NODE_HEIGHT)
            .positioned(0.0, 0.0); // Focal is always at the origin
        view_nodes.push(focal_view_node);

        // 3. Position Children Nodes in a Grid
        children_nodes.iter().enumerate().for_each(|(i, child_node)| {
            let col = i % GRID_COLUMNS;
            let row = i / GRID_COLUMNS;
            let child_x = grid_offset_x + ((col as f32) * (NODE_WIDTH + GAPX));
            let child_y = NODE_HEIGHT + GAPY + ((row as f32) * (NODE_HEIGHT + GAPX)); // Below focal
            let child_view_node = ViewNode::from_data_node((*child_node).clone())
                .sized(NODE_WIDTH, NODE_HEIGHT)
                .positioned(child_x, child_y);
            view_nodes.push(child_view_node);
        });

        // Find existing file.
        let existingFile = self.get_context_file(focal_uuid);
        match existingFile {
            Ok(context) => {
                // If context file exists, add to it the new nodes that dont exist in it.
                // The context file is the source of truth.
                let mut context: Context = context;

                view_nodes.iter().for_each(|node| {
                    if !context.viewnodes().iter().any(|n| n.uuid() == node.uuid()) {
                        context.add_node(node.clone());
                    }
                });
                
                context
            },
            Err(_) => Context::with_viewnodes(focal_uuid, view_nodes),
        }
    }

    pub fn get_context_file(&self, uuid: Uuid) -> Result<Context, Box<dyn Error>> {
        let mut file_name: String = uuid.to_string();
        file_name.push_str(".ctx");
        let full_path = self.get_contexts_dir().join(file_name);

        let context_file = std::fs::File::open(full_path)?;
        let context: Context = ron::de::from_reader(context_file)?;

        Ok(context)
    }

    pub fn save_context(&self, context: &Context) -> Result<(), Box<dyn Error>> {
        let contexts_dir = self.get_contexts_dir();
        std::fs::create_dir_all(&contexts_dir)?; // Ensure the contexts directory exists

        let mut file_name: String = context.focal().to_string();
        file_name.push_str(".ctx");
        let full_path = contexts_dir.join(file_name);

        // If the context we are asked to save has no nodes, it means all nodes
        // are in their default positions. In this case, we should remove the
        // context file if it exists to fall back to the default generated layout.
        if context.viewnodes().is_empty() {
            if full_path.exists() {
                std::fs::remove_file(full_path)?;
            }
            return Ok(());
        }

        // Otherwise, serialize the provided context (with its modified nodes)
        // and overwrite the file.
        let mut context_file = std::fs::File::create(full_path)?;
        let pretty_config = ron::ser::to_string_pretty(&context, Default::default())?;
        context_file.write_all(pretty_config.as_bytes())?;
        Ok(())
    }
}