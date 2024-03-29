
use std::{collections::HashMap, path::PathBuf};

use bevy::prelude::*;
use bevy_mod_picking::picking_core::PickSet;
use serde::{Deserialize, Serialize};


use crate::{bevy_overlay_graph::{events::edges::*, input::pointer::*}, graph::attribute::Attributes};

use super::{nodes::GraphNodeEdges, context::PathsToEntitiesIndex};

pub struct EdgesPlugin;

impl Plugin for EdgesPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(PreUpdate, (
                handle_edge_click
            )
                .before(update_cursor_info)
                .after(PickSet::Last))
            .add_systems(Last, despawn_edges
                // TODO: Does this have to run every frame?
                //.run_if(resource_changed::<CurrentContext>())
            )
            .add_systems(PostUpdate, add_edge_to_node_indexes)
        ;
    }
}

// ----------------------------------------------------------------
// Component definitions

// A component for the most basic data of an EDGE
#[derive(Component, Reflect, Debug)]
pub struct GraphDataEdge {
    pub source: PathBuf,
    pub target: PathBuf,
}

impl GraphDataEdge {
    pub fn same_pair(&self, other: &GraphDataEdge) -> bool {
        if (self.source == other.source && self.target == other.target) || (self.source == other.target && self.target == other.source) {
            return true;
        }
        false
    }
}

#[derive(Component, Clone, Debug, PartialEq, Reflect)]
pub struct EdgeType {
    pub etype: EdgeTypes,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Reflect)]
pub enum EdgeTypes {
    Base,
    Parent,
}

impl Default for EdgeTypes {
    fn default() -> Self {
        EdgeTypes::Base
    }
}

fn handle_edge_click(
    mut event: EventReader<EdgeClickEvent>,

    mut input_data: ResMut<InputData>,

    edges: Query<(Entity, &GraphDataEdge)>,
){
    if event.is_empty(){
        return
    }

    // TODO: Handle multiple events
    match event.read().next().unwrap().target {
        None => {
            //println!("No event");
            input_data.latest_click_nodepath = None;
        }
        Some(target) => {
            //println!("Event: {:?}", target);
            
            match edges.get(target){
                Ok(edge) => {
                    let (entity, _data) = edge;
                    input_data.latest_edge_entity = Some(entity);
                },
                Err(_) => {
                    //println!("No node found");
                }
            }
        },
    }
    event.clear();
}


pub fn create_edge(
    from: &PathBuf, 
    to: &PathBuf, 
    etype: EdgeTypes,
    commands: &mut Commands,
    edges: &Query<(&GraphDataEdge, &EdgeType)>,
){
    // Check if an edge already exists between the node pair
    for (edge, _edge_type) in edges.iter() {
        if edge.same_pair(&GraphDataEdge {
            source: (*from).to_path_buf(),
            target: (*to).to_path_buf(),
        }) {
            println!("Edge already exists between {:?} and {:?}", from, to);
            return
        }
    }

    println!("Creating edge from {:?} to {:?}", from, to);

    let mut initial_attributes: HashMap<String, Option<f32>> = HashMap::new();

    initial_attributes.insert(
        "k".to_string(), Some(0.85),
    );
    initial_attributes.insert(
        "length".to_string(), Some(210.0),
    );

    commands.spawn((
        GraphDataEdge {
            source: (*from).to_path_buf(),
            target: (*to).to_path_buf(),
        },
        EdgeType {
            etype: etype.clone(),
        },
        Attributes {
            attributes: initial_attributes,
        }),
    );
}

/// Function to update the GraphNodeEdges component of nodes when an edge is spawned
/// The event could possibly be modified to include the source and target paths, 
/// to simplify the function. 
pub fn add_edge_to_node_indexes(
    new_edges: Query<(Entity, &GraphDataEdge), Added<GraphDataEdge>>,
    pe_index: Res<PathsToEntitiesIndex>,
    mut node_edges: Query<&mut GraphNodeEdges>,
) {
    for (edge_entity, edge_data) in new_edges.iter() {

        let source_entity = match pe_index.0.get(&edge_data.source){
            Some(entity) => entity,
            None => {continue},
        };
        let mut source_edges = match node_edges.get_mut(*source_entity){
            Ok(edges) => edges,
            Err(_) => {continue},
        };

        source_edges.insert_edge(edge_data.target.clone(), edge_entity);
 
        let target_entity = match pe_index.0.get(&edge_data.target){
            Some(entity) => entity,
            None => {continue},
        };
        let mut target_edges = match node_edges.get_mut(*target_entity){
            Ok(edges) => edges,
            Err(_) => {continue},
        };

        target_edges.insert_edge(edge_data.source.clone(), edge_entity);
    }
}

pub fn despawn_edges(
    mut commands: Commands,
    mut edges: Query<(Entity, &GraphDataEdge)>,
    pe_index: Res<PathsToEntitiesIndex>,
) {
    for (edge_entity, edge_data) in edges.iter_mut() {
        let source_entity = pe_index.0.get(&edge_data.source);
        let target_entity = pe_index.0.get(&edge_data.target);

        if source_entity.is_none() || target_entity.is_none() {
            // println!("Despawning edge");
            commands.entity(edge_entity).despawn_recursive();

        }
    }
}
