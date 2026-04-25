#[forbid(unsafe_code)]
use std::path::Path;

use crate::{extensions::Bridge, factory::plugin::PluginFactoryBuilder};

pub mod events;
pub mod extensions;
pub mod factory;
pub mod plugin;
pub mod process;

pub mod prelude {
    pub use crate::{
        Entry, PluginApi,
        events::{
            Event, EventFlags, HostEvent, HostParamEvent, NoteContext, NoteExpression, PluginEvent,
            PluginParamEvent,
        },
        extensions::{audio_ports::*, params::*},
        factory::plugin::{PluginBuilder, PluginFactoryBuilder},
        plugin::{Plugin, PluginDescriptor, PluginFeature},
        process::{AudioBuffer, Process, Status},
    };

    pub use std::path::Path;
}

/// Trait defining the plugin API for a given audio plugin standard (e.g., CLAP).
///
/// Implement this to create an adapter for a new plugin format.
/// The associated type specifies which extensions are available in that standard.
pub trait PluginApi {
    /// The extension bridge type for this plugin standard.
    type Extension: Bridge + Clone;
}

/// Entry point for plugin discovery and initialization.
///
/// Implement this trait in your plugin binary and use the `export!` macro
/// to expose your plugin to the host. The host will call `init()` when loading the plugin,
/// and `deinit()` when unloading.
pub trait Entry<A: PluginApi> {
    /// Initialize the plugin factory (called once on library load).
    ///
    /// # Arguments
    /// - `path`: Optional path to the plugin library file
    ///
    /// # Returns
    /// - `Some(factory)`: Plugin factory ready for instantiation
    /// - `None`: Plugin initialization failed
    fn init(path: Option<&Path>) -> Option<PluginFactoryBuilder<A>>;

    /// Deinitialize the plugin factory (called on library unload).
    fn deinit() {}
}
