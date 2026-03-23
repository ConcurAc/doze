use std::{
    borrow::Borrow,
    ffi::{CStr, c_char, c_void},
    ptr::null,
};

use clap_sys::{factory::plugin_factory::clap_plugin_factory, host::*, plugin::*};

use parking_lot::Mutex;

use doze_common::identifier::WeakIdentifier;
use doze_plugin::{
    factory::plugin::{PluginDefinition, PluginFactory, PluginFactoryBuilder},
    plugin::{Plugin, descriptor::PluginDescriptor},
};

use crate::{
    Clap,
    plugin::{ClapPluginExtern, ClapPluginWrapper, descriptor::ClapPluginDescriptor},
};

pub const CLAP_PLUGIN_FACTORY_VTABLE: clap_plugin_factory = clap_plugin_factory {
    get_plugin_count: Some(get_plugin_count),
    get_plugin_descriptor: Some(get_plugin_descriptor),
    create_plugin: Some(create_plugin),
};

pub static CLAP_PLUGIN_FACTORY: Mutex<Option<ClapPluginFactory>> = Mutex::new(None);

#[repr(C)]
pub struct ClapPluginFactory {
    vtable: clap_plugin_factory,
    definitions: Vec<ClapPluginDefinition>,
}

struct ClapPluginDefinition {
    descriptor: ClapPluginDescriptor,
    definition: PluginDefinition<Clap>,
}

impl PluginFactory for ClapPluginFactory {
    fn descriptor_count(&self) -> usize {
        self.definitions.iter().len()
    }
    fn get_descriptor(&self, index: usize) -> Option<impl Borrow<PluginDescriptor>> {
        self.definitions
            .get(index)
            .map(|d| &d.definition.descriptor)
    }
    fn create_plugin(&self, identifier: WeakIdentifier) -> Option<Box<dyn Plugin>> {
        let definition = self
            .definitions
            .iter()
            .find(|d| d.definition.descriptor.id.downgrade() == identifier)?;

        Some((definition.definition.creator)())
    }
}

impl From<PluginFactoryBuilder<Clap>> for ClapPluginFactory {
    fn from(builder: PluginFactoryBuilder<Clap>) -> Self {
        ClapPluginFactory {
            vtable: CLAP_PLUGIN_FACTORY_VTABLE,
            definitions: builder
                .into_iter()
                .map(|definition| ClapPluginDefinition {
                    descriptor: definition.descriptor.clone().into(),
                    definition,
                })
                .collect(),
        }
    }
}

pub unsafe extern "C" fn get_plugin_count(factory: *const clap_plugin_factory) -> u32 {
    let factory = unsafe {
        match (factory as *const ClapPluginFactory).as_ref() {
            Some(f) => f,
            None => return 0,
        }
    };

    factory.descriptor_count() as u32
}

pub unsafe extern "C" fn get_plugin_descriptor(
    factory: *const clap_plugin_factory,
    index: u32,
) -> *const clap_plugin_descriptor {
    let factory = unsafe {
        match (factory as *const ClapPluginFactory).as_ref() {
            Some(f) => f,
            None => return null(),
        }
    };

    factory
        .definitions
        .get(index as usize)
        .map_or_else(null, |d| {
            d.descriptor.get() as *const clap_plugin_descriptor
        })
}

pub unsafe extern "C" fn create_plugin(
    factory: *const clap_plugin_factory,
    _host: *const clap_host,
    plugin_id: *const c_char,
) -> *const clap_plugin {
    if plugin_id.is_null() {
        return null();
    }

    let id = unsafe { CStr::from_ptr(plugin_id) };
    let factory = unsafe {
        match (factory as *const ClapPluginFactory).as_ref() {
            Some(f) => f,
            None => return null(),
        }
    };

    factory
        .create_plugin(id.into())
        .map(|p| {
            factory
                .definitions
                .iter()
                .find(|&d| d.definition.descriptor.id.as_ref() == id)
                .map(|d| {
                    let wrapper = ClapPluginWrapper::new(p, &d.definition);
                    let plugin = clap_plugin {
                        init: Some(ClapPluginExtern::init),
                        activate: Some(ClapPluginExtern::activate),
                        deactivate: Some(ClapPluginExtern::deactivate),
                        reset: Some(ClapPluginExtern::reset),
                        destroy: Some(ClapPluginExtern::destroy),
                        start_processing: Some(ClapPluginExtern::start_processing),
                        stop_processing: Some(ClapPluginExtern::stop_processing),
                        process: Some(ClapPluginExtern::process),
                        on_main_thread: Some(ClapPluginExtern::on_main_thread),
                        get_extension: Some(ClapPluginExtern::get_extension),
                        desc: d.descriptor.get() as *const _,
                        plugin_data: Box::into_raw(Box::new(wrapper)) as *mut c_void,
                    };
                    Box::into_raw(Box::new(plugin)) as *const clap_plugin
                })
        })
        .flatten()
        .unwrap_or_else(null)
}
