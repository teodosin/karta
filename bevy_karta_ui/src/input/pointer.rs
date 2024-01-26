use bevy::{
    ecs::{entity::Entity, event::EventReader}, hierarchy::Parent, prelude::{
        Camera, Camera2d, GlobalTransform, Input, MouseButton, Query, Res, ResMut, Resource, Vec2,
        With,
    }, window::Window
};

use crate::{prelude::{graph_cam::GraphCamera, node_events::{NodeClickEvent, NodeHoverEvent, NodeHoverStopEvent, NodePressedEvent}}, ui::nodes::{GraphViewNode, NodeOutline}};

#[derive(Resource, Debug)]
pub struct InputData {
    pub latest_click_entity: Option<Entity>,
    pub latest_press_entity: Option<Entity>,
    pub latest_hover_entity: Option<Entity>,

    pub latest_edge_entity: Option<Entity>,

    pub target_type: GraphPickingTarget,

    pub left_just_released: bool,

    pub drag_position: Vec2,
    pub prev_position: Vec2,
    pub curr_position: Vec2,
}

#[derive(Debug, Clone, Copy)]
pub enum GraphPickingTarget {
    Node,
    NodeOutline,
    Edge,
    None,
}

impl Default for InputData {
    fn default() -> Self {
        InputData {
            latest_click_entity: None,
            latest_press_entity: None,
            latest_hover_entity: None,

            latest_edge_entity: None,

            target_type: GraphPickingTarget::None,

            left_just_released: false,

            drag_position: Vec2::ZERO,
            prev_position: Vec2::ZERO,
            curr_position: Vec2::ZERO,
        }
    }
}

impl InputData {
    pub fn latest_is_outline(&self) -> bool {
        match self.target_type {
            GraphPickingTarget::NodeOutline => true,
            _ => false,
        }
    }

    pub fn latest_is_node(&self) -> bool {
        match self.target_type {
            GraphPickingTarget::Node => true,
            _ => false,
        }
    }

    pub fn latest_is_edge(&self) -> bool {
        match self.target_type {
            GraphPickingTarget::Edge => true,
            _ => false,
        }
    }

    pub fn set_target_type(&mut self, target_type: GraphPickingTarget) {
        self.target_type = target_type;
    }
}

pub fn left_click_just_released(input: Res<InputData>) -> bool {
    input.left_just_released
}

/// System that updates the InputData resource with the current cursor position.
pub fn update_cursor_info(
    mut cursor_history: ResMut<InputData>,
    mouse: Res<Input<MouseButton>>,
    window: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform), (With<Camera2d>, With<GraphCamera>)>,
) {
    let (camera, camera_transform) = camera_q.single();

    cursor_history.prev_position = cursor_history.curr_position;

    if mouse.just_pressed(MouseButton::Middle) {
        cursor_history.drag_position = cursor_history.curr_position;
    }

    if let Some(world_position) = window
        .single()
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate())
    {
        cursor_history.curr_position = world_position;
    }

    if mouse.just_released(MouseButton::Left) {
        cursor_history.left_just_released = true;
    } else {
        cursor_history.left_just_released = false;
    }
}



pub fn handle_node_click(
    mut event: EventReader<NodeClickEvent>,
    mut input_data: ResMut<InputData>,

    nodes: Query<(Entity, &GraphViewNode)>,
    outlines: Query<&Parent, With<NodeOutline>>,
){
    if event.is_empty(){
        return
    }

    // TODO: Handle multiple events
    match event.read().next().unwrap().target {
        None => {
            //println!("No event");
            input_data.latest_click_entity = None;
        }
        Some(target) => {
            //println!("Event: {:?}", target);
            
            match nodes.get(target){
                Ok(node) => {
                    let (entity, data) = node;
                    let target_path = data.path.clone();
                    input_data.latest_click_entity = Some(target_path.clone());
                },
                Err(_) => {
                    //println!("No node found");
                }
            }

            match outlines.get(target){
                Ok(outline) => {
                    let outline_path = nodes.get(outline.get()).unwrap().1.path.clone();
                    input_data.latest_click_entity = Some(outline_path.clone());
                    //println!("Clicking outline: {}", outline_path);
                },
                Err(_) => {
                    //println!("No outline found");
                }
            }
        },
    }
    event.clear();
}

pub fn handle_node_press(
    mut event: EventReader<NodePressedEvent>,
    mut input_data: ResMut<InputData>,
    nodes: Query<&GraphViewNode>,
    outlines: Query<&Parent, With<NodeOutline>>,
){
    if event.is_empty() {
        return
    }
    match event.read().next().unwrap().target {
        None => {
            //println!("No event");
            input_data.latest_press_entity = None;
        }
        Some(target) => {
            //println!("Event: {:?}", target);
            
            match nodes.get(target){
                Ok(node) => {
                    let target_path = node.path.clone();
                    input_data.latest_press_entity = Some(target_path.clone());
                    input_data.set_target_type(GraphPickingTarget::Node);
                    println!("Pressing path: {}", input_data.latest_press_entity.clone().unwrap().display());
                },
                Err(_) => {
                    //println!("No node found for press");
                }
            }

            match outlines.get(target){
                Ok(outline) => {
                    let outline_path = nodes.get(outline.get()).unwrap().path.clone();
                    input_data.latest_press_entity = Some(outline_path.clone());
                    input_data.set_target_type(GraphPickingTarget::NodeOutline);
                    //println!("Pressing outline: {}", outline_path);
                },
                Err(_) => {
                    //println!("No outline found");
                }
            }
        },
    }
    event.clear();
}


pub fn handle_node_hover(
    mut event: EventReader<NodeHoverEvent>,
    mut input_data: ResMut<InputData>,
    nodes: Query<&GraphViewNode>,
    outlines: Query<&Parent, With<NodeOutline>>,
){
    if event.is_empty() {
        return
    }
    
    match event.read().next().unwrap().target {
        None => {
            //println!("No event");
            input_data.latest_hover_entity = None;
        }
        Some(target) => {
            //println!("Event: {:?}", target);
            
            match nodes.get(target){
                Ok(node) => {
                    let target_path = node.path.clone();
                    input_data.latest_hover_entity = Some(target_path.clone());
                    //println!("Hovering over path: {}", target_path);
                },
                Err(_) => {
                    //println!("No node found");
                }
            }

            match outlines.get(target){
                Ok(outline) => {
                    let outline_path = nodes.get(outline.get()).unwrap().path.clone();
                    input_data.latest_hover_entity = Some(outline_path.clone());
                    //println!("Hovering over outline: {}", outline_path);
                },
                Err(_) => {
                    //println!("No outline found");
                }
            }
        },
    }
    event.clear();
}

pub fn handle_node_hover_stop(
    mut event: EventReader<NodeHoverStopEvent>,
    mut input_data: ResMut<InputData>,

){
    if event.is_empty() {
        return
    }

    input_data.latest_hover_entity = None;

    event.clear();
}