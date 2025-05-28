use std::path::PathBuf;

use agdb::{DbError, DbValue};
use uuid::Uuid;

use super::nodetype::ARCHETYPES;

pub enum NodeHandle {
    Path(NodePath),
    Uuid(Uuid)
}

/// Newtype wrapper for the node path. Acts as the main struct for
/// creating and modifying node paths, turning them into db aliases/strings and
/// back. Path includes the name of the node itself.
#[derive(Debug, Clone, Hash, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct NodePath(PathBuf);

impl NodePath {
    /// Get NodePath of the root node. Note that this is not
    /// the vault, which must be accessed through the graph.
    pub fn root() -> Self {
        NodePath(PathBuf::from(""))
    }

    /// Get the vault of the graph
    pub fn vault() -> Self {
        NodePath(PathBuf::from("vault"))
    }

    /// Create a new NodePath from a pathbuf relative to the vault.
    /// Supplying an empty pathbuf will create a NodePath to the userroot (vault).
    /// If the path is "vault", it also returns NodePath::vault().
    /// Otherwise, it prepends "vault/" to the given path.
    pub fn new(path: PathBuf) -> Self {
        if path.to_str().unwrap_or("").is_empty() { // Handles empty path
            return NodePath::vault();
        }
        if path == PathBuf::from("vault") { // Handles "vault" path
            return NodePath::vault();
        }
        // For other paths, prepend "vault/"
        let vault_prefix = NodePath::vault().0; // This is PathBuf::from("vault")
        return NodePath(vault_prefix.join(path));
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

    pub fn is_atype(&self) -> bool {
        let atypes = ARCHETYPES;
        for atype in atypes {
            if NodePath::atype(atype) == *self {
                return true;
            }
        } 
        false
    }

    /// Get the absolute file path, including the vault. Root path must be provided.
    /// Note that this function doesn't take into account whether the path exists or not.
    pub fn full(&self, root_path: &PathBuf) -> PathBuf {
        let full_path = root_path.clone();
        let path_to_join = self.0.strip_prefix("vault").unwrap_or(&self.0).to_path_buf();
        full_path.join(path_to_join)
    }

    /// Get a NodePath from an absolute path and the vault vault path.
    /// users/me/projects/my_vault/big/medium/small.txt -> NodePath("big/medium/small.txt")
    /// NodePath("big/medium/small.txt").alias() -> "/vault/big/medium/small.txt"
    pub fn from_dir_path(root_path: &PathBuf, file_path: &PathBuf) -> Self {
        let relative_path = file_path.strip_prefix(root_path).unwrap_or(file_path);
        NodePath::new(relative_path.to_path_buf())
    }

    pub fn parent(&self) -> Option<NodePath> {
        let parent = self.0.parent();
        match parent {
            Some(p) => Some(NodePath(p.to_path_buf())),
            None => None,
        }
    }

    /// Checks if the NodePath represents the virtual root node ("").
    pub fn is_root(&self) -> bool {
        self.0 == PathBuf::from("")
    }

    /// Checks if the NodePath represents the vault root node ("vault").
    pub fn is_vault_root(&self) -> bool {
        self.0 == PathBuf::from("vault")
    }

    /// If the path is under "vault/" (e.g., "vault/foo/bar"), returns "foo/bar".
    /// If the path is "vault", returns an empty string.
    /// Otherwise (e.g., for the virtual root ""), returns None.
    pub fn strip_vault_prefix(&self) -> Option<String> {
        let path_str = self.0.to_string_lossy();
        if path_str == "vault" {
            Some("".to_string())
        } else if path_str.starts_with("vault/") {
            Some(path_str["vault/".len()..].to_string())
        } else {
            None
        }
    }
}

impl From<String> for NodePath {
    fn from(path_str: String) -> Self {
        if path_str == "vault" {
            return NodePath::vault();
        }
        // NodePath::new will handle empty string correctly (returns NodePath::vault())
        // and prepend "vault/" to other non-empty, non-"vault" strings.
        NodePath::new(PathBuf::from(path_str))
    }
}

impl From<&str> for NodePath {
    fn from(path_str: &str) -> Self {
        if path_str == "vault" {
            return NodePath::vault();
        }
        NodePath::new(PathBuf::from(path_str))
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

#[cfg(test)]
mod tests {
    use super::*;

    
}