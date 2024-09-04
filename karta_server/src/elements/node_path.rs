use std::path::PathBuf;

use agdb::{DbError, DbValue};

/// Newtype wrapper for the node path. 
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct NodePath(PathBuf);

impl NodePath {
    pub fn root() -> Self {
        NodePath(PathBuf::from(""))
    }
    /// Create a new NodePath from a pathbuf. 
    /// Supplying an empty pathbuf will create a NodePath to the root. 
    pub fn new(path: PathBuf) -> Self {
        NodePath(path)
    }

    pub fn name(&self) -> &str {
        self.0.file_name().unwrap().to_str().unwrap()
    }

    /// Get the path as a pathbuf, excluding the root. 
    pub fn buf(&self) -> &PathBuf {
        &self.0
    }

    /// Get the path as a string, excluding the root path but prefixed with root/.
    pub fn alias(&self) -> String {
        let str: String = self.0.to_str().unwrap().into();

        let mut alias;
        
        // Add root/ prefix to path if not empty. If empty, just root
        if str.len() > 0 {
            alias = format!("root/{}", str);
        } else {
            alias = String::from("root");
        }
    
        alias
    }

    // Turn alias (root/path) into NodePath
    pub fn from_alias(alias: &str) -> Self {
        let buf = PathBuf::from(alias);

        // Remove root/ prefix from path
        let newbuf =  match buf.strip_prefix("root/") {
            Ok(buf) => PathBuf::from(buf),
            Err(_) => buf,
        };
    
        NodePath(newbuf)
    }

    /// Get the full path, including the root. Root path must be provided. 
    pub fn full(&self, root_path: &PathBuf) -> PathBuf {
        let full_path = root_path.clone();
        full_path.join(self.0.clone())
    }

    pub fn parent(&self) -> Option<NodePath> {
        let parent = self.0.parent();
        match parent {
            Some(p) => Some(NodePath(p.to_path_buf())),
            None => None,
        }
    }
}

impl From<String> for NodePath {
    fn from(path: String) -> Self {
        NodePath::new(PathBuf::from(path))
    }
}

impl From<&str> for NodePath {
    fn from(path: &str) -> Self {
        NodePath::new(PathBuf::from(path))
    }
}

impl TryFrom<DbValue> for NodePath {
    type Error = DbError;

    fn try_from(value: DbValue) -> Result<Self, Self::Error> {
        Ok(NodePath::from_alias(&value.to_string()))
    }
}

impl From<NodePath> for DbValue {
    fn from(path: NodePath) -> Self {
        path.alias().into()
    }
}