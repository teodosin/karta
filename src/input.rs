use bevy::prelude::{Plugin, PreStartup, PreUpdate, App, PostUpdate};

use self::pointer::{InputData, update_cursor_info};

pub mod input_map;
pub mod pointer;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(PreStartup, input_map::setup_input_map)

            .insert_resource(InputData::default())
            
            .add_systems(PostUpdate, update_cursor_info)

            
            // Add the update when the ui for input map editing is in place.
            // No point before that. 
            //.add_systems(PreUpdate, input_map::update_input_map)
        ;
    }
}