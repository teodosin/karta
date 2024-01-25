use std::any::TypeId;

use bevy::{app::{App, Plugin}, ecs::{component::Component, entity::Entity, system::{EntityCommand, Resource}, world::World}, transform::components::Transform, utils::HashMap};

pub struct ContextCommandsPlugin;

impl Plugin for ContextCommandsPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(ComponentCommands::new()
                .with::<Transform, DeleteEntityCommand>(DeleteEntityCommand, "Delete Entity".to_string()))
        ;
    }
}

pub struct CustomCommand {
    name: String,
    command: Box<dyn EntityCommand + Sync>,
}

impl CustomCommand {
    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn command(&self) -> &Box<dyn EntityCommand + Sync> {
        &self.command
    }
}

/// A resource storing the component-specific commands in an application. 
/// These may be used in the construction of the context menu. 
/// 
/// EntityCommand is a trait that doesn't have much documentation yet.
/// https://docs.rs/bevy/latest/bevy/ecs/system/trait.EntityCommand.html
#[derive(Resource)]
pub struct ComponentCommands {
    map: HashMap<TypeId, Vec<CustomCommand>>,
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
    pub fn with<T: Component, C: EntityCommand + Sync + 'static>(mut self, command: C, name: String) -> Self {
        self.insert::<T, C>(command, name);
        self
    }

    /// Add a command to the ContextCommands resource. This command will be available in
    /// the context menu for all entities with the given component.
    pub fn insert<T: Component, C: EntityCommand + Sync + 'static>(&mut self, command: C, name: String) {
        let type_id = TypeId::of::<T>();
        self.map.entry(type_id).or_insert(Vec::new()).push(CustomCommand {
            name,
            command: Box::new(command),
        });
    }

    /// Get the commands for a given component type. 
    pub fn get<T: Component>(&self) -> Option<&Vec<CustomCommand>> {
        let type_id = TypeId::of::<T>();
        self.map.get(&type_id)
    }
    
    /// Alternative getter that uses the component type id directly.
    /// Mostly used by the crate itself to construct the context menu. 
    pub fn get_by_id(&self, type_id: TypeId) -> Option<&Vec<CustomCommand>> {
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