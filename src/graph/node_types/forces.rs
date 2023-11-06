// Forces and constraints for the graph

// They are implemented as systems that iterate over the graph nodes and edges,
// and apply forces to them. Because they are regular bevy systems, and always running, 
// they don't need to be called manually, and whether they run depends on if there 
// are node entities with the required components.

// All these systems do is calculate forces and add them to the Velocity component
// of the node. The Velocity component is then used by another system that actually
// applies all the forces. 

struct ForceNodesPlugin;

impl Plugin for ForceNodesPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, edge_spring_constraint)
            .add_systems(Update, repulsion_constraint)
    }
}

#[derive(Component)]
struct NodeForce {
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
// The resting length and stiffness values are inputs to the node.
fn edge_spring_constraint (
    forces: Query<(&GraphNode, &mut NodeForce)>,
    mut nodes: Query<(Entity, &GraphNode, &Transform, &mut Velocity)>,
    edges: Query<(&GraphEdge, &Attributes)>,
){

}

// Constraint: Repulsion
// ----------------------------------------------------------------
// This constraint applies a repulsive force to each node, based on the distance between them.
// The force is inversely proportional to the distance squared.
fn repulsion_constraint (
    forces: Query<&GraphNode, With<NodeForce>>,
    mut nodes: Query<(Entity, &GraphNode, &Transform, &mut Velocity)>,
){
    
}

// Constraint: Radial Spread
// ----------------------------------------------------------------
// This constraint applies a force to each node, trying to make the
// angles between their edges with the root equal.

// Constraint: Line Spread
// ----------------------------------------------------------------
// This constraint applies a force to each node, trying to spread them
// evenly along a line. 

