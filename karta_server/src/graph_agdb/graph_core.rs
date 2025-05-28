use std::{
    error::Error,
    path::{self, PathBuf},
};

use agdb::QueryBuilder;

use crate::{
    elements::nodetype::ARCHETYPES,
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

            // println!("Length of archetypes {}", archetypes.len());

            archetypes.iter().for_each(|at| {
                // println!("{}", at);
            });

            archetypes.iter().for_each(|atype| {
                let atype_path = NodePath::atype(*atype);
                // println!("Atypepath {:?}", atype_path);

                // println!("Creating archetype node: {}", atype_path.alias());

                let ntype = if atype_path == NodePath::root() {
                    // println!("Root node in question");
                    NodeTypeId::root_type()
                } else if atype_path == NodePath::vault() { // Check for vault specifically
                    // println!("Vault node in question");
                    NodeTypeId::dir_type() // Assign core/dir to vault
                } else {
                    // println!("Other archetype node in question");
                    NodeTypeId::archetype_type() // Other archetypes get core/archetype
                };

                let node: DataNode = DataNode::new(&atype_path, ntype);

                // println!("alias is {}", atype_path.alias());

                let query = giraphe.db.exec_mut(
                    &QueryBuilder::insert()
                        .nodes()
                        .aliases(atype_path.alias())
                        .values(node)
                        .query(),
                );

                match query {
                    Ok(_) => {
                        // println!("Created archetype node: {}", atype_path.alias());
                    }
                    Err(ref err) => {
                        panic!("Failed to create archetype node: {}", err);
                        // println!("Failed to create archetype node: {}", err);
                    }
                }

                if atype_path != NodePath::root() {
                    let root_to_atype_edge =
                        crate::prelude::Edge::new_cont(&NodePath::root(), &atype_path);

                    giraphe.insert_edges(vec![root_to_atype_edge]);
                } else {
                    // println!("Root node, no autoparenting");
                }
            });
        }

        // Indexes for faster lookup based on attributes
        giraphe.db.exec_mut(QueryBuilder::insert().index("uuid").query());
        giraphe.db.exec_mut(QueryBuilder::insert().index("ntype").query());

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
