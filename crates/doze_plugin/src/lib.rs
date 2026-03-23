#[forbid(unsafe_code)]
pub mod events;
pub mod extensions;
pub mod factory;
pub mod features;
pub mod plugin;
pub mod process;

use std::path::Path;

use crate::{extensions::Bridge, factory::plugin::PluginFactoryBuilder};

pub mod prelude {
    pub use crate::{
        Entry, PluginApi,
        events::{
            Event, EventFlags, HostEvent, HostParamEvent, NoteContext, NoteExpression, PluginEvent,
            PluginParamEvent,
        },
        extensions::{audio_ports::*, params::*},
        factory::plugin::{PluginBuilder, PluginFactoryBuilder},
        features::PluginFeature,
        plugin::{Plugin, descriptor::PluginDescriptor},
        process::{Process, Status},
    };

    pub use std::path::Path;
}

pub trait PluginApi {
    type Extension: Bridge + Clone;
}

pub trait Entry<A: PluginApi> {
    fn init(path: Option<&Path>) -> Option<PluginFactoryBuilder<A>>;
    fn deinit();
}
