// FORCE SIMULATION

use bevy::{prelude::*, utils::HashMap};
use bevy_karta_client::prelude::{pe_index::PathsToEntitiesIndex, Attributes, DataEdge, ViewNode};
use bevy_mod_picking::selection::PickSelection;

use crate::prelude::Pins;

use super::nodes::{TargetPosition, Velocity2D};

pub struct GraphSimPlugin;

impl Plugin for GraphSimPlugin {
    fn build(&self, app: &mut App) {
        app
            //.add_systems(Update, simulation.before(move_node_selection))

            .insert_resource(GraphSimSettings::default())
            .add_systems(Update, (
                repulsion_constraints,
                edge_spring_constraints
            ).chain())
            .add_systems(PostUpdate, apply_forces)
        ;
    }

}

const _FORCE_UPPER_LIMIT: f32 = 50.0;
const _FORCE_LOWER_LIMIT: f32 = 0.5;
const _DAMPING_FACTOR: f32 = 0.95;
const _SIMULATION_STEPS: usize = 8;

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
            force_upper_limit: 350.0,
            force_lower_limit: 0.5,
            damping_factor: 0.85,
            simulation_steps: 8,
        }
    }
}

// Constraint: Edge Spring
// ----------------------------------------------------------------
/// This constraint treats edges like springs, and applies a force to each node.
/// For now, the attributes that will be read will be hard-coded (k and length).
/// In the future, the resting length and stiffness values will be inputs to the node.
pub fn edge_spring_constraints (
    // _forces: Query<(&ViewNode, &mut NodeForce)>,
    mut nodes: Query<(Entity, &ViewNode, &Transform, &mut Velocity2D)>,
    edges: Query<(&DataEdge, &Attributes)>,
    pe_index: Res<PathsToEntitiesIndex>,
){
    // When this force is implemented as a node, we will need to handle multiple of them.

    for (edge, attr) in edges.iter(){
        let source_entity = match pe_index.get_view(&edge.source){
            Some(entity) => entity,
            None => continue,
        };
        let target_entity = match pe_index.get_view(&edge.target){
            Some(entity) => entity,
            None => continue,
        };

        let from = match nodes.get(source_entity){
            Ok(node) => node,
            Err(_) => continue,
        };
        let to = match nodes.get(target_entity){
            Ok(node) => node,
            Err(_) => continue,
        };
        
        let diff = Vec2::new(
            from.2.translation.x - to.2.translation.x,
            from.2.translation.y - to.2.translation.y,
        );
        
        // distance between the two positions
        let dist = diff.length() + 0.0001;
        
        let len = 200.0;
        
        let displacement = dist - len;
        
        let attractive_force = 0.85 * displacement;      
            
        match nodes.get_mut(source_entity){
            Ok(mut node) => {
                node.3.velocity -= diff / dist * attractive_force;
            },
            Err(_) => continue,
        }
        
        match nodes.get_mut(target_entity){
            Ok(mut node) => {                
                node.3.velocity += diff / dist * attractive_force;                
            },
            Err(_) => continue,
        }        
    } 
}

// Constraint: Repulsion
// ----------------------------------------------------------------
// This constraint applies a repulsive force to each node, based on the distance between them.
// The force is inversely proportional to the distance squared.

// Same current restrictions and future plans as for the edge spring constraints apply here. 
pub fn repulsion_constraints (
    mut nodes: Query<(Entity, &ViewNode, &Transform, &mut Velocity2D)>,
){
    let mut forces: HashMap<Entity, Vec2> = HashMap::new();

    for (node_a, _view_a, pos_a, mut _vel_a) in nodes.iter(){
        for (node_b, _view_b, pos_b, mut _vel_b) in nodes.iter(){
            if node_a == node_b {
                continue
            }
            
            let diff = Vec2::new(
                pos_a.translation.x - pos_b.translation.x,
                pos_a.translation.y - pos_b.translation.y,
            );
            
            // distance between the two positions
            let dist = diff.length();

            // if dist > 150.0 {
            //     continue
            // }
             
            let repulsive_force = 20000.0 / dist.powf(1.25);

            *forces.entry(node_a).or_insert(Vec2::ZERO) += diff / dist * repulsive_force;
            *forces.entry(node_b).or_insert(Vec2::ZERO) -= diff / dist * repulsive_force;
        }
    }

    for (node, _view, _pos, mut vel) in nodes.iter_mut(){
        if forces.contains_key(&node){
            vel.velocity = forces[&node];
        }
    }
}

/// System that applies the forces calculated by force nodes
fn apply_forces(
    sim_settings: Res<GraphSimSettings>,
    time: Res<Time>,
    mut nodes: Query<
        (Entity, &ViewNode, &mut Transform, &mut Velocity2D, &Pins, &PickSelection), 
        Without<TargetPosition>
    >,
    mut _gizmos: Gizmos,
    keys: Res<ButtonInput<KeyCode>>,
) {
    // if !keys.pressed(KeyCode::Space) {
    //     return
    // }
    //for step in 0..sim_settings.simulation_steps {
        for (_node, _view, mut pos, mut vel, pins, pick) in nodes.iter_mut() {

            if pick.is_selected || pins.position {continue};

            let mut force = vel.velocity;
            vel.velocity = Vec2::ZERO;
                
            if force.length() < sim_settings.force_lower_limit {
                continue
            }

            if force.length() > sim_settings.force_upper_limit {
                force = force.normalize() * sim_settings.force_upper_limit;
            }
            
            force = force * sim_settings.damping_factor * time.delta().as_secs_f32();
            
            // Lines for debugging the forces
            // _gizmos.line_2d(
            //     pos.translation.truncate(), 
            //     pos.translation.truncate() + force * 100.0, 
            //     LinearRgba::RED,
            // );
                
            pos.translation.x += force.x;
            pos.translation.y += force.y;


        }
    //}
}

