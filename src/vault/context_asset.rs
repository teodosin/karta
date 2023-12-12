//

use std::{path::PathBuf, fs::DirEntry, io::Write};

use bevy::{prelude::Vec2, ecs::{system::{Res, Query}, entity::Entity, query::{With, Without}}, reflect::TypePath, utils::HashMap, transform::components::Transform};
use serde::{Deserialize, Serialize};

use crate::{graph::{attribute::Attributes, node_types::NodeTypes, edges::{EdgeTypes, EdgeType, GraphEdge}, nodes::{GraphDataNode, GraphNodeEdges, PinnedToPosition, PinnedToPresence, Visitor, ContextRoot}, context::CurrentContext}, events::{context, edges}};

use super::CurrentVault;

const CONTEXT_FILE_EXTENSION: &str = "context";

const VERSION: &str = env!("CARGO_PKG_VERSION");

// Serialized Structs
// --------------------------------------------------------------------------
#[derive(Debug, Serialize, Deserialize, TypePath, Default)]
pub struct ContextAsset {
    pub karta_version: String,

    pub nself: RootNodeSerial,

    #[serde(default = "Vec::new")]
    pub nodes: Vec<NodeSerial>,

    #[serde(default = "Vec::new")]
    pub edges: Vec<RootEdgeSerial>,
}

// The root node. The only node that stores complete data. 
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct RootNodeSerial {
    pub path: String,

    #[serde(default)]
    pub ntype: NodeTypes,

    #[serde(default)]
    pub attributes: Option<Attributes>,

    #[serde(default)]
    pub pin_to_position: bool,

    #[serde(default)]
    pub pin_to_presence: bool,
}

// The nodes present in the context (non-visitors.)
// Their data is stored in their respective context files. 
// This struct only stores their relative transforms to the root node and their pins. 
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NodeSerial {
    pub path: String,

    // A context file only stores one node, and its connections to other nodes. 
    // These are the relative transforms of the other node for a connection. 
    #[serde(default)]
    pub relative_position: Option<Vec2>,
    
    #[serde(default)]
    pub relative_size: Option<Vec2>,

    // Pins that are only relevant in the current root's context
    #[serde(default)]
    pub pin_to_position: bool,

    #[serde(default)]
    pub pin_to_presence: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RootEdgeSerial {
    pub source: String,
    pub target: String,

    #[serde(default)]
    pub directed: bool,

    #[serde(default)]
    pub etype: EdgeTypes,
    
    #[serde(default)]
    pub attributes: Option<Attributes>,
}

// Functions
// --------------------------------------------------------------------------
pub fn open_context_from_node_path(
    vault_root_path: &PathBuf,
    vault_dir_path: &PathBuf,
    node_path: &PathBuf,
) -> Result<ContextAsset, String> {
    let context_path = node_path_to_context_path(vault_root_path, vault_dir_path, node_path);

    if context_path.exists() {

        println!("Context file exists: {:?}", context_path);

        // Load the file
        let context_file = std::fs::read_to_string(context_path).expect("Could not read context file");
        // Deserialize the file
        let context_asset = match ron::de::from_str(&context_file) {
            Ok(context_asset) => {
                println!("Vault assets: {:?}", context_asset);
                Ok(context_asset)
            },
            Err(e) => {
                println!("Error: {:?}", e);
                Err(e.to_string())
            }
        };
        context_asset
    } else {
        println!("Context file does not exist: {:?}", context_path);
        Err("Context file does not exist".to_string())
    }
}

pub fn save_context(
    vault: Res<CurrentVault>,
    root_node: Query<(
        Entity, 
        &GraphDataNode, 
        &GraphNodeEdges,
        Option<&Transform>, // TODO: Remove the option?
        Option<&PinnedToPosition>,
        Option<&PinnedToPresence>,
        Option<&Attributes>,
        ),
        With<ContextRoot>,
    >,
    all_nodes: Query<(
        Entity, 
        Option<&Visitor>,
        &GraphDataNode, 
        &GraphNodeEdges,
        Option<&Transform>, // TODO: Remove the option?
        Option<&PinnedToPosition>,
        Option<&PinnedToPresence>,
        Option<&Attributes>,
        ),
        Without<ContextRoot>,
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

    // What needs to be saved:
    // - The root node
    // - All nodes that are not visitors
    // - All edges between the present nodes
    // - The relative position of the nodes to the root node

    // The other context files may have connections not present in this context. We do not want to
    // overwrite those. We only want to modify the edges where the source and target are present in
    // this context.

    // Root node setup
    let root_node = match root_node.iter().next() {
        Some(root_node) => root_node,
        None => {
            println!("No root node");
            return
        }
    };

    let (rn_entity, rn_data, rn_edges, rn_transform, rn_pin_pos, rn_pin_pres, rn_attr) = root_node;

    // Data structure to store the index of the node entity to the path of the node.
    // Needed for the edge serialization
    let mut node_entity_to_path_index: HashMap<Entity, PathBuf> = all_nodes.iter().map(|(entity, _, node, _, _, _, _, _)| {
        (entity, node.path.clone())
    }).collect();
    node_entity_to_path_index.insert(rn_entity, rn_data.path.clone());



    // Save the root node context. 
    // All the connected nodes are guaranteed to be present in the context. 
    // Relative positions of the other nodes are saved in their corresponding edges.
    let root_serial = RootNodeSerial {
        path: rn_data.path.to_str().unwrap().to_string(),
        ntype: rn_data.ntype,
        attributes: rn_attr.cloned(),
        pin_to_position: rn_pin_pos.is_some(),
        pin_to_presence: rn_pin_pres.is_some(),
    };

    println!("Edges on root node: {:?}", rn_edges.edges.len());

    let nodes_serial: Vec<NodeSerial> = rn_edges.edges.iter().map(|edge| {
        let edge_entity = all_edges.get(*edge);
        if edge_entity.is_err() {
            return None
        }
        let (_entity, edge, _etype, _attributes) = edge_entity.unwrap();

        let other_node_entity = if edge.source == rn_entity {
            edge.target
        } else {
            edge.source
        };

        let other_node = match all_nodes.get(other_node_entity) {
            Ok(other_node) => other_node,
            Err(_) => return None,
        };

        let (_, _, on_node, _, on_transform, on_pin_pos, on_pin_pres, _) = other_node;

        let relative_position = match on_transform {
            Some(transform) => {
                let node_position = Vec2::new(transform.translation.x, transform.translation.y);
                let relative_position = node_position - rn_transform.unwrap().translation.truncate();
                Some(relative_position)
            },
            None => None,
        };

        let relative_size = match on_transform {
            Some(transform) => {
                let node_size = Vec2::new(transform.scale.x, transform.scale.y);
                let rel_size = node_size / rn_transform.unwrap().scale.truncate();
                Some(rel_size)
            },
            None => None,
        };

        let node_serial = NodeSerial {
            path: on_node.path.to_str().unwrap().to_string(),
            relative_position,
            relative_size,
            pin_to_position: on_pin_pos.is_some(),
            pin_to_presence: on_pin_pres.is_some(),
        };

        Some(node_serial)

    }).filter(|node_serial| node_serial.is_some()).map(|node_serial| node_serial.unwrap()).collect();

    println!("Saving {} nodes", nodes_serial.len());

    let edges_serial: Vec<RootEdgeSerial> = rn_edges.edges.iter().map(|edge| {
        let edge_entity = all_edges.get(*edge);
        if edge_entity.is_err() {
            return None
        }
        let (_entity, edge, etype, attributes) = edge_entity.unwrap();

        let edge_serial = RootEdgeSerial {
            source: node_entity_to_path_index.get(&edge.source).unwrap().to_str().unwrap().to_string(),
            target: node_entity_to_path_index.get(&edge.target).unwrap().to_str().unwrap().to_string(),
            directed: true,
            etype: etype.etype.clone(),
            attributes: attributes.cloned(),
        };

        Some(edge_serial)

    }).filter(|edge_serial| edge_serial.is_some()).map(|edge_serial| edge_serial.unwrap()).collect();

    let root_asset = ContextAsset {
        karta_version: VERSION.to_string(),
        nself: root_serial,
        nodes: nodes_serial,
        edges: edges_serial,
    };

    save_context_file(
        vault_root_path,
        &vault_dir_path,
        &rn_data.path,
        &root_asset,
    );



    // Modify the other context files
    // Ignore visitors 
    for node in all_nodes.iter() {
        let (_, visitor, node, edges, _, pin_pos, pin_pres, attr) = node;

        if visitor.is_some() {
            continue
        }

        // Load the context file if it exists
        // Destructure it into its components
        let (_, existing_nodes, existing_edges): (RootNodeSerial, Vec<NodeSerial>, Vec<RootEdgeSerial>) = match open_context_from_node_path(
            vault_root_path,
            &vault_dir_path,
            &node.path,
        ) {
            Ok(context_asset) => {
                let existing_root = context_asset.nself;
                let existing_nodes = context_asset.nodes;
                let existing_edges = context_asset.edges;
                (existing_root, existing_nodes, existing_edges)
            },
            Err(_) => (RootNodeSerial::default(), Vec::new(), Vec::new()),
        };


        // Overwrite the root node
        let root_serial = RootNodeSerial {
            path: node.path.to_str().unwrap().to_string(),
            ntype: node.ntype,
            attributes: attr.cloned(),
            pin_to_position: pin_pos.is_some(),
            pin_to_presence: pin_pres.is_some(),
        };

        // Leave nodes as they were originally.
        let nodes_serial = existing_nodes;

        // We must selectively overwrite only the edges that are present in this context.

        let all_node_paths: Vec<String> = all_nodes.iter().map(|(_, _, node, _, _, _, _, _)| {
            node.path.to_str().unwrap().to_string()
        }).collect();

        println!("All node paths: {:?}", all_node_paths);


        let mut edges_serial: Vec<RootEdgeSerial> = existing_edges.iter().filter_map(|edge_serial| {
            let (source, target) = (&edge_serial.source, &edge_serial.target);

            if all_node_paths.contains(source) && all_node_paths.contains(target) {
                None
            } else {
                println!("Keeping this one: {:?}", edge_serial);
                Some(edge_serial.clone())
            }
        }).collect();

        println!("Edges serial existing file: {:?}", edges_serial);

        for edge in edges.edges.iter() {
            let edge_entity = all_edges.get(*edge);
            if edge_entity.is_err() {
                continue
            }
            let (_entity, edge, etype, attributes) = edge_entity.unwrap();

            let edge_serial = RootEdgeSerial {
                source: node_entity_to_path_index.get(&edge.source).unwrap().to_str().unwrap().to_string(),
                target: node_entity_to_path_index.get(&edge.target).unwrap().to_str().unwrap().to_string(),
                directed: true,
                etype: etype.etype.clone(),
                attributes: attributes.cloned(),
            };

            if !edges_serial.iter().any(|existing_edge| existing_edge.source == edge_serial.source && existing_edge.target == edge_serial.target) {
                edges_serial.push(edge_serial);
            }        
        }

        let asset = ContextAsset {
            karta_version: VERSION.to_string(),
            nself: root_serial,
            nodes: nodes_serial,
            edges: edges_serial,
        };

        save_context_file(
            vault_root_path,
            &vault_dir_path,
            &node.path,
            &asset,
        );
    }

}

fn save_context_file(
    vault_root_path: &PathBuf,
    vault_dir_path: &PathBuf,
    node_path: &PathBuf,
    context_asset: &ContextAsset,
) {
    // Check that we are not accidentally saving the .kartaVault.context file.
    if node_path.file_name().unwrap().to_str().unwrap() == ".kartaVault.context" {
        println!("Not saving context file for .kartaVault.context");
        return
    }

    let context_path = node_path_to_context_path(vault_root_path, vault_dir_path, node_path);

    // Create the directory if it doesn't exist
    if let Some(parent) = context_path.parent() {
        if std::fs::create_dir_all(parent).is_err() {
            eprintln!("Failed to create directory: {:?}", parent);
        }
    }

    let pretty_config = ron::ser::PrettyConfig::default();
    let data = ron::ser::to_string_pretty(&context_asset, pretty_config).unwrap();
    
    let mut file = std::fs::File::create(&context_path).expect("Could not create context file");
    file.write_all(data.as_bytes()).expect("Could not write to context file");
    
    println!("Saved context: {:?}", context_path);
}

fn node_path_to_context_path(
    vault_root_path: &PathBuf,
    vault_dir_path: &PathBuf,
    node_path: &PathBuf,
) -> PathBuf {
    let vault_dir_name: PathBuf = vault_root_path.file_name().unwrap().into();
    

    let mut context_path = vault_dir_path.join(vault_dir_name).join(&node_path.strip_prefix(vault_root_path).unwrap());

        // TODO: Add support for non-unicode characters
    if let Some(stem) = context_path.file_name().and_then(|s| s.to_str()) {
        let new_name = format!("{}.{}", stem, CONTEXT_FILE_EXTENSION);
        context_path.set_file_name(new_name);
    }

    println!("Saving context to: {:?}", context_path);
    context_path
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

