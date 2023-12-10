//

use bevy::prelude::*;

use crate::{
    graph::{context::{update_context, Selected}, nodes::GraphDataNode, graph_cam}, 
    events::nodes::MoveNodesEvent, input::pointer::InputData
};

use super::KartaModeState;

pub struct MovePlugin;

impl Plugin for MovePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, move_node_selection
                .before(update_context)
                .run_if(in_state(KartaModeState::Move))
            )
        ;

    }
}

pub fn move_node_selection(
    mut ev_mouse_drag: EventReader<MoveNodesEvent>,
    mouse: Res<Input<MouseButton>>,
    cursor: Res<InputData>,
    mut query: Query<(Entity, &GraphDataNode, &mut Transform), With<Selected>>,
    mut view_data: ResMut<graph_cam::ViewData>,
) {

    if mouse.just_pressed(MouseButton::Left) {
        for (_entity, _node, mut transform) in query.iter_mut() {
            transform.translation.z = 60.0;
            view_data.top_z += 1.0;
        }
    }

    for _ev in ev_mouse_drag.read() {
        if mouse.pressed(MouseButton::Left){
            for (_entity, _node, mut transform) in query.iter_mut() {
                    transform.translation.x += cursor.curr_position.x - cursor.prev_position.x;
                    transform.translation.y += cursor.curr_position.y - cursor.prev_position.y;     
            }
        }
        break
    }
}