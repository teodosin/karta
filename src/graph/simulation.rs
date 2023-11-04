// FORCE SIMULATION

use std::collections::HashMap;

use bevy::{prelude::{Query, Transform, With, Without, Vec2, Plugin, App, Update, Entity, Res, IntoSystemConfigs, Gizmos, Color}, time::Time};

use crate::modes::r#move::move_node_selection;

use super::{nodes::{GraphNode, PinnedToPosition, GraphNodeEdges}, edges::GraphEdge, attribute::Attributes, context::{Selected, CurrentContext}};

pub struct GraphSimPlugin;

impl Plugin for GraphSimPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, simulation.before(move_node_selection))
        ;
    }

}

const FORCE_UPPER_LIMIT: f32 = 50.0;
const FORCE_LOWER_LIMIT: f32 = 0.5;
const DAMPING_FACTOR: f32 = 0.95;
const SIMULATION_STEPS: usize = 8;

fn simulation(
    context: Res<CurrentContext>,
    time: Res<Time>,
    mut nodes: Query<(Entity, &mut Transform, &GraphNode)>,
    edges: Query<(&GraphEdge, &Attributes)>,
    selected: Query<&Selected, With<GraphNode>>,
    pinned: Query<&PinnedToPosition, With<GraphNode>>,
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
