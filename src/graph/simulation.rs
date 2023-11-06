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
            damping_factor: 0.95,
            simulation_steps: 8,
        }
    }
}

fn apply_forces(
    sim_settings: Res<GraphSimSettings>,
    time: Res<Time>,
    mut nodes: Query<(Entity, &GraphViewNode, &mut Transform, &mut Velocity2D)>,
    selected: Query<&Selected, With<GraphViewNode>>,
    pinned: Query<&PinnedToPosition, With<GraphViewNode>>,
) {
    //for step in 0..sim_settings.simulation_steps {
        for (node, view, mut pos, vel) in nodes.iter_mut() {
            let mut force = vel.velocity;
                
            if force.length() < FORCE_LOWER_LIMIT {
                continue
            }

            match selected.get(node){
                Ok(_) => continue,
                Err(_) => (),
            }

            if force.length() > FORCE_UPPER_LIMIT {
                force = force.normalize() * FORCE_UPPER_LIMIT;
            }
            
            force = force * DAMPING_FACTOR / time.delta().as_millis() as f32;
            
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

fn simulation(
    context: Res<CurrentContext>,
    time: Res<Time>,
    mut nodes: Query<(Entity, &mut Transform, &GraphViewNode)>,
    edges: Query<(&GraphEdge, &Attributes)>,
    selected: Query<&Selected, With<GraphViewNode>>,
    pinned: Query<&PinnedToPosition, With<GraphViewNode>>,
    mut gizmos: Gizmos,
){
    for _ in 0..SIMULATION_STEPS {

        // First we will calculate and collect all the forces to this hashmap
        let mut forces: HashMap<Entity, Vec2> = HashMap::new();
        
        // REPULSIVE FORCES
        for (node_a, pos_a, name_a) in nodes.iter(){
            for (node_b, pos_b, name_a) in nodes.iter(){
                if node_a == node_b {
                    continue
                }
                
                let diff = Vec2::new(
                    pos_a.translation.x - pos_b.translation.x,
                    pos_a.translation.y - pos_b.translation.y,
                );
                
                // distance between the two positions
                let dist = diff.length();
                
                let repulsive_force = 30000.0 / dist.powi(2);
                
                *forces.entry(node_a).or_insert(Vec2::ZERO) += diff / dist * repulsive_force;
                *forces.entry(node_b).or_insert(Vec2::ZERO) -= diff / dist * repulsive_force;
                
            }
        }
        
        // ATTRACTIVE/SPRING FORCES
        for (edge, attr) in edges.iter(){
            let from = match nodes.get(edge.from){
                Ok(node) => node,
                Err(_) => continue,
            };
            let to = match nodes.get(edge.to){
                Ok(node) => node,
                Err(_) => continue,
            };
            
            let diff = Vec2::new(
                from.1.translation.x - to.1.translation.x,
                from.1.translation.y - to.1.translation.y,
            );
            
            // distance between the two positions
            let dist = diff.length() + 0.0001;
            
            let len = match attr.attributes.get("length") {
                Some(len) => match len {
                    Some(len) => *len,
                    None => continue,
                },
                None => continue,
            };
            
            let displacement = dist - len;
            
            let mut attractive_force = match attr.attributes.get("k") {
                Some(k) => match k {
                    Some(k) => *k,
                    None => continue,
                },
                None => continue,
            } * displacement;
            
            
            if attractive_force.abs() < FORCE_LOWER_LIMIT {
                continue
            }
            
            if attractive_force.abs() > FORCE_UPPER_LIMIT {
                    attractive_force = attractive_force / attractive_force * FORCE_UPPER_LIMIT;
            }
                
                
            match nodes.get_mut(edge.from){
                Ok(node) => {
                    match selected.get(edge.from){
                        Ok(_) => continue,
                        Err(_) => (),
                    }
                    
                    *forces.entry(node.0).or_insert(Vec2::ZERO) -= diff / dist * attractive_force;
                    
                },
                Err(_) => continue,
            }
            
            match nodes.get_mut(edge.to){
                Ok(node) => {
                    
                    *forces.entry(node.0).or_insert(Vec2::ZERO) += diff / dist * attractive_force;
                    
                },
                Err(_) => continue,
            }        
        }
        
        // APPLY FORCES
        for (node, mut pos, name) in nodes.iter_mut(){
            
            let mut force = forces[&node];
            
            if force.length() < FORCE_LOWER_LIMIT {
                continue
            }

            match selected.get(node){
                Ok(_) => continue,
                Err(_) => (),
            }

            if force.length() > FORCE_UPPER_LIMIT {
                force = force.normalize() * FORCE_UPPER_LIMIT;
            }
            
            force = force * DAMPING_FACTOR / time.delta().as_millis() as f32;
            
            // Lines for debugging the forces
            // gizmos.line_2d(
            //     pos.translation.truncate(), 
            //     pos.translation.truncate() + force * 100.0, 
            //     Color::RED,
            // );
                
            pos.translation.x += force.x;
            pos.translation.y += force.y;
        }
    }
}
