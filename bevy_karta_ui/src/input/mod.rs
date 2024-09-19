use bevy::prelude::{Plugin, PreUpdate, App};

use self::pointer::{handle_node_click, handle_node_hover, handle_node_press, update_cursor_info, InputData};

pub mod pointer;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(InputData::default())
            
            .add_systems(PreUpdate, (
                update_cursor_info,
                handle_node_click,
                handle_node_press,
                handle_node_hover,
            ))

            
            // Add the update when the ui for input map editing is in place.
            // No point before that. 
            //.add_systems(PreUpdate, input_map::update_input_map)
        ;
    }
}