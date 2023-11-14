// Here's an idea

// Combine the performance mode / animated traversal with the changing 
// of the context and the active node into one mode. They seem to fit
// well together, so it would remove some redundancy in modes. 

use std::path::PathBuf;

use bevy::prelude::*;
use bevy_mod_picking::prelude::PointerButton;

use crate::{
    graph::context::{
            CurrentContext, update_context
        }, 
    events::nodes::NodeClickEvent,
    vault::KartaVault, input::pointer::InputData
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
    mut event: EventReader<NodeClickEvent>,
    input_data: Res<InputData>,
    vault: Res<KartaVault>,
    mut context: ResMut<CurrentContext>,
){
    // Only run the system if there has been a node input
    if event.is_empty(){
        return
    }

    if event.read().next().unwrap().button != PointerButton::Primary {
        return
    }

    let path: PathBuf = input_data.latest_click_entity.clone()
    .unwrap_or(context.get_current_context_path());

    if path == context.get_current_context_path() && path != vault.get_root_path(){
        println!("Already in context: {}", path.display());
        return
    }

    println!("Changing context to: {}", path.display());
    context.set_current_context(path.clone());

}