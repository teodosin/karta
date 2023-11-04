//

use super::Action;

// ------------------ CreateNodeAction ------------------

#[derive(Clone)]
pub struct CreateNodeAction {
    node: String,
}

impl Action for CreateNodeAction {
    fn execute(&mut self, world: &mut bevy::prelude::World) {
        println!("Performing CreateNodeAction");
    }

    fn undo(&mut self, world: &mut bevy::prelude::World) {
        println!("Undoing CreateNodeAction");
    }

    fn redo(&mut self, world:  &mut bevy::prelude::World) {
        println!("Redoing CreateNodeAction");
    }
}

impl Default for CreateNodeAction {
    fn default() -> Self {
        CreateNodeAction {
            node: String::from("Default Node"),
        }
    }
}

impl CreateNodeAction {
    pub fn new(node: String) -> Self {
        CreateNodeAction {
            node,
        }
    }
}

// ------------------ DeleteNodeAction ------------------

// ------------------ EditNodeAction ------------------

// ------------------ PinToPositionAction ------------------

// ------------------ UnpinToPositionAction ------------------

// ------------------ PinToPresenceAction ------------------

// ------------------ UnpinToPresenceAction ------------------