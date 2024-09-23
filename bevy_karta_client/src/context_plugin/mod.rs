//TODO: Would using Bevy's Hashmap be better for performance?
use std::{collections::HashMap, path::{Path, PathBuf}};

use bevy::prelude::*;
use events::{ChangeContextEvent, ContextEventsPlugin};
use fs_graph::prelude::*;

use crate::{node_plugin, prelude::{CurrentVault, DataEdge, DataEdgeBundle, DataNode, DataNodeBundle, Relation, Relations, ToBeDespawned, ViewNode}};

pub mod pe_index;
pub mod events;

use pe_index::*;

// -----------------------------------------------------------------
// Plugin
pub struct ContextPlugin;

impl Plugin for ContextPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(ContextEventsPlugin)
            .insert_resource(CurrentContext::empty())
            .insert_resource(PathsToEntitiesIndex::new())

            .add_systems(PreUpdate, (
                change_context,
                on_context_change.run_if(resource_changed::<CurrentContext>)
            ).chain())

            .add_systems(Update, (
                update_pe_index_on_datanode_spawn,
                update_pe_index_on_viewnode_spawn,
                update_relations_on_edge_spawn,
            ).chain())

            .add_systems(PostUpdate, cleanup_to_be_despawned)
        ;
    }
}

// -----------------------------------------------------------------
// Resources
#[derive(Resource)]
pub struct CurrentContext {
    undo_stack: Vec<KartaContext>,
    redo_stack: Vec<KartaContext>,
    setting_from_undo_redo: bool,

    context: Option<KartaContext>,
}

impl CurrentContext {
    pub fn empty() -> Self {
        CurrentContext {
            context: None,

            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            setting_from_undo_redo: false,
        }
    }

    pub fn set(&mut self, ctx_path: NodePath){
        self.context = Some(KartaContext {
            path: ctx_path,
        });
    }
}




pub struct KartaContext {
    path: NodePath, 
}

impl KartaContext {
    pub fn root() -> Self{
        KartaContext {
            path: NodePath::root(),
        }
    }

    pub fn user_root() -> Self {
        KartaContext {
            path: NodePath::user_root(),
        }
    }
}

// --------------------------------------------------------------------
// Systems

fn on_context_change(
    mut commands: Commands,
    context: Res<CurrentContext>,
    mut vault: ResMut<CurrentVault>,
    open_nodes: Query<Entity, With<DataNode>>,
    pe_index: Res<PathsToEntitiesIndex>,
){
    let graph = match &mut vault.graph {
        Some(graph) => graph,
        None => return, // TODO: Better error handling here
    };

    let ctx = match &context.context {
        Some(ctx) => ctx,
        None => return, // TODO: Better error handling here
    };
    
    let ctx_root_nodepath = ctx.path.clone();

    
    let node = graph.open_node(&ctx_root_nodepath);



    let node: fs_graph::prelude::Node = match node {
        Ok(node) => {
            println!("Node found: {:#?}", node);
            node
        },
        Err(e) => {
            println!("Node not found: {}", e);
            return;
        }
    };
    
    let nodepath = node.path().clone();
    let name = node.name().to_string();
    let ctx_rt_e: Option<Entity>;
    
    graph.index_node_context(&nodepath);

    if !pe_index.0.contains_key(&nodepath){
        ctx_rt_e = Some(commands.spawn(DataNodeBundle{
            name: Name::new(name),
            data_node: DataNode {
                path: nodepath,
                created_time: node.created_time().clone(),
                modified_time: node.modified_time().clone(),
            },
            data_node_type: node_plugin::DataNodeType(node.ntype_name()),
            attributes: node_plugin::Attributes(node.attributes().clone()),
        }).id());
    } else {
        ctx_rt_e = match pe_index.get_data(&nodepath) {
            Some(e) => Some(e),
            None => None,
        };
    }

    for node in open_nodes.iter(){
        match ctx_rt_e {
            Some(rt_e) => {
                if node == rt_e {
                    println!("Not deleting new context root: {}", rt_e);
                    continue;
                }
            },
            None => {}
        }

        commands.entity(node).insert(ToBeDespawned);
    }


    let connections: Vec<(fs_graph::prelude::Node, Edge)> = graph.open_node_connections(&ctx_root_nodepath);

    for node in connections.iter() {
        println!("Node: {:#?}", node);
        let (node, edge) = node;
        let node_path = node.path().clone();
        let name = node.name().to_string();

        if pe_index.0.contains_key(&node_path) {
            let rt_e = pe_index.get_data(&node_path);
            match rt_e {
                Some(rt_e) => {
                    match commands.get_entity(rt_e) {
                        Some(mut entity) => {
                            entity.remove::<ToBeDespawned>();
                        },
                        None => {}
                    }
                },
                None => {}
            }
            continue;
        }

        commands.spawn(DataNodeBundle{
            name: Name::new(name),
            data_node: DataNode {
                path: node_path,
                created_time: node.created_time().clone(),
                modified_time: node.modified_time().clone(),
            },
            data_node_type: node_plugin::DataNodeType(node.ntype_name()),
            attributes: node_plugin::Attributes(node.attributes().clone()),
        });

        commands.spawn(DataEdgeBundle {
            data_edge: DataEdge {
                source: edge.source().clone(),
                target: edge.target().clone(),
                created_time: edge.created_time().clone(),
                modified_time: edge.modified_time().clone(),
            },
            attributes: node_plugin::Attributes(edge.attributes().clone()),
        });
    }
}

fn change_context(
    mut event: EventReader<ChangeContextEvent>,
    mut context: ResMut<CurrentContext>,
){
    for event in event.read() {
        context.context = Some(KartaContext {
            path: event.new_ctx.clone(),
        });
    }
}

fn cleanup_to_be_despawned(
    mut commands: Commands,
    to_be_despawnd: Query<(Entity, &DataNode), With<ToBeDespawned>>,
    mut pe_index: ResMut<PathsToEntitiesIndex>,
){
    if to_be_despawnd.is_empty() {
        return;
    }
    println!("Cleaning up to be despawned: {}", to_be_despawnd.iter().count());
    for (e, data) in to_be_despawnd.iter() {
        pe_index.remove(&data.path);
        commands.entity(e).despawn_recursive();
    }
}