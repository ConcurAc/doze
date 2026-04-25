use std::{
    ffi::{CStr, c_char, c_void},
    ptr::null,
};

use clap_sys::{
    plugin::clap_plugin,
    process::{CLAP_PROCESS_ERROR, clap_process, clap_process_status},
};

use doze_common::{collections::HashMap, identifier::WeakIdentifier, storage::alloc::History};
use doze_plugin::{
    events::{Event, TransportEvent},
    factory::plugin::PluginDefinition,
    plugin::{Entity, EntityRegistry, ExtensionRegistry, Plugin},
};

use crate::{
    Clap, ClapId,
    events::{
        ClapEvent, ClapEventSender, iter::ClapHostEventIter,
        transport::clap_transport_to_host_event,
    },
    process::{ClapProcess, status_to_clap},
};

pub mod descriptor;
pub mod feature;

pub struct ClapPluginWrapper {
    pub plugin: Box<dyn Plugin>,
    pub extensions: ExtensionRegistry<Clap>,
    pub entities: EntityRegistry<ClapId>,

    pub sent_events: History<ClapEvent>,
    pub transport: Option<Event<TransportEvent>>,

    vtables: HashMap<WeakIdentifier<'static>, *const c_void>,
}

impl ClapPluginWrapper {
    pub fn new(plugin: Box<dyn Plugin>, definition: &PluginDefinition<Clap>) -> Self {
        let extensions = definition.extensions.iter().cloned().collect();

        let entities = definition
            .extensions
            .iter()
            .filter_map(|e| e.extension.as_registry_source())
            .map(|c| c.identifiers(plugin.as_ref()).into_iter().enumerate())
            .flatten()
            .map(|(index, identifier)| {
                (identifier.downgrade().into(), Entity { index, identifier })
            })
            .collect();

        let vtables = definition
            .extensions
            .iter()
            .map(|c| c.payload.iter())
            .flatten()
            .map(|c| (c.id, c.vtable))
            .collect();

        Self {
            plugin,
            extensions,
            entities,

            sent_events: History::with_capacity(definition.context.event_capacity),
            transport: None,

            vtables,
        }
    }

    pub unsafe fn from_raw(plugin: *const clap_plugin) -> Option<&'static Self> {
        unsafe { (plugin.as_ref()?.plugin_data as *const Self).as_ref() }
    }

    pub unsafe fn from_raw_mut(plugin: *const clap_plugin) -> Option<&'static mut Self> {
        unsafe { (plugin.as_ref()?.plugin_data as *mut Self).as_mut() }
    }
}

pub struct ClapPluginExtern;

impl ClapPluginExtern {
    pub unsafe extern "C" fn init(clap_plugin: *const clap_plugin) -> bool {
        let Some(wrapper) = (unsafe { ClapPluginWrapper::from_raw_mut(clap_plugin) }) else {
            return false;
        };

        wrapper.plugin.init();

        return true;
    }

    pub unsafe extern "C" fn destroy(clap_plugin: *const clap_plugin) {
        if clap_plugin.is_null() {
            return;
        }

        let clap_plugin = unsafe { Box::from_raw(clap_plugin as *mut clap_plugin) };

        if clap_plugin.plugin_data.is_null() {
            return;
        }

        drop(unsafe { Box::from_raw(clap_plugin.plugin_data as *mut ClapPluginWrapper) });
    }

    pub unsafe extern "C" fn activate(
        clap_plugin: *const clap_plugin,
        sample_rate: f64,
        min_frames_count: u32,
        max_frames_count: u32,
    ) -> bool {
        let Some(wrapper) = (unsafe { ClapPluginWrapper::from_raw_mut(clap_plugin) }) else {
            return false;
        };

        wrapper
            .plugin
            .activate(sample_rate, min_frames_count, max_frames_count)
    }

    pub unsafe extern "C" fn deactivate(clap_plugin: *const clap_plugin) {
        let Some(wrapper) = (unsafe { ClapPluginWrapper::from_raw_mut(clap_plugin) }) else {
            return;
        };

        wrapper.plugin.deactivate();
    }

    pub unsafe extern "C" fn start_processing(clap_plugin: *const clap_plugin) -> bool {
        let Some(wrapper) = (unsafe { ClapPluginWrapper::from_raw_mut(clap_plugin) }) else {
            return false;
        };

        wrapper.plugin.start_processing()
    }

    pub unsafe extern "C" fn stop_processing(clap_plugin: *const clap_plugin) {
        let Some(wrapper) = (unsafe { ClapPluginWrapper::from_raw_mut(clap_plugin) }) else {
            return;
        };

        wrapper.plugin.stop_processing();
    }

    pub unsafe extern "C" fn reset(clap_plugin: *const clap_plugin) {
        let Some(wrapper) = (unsafe { ClapPluginWrapper::from_raw_mut(clap_plugin) }) else {
            return;
        };

        wrapper.plugin.reset();
    }

    pub unsafe extern "C" fn process(
        clap_plugin: *const clap_plugin,
        clap_process: *const clap_process,
    ) -> clap_process_status {
        let Some(wrapper) = (unsafe { ClapPluginWrapper::from_raw_mut(clap_plugin) }) else {
            return CLAP_PROCESS_ERROR;
        };

        wrapper.sent_events.clear();

        let Some(clap_process) = (unsafe { clap_process.as_ref() }) else {
            return CLAP_PROCESS_ERROR;
        };

        let Some(events) = (unsafe {
            clap_process
                .in_events
                .as_ref()
                .and_then(|i| ClapHostEventIter::new(i, &wrapper.entities))
        }) else {
            return CLAP_PROCESS_ERROR;
        };

        let Some(sender) = (unsafe {
            clap_process
                .out_events
                .as_ref()
                .and_then(|o| ClapEventSender::new(o, &mut wrapper.sent_events))
        }) else {
            return CLAP_PROCESS_ERROR;
        };

        let transport =
            unsafe { clap_process.transport.as_ref() }.map(clap_transport_to_host_event);

        let mut iter = transport.into_iter().chain(events);

        let Some(mut process) = ClapProcess::new(clap_process, &mut iter, sender) else {
            return CLAP_PROCESS_ERROR;
        };

        status_to_clap(wrapper.plugin.process((&mut process).into()))
    }

    pub unsafe extern "C" fn get_extension(
        clap_plugin: *const clap_plugin,
        id: *const c_char,
    ) -> *const c_void {
        let Some(wrapper) = (unsafe { ClapPluginWrapper::from_raw(clap_plugin) }) else {
            return null();
        };

        if id.is_null() {
            return null();
        }
        let id = unsafe { CStr::from_ptr(id) };

        wrapper
            .vtables
            .get(&WeakIdentifier::from(id))
            .map(|&r| r as *const _ as *const c_void)
            .unwrap_or_else(null)
    }

    pub unsafe extern "C" fn on_main_thread(clap_plugin: *const clap_plugin) {
        let Some(wrapper) = (unsafe { ClapPluginWrapper::from_raw_mut(clap_plugin) }) else {
            return;
        };

        wrapper.plugin.on_main_thread();
    }
}
