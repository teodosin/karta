
use std::collections::HashMap;

use bevy::prelude::*;

use crate::{graph::attribute::Attributes, ui::edges::add_edge_ui};

use super::{nodes::GraphNode, context::CurrentContext, graph_cam::ViewData};

pub struct EdgesPlugin;

impl Plugin for EdgesPlugin {
    fn build(&self, app: &mut App) {
        app

            .add_systems(Last, despawn_edges
                .run_if(resource_changed::<CurrentContext>())
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

// TODO 0.12: Convert to One-Shot System
// And use EdgeDefaults resource to set the default length
pub fn create_edge(
    from: &Entity, 
    to: &Entity, 
    commands: &mut Commands,
    view_data: &mut ViewData
){

    println!("Creating edge from {:?} to {:?}", from, to);

    let mut initial_attributes: HashMap<String, Option<f32>> = HashMap::new();

    initial_attributes.insert(
        "k".to_string(), Some(0.2),
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
    add_edge_ui(
        commands, 
        edge,
        view_data
    )
}

pub fn despawn_edges(
    mut commands: Commands,
    mut edges: Query<(Entity, &GraphEdge)>,
    nodes: Query<&GraphNode>,
) {
    for (edge_entity, edge_data) in edges.iter_mut() {
        if nodes.get(edge_data.from).is_err() || nodes.get(edge_data.to).is_err() {
            println!("Despawning edge");
            commands.entity(edge_entity).despawn_recursive();
        }
    }
}
