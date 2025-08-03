use agdb::{DbError, DbValue};

use crate::elements::node;

// Some of the structs and enums in this file are currently not used.
// Determining a sound architecture for node types is difficult and
// not urgent quite yet.

pub const KARTA_VERSION: &str = "0.1.0";


// pub const ARCHETYPES: [&str; 5] = ["", "vault", "attributes", "nodetypes", "settings"];
pub const ARCHETYPES: [&str; 2] = ["", "vault"];

#[derive(Debug, Clone, Hash, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct NodeTypeId {
    type_path: String,
    version: String,
}

impl NodeTypeId {
    pub fn to_string(&self) -> String {
        format!("{}@{}", self.type_path, self.version)
    }

    pub fn new(type_path: &str) -> Self {
        Self {
            type_path: type_path.to_string(),
            version: KARTA_VERSION.to_string(),
        }
    }

    pub fn root_type() -> Self {
        Self {
            type_path: "core/root".to_string(),
            version: KARTA_VERSION.to_string(),
        }
    }

    pub fn archetype_type() -> Self {
        Self {
            type_path: "core/archetype".to_string(),
            version: KARTA_VERSION.to_string(),
        }
    }

    pub fn dir_type() -> Self {
        Self {
            type_path: "core/fs/dir".to_string(),
            version: KARTA_VERSION.to_string(),
        }
    }

    /// Generic file type. 
    pub fn file_type() -> Self {
        Self {
            type_path: "core/fs/file".to_string(),
            version: KARTA_VERSION.to_string(),
        }
    }

    pub fn image_type() -> Self {
    	Self {
    		type_path: "core/image".to_string(),
    		version: KARTA_VERSION.to_string(),
    	}
    }
   
    pub fn virtual_generic() -> Self {
        Self {
            type_path: "core/virtual_generic".to_string(),
            version: KARTA_VERSION.to_string(),
        }
    }

    /// Check if this is the root node specifically
    pub fn is_root_node(&self) -> bool {
        self.type_path == "core/root"
    }

}

impl TryFrom<DbValue> for NodeTypeId {
    type Error = DbError;

    fn try_from(value: DbValue) -> Result<Self, Self::Error> {
        let type_str = value.string()?;
        let parts: Vec<&str> = type_str.split('@').collect();
        if parts.len() != 2 {
            return Err(DbError::from("Invalid NodeTypeId format"));
        }
        
        Ok(NodeTypeId {
            type_path: parts[0].to_string(),
            version: parts[1].to_string(),
        })
    }
}

impl From<NodeTypeId> for DbValue {
    fn from(type_id: NodeTypeId) -> Self {
        type_id.to_string().into()
    }
}