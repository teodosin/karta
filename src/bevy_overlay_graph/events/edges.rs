//

use bevy::prelude::{Event, Entity};
use bevy_mod_picking::{pointer::PointerButton, prelude::ListenerInput, events::{Pointer, Click}};

#[derive(Event)]
pub struct EdgeClickEvent {
    pub target: Option<Entity>,
    pub button: PointerButton,
}

// Implementation required by bevy_mod_picking
impl From<ListenerInput<Pointer<Click>>> for EdgeClickEvent {
    fn from(event: ListenerInput<Pointer<Click>>) -> Self {
        EdgeClickEvent {
            target: Some(event.target),
            button: event.button
        }
    }
}