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
