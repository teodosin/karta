//

use bevy::prelude::{Plugin, PreStartup, Update, App, World, Resource, ResMut, Component};

pub(crate) mod move_actions;
pub(crate) mod node_actions;
pub(crate) mod edge_actions;


pub struct ActionPlugin;

impl Plugin for ActionPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(PreStartup, move_actions::setup_move_actions)
        
        .insert_resource(ActionManager::new())
            
        .add_systems(Update, execute_actions)
        ;
    }
}

// All undoable actions must implement this trait
pub trait Action: Send + Sync + 'static {
    fn execute(&mut self, world: &mut World);
    fn undo(&mut self, world: &mut World);
    fn redo(&mut self, world:  &mut World){
        self.execute(world);
    }
}

// A component that stores an action 
#[derive(Component)]
pub struct ActionComponent {
    pub action: ActionFactory,
}

pub type ActionFactory = Box<dyn Fn() -> Box<dyn Action> + Send + Sync>;


#[derive(Resource)]
pub struct ActionManager {
    queue: Vec<Box<dyn Action>>,
    undo_stack: Vec<Box<dyn Action>>,
    redo_stack: Vec<Box<dyn Action>>,
}

impl ActionManager {
    pub fn new() -> Self {
        ActionManager {
            queue: Vec::new(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    pub fn queue_action(&mut self, action: Box<dyn Action>) {
        self.queue.push(action);
    }

    fn perform_action(&mut self, mut action: Box<dyn Action>, world: &mut World) {
        action.execute(world);
        self.undo_stack.push(action);
        self.redo_stack.clear();
    }

    fn undo(&mut self, world: &mut World) {
        if let Some(mut action) = self.undo_stack.pop() {
            action.undo(world);
            self.redo_stack.push(action);
        }
    }

    fn redo(&mut self, world: &mut World) {
        if let Some(mut action) = self.redo_stack.pop() {
            action.redo(world);
            self.undo_stack.push(action);
        }
    }
}

fn execute_actions(
    world: &mut World,
){
    let mut manager = world.remove_resource::<ActionManager>().expect("ActionManager resource not found");

    let actions = std::mem::take(&mut manager.queue);

    for action in actions {
        manager.perform_action(action, world);
    }

    world.insert_resource(manager);
}