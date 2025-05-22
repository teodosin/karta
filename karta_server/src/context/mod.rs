use std::{error::Error, io::Write, path::PathBuf};

use context_db::ContextDb;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod context_db;
pub mod context;
pub mod context_settings;


pub fn test_ctx_db(test_name: &str) -> ContextDb {
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

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn test_context_db() {
        let func_name = "test_context_db";
        let context_db = test_ctx_db(func_name);
    }

    #[test]
    fn context_file_name_is_uuid_with_ctx_extension() {
        let func_name = "context_file_name_is_uuid_with_ctx_extension";
        let context_db = test_ctx_db(func_name);

        // What are we testing here?
        todo!();
    }
}