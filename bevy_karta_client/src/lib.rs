
/// Core plugin. Assembles all of the other plugins.
pub mod core_plugin;

mod vault_plugin;

mod context_plugin;

mod node_plugin;

pub mod prelude {
    pub use crate::core_plugin::*;
    pub use crate::vault_plugin::*;
    pub use crate::context_plugin::*;
    pub use crate::node_plugin::*;
}