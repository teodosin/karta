use std::{error::Error, path::PathBuf, sync::Arc};

use tokio::sync::RwLock;

use crate::{context::{context::Context, context_db::ContextDb}, elements::node_path::NodeHandle, prelude::*};


pub struct KartaService {
    data: GraphAgdb,
    view: ContextDb,
}

impl KartaService {
    pub fn new(
        name: &str,
        root_path: PathBuf,
        storage_dir: PathBuf,
    ) -> Self {

        // Check if the storage dir is called .karta.
        // If not, create it.
        // This might be a bit crude, but it will do for now.
        let mut storage_dir = storage_dir;
        if storage_dir.file_name().unwrap() != ".karta" {
            storage_dir = storage_dir.join(".karta");
            std::fs::create_dir_all(&storage_dir).unwrap();
        }

        let data = GraphAgdb::new(
            name,
            root_path.clone(),
            storage_dir.clone(),
        );
        let view = ContextDb::new(
                name.to_owned(),
                root_path.clone(),
                storage_dir.clone(),
        );

        Self { data, view }
    }

    
    pub fn data(&self) -> &GraphAgdb {
        &self.data
    }

    
    pub fn view(&self) -> &ContextDb {
        &self.view
    }

    
    pub fn data_mut(&mut self) -> &mut GraphAgdb {
        &mut self.data
    }

    
    pub fn view_mut(&mut self) -> &mut ContextDb {
        &mut self.view
    }

    /// Opens a context's Data and View.
    /// This is the main function for opening a context.
    /// Reconciles the indexed data with the physical data.
    pub fn open_context_from_path(&self, path: NodePath) 
        -> Result<(Vec<DataNode>, Vec<Edge>, Context), Box<dyn Error>> {

        let mut finaldatanodes: Vec<DataNode> = Vec::new();
        let mut finaledges: Vec<Edge> = Vec::new();
        
        let focal_handle: NodeHandle = NodeHandle::Path(path.clone());
        let focal_node = self.data().open_node(&focal_handle)?;
        let focal_uuid = focal_node.uuid().unwrap();

        finaldatanodes.push(focal_node);

        let datanodes = self.data().open_node_connections(&path);
        for (node, edge) in datanodes {
            finaldatanodes.push(node);
            finaledges.push(edge);
        }

        let context = self.view.generate_context(
            focal_uuid,
            finaldatanodes.clone(),
        );

        return Ok((finaldatanodes, finaledges, context));
    }
}