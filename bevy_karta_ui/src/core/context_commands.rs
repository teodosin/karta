use std::any::TypeId;

use bevy::{app::{App, Plugin}, ecs::{component::Component, entity::Entity, system::{EntityCommand, Resource}, world::World}, transform::components::Transform, utils::HashMap};

pub struct ContextCommandsPlugin;

impl Plugin for ContextCommandsPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(ComponentCommands::new()
                .with::<Transform, DeleteEntityCommand>(DeleteEntityCommand))
        ;
    }
}

/// A resource storing the component-specific commands in an application. 
/// These may be used in the construction of the context menu. 
/// 
/// EntityCommand is a trait that doesn't have much documentation yet.
/// https://docs.rs/bevy/latest/bevy/ecs/system/trait.EntityCommand.html
#[derive(Resource)]
pub struct ComponentCommands {
    map: HashMap<TypeId, Vec<Box<dyn EntityCommand + Sync>>>,
}

impl ComponentCommands {
    /// Create a new ContextCommands resource with no commands.
    fn new() -> Self {
        ComponentCommands {
            map: HashMap::new(),
        }
    }

    /// Add a command to the ContextCommands resource using the builder pattern. 
    /// Mostly used by the crate itself for initialising some default commands. 
    fn with<T: Component, C: EntityCommand + Sync + 'static>(mut self, command: C) -> Self {
        self.insert::<T, C>(command);
        self
    }

    /// Add a command to the ContextCommands resource. This command will be available in
    /// the context menu for all entities with the given component.
    fn insert<T: Component, C: EntityCommand + Sync + 'static>(&mut self, command: C) {
        let type_id = TypeId::of::<T>();
        self.map.entry(type_id).or_insert(Vec::new()).push(Box::new(command));
    }

    /// Get the commands for a given component type. Mostly used by the crate itself
    /// to construct the context menu. 
    fn get<T: Component>(&self) -> Option<&Vec<Box<dyn EntityCommand + Sync>>> {
        let type_id = TypeId::of::<T>();
        self.map.get(&type_id)
    }
}

/// A generic command for deleting an entity.
/// 
/// Temporarily bound to the Transform component for testing. 
/// Since this action could be applied to any entity regardless of components, 
/// perhaps there should be a generic EntityCommands resource too, where users could
/// add commands that apply to 
struct DeleteEntityCommand;

impl EntityCommand for DeleteEntityCommand {
    fn apply(self, entity: Entity, world: &mut World) {
        world.despawn(entity);
    }
}