// This file is not yet complete. It's just a sketch of what I'm thinking of. It will be split into multiple files later, most likely. 





/// For the ECS, it would be incredibly useful to have a spreadsheet view of the entities
/// and their components.
/// 
/// So what if there was a smooth transition between the spreadsheet view and the graph view?
/// To do this, we would need to have a way to store the position of the entity in the graph view.
#[derive(Component)]
pub struct GraphPosition {
    pub x: f32,
    pub y: f32,
}

impl GraphPosition {
    pub fn new (x: f32, y: f32) -> Self {
        GraphPosition {
            x,
            y,
        }
    }

    pub fn from_vec (vec: Vec2) -> Self {
        GraphPosition {
            x: vec.x,
            y: vec.y,
        }
    }

    pub fn to_vec (&self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }

    pub fn set_vec (&mut self, vec: Vec2) {
        self.x = vec.x;
        self.y = vec.y;
    }
}

// There needs to be an ergonomic API for users to define their own actions for specific entities. 
// More precisely, for specific components and component combinations. 
// Maybe queries could be used to define the conditions for an action to be available? 

/// This is a sketch for a component that stores all the possible actions for a given entity. To get 
/// actions for the entity's context menu, for example. 
/// 
/// The idea is that each GraphEntity would get this component added to it by bevy_karta_ui.
/// Users wouldn't manage it themselves, necessarily. There could be a change-detecting system or something.
#[derive(Component)]
pub struct ContextActions {
    pub actions: Vec<Box<dyn Action>>,
}

/// Alternatively, the custom action trait could be abandoned in favor of Bevy's existing command system.
/// Bevy already has a command queue and exclusive ways to execute commands in a deterministic order. 
#[derive(Component)]
pub struct ContextCommands {
    pub commands: Vec<Box<dyn Command>>,
}

/// A resource storing the context menu commands in an application.
/// 
/// EntityCommand is a trait that doesn't have much documentation yet.
/// I didn't know it existed before I talked with "James." on the Bevy Discord server.
/// https://docs.rs/bevy/latest/bevy/ecs/system/trait.EntityCommand.html
#[derive(Resource)]
pub struct ContextCommands {
    map: HashMap<TypeId, Vec<Box<dyn EntityCommand>>>,
}

impl ContextCommands {
    /// Create a new ContextCommands resource with no commands.
    fn new() -> Self {
        ContextCommands {
            map: HashMap::new(),
        }
    }

    /// Add a command to the ContextCommands resource using the builder pattern. 
    /// Mostly used by the crate itself for initialising some default commands. 
    fn with<T: Component, C: EntityCommand + 'static>(mut self, command: C) -> Self {
        self.insert::<T, C>(command);
        self
    }

    /// Add a command to the ContextCommands resource. This command will be available in
    /// the context menu for all entities with the given component.
    fn insert<T: Component, C: EntityCommand + 'static>(&mut self, command: C) {
        let type_id = TypeId::of::<T>();
        self.map.entry(type_id).or_insert(Vec::new()).push(Box::new(command));
    }

    /// Get the commands for a given component type. Mostly used by the crate itself
    /// to construct the context menu. 
    fn get<T: Component>(&self) -> Option<&Vec<Box<dyn EntityCommand>>> {
        let type_id = TypeId::of::<T>();
        self.map.get(&type_id)
    }
}

/// A bare sketch for a new Action trait. 
pub trait Action {
    fn execute (
        &self, 
        /// Actions are executed in an exclusive system. 
        /// 
        world: &mut World,
        /// An action could specify an entity to act on, or not. 
        /// There is flexibility to accomodate both.
        entity: Option<Entity>, 
    );
}

/// This is the Action trait as it is implemented in Karta. 
/// As you can see, it was built with undo/redo in mind. 
/// That brings up an important question that needs answering:
/// Should bevy_karta_ui have an undo/redo system? Or should that 
/// be left to the user to implement?
/// 
/// Considering that bevy_karta_ui is an editor candidate similar to 
/// bevy_editor_pls, it would be nice to have an undo/redo system built in.
/// People building games don't necessarily need an undo/redo system, so it 
/// makes sense to provide one. 
pub trait Action: Send + Sync + 'static {
    fn execute(&mut self, world: &mut World);
    fn undo(&mut self, world: &mut World);
    fn redo(&mut self, world:  &mut World){
        self.execute(world);
    }
}

/// What if we could use Bevy's command system to implement undo/redo?
/// What if an Action is a struct that bundles Commands together?
/// Is that silly? What are the benefits?
/// 
/// I wonder if it would be possible to generate this using a macro on a Command. 
/// It feels like Command is so close to being an Action already.  
pub struct Action {
    applier: Command,
    undoer: Command,
    redoer: Command,
}

fn execute_action (
    mut commands: Commands,
){

}