use bevy::prelude::{Plugin, PreUpdate, App, PostUpdate};

use self::pointer::{InputData, update_cursor_info};

pub mod keymap;
pub mod pointer;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(InputData::default())
            .insert_resource(keymap::KeyMap::default())
            
            .add_systems(PostUpdate, update_cursor_info)
            .add_systems(PreUpdate, keymap::handle_key_input)

            
            // Add the update when the ui for input map editing is in place.
            // No point before that. 
            //.add_systems(PreUpdate, input_map::update_input_map)
        ;
    }
}