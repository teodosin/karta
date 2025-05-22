use std::{path::PathBuf, sync::Arc};

use tokio::sync::RwLock;

use crate::{context::context_db::ContextDb, prelude::*};


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


}