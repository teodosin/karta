use std::{error::Error, io::Write, path::PathBuf};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{elements::view_node::ViewNode, SERVER_VERSION};


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

    fn get_context(&self, uuid: Uuid) -> Result<Context, Box<dyn Error>> {
        let mut file_name: String = uuid.to_string();
        file_name.push_str(".ctx");
        let full_path = self.get_contexts_dir().join(file_name);

        let context_file = std::fs::File::open(full_path)?;
        let context: Context = ron::de::from_reader(context_file)?;

        Ok(context)
    }

    fn save_context(&self, context: &Context) -> Result<(), Box<dyn Error>> {
        let mut file_name: String = context.focal.uuid().to_string();
        file_name.push_str(".ctx");
        let full_path = self.get_contexts_dir().join(file_name);

        let mut context_file = std::fs::File::create(full_path)?;
        let pretty_config = ron::ser::to_string_pretty(&context, Default::default()).unwrap();
        context_file.write_all(pretty_config.as_bytes()).unwrap();
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
struct Context {
    karta_version: String,
    focal: ViewNode,
    nodes: Vec<ViewNode>,
}

impl Context {
    fn new(focal: ViewNode) -> Self {
        Self {
            karta_version: SERVER_VERSION.to_string(),
            focal,
            nodes: Vec::new(),
        }
    }

    fn add_node(&mut self, node: ViewNode) {
        self.nodes.push(node);
    }
}

mod tests {
    use super::*;

    fn test_ctx_db(test_name: &str) -> ContextDb {
        let strg_name = "karta_server";

        let root = directories::ProjectDirs::from("com", "karta_server", strg_name)
            .unwrap()
            .data_dir()
            .to_path_buf();

        let full_path = root.join(&test_name);
        let strg_dir = full_path.join(".karta");

        let context_db = ContextDb::new("test".to_string(), PathBuf::from("."), PathBuf::from("."));
        context_db
    }

    #[test]
    fn test_context_db() {
        let func_name = "test_context_db";
        let context_db = test_ctx_db(func_name);
    }
}