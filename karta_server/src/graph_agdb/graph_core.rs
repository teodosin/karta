use std::{
    error::Error,
    path::{self, PathBuf},
};

use agdb::QueryBuilder;

use crate::{
    elements::{node::ROOT_UUID, nodetype::ARCHETYPES},
    graph_traits::{self, graph_core::GraphCore, graph_node::GraphNodes},
    prelude::{DataNode, GraphEdge, NodePath, NodeTypeId, StoragePath},
};

use super::GraphAgdb;

/// Implementation block for the Graph struct itself.
/// Includes constructors and utility functions.
impl GraphCore for GraphAgdb {
    fn storage_path(&self) -> PathBuf {
        self.storage_path.clone()
    }

    fn vault_dirpath(&self) -> PathBuf {
        let path = self.vault_fs_path.clone();
        // println!("root_path: {:?}", path);
        path
    }

    fn root_nodepath(&self) -> NodePath {
        NodePath::root()
    }

    /// Gets the name of the root directory without the full path
    fn root_name(&self) -> String {
        self.vault_fs_path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string()
    }

    /// Constructor. Panics if the db cannot be created.
    ///
    /// Takes the desired root directory of the graph as a parameter and the name for the db.
    /// The name of the root directory will become the vault of the graph,
    /// as first child of the root node.
    ///
    /// Creates the db at the storage_path, or initialises the db if it already exists there.
    ///
    /// TODO: Add error handling.
    fn new(name: &str, vault_fs_path: PathBuf, storage_dir: PathBuf) -> Self {
        if !storage_dir.exists() {
            std::fs::create_dir_all(&storage_dir).expect("Failed to create storage path");
        }
        let db_path = storage_dir.join(format!("{}.agdb", name));

        // Check if the database already exists
        let open_existing = db_path.exists();

        let db = agdb::Db::new(db_path.to_str().unwrap()).expect("Failed to create new db");

        let mut giraphe = GraphAgdb {
            name: name.to_string(),
            db,
            vault_fs_path: vault_fs_path.into(),
            storage_path: storage_dir,
        };

        if !open_existing {
            let archetypes = ARCHETYPES;

            archetypes.iter().for_each(|atype| {
                let atype_path = NodePath::atype(*atype);
                let ntype = if atype_path == NodePath::root() {
                    NodeTypeId::root_type()
                } else if atype_path == NodePath::vault() {
                    NodeTypeId::dir_type()
                } else {
                    NodeTypeId::archetype_type()
                };

                let node = DataNode::new(&atype_path, ntype);
                let node_uuid = node.uuid();

                let query_result = giraphe.db.exec_mut(
                    &QueryBuilder::insert()
                        .nodes()
                        .aliases(node_uuid.to_string())
                        .values(node)
                        .query(),
                );

                if let Err(err) = query_result {
                    panic!("Failed to create archetype node: {}", err);
                }

                if atype_path != NodePath::root() {
                    let root_to_atype_edge =
                        crate::prelude::Edge::new_cont(ROOT_UUID, node_uuid);

                    giraphe.insert_edges(vec![root_to_atype_edge]);
                }
            });
        }

        // Indexes for faster lookup based on attributes
        giraphe.db.exec_mut(QueryBuilder::insert().index("uuid").query());
        giraphe.db.exec_mut(QueryBuilder::insert().index("ntype").query());
        giraphe.db.exec_mut(QueryBuilder::insert().index("path").query());

        return giraphe;
    }

    fn get_all_aliases(&self) -> Vec<String> {
        let all = self.db().exec(&QueryBuilder::select().aliases().query());
        match all {
            Ok(aliases) => {
                let all: Vec<String> = aliases
                    .elements
                    .iter()
                    .map(|alias| alias.values[0].value.to_string())
                    .collect();

                all
            }
            Err(err) => {
                // println!("Error: {}", err);
                vec![]
            }
        }
    }
}
