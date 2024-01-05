// Mode for creation and editing of edges



use bevy::prelude::*;


use crate::{ graph::{context::PathsToEntitiesIndex, edges::{create_edge, EdgeTypes, EdgeType, GraphDataEdge}, nodes::GraphDataNode}, bevy_overlay_graph::input::pointer::{InputData, left_click_just_released}};

pub struct EdgesPlugin;

impl Plugin for EdgesPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, create_edge_from_drag
                //.after()
                // .run_if(in_state(KartaModeState::Edges)
                //     .and_then(left_click_just_released)
                .run_if(left_click_just_released)
                )
            .add_systems(Update, draw_edge_preview
                // .run_if(in_state(KartaModeState::Edges))
            )
        ;

    }
}

fn create_edge_from_drag(
    input_data: Res<InputData>,
    mut commands: Commands,
    edges: Query<(&GraphDataEdge, &EdgeType)>,
) {
    println!("Creating edge from drag");

    if input_data.latest_press_nodepath.is_none() {

        //println!("No press entity");
        return
    }
    if input_data.latest_hover_nodepath.is_none() {
        //println!("No hover entity");
        return
    }
    if input_data.latest_press_nodepath == input_data.latest_hover_nodepath {
        println!("Same entity");
        return
    }

    // Enforce only being able to create edges from the outline
    if !input_data.latest_is_outline() {
        return
    }

    let from = input_data.latest_press_nodepath.clone().unwrap();
    let to = input_data.latest_hover_nodepath.clone().unwrap();

    println!("Creating edge from {:?} to {:?}", from, to);

    create_edge(
        &from, 
        &to, 
        EdgeTypes::Base,
        &mut commands,
        &edges,
    );


}

fn draw_edge_preview(
    mut input_data: ResMut<InputData>,
    _mouse: Res<Input<MouseButton>>,
    nodes: Query<&Transform, With<GraphDataNode>>,
    pe_index: Res<PathsToEntitiesIndex>,
    mut gizmos: Gizmos,
) {
    if input_data.latest_press_nodepath.is_none() {
        return
    }

    if !input_data.latest_is_outline() {
        return
    }

    if input_data.left_just_released {
        input_data.latest_press_nodepath = None;
    }

    let cursor = input_data.curr_position;
    
    let from = match input_data.latest_press_nodepath.clone() {
        Some(from) => from,
        None => return,
    };
    let from = pe_index.0.get(&from).unwrap();
    let to = cursor;
    
    
    let start = match nodes.get(*from) {
        Ok(node) => node,
        Err(_) => return,
    };
    
    gizmos.line_2d(
        Vec2::new(start.translation.x, start.translation.y),
        Vec2::new(to.x, to.y),
        Color::YELLOW_GREEN,
    );
    
    
}

// -------------------------------------------
// ------------------ Tests ------------------
// -------------------------------------------


// A test for the edge creation
// #[test]
fn test_create_edge_from_drag() {
    use bevy::utils::HashMap;
    use std::path::PathBuf;
    use crate::graph::node_types::NodeTypes;
    // Setup a world and schedule for Bevy ECS (assuming Bevy is being used)
    let mut app = App::new();

    let entity1 = app.world.spawn(
        GraphDataNode {
            path: PathBuf::from("path/to/entity1"),
            ntype: NodeTypes::Base,
            data: None,
        }
    ).id();
    
    let entity2 = app.world.spawn(
        GraphDataNode {
            path: PathBuf::from("path/to/entity2"),
            ntype: NodeTypes::Base,
            data: None,
        }
    ).id();
    
    let node1 = &app.world.get::<GraphDataNode>(entity1).unwrap().path;
    let node2 = &app.world.get::<GraphDataNode>(entity2).unwrap().path;

    // // Mock InputData, Valid
    // let input_data_mock_valid = InputData {
    //     latest_press_entity: Some(node1.clone()), // Assuming Entity can be created like this
    //     latest_hover_entity: Some(node2.clone()),
    //     target_type: InputTarget::NodeOutline,
    //     ..default()
    // };

    // // Mock InputData, Invalid: Same entity
    // let input_data_mock_invalid_same_entity = InputData {
    //     latest_press_entity: Some(node1.clone()),
    //     latest_hover_entity: Some(node1.clone()),
    //     target_type: InputTarget::NodeOutline,
    //     ..default()
    // };

    // Create a mock PathsToEntitiesIndex
    let mut pe_index = PathsToEntitiesIndex(HashMap::new());
    pe_index.0.insert(PathBuf::from("path/to/entity1"), entity1);
    pe_index.0.insert(PathBuf::from("path/to/entity2"), entity2);
    app.world.insert_resource(pe_index);


    // 1. Test valid input_data scenario
    // app.world.insert_resource(input_data_mock_valid);
    app.add_systems(Update, create_edge_from_drag);

    app.update();
    assert_eq!(app.world.query::<&GraphDataEdge>().iter(&app.world).len(), 1);
    
    app.world.remove_resource::<InputData>();
    
    // 2. Test invalid scenario: same entity
    // app.world.insert_resource(input_data_mock_invalid_same_entity);

    app.update();
    assert_eq!(app.world.query::<&GraphDataEdge>().iter(&app.world).len(), 1);
    
    
    //app.world.insert_resource(input_data_mock_invalid_same_entity);


}
