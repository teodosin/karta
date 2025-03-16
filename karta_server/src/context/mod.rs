use std::path::PathBuf;

use crate::elements::view_node::ViewNode;


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
}


struct Context {
    karta_version: String,
    focal: ViewNode,
    nodes: Vec<ViewNode>,
}