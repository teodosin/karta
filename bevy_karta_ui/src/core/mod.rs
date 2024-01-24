// This module defines resources and components that users of the library would use in their own projects. 
// That's what I'm thinking of, anyway.

use bevy::{app::{Plugin, PreStartup, App}, core::Name, ecs::{component::Component, world::World}};

mod context_commands;

pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app
            // Disabled for now, because there are a ridiculous amount of components already. 
            // .add_systems(PreStartup, spawn_components)  
        ;
    }
}

/// A marker component.
/// Entities with this component will have a graph node created and managed for them. 
#[derive(Component)]
pub struct GraphEntity;

/// A component that stores the pin state of a graph node. 
#[derive(Component, Default)]
pub struct Pins {
    /// If true, the node will not be moved by the graph simulation.
    pub position: bool,
    /// If true, the node will not be despawned even if its corresponding entity is despawned.
    pub presence: bool,
    /// If true, the node will exist on the ui layer. 
    pub ui: bool,
}

impl Pins {
    pub fn new_pinpos () -> Self {
        Pins {
            position: true,
            presence: false,
            ui: false,
        }
    }
    pub fn pinpres () -> Self {
        Pins {
            position: false,
            presence: true,
            ui: false,
        }
    }
    pub fn pinui () -> Self {
        Pins {
            position: false,
            presence: false,
            ui: true,
        }
    }
}

/// Function that spawns all component types as entities to be visualised
/// in the graph. 
/// 
/// Currently does nothing because there doesn't seem to be 
/// a way to get a list of components from the World, but that will probably
/// be trivial once components are entities. 
fn spawn_components(
    world: &mut World,
){
    let components = world.components();
    let comps = components.iter().map(|c| c.name().to_owned()).collect::<Vec<String>>();

    for component in comps.iter() {
        world.spawn((
            GraphEntity,
            Name::new(component.clone()),
        ));
    }
}