use std::{
    ffi::{CStr, c_char, c_void},
    ptr::null,
};

use clap_sys::factory::plugin_factory::CLAP_PLUGIN_FACTORY_ID;

use plugin::CLAP_PLUGIN_FACTORY;

pub mod plugin;

pub unsafe extern "C" fn get_factory(factory_id: *const c_char) -> *const c_void {
    if factory_id.is_null() {
        return null();
    }
    let id = unsafe { CStr::from_ptr(factory_id) };

    match id {
        id if id == CLAP_PLUGIN_FACTORY_ID => CLAP_PLUGIN_FACTORY.data_ptr() as *const c_void,
        _ => null(),
    }
}
