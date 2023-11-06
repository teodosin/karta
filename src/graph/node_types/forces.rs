// Forces and constraints for the graph

// They are implemented as systems that iterate over the graph nodes and edges,
// and apply forces to them. Because they are regular bevy systems, and always running, 
// they don't need to be called manually, and whether they run depends on if there 
// are node entities with the required components.

// All these systems do is calculate forces and add them to the Velocity component
// of the node. The Velocity component is then used by another system that actually
// applies all the forces. 

use bevy::{prelude::*, utils::HashMap};

use crate::{graph::{edges::GraphEdge, attribute::Attributes, simulation::GraphSimSettings}, ui::nodes::{GraphViewNode, Velocity2D}};

pub struct ForceNodesPlugin;

impl Plugin for ForceNodesPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, edge_spring_constraints)
            .add_systems(Update, repulsion_constraints)
        ;
    }
}

#[derive(Component)]
pub struct NodeForce {
    running: bool,
}

// Graph Simulation Root
// ----------------------------------------------------------------
// An implicit root node is create, and all nodes are connected to it.

// What if the user doesn't have to create this node, and it is created automatically 
// if there are any force nodes in the current context? 



// Constraint: Edge Spring
// ----------------------------------------------------------------
// This constraint treats edges like springs, and applies a force to each node.
// For now, the attributes that will be read will be hard-coded (k and length).
// In the future, the resting length and stiffness values will be inputs to the node.
pub fn edge_spring_constraints (
    sim_settings: Res<GraphSimSettings>,
    forces: Query<(&GraphViewNode, &mut NodeForce)>,
    mut nodes: Query<(Entity, &GraphViewNode, &Transform, &mut Velocity2D)>,
    edges: Query<(&GraphEdge, &Attributes)>,
){
    // When this force is implemented as a node, we will need to handle multiple of them.

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
            from.2.translation.x - to.2.translation.x,
            from.2.translation.y - to.2.translation.y,
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
        
        
        if attractive_force.abs() < sim_settings.force_lower_limit {
            continue
        }
        
        if attractive_force.abs() > sim_settings.force_upper_limit {
                attractive_force = attractive_force / attractive_force * sim_settings.force_upper_limit;
        }
            
            
        match nodes.get_mut(edge.from){
            Ok(mut node) => {
                
                node.3.velocity -= diff / dist * attractive_force;
                
            },
            Err(_) => continue,
        }
        
        match nodes.get_mut(edge.to){
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
    force_nodes: Query<&GraphViewNode, With<NodeForce>>,
    mut nodes: Query<(Entity, &GraphViewNode, &Transform, &mut Velocity2D)>,
){
    let mut forces: HashMap<Entity, Vec2> = HashMap::new();

    for (node_a, view_a, pos_a, mut vel_a) in nodes.iter(){
        for (node_b, view_b, pos_b, mut vel_b) in nodes.iter(){
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

    for (node, _view, _pos, mut vel) in nodes.iter_mut(){
        vel.velocity = forces[&node];
    }
}

// Constraint: Radial Spread
// ----------------------------------------------------------------
// This constraint applies a force to each node, trying to make the
// angles between their edges with the root equal.

// Constraint: Line Spread
// ----------------------------------------------------------------
// This constraint applies a force to each node, trying to spread them
// evenly along a line. 

