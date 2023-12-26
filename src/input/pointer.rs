use std::path::PathBuf;

use bevy::{prelude::{Vec2, ResMut, Input, Res, MouseButton, Query, Camera, GlobalTransform, With, Camera2d, Resource}, window::Window, ecs::entity::Entity};



#[derive(Resource, Debug)]
pub struct InputData {
    pub latest_click_nodepath: Option<PathBuf>,
    pub latest_press_nodepath: Option<PathBuf>,
    pub latest_hover_nodepath: Option<PathBuf>,

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
            latest_click_nodepath: None,
            latest_press_nodepath: None,
            latest_hover_nodepath: None,

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

pub fn left_click_just_released(
    input: Res<InputData>,
) -> bool {
    input.left_just_released
}

pub fn update_cursor_info(
    mut cursor_history: ResMut<InputData>,
    mouse: Res<Input<MouseButton>>,
    window: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
) {
    let (camera, camera_transform) = camera_q.single();

    cursor_history.prev_position = cursor_history.curr_position;

    if mouse.just_pressed(MouseButton::Middle) {
        cursor_history.drag_position = cursor_history.curr_position;
    }

    if let Some(world_position) = window.single().cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate())
    {
        cursor_history.curr_position = world_position;
    }

    if mouse.just_released(MouseButton::Left) {
        //cursor_history.latest_press_entity = None;
        cursor_history.left_just_released = true;
    }
    else {
        cursor_history.left_just_released = false;
    }
}