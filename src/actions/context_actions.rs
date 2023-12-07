use std::path::PathBuf;

use crate::graph::context::CurrentContext;

use super::Action;

#[derive(Clone)]
pub struct MoveToContextAction {
    previous: Option<PathBuf>,
    next: PathBuf,
}

impl Action for MoveToContextAction {
    fn execute(&mut self, world: &mut bevy::prelude::World) {
        
        let mut current_context = world.get_resource_mut::<CurrentContext>().unwrap();

        match current_context.cxt.clone() {
            Some(cxt) => {
                self.previous = Some(cxt.current_context);
                self.next = self.next.clone();
            },
            None => {
                println!("No current context");
                self.previous = None;
                self.next = self.next.clone();
            }
        };

        current_context.set_current_context(self.next.clone());

        println!("Performing MoveToContextAction");
    }

    fn undo(&mut self, world: &mut bevy::prelude::World) {
        let mut current_context = world.get_resource_mut::<CurrentContext>().unwrap();
        match self.previous.clone() {
            Some(previous) => {
                current_context.set_current_context(previous);
                println!("Undoing MoveToContextAction");
            },
            None => {
                println!("No previous context");
            }
        }
    }

    fn redo(&mut self, world:  &mut bevy::prelude::World) {
        let mut current_context = world.get_resource_mut::<CurrentContext>().unwrap();
        current_context.set_current_context(self.next.clone());
        println!("Redoing MoveToContextAction");
    }
}

impl MoveToContextAction {
    pub fn new(next: PathBuf) -> Self {
        MoveToContextAction {
            previous: None,
            next,
        }
    }

}