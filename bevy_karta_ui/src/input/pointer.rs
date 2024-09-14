use bevy::{
    ecs::{entity::Entity, event::EventReader, query::Without},
    hierarchy::Parent,
    input::ButtonInput,
    prelude::{
        Camera, Camera2d, GlobalTransform, MouseButton, Query, Res, ResMut, Resource, Vec2, With,
    },
    render::view,
    time::{self, Time},
    window::Window,
};
use bevy_fs_graph::prelude::ViewNode;
use bevy_mod_picking::pointer::PointerButton;

use crate::{
    prelude::{
        graph_cam::GraphCamera,
        node_events::{NodeClickEvent, NodeHoverEvent, NodeHoverStopEvent, NodePressedEvent},
    },
    ui::nodes::NodeOutline,
};

// #[derive(Resource, Debug)]
// pub struct InputData {
//     pub latest_click_entity: Option<Entity>,
//     pub latest_press_entity: Option<Entity>,
//     pub latest_hover_entity: Option<Entity>,

//     pub latest_edge_entity: Option<Entity>,

//     pub target_type: GraphPickingTarget,

//     pub left_just_released: bool,

//     pub drag_position: Vec2,
//     pub prev_position: Vec2,
//     pub curr_position: Vec2,
// }

/// Struct that stores information about a single input event. Used by the
/// InputData resource to store the recent history of input events.
#[derive(Debug, Clone)]
struct SingleInputData {
    /// The target graph node of the input event.
    pub target: Option<Entity>,
    pub target_type: GraphPickingTarget,
    pub interaction_type: InteractionType,
    pub time: f64,
}

/// Struct that stores information about a single pointer position. Used by the
/// InputData resource to store the recent history of pointer positions.
#[derive(Debug, Clone)]
pub struct SinglePositionData {
    pub viewport_position: Vec2,
    pub world_position: Vec2,
    pub time: f64,
}

/// Enum of potential targets for picking in the graph and in the scene.
///
/// Will eventually also include node sockets and the ends of edges for disconnection/reconnection.
#[derive(Debug, Clone, Copy)]
pub enum GraphPickingTarget {
    Node,
    NodeOutline,
    Edge,
    SceneEntity,
    None,
}

/// Enum of potential interaction types for picking with entities.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InteractionType {
    Click(PointerButton),
    Press(PointerButton),
    Hover,
    HoverStop,
}

/// Resource for storing rich input data over time. Has functions
/// for accessing this data in various ways, for example in order to implement
/// double-clicking.
#[derive(Resource, Debug)]
pub struct InputData {
    input_limit: usize,
    inputs: Vec<SingleInputData>,

    /// Store the latest input events for each interaction type. Ensures that we always know
    /// which entity was the target of the latest click, press, hover, etc. no matter how
    /// many other events have occurred since then.
    latest_click_data: Option<SingleInputData>,
    latest_press_data: Option<SingleInputData>,
    latest_hover_data: Option<SingleInputData>,
    latest_hover_stop_data: Option<SingleInputData>,

    pos_limit: usize,
    positions: Vec<SinglePositionData>,
    // Should this struct also have data
}

impl Default for InputData {
    fn default() -> Self {
        InputData {
            input_limit: 50,
            inputs: Vec::new(),

            latest_click_data: None,
            latest_press_data: None,
            latest_hover_data: None,
            latest_hover_stop_data: None,

            pos_limit: 300,
            positions: Vec::new(),
        }
    }
}

impl InputData {
    /// Function for inserting a new input event into the InputData resource.
    /// Also updates the latest input event for the given interaction type and
    /// manages the length of the input history.
    pub fn insert_input(&mut self, input: SingleInputData) {
        let input_copy = input.clone();
        self.inputs.push(input_copy);
        if self.inputs.len() > self.input_limit {
            self.inputs.remove(0);
        }

        match input.interaction_type {
            InteractionType::Click(_) => {
                self.latest_click_data = Some(input);
            }
            InteractionType::Press(_) => {
                self.latest_press_data = Some(input);
            }
            InteractionType::Hover => {
                self.latest_hover_data = Some(input);
            }
            InteractionType::HoverStop => {
                self.latest_hover_stop_data = Some(input);
            }
        }
    }

    /// Function for inserting a cursor position into the InputData resource.
    /// Used by the update_cursor_info system.
    pub fn insert_cursor_position(&mut self, position: SinglePositionData) {
        self.positions.push(position);
        if self.positions.len() > self.pos_limit {
            self.positions.remove(0);
        }
    }

    /// Function for checking if the most recent input event was a click of the left mouse button.
    pub fn left_just_released(&self) -> bool {
        match &self.latest_click_data {
            Some(data) => {
                if let InteractionType::Click(button) = data.interaction_type {
                    if button == PointerButton::Primary {
                        return true;
                    }
                }
            }
            None => {}
        }
        false
    }

    pub fn latest_click_entity(&self) -> Option<Entity> {
        match &self.latest_click_data {
            Some(data) => data.target,
            None => None,
        }
    }

    pub fn cursor_world_current_position(&self) -> Vec2 {
        match self.positions.last() {
            Some(data) => data.world_position,
            None => Vec2::ZERO,
        }
    }

    pub fn cursor_world_previous_position(&self) -> Vec2 {
        match self.positions.get(self.positions.len() - 2) {
            Some(data) => data.world_position,
            None => Vec2::ZERO,
        }
    }
}

pub fn left_click_just_released(input: Res<InputData>) -> bool {
    input.left_just_released()
}

/// System that updates the InputData resource with the current cursor position.
pub fn update_cursor_info(
    time: Res<Time>,
    mut cursor_history: ResMut<InputData>,
    mouse: Res<ButtonInput<MouseButton>>,
    window: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform), (With<Camera2d>, With<GraphCamera>)>,
) {
    let (camera, camera_transform) = camera_q.single();

    let time = time.elapsed_seconds_f64();

    let window = window.single();

    let view_pos = match window.cursor_position() {
        Some(pos) => pos,
        None => return,
    };

    let world_position = camera
        .viewport_to_world(camera_transform, view_pos)
        .map(|ray| ray.origin.truncate())
        .unwrap();

    let data = SinglePositionData {
        viewport_position: view_pos,
        world_position,
        time,
    };

    cursor_history.insert_cursor_position(data);
}

/// System that updates InputData whenever a node is clicked. This system and its siblings
/// run in the PreUpdate stage.
pub fn handle_node_click(
    time: Res<Time>,

    mut event: EventReader<NodeClickEvent>,
    mut input_data: ResMut<InputData>,

    nodes: Query<(Entity, &ViewNode), Without<NodeOutline>>,
    outlines: Query<&Parent, (With<NodeOutline>, Without<ViewNode>)>,
) {
    if event.is_empty() {
        return;
    }

    for ev in event.read() {
        let interaction_type: InteractionType = InteractionType::Click(ev.button);
        let time = time.elapsed_seconds_f64();

        let target = ev.target.unwrap();

        // Determine the type of the picking target.
        // Currently this is done by checking if the target can be found in a
        // few mutually exclusive queries. Would be nice if there was a better way.
        let mut target_type = GraphPickingTarget::None;

        if nodes.get(target).is_ok() {
            target_type = GraphPickingTarget::Node;
        } else if outlines.get(target).is_ok() {
            target_type = GraphPickingTarget::NodeOutline;
        }

        let data = SingleInputData {
            target: Some(target),
            target_type,
            interaction_type,
            time,
        };

        input_data.insert_input(data);
    }
}

/// System that updates InputData whenever a node is pressed. This system and its siblings
/// run in the PreUpdate stage.
pub fn handle_node_press(
    time: Res<Time>,

    mut event: EventReader<NodePressedEvent>,
    mut input_data: ResMut<InputData>,

    nodes: Query<(Entity, &ViewNode), Without<NodeOutline>>,
    outlines: Query<&Parent, (With<NodeOutline>, Without<ViewNode>)>,
) {
    if event.is_empty() {
        return;
    }

    for ev in event.read() {
        let interaction_type: InteractionType = InteractionType::Press(ev.button);
        let time = time.elapsed_seconds_f64();

        let target = ev.target.unwrap();

        // Determine the type of the picking target.
        // Currently this is done by checking if the target can be found in a
        // few mutually exclusive queries. Would be nice if there was a better way.
        let mut target_type = GraphPickingTarget::None;

        if nodes.get(target).is_ok() {
            target_type = GraphPickingTarget::Node;
        } else if outlines.get(target).is_ok() {
            target_type = GraphPickingTarget::NodeOutline;
        }

        let data = SingleInputData {
            target: Some(target),
            target_type,
            interaction_type,
            time,
        };

        input_data.insert_input(data);
    }
}

/// System that updates InputData whenever a node is hovered over. This system and its siblings
/// run in the PreUpdate stage.
pub fn handle_node_hover(
    time: Res<Time>,

    mut event: EventReader<NodePressedEvent>,
    mut input_data: ResMut<InputData>,

    nodes: Query<(Entity, &ViewNode), Without<NodeOutline>>,
    outlines: Query<&Parent, (With<NodeOutline>, Without<ViewNode>)>,
) {
    if event.is_empty() {
        return;
    }

    for ev in event.read() {
        let interaction_type: InteractionType = InteractionType::Hover;
        let time = time.elapsed_seconds_f64();

        let target = ev.target.unwrap();

        // Determine the type of the picking target.
        // Currently this is done by checking if the target can be found in a
        // few mutually exclusive queries. Would be nice if there was a better way.
        let mut target_type = GraphPickingTarget::None;

        if nodes.get(target).is_ok() {
            target_type = GraphPickingTarget::Node;
        } else if outlines.get(target).is_ok() {
            target_type = GraphPickingTarget::NodeOutline;
        }

        let data = SingleInputData {
            target: Some(target),
            target_type,
            interaction_type,
            time,
        };

        input_data.insert_input(data);
    }
}

/// System that updates InputData whenever a node is no longer hovered over. This system and its siblings
/// run in the PreUpdate stage.
pub fn handle_node_hover_stop(
    time: Res<Time>,
    mut event: EventReader<NodeHoverStopEvent>,
    mut input_data: ResMut<InputData>,
) {
    if event.is_empty() {
        return;
    }

    for ev in event.read() {
        let interaction_type: InteractionType = InteractionType::HoverStop;
        let time = time.elapsed_seconds_f64();

        let data = SingleInputData {
            target: None,
            target_type: GraphPickingTarget::None,
            interaction_type,
            time,
        };

        input_data.insert_input(data);
    }
}
