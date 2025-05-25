use std::{error::Error, io::Write, path::PathBuf};

use uuid::Uuid;

use crate::{elements::view_node::ViewNode, prelude::DataNode, SERVER_VERSION};

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
    pub fn generate_context(&self, focal: Uuid, dataNodes: Vec<DataNode>) -> Context {
        let mut viewNodes: Vec<ViewNode> = Vec::new();

        let grid_columns: usize = 5;
        let node_width: f32 = 200.0;
        let node_height: f32 = 200.0;
        let gap: f32 = 24.0;

        let focalNode = dataNodes
            .iter().find(|node| node.uuid() == focal);
        
        if focalNode.is_none() {
            panic!("Focal node not found in dataNodes");
        }

        // Generate the focalnode viewnode. It has a relative position of 0,0.
        // In the frontend, this node will be ignored if it was already present
        // among the loaded nodes.
        let focalNode = focalNode.unwrap();
        let focalViewNode = ViewNode::from_data_node(focalNode.clone())
            .sized(node_width, node_height);

        viewNodes.push(focalViewNode);

        // Iterate over the datanodes.
        dataNodes.iter().enumerate()
            .filter(|(_, node)| node.uuid() != focal)
            .for_each(|(i, node)| {
                // Calculate the position of the node.
                let column = i % grid_columns;
                let row = i / grid_columns;

                let relX = (column as f32) * (node_width + gap);
                let relY = (row as f32) * (node_height + gap);

                let viewNode = ViewNode::from_data_node(node.clone())
                    .sized(node_width, node_height)
                    .positioned(relX, relY);

                viewNodes.push(viewNode);
            });

        // Find existing file.
        let existingFile = self.get_context_file(focal);
        match existingFile {
            Ok(context) => {
                // If context file exists, add to it the new nodes that dont exist in it.
                // The context file is the source of truth.
                let mut context: Context = context;

                viewNodes.iter().for_each(|node| {
                    if !context.viewnodes().iter().any(|n| n.uuid() == node.uuid()) {
                        context.add_node(node.clone());
                    }
                });
                
                context
            },
            Err(_) => Context::with_viewnodes(focal, viewNodes),
        }
    }

    fn get_context_file(&self, uuid: Uuid) -> Result<Context, Box<dyn Error>> {
        let mut file_name: String = uuid.to_string();
        file_name.push_str(".ctx");
        let full_path = self.get_contexts_dir().join(file_name);

        let context_file = std::fs::File::open(full_path)?;
        let context: Context = ron::de::from_reader(context_file)?;

        Ok(context)
    }

    fn save_context(&self, context: &Context) -> Result<(), Box<dyn Error>> {
        let mut file_name: String = context.focal().to_string();
        file_name.push_str(".ctx");
        let full_path = self.get_contexts_dir().join(file_name);

        let mut context_file = std::fs::File::create(full_path)?;
        let pretty_config = ron::ser::to_string_pretty(&context, Default::default()).unwrap();
        context_file.write_all(pretty_config.as_bytes()).unwrap();
        Ok(())
    }
}