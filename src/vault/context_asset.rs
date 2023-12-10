//

use std::{path::PathBuf, fs::DirEntry};

use bevy::{prelude::Vec2, ecs::{system::{Resource, Commands, Res, ResMut, Query}, entity::Entity}, asset::{AssetServer, Assets, Handle, Asset, AssetLoader, io::Reader, LoadContext, AsyncReadExt}, reflect::TypePath, utils::BoxedFuture, transform::components::Transform};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::graph::{attribute::{Attribute, Attributes}, node_types::NodeTypes, edges::{EdgeTypes, EdgeType, GraphEdge}, nodes::{GraphDataNode, GraphNodeEdges, PinnedToPosition, PinnedToPresence, Visitor}, context::CurrentContext};

use super::CurrentVault;

const CONTEXT_FILE_EXTENSION: &str = "context";

#[derive(Resource, Default)]
pub struct ContextAssetState {
    pub _handle: Handle<ContextAsset>,
}

#[derive(Asset, Debug, Deserialize, TypePath, Default)]
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
    nodes: Query<(
        Entity, 
        Option<&Visitor>,
        &GraphDataNode, 
        &GraphNodeEdges,
        Option<&Transform>,
        Option<&PinnedToPosition>,
        Option<&PinnedToPresence>,
        Option<&Attributes>,
    )>,
    edges: Query<(
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
    let context = match &context.cxt {
        Some(context) => context,
        None => {
            println!("No context set");
            return
        }
    };

    let vault_root_path = &vault.root;
    let vault_dir_path = vault_root_path.join(&vault.vault_folder_name);
    let mut current_context_path = vault_dir_path.join(&context.current_context.strip_prefix(vault_root_path).unwrap());

    // TODO: Add support for non-unicode characters
    if let Some(stem) = current_context_path.file_name().and_then(|s| s.to_str()) {
        let new_name = format!("{}.{}", stem, CONTEXT_FILE_EXTENSION);
        current_context_path.set_file_name(new_name);
    }

    println!("Vault root path: {:?}", vault_root_path);
    println!("Vault dir path: {:?}", vault_dir_path);
    println!("Saving context to: {:?}", current_context_path);



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

