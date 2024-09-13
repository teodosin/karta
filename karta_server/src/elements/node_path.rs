use std::path::PathBuf;

use agdb::{DbError, DbValue};

use super::nodetype::ARCHETYPES;

/// Newtype wrapper for the node path. Acts as the main struct for
/// creating and modifying node paths, turning them into db aliases/strings and
/// back. Path includes the name of the node itself.
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct NodePath(PathBuf);

impl NodePath {
    /// Get NodePath of the root node. Note that this is not
    /// the user_root, which must be accessed through the graph.
    pub fn root() -> Self {
        NodePath(PathBuf::from(""))
    }

    /// Get the user_root of the graph
    pub fn user_root() -> Self {
        NodePath(PathBuf::from("user_root"))
    }

    /// Create a new NodePath from a pathbuf relative to the user_root.
    /// Supplying an empty pathbuf will create a NodePath to the userroot.
    pub fn new(path: PathBuf) -> Self {
        if path.to_str().unwrap().is_empty() {
            return NodePath::user_root();
        }
        let root = NodePath::user_root().buf().clone();
        return NodePath(root.join(path));
    }

    pub fn join(&self, path: &str) -> Self {
        let mut path_buf = self.0.clone();
        path_buf.push(path);
        NodePath(path_buf)
    }

    fn raw(path: PathBuf) -> Self {
        NodePath(path)
    }

    pub fn name(&self) -> String {
        // Get the name of the node
        let name = self.0.clone();
        let name = name.file_name();

        match name {
            Some(name) => name.to_string_lossy().to_string(),
            None => "root".to_string(),
        }
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
            alias = format!("/{}", str);
        } else {
            alias = String::from("/");
        }

        alias
    }

    // Turn alias (root/path) into NodePath
    pub fn from_alias(alias: &str) -> Self {
        let buf = PathBuf::from(alias);

        // Remove root/ prefix from path
        let newbuf = match buf.strip_prefix("/") {
            Ok(buf) => PathBuf::from(buf),
            Err(_) => buf,
        };

        NodePath(newbuf)
    }

    pub fn atype(name: &str) -> Self {
        let atypes = ARCHETYPES;
        if atypes.contains(&name) {
            return NodePath::raw(name.into());
        } else {
            panic!("{} is not an archetype", name);
        }
    }

    /// Get the full path, including the root. Root path must be provided.
    pub fn full(&self, root_path: &PathBuf) -> PathBuf {
        let full_path = root_path.clone();
        let path_to_join = self.0.strip_prefix("user_root").unwrap_or(&self.0).to_path_buf();
        full_path.join(path_to_join)
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
