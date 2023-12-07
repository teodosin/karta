
use std::collections::HashMap;

use bevy::prelude::*;

use crate::{graph::attribute::Attributes, events::edges::EdgeSpawnedEvent};

use super::{nodes::GraphDataNode, context::CurrentContext, graph_cam::ViewData};

pub struct EdgesPlugin;

impl Plugin for EdgesPlugin {
    fn build(&self, app: &mut App) {
        app

            .add_systems(Last, despawn_edges
                //.run_if(resource_changed::<CurrentContext>())
            )
        ;
    }
}

// ----------------------------------------------------------------
// Component definitions

// A component for the most basic data of an EDGE
#[derive(Component, Reflect)]
pub struct GraphEdge {
    pub from: Entity,
    pub to: Entity,
}

#[derive(Component)]
pub struct EdgeType {
    pub etype: EdgeTypes,
}

pub enum EdgeTypes {
    Base,
    _Parent,
}

// TODO 0.12: Convert to One-Shot System
// And use EdgeDefaults resource to set the default length
pub fn create_edge(
    event: &mut EventWriter<EdgeSpawnedEvent>,
    from: &Entity, 
    to: &Entity, 
    commands: &mut Commands,
    _view_data: &mut ViewData
){

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
            from: *from,
            to: *to,
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

pub fn despawn_edges(
    mut commands: Commands,
    mut edges: Query<(Entity, &GraphEdge)>,
    nodes: Query<&GraphDataNode>,
) {
    for (edge_entity, edge_data) in edges.iter_mut() {
        if nodes.get(edge_data.from).is_err() || nodes.get(edge_data.to).is_err() {
            println!("Despawning edge");
            commands.entity(edge_entity).despawn_recursive();
        }
    }
}
