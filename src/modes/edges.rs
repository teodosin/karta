// Mode for creation and editing of edges

use bevy::prelude::*;


use crate::{input::pointer::{left_click_just_released, InputData}, graph::{context::PathsToEntitiesIndex, edges::create_edge, nodes::GraphNode}};

use super::KartaModeState;

pub struct EdgesPlugin;

impl Plugin for EdgesPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, create_edge_from_drag
                //.after()
                .run_if(in_state(KartaModeState::Edges)
                    .and_then(left_click_just_released)
                )
            )
            .add_systems(Update, draw_edge_preview
                .run_if(in_state(KartaModeState::Edges))
            )
        ;

    }
}

fn create_edge_from_drag(
    input_data: Res<InputData>,
    pe_index: Res<PathsToEntitiesIndex>,
    mut commands: Commands,
) {
    println!("Creating edge from drag");

    if input_data.latest_press_entity.is_none() {
        println!("No press entity");
        return
    }
    if input_data.latest_hover_entity.is_none() {
        //println!("No hover entity");
        return
    }
    if input_data.latest_press_entity == input_data.latest_hover_entity {
        return
    }

    let from = input_data.latest_press_entity.clone().unwrap();
    let to = input_data.latest_hover_entity.clone().unwrap();

    let from = pe_index.0.get(&from).unwrap();
    let to = pe_index.0.get(&to).unwrap();

    println!("Creating edge from {:?} to {:?}", from, to);

    create_edge(from, to, &mut commands);


}

fn draw_edge_preview(
    mut input_data: ResMut<InputData>,
    _mouse: Res<Input<MouseButton>>,
    nodes: Query<&Transform, With<GraphNode>>,
    pe_index: Res<PathsToEntitiesIndex>,
    mut gizmos: Gizmos,
) {
    if input_data.latest_press_entity.is_none() {
        return
    }

    if input_data.left_just_released {
        input_data.latest_press_entity = None;
    }

    let cursor = input_data.curr_position;
    
    let from = match input_data.latest_press_entity.clone() {
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

