
use std::collections::HashMap;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{graph::attribute::Attributes, events::edges::EdgeSpawnedEvent};

use super::{nodes::{GraphDataNode, GraphNodeEdges}, graph_cam::ViewData};

pub struct EdgesPlugin;

impl Plugin for EdgesPlugin {
    fn build(&self, app: &mut App) {
        app

            .add_systems(Last, despawn_edges
                // TODO: Does this have to run every frame?
                //.run_if(resource_changed::<CurrentContext>())
            )
            .add_systems(PostUpdate, add_edge_to_node_indexes
                .run_if(on_event::<EdgeSpawnedEvent>()))
        ;
    }
}

// ----------------------------------------------------------------
// Component definitions

// A component for the most basic data of an EDGE
#[derive(Component, Reflect)]
pub struct GraphEdge {
    pub source: Entity,
    pub target: Entity,
}

impl GraphEdge {
    pub fn same_pair(&self, other: &GraphEdge) -> bool {
        if self.source == other.source && self.target == other.target {
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

// TODO 0.12: Convert to One-Shot System
// And use EdgeDefaults resource to set the default length
pub fn create_edge(
    event: &mut EventWriter<EdgeSpawnedEvent>,
    from: &Entity, 
    to: &Entity, 
    etype: EdgeTypes,
    commands: &mut Commands,
    edges: &Query<(&GraphEdge, &EdgeType)>,
){
    // Check if an edge already exists between the node pair


    for (edge, edge_type) in edges.iter() {
        if edge.same_pair(&GraphEdge {
            source: *from,
            target: *to,
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

    let edge = commands.spawn((
        GraphEdge {
            source: *from,
            target: *to,
        },
        EdgeType {
            etype,
        },
        Attributes {
            attributes: initial_attributes,
        }),
    ).id();

    event.send(EdgeSpawnedEvent {
        entity: edge,
        connected_to_focal: true,
        edge_type: EdgeTypes::Base,
    });
}

pub fn add_edge_to_node_indexes(
    mut event: EventReader<EdgeSpawnedEvent>,
    mut edges: Query<&GraphEdge>,
    nodes: Query<Entity, With<GraphNodeEdges>>,
    mut node_edges: Query<&mut GraphNodeEdges>,
) {
    for ev in event.read(){
        let edge_entity = ev.entity;
        let edge_data = edges.get_mut(edge_entity).unwrap();

        let mut source_edges = match node_edges.get_mut(edge_data.source){
            Ok(edges) => edges,
            Err(_) => {continue},
        };
        let target_entity = nodes.get(edge_data.target).unwrap();

        source_edges.add_edge(target_entity, edge_entity);
 
        let mut target_edges = match node_edges.get_mut(edge_data.target){
            Ok(edges) => edges,
            Err(_) => {continue},
        };
        let source_entity = nodes.get(edge_data.source).unwrap();

        target_edges.add_edge(source_entity, edge_entity);
    }
}

pub fn despawn_edges(
    mut commands: Commands,
    mut edges: Query<(Entity, &GraphEdge)>,
    nodes: Query<&GraphNodeEdges>,
) {
    for (edge_entity, edge_data) in edges.iter_mut() {
        if nodes.get(edge_data.source).is_err() || nodes.get(edge_data.target).is_err() {
            println!("Despawning edge");
            commands.entity(edge_entity).despawn_recursive();

        }
    }
}
