use std::any::TypeId;

use bevy::{
    app::{App, Plugin, PreStartup},
    ecs::{
        component::Component,
        system::{Commands, Res, ResMut, Resource, SystemId, SystemState},
        world::World,
    },
    hierarchy::DespawnRecursiveExt,
    utils::HashMap,
};

use crate::prelude::pointer::InputData;

pub struct ContextCommandsPlugin;

impl Plugin for ContextCommandsPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(ContextComponentSystems::new())
            .insert_resource(ContextEntitySystems::new())
            .add_systems(PreStartup, register_default_systems)
        ;
    }
}

#[derive(Clone)]
pub struct ContextSystem {
    name: String,
    command: SystemId,
}

impl ContextSystem {
    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn command(&self) -> SystemId {
        self.command
    }
}

/// A resource storing SystemId's for the context menu. Systems registered here will be
/// available in the context menu for all entities, regardless of components.
#[derive(Resource)]
pub struct ContextEntitySystems {
    list: Vec<ContextSystem>,
}

impl ContextEntitySystems {
    fn new() -> Self {
        ContextEntitySystems { list: Vec::new() }
    }

    pub fn insert(&mut self, name: String, command: SystemId) {
        let command = ContextSystem { name, command };
        self.list.push(command);
    }

    pub fn get(&self) -> Vec<ContextSystem> {
        self.list.clone()
    }
}

/// A resource storing the component-specific SystemId's in an application.
/// These may be used in the construction of the context menu.
#[derive(Resource)]
pub struct ContextComponentSystems {
    map: HashMap<TypeId, Vec<ContextSystem>>,
}

impl ContextComponentSystems {
    /// Create a new ContextCommands resource with no commands.
    fn new() -> Self {
        ContextComponentSystems {
            map: HashMap::new(),
        }
    }

    /// Add a command to the ContextCommands resource. This command will be available in
    /// the context menu for all entities with the given component.
    pub fn insert<T: Component>(&mut self, name: String, command: SystemId) {
        let type_id = TypeId::of::<T>();
        let command = ContextSystem { name, command };
        if let Some(commands) = self.map.get_mut(&type_id) {
            commands.push(command);
        } else {
            self.map.insert(type_id, vec![command]);
        }
    }

    /// Get the commands for a given component type.
    pub fn get<T: Component>(&self) -> Option<Vec<ContextSystem>> {
        let type_id = TypeId::of::<T>();
        let sys_id = self.map.get(&type_id);
        sys_id.cloned()
    }

    /// Alternative getter that uses the component type id directly.
    /// Mostly used by the crate itself to construct the context menu.
    pub fn get_by_id(&self, type_id: TypeId) -> Option<Vec<ContextSystem>> {
        self.map.get(&type_id).cloned()
    }
}

/// A generic system for deleting an entity.
fn delete_entity(mut commands: Commands, input_data: Res<InputData>) {
    let target = input_data.latest_click_entity.unwrap();
    commands.entity(target).despawn_recursive();
}

fn register_default_systems(mut world: &mut World) {
    let mut system_state: SystemState<(
        ResMut<ContextEntitySystems>,
        ResMut<ContextComponentSystems>,
    )> = SystemState::new(&mut world);

    
    let del = world.register_system(delete_entity);
    let (mut entity_systems, component_systems) = system_state.get_mut(&mut world);

    entity_systems.insert("Delete".to_string(), del);
}
