//

use bevy::{prelude::{EventReader, ResMut, Res, Resource, KeyCode}, input::keyboard::KeyboardInput, utils::HashMap, time::Time};

use crate::actions::{Action, ActionManager, ActionFactory, node_actions::CreateNodeAction};

use super::pointer::InputData;

pub fn setup_input_map(){
    println!("Setting up input map");
}

#[derive(Resource)]
pub struct KeyMap {
    key_action_pairs: HashMap<KeyCode, ActionFactory>,
}

impl KeyMap {
    pub fn add_key_action_pair(&mut self, key: KeyCode, action: ActionFactory) {
        self.key_action_pairs.insert(key, action);
    }
}

impl Default for KeyMap {
    fn default() -> Self {

        let mut map = KeyMap {
            key_action_pairs: HashMap::default(),
        };

        map.add_key_action_pair(KeyCode::N, Box::new(|| Box::new(CreateNodeAction::default())));

        map
    }
}

pub fn handle_key_input(
    mut event: EventReader<KeyboardInput>,
    mut manager: ResMut<ActionManager>,
    keymap: Res<KeyMap>,
    time: Res<Time>,
){
    if event.is_empty() {
        return
    }

    let ev = event.iter().next().unwrap();

    if ev.state.is_pressed() {
        return
    }

    match ev.key_code {
        Some(key) => {
            println!("Key: {:?}", key);

            match keymap.key_action_pairs.get(&key) {
                Some(factory) => {
                    let action = factory();
                    manager.queue_action(action);
                    println!("{}", time.elapsed().as_millis())
                },
                None => {
                    println!("No action");
                }
            }
        },
        None => {
            println!("No key");
        }
    }
}


