#![feature(min_specialization)]

use doze_common::identifier::IdentifierHash;
use doze_plugin::PluginApi;

use crate::extensions::ClapBridge;

pub use doze_plugin;

pub use clap_sys;

pub mod events;
pub mod extensions;
pub mod factory;
pub mod features;
pub mod plugin;
pub mod process;

#[derive(Clone)]
pub struct Clap;

impl PluginApi for Clap {
    type Extension = ClapBridge;
}

pub type ClapId = IdentifierHash<u32>;

#[macro_export]
macro_rules! export {
    ($entry:ty) => {
        mod clap {
            use super::*;

            use $crate::{
                Clap,
                factory::{get_factory, plugin::CLAP_PLUGIN_FACTORY},
            };

            use $crate::doze_plugin::Entry;

            use $crate::clap_sys::{
                entry::clap_plugin_entry,
                factory::{
                    plugin_factory::CLAP_PLUGIN_FACTORY_ID,
                    preset_discovery::{
                        CLAP_PRESET_DISCOVERY_FACTORY_ID, CLAP_PRESET_DISCOVERY_FACTORY_ID_COMPAT,
                    },
                },
                version::CLAP_VERSION,
            };

            use std::{
                ffi::{CStr, c_char, c_void},
                path::PathBuf,
                ptr::null,
            };

            pub unsafe extern "C" fn init(plugin_path: *const c_char) -> bool {
                let pathbuf = if plugin_path.is_null() {
                    None
                } else {
                    unsafe {
                        Some(PathBuf::from(
                            CStr::from_ptr(plugin_path).to_string_lossy().as_ref(),
                        ))
                    }
                };

                let path = pathbuf.as_ref().map(PathBuf::as_path);

                let Some(builder) = <$entry as Entry<Clap>>::init(path) else {
                    return false;
                };

                *CLAP_PLUGIN_FACTORY.lock() = Some(builder.into());

                true
            }

            pub unsafe extern "C" fn deinit() {
                <$entry as Entry<Clap>>::deinit();
                *CLAP_PLUGIN_FACTORY.lock() = None;
            }

            #[allow(non_upper_case_globals)]
            #[unsafe(no_mangle)]
            #[used]
            pub static clap_entry: clap_plugin_entry = clap_plugin_entry {
                init: Some(init),
                deinit: Some(deinit),
                get_factory: Some(get_factory),
                clap_version: CLAP_VERSION,
            };
        }
    };
}
