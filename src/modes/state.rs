// Here's an idea

// Combine the performance mode / animated traversal with the changing 
// of the context and the active node into one mode. They seem to fit
// well together, so it would remove some redundancy in modes. 

use bevy::prelude::*;

use crate::{
    graph::{
        nodes::NodeClickEvent, graph_cam, context::{
            CurrentContext, update_context
        }
    }, 
    vault::KartaVault
};

use super::KartaModeState;

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, change_context_path
                .before(update_context)
                .run_if(in_state(KartaModeState::State))
            )
        ;

    }
}

fn change_context_path(
    event: EventReader<NodeClickEvent>,
    input_data: Res<graph_cam::InputData>,
    vault: Res<KartaVault>,
    mut context: ResMut<CurrentContext>,
){
    // Only run the system if there has been a node input
    if event.is_empty(){
        return
    }

    let path: String = input_data.latest_click_entity.clone()
    .unwrap_or(context.get_current_context_path());

    if path == context.get_current_context_path() && path != vault.get_root_path(){
        println!("Already in context: {}", path);
        return
    }

    context.set_current_context(path.clone());

}