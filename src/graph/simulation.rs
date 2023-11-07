// FORCE SIMULATION

use std::collections::HashMap;

use bevy::{prelude::{Query, Transform, With, Without, Vec2, Plugin, App, Update, Entity, Res, IntoSystemConfigs, Gizmos, Color, Resource, PostUpdate}, time::Time};

use crate::{ui::nodes::{GraphViewNode, Velocity2D}};

use super::{nodes::{PinnedToPosition, GraphNodeEdges}, edges::GraphEdge, attribute::Attributes, context::{Selected, CurrentContext}};

pub struct GraphSimPlugin;

impl Plugin for GraphSimPlugin {
    fn build(&self, app: &mut App) {
        app
            //.add_systems(Update, simulation.before(move_node_selection))

            .insert_resource(GraphSimSettings::default())
            .add_systems(PostUpdate, apply_forces)
        ;
    }

}

const FORCE_UPPER_LIMIT: f32 = 50.0;
const FORCE_LOWER_LIMIT: f32 = 0.5;
const DAMPING_FACTOR: f32 = 0.95;
const SIMULATION_STEPS: usize = 8;

// Should the simulation settings be a part of the sim root node?
#[derive(Debug, Resource)]
pub struct GraphSimSettings {
    pub force_upper_limit: f32,
    pub force_lower_limit: f32,
    pub damping_factor: f32,
    pub simulation_steps: usize,
}

impl Default for GraphSimSettings {
    fn default() -> Self {
        GraphSimSettings {
            force_upper_limit: 50.0,
            force_lower_limit: 0.5,
            damping_factor: 0.85,
            simulation_steps: 8,
        }
    }
}

fn apply_forces(
    sim_settings: Res<GraphSimSettings>,
    time: Res<Time>,
    mut nodes: Query<
        (Entity, &GraphViewNode, &mut Transform, &mut Velocity2D), 
        (Without<PinnedToPosition>, Without<Selected>)
    >,
    mut gizmos: Gizmos,
) {
    //for step in 0..sim_settings.simulation_steps {
        for (node, view, mut pos, mut vel) in nodes.iter_mut() {

            let mut force = vel.velocity;
            vel.velocity = Vec2::ZERO;
                
            if force.length() < sim_settings.force_lower_limit {
                continue
            }

            if force.length() > sim_settings.force_upper_limit {
                force = force.normalize() * sim_settings.force_upper_limit;
            }
            
            force = force * sim_settings.damping_factor / time.delta().as_millis() as f32;
            
            // Lines for debugging the forces
            // gizmos.line_2d(
            //     pos.translation.truncate(), 
            //     pos.translation.truncate() + force * 100.0, 
            //     Color::RED,
            // );
                
            pos.translation.x += force.x;
            pos.translation.y += force.y;


        }
    //}
}

