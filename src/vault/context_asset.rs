//

use std::{path::PathBuf, fs::DirEntry, io::Write};

use bevy::{prelude::Vec2, ecs::{system::{Resource, Res, Query}, entity::Entity, query::With}, asset::{Handle, Asset, AssetLoader, io::Reader, LoadContext, AsyncReadExt}, reflect::TypePath, utils::{BoxedFuture, HashMap}, transform::components::Transform};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{graph::{attribute::{Attribute, Attributes}, node_types::NodeTypes, edges::{EdgeTypes, EdgeType, GraphEdge}, nodes::{GraphDataNode, GraphNodeEdges, PinnedToPosition, PinnedToPresence, Visitor, ContextRoot}, context::CurrentContext}, events::context};

use super::CurrentVault;

const CONTEXT_FILE_EXTENSION: &str = "context";

#[derive(Resource, Default)]
pub struct ContextAssetState {
    pub _handle: Handle<ContextAsset>,
}

#[derive(Asset, Debug, Serialize, Deserialize, TypePath, Default)]
pub struct ContextAsset {
    pub nself: NodeSerial,

    #[serde(default = "Vec::new")]
    pub edges: Vec<EdgeSerial>,

    #[serde(default = "Vec::new")]
    pub nodes: Vec<NodeSerial>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct EdgeSerial {
    pub source: String,
    pub target: String,

    #[serde(default)]
    pub directed: bool,

    #[serde(default)]
    pub etype: EdgeTypes,

    #[serde(default)]
    pub attributes: Option<Vec<Attribute>>,
}
 
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct NodeSerial {
    pub path: String,

    #[serde(default)]
    pub ntype: NodeTypes,

    #[serde(default)]
    pub relative_position: Option<Vec2>,

    #[serde(default)]
    pub relative_size: Option<Vec2>,

    #[serde(default)]
    pub attributes: Option<Vec<Attribute>>,

    #[serde(default)]
    pub pin_to_position: bool,

    #[serde(default)]
    pub pin_to_presence: bool,
}

pub fn save_context(
    vault: Res<CurrentVault>,
    context: Res<CurrentContext>,
    root_node: Query<(
        Entity, 
        Option<&Transform>,
        ),
        With<ContextRoot>,
    >,
    all_nodes: Query<(
        Entity, 
        Option<&Visitor>,
        &GraphDataNode, 
        &GraphNodeEdges,
        Option<&Transform>,
        Option<&PinnedToPosition>,
        Option<&PinnedToPresence>,
        Option<&Attributes>,
        ),
        // Without<ContextRoot>,
    >,
    all_edges: Query<(
        Entity, 
        &GraphEdge,
        &EdgeType,
        Option<&Attributes>,
    )>,
){
    // Evaluate the correct location for the save file
    let vault = match &vault.vault {
        Some(vault) => vault,
        None => {
            println!("No vault set");
            return
        }
    };

    let vault_root_path = &vault.root;
    let vault_dir_path = vault_root_path.join(&vault.vault_folder_name);

    #[cfg(debug_assertions)]
    println!("Vault root path: {:?}", vault_root_path);
    #[cfg(debug_assertions)]
    println!("Vault dir path: {:?}", vault_dir_path);

    let root_node_position: Vec2 = match root_node.iter().next() {
        Some((_, Some(transform))) => {
            Vec2::new(transform.translation.x, transform.translation.y)
        },
        _ => {
            println!("No root node found");
            return
        }
    };

    let node_entity_to_path_index: HashMap<Entity, PathBuf> = all_nodes.iter().map(|(entity, _, node, _, _, _, _, _)| {
        (entity, node.path.clone())
    }).collect();


    for node in all_nodes.iter() {
        let (entity, visitor, node, nedges, transform, pin_to_position, pin_to_presence, attributes) = node;

        if visitor.is_some() {
            continue
        }

        // Ignore the file .kartaVault
        if node.path.file_name().unwrap() == vault.vault_folder_name {
            continue
        }

        let node_path = &node.path;
        let context_path = node_path_to_context_path(vault_root_path, &vault_dir_path, node_path);

        // Calculate position relative to root node
        let relative_position = match transform {
            Some(transform) => {
                let node_position = Vec2::new(transform.translation.x, transform.translation.y);
                let relative_position = node_position - root_node_position;
                Some(relative_position)
            },
            None => None,
        };

        let node_serial = NodeSerial {
            path: node.path.to_str().unwrap().to_string(),
            ntype: node.ntype,
            relative_position,
            relative_size: None,
            attributes: None,
            pin_to_position: pin_to_position.is_some(),
            pin_to_presence: pin_to_presence.is_some(),
        };

        let mut edges_serial: Vec<EdgeSerial> = Vec::new();


        for edge in nedges.edges.iter() {
            let entity = all_edges.get(*edge);
            if entity.is_err() {
                continue
            }
            let (_entity, edge, etype, _attributes) = entity.unwrap();

            let edge_serial = EdgeSerial {
                source: node_entity_to_path_index.get(&edge.source).unwrap().to_str().unwrap().to_string(),
                target: node_entity_to_path_index.get(&edge.target).unwrap().to_str().unwrap().to_string(),
                directed: true,
                etype: etype.etype.clone(),
                attributes: None,
            };

            edges_serial.push(edge_serial);
        }

        let asset = ContextAsset {
            nself: node_serial,
            edges: edges_serial,
            nodes: Vec::new(),
        };

        // Create the directory if it doesn't exist
        if let Some(parent) = context_path.parent() {
            if std::fs::create_dir_all(parent).is_err() {
                eprintln!("Failed to create directory: {:?}", parent);
                continue;
            }
        }

        let pretty_config = ron::ser::PrettyConfig::default();
        let data = ron::ser::to_string_pretty(&asset, pretty_config).unwrap();
        
        let mut file = std::fs::File::create(&context_path).expect("Could not create context file");
        file.write_all(data.as_bytes()).expect("Could not write to context file");
        
        println!("Saved context: {:?}", context_path);

    }
}

fn node_path_to_context_path(
    vault_root_path: &PathBuf,
    vault_dir_path: &PathBuf,
    node_path: &PathBuf,
) -> PathBuf {
    let mut context_path = vault_dir_path.join(&node_path.strip_prefix(vault_root_path).unwrap());

        // TODO: Add support for non-unicode characters
    if let Some(stem) = context_path.file_name().and_then(|s| s.to_str()) {
        let new_name = format!("{}.{}", stem, CONTEXT_FILE_EXTENSION);
        context_path.set_file_name(new_name);
    }

    println!("Saving context to: {:?}", context_path);
    context_path


}

pub fn load_context(

){

}


#[derive(Default)]
pub struct ContextAssetLoader;

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum ContextAssetLoaderError {
    /// An [IO](std::io) Error
    #[error("Could load shader: {0}")]
    Io(#[from] std::io::Error),
    /// A [RON](ron) Error
    #[error("Could not parse RON: {0}")]
    RonSpannedError(#[from] ron::error::SpannedError),
}

impl AssetLoader for ContextAssetLoader {
    type Asset = ContextAsset;
    type Settings = ();
    type Error = ContextAssetLoaderError;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a Self::Settings,
        _load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            let asset: ContextAsset = ron::de::from_bytes::<ContextAsset>(&bytes)?;
            Ok(asset)
        })
    }

    fn extensions(&self) -> &[&str] {
        &["context"]
    }

}

pub fn _dir_or_file_is_hidden(
    entry: &DirEntry,
) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| !s.starts_with("."))
        .unwrap_or(false)
}

