use std::{
    ffi::{CStr, c_char, c_void},
    marker::PhantomData,
    ptr::null_mut,
};

use clap_sys::{
    events::{clap_input_events, clap_output_events},
    ext::params::{CLAP_EXT_PARAMS, clap_param_info, clap_plugin_params},
    id::clap_id,
    plugin::clap_plugin,
};

use clap_sys::ext::params::{
    CLAP_PARAM_IS_AUTOMATABLE, CLAP_PARAM_IS_AUTOMATABLE_PER_CHANNEL,
    CLAP_PARAM_IS_AUTOMATABLE_PER_KEY, CLAP_PARAM_IS_AUTOMATABLE_PER_NOTE_ID,
    CLAP_PARAM_IS_AUTOMATABLE_PER_PORT, CLAP_PARAM_IS_BYPASS, CLAP_PARAM_IS_ENUM,
    CLAP_PARAM_IS_HIDDEN, CLAP_PARAM_IS_MODULATABLE, CLAP_PARAM_IS_MODULATABLE_PER_CHANNEL,
    CLAP_PARAM_IS_MODULATABLE_PER_KEY, CLAP_PARAM_IS_MODULATABLE_PER_NOTE_ID,
    CLAP_PARAM_IS_MODULATABLE_PER_PORT, CLAP_PARAM_IS_PERIODIC, CLAP_PARAM_IS_READONLY,
    CLAP_PARAM_IS_STEPPED, CLAP_PARAM_REQUIRES_PROCESS,
};

use doze_common::{fmt::NullTermMessage, identifier::WeakIdentifier};
use doze_plugin::{
    extensions::{
        PluginAccess,
        params::{Param, ParamFlags, ParamInterpolation, Params},
    },
    plugin::Plugin,
};

use crate::{
    ClapId,
    events::{ClapEventSender, iter::ClapHostEventIter},
    extensions::{ClapExtensionDescriptor, ClapExtensions, ClapExtern, ClapPayload},
    plugin::ClapPluginWrapper,
};

struct ClapParamInfo<'p> {
    pub id: ClapId,
    pub param: &'p Param,
    pub cookie: *const f64,
}

impl<P: Plugin> ClapPayload<Params<P>> for ClapExtern {
    fn get() -> ClapExtensions {
        ClapParams::<P>::CLAP_EXTENSION
    }
}

pub struct ClapParams<P: Plugin>(PhantomData<P>);

impl<P: Plugin> ClapParams<P> {
    const CLAP_EXTENSION: ClapExtensions = &[ClapExtensionDescriptor {
        id: WeakIdentifier::from_cstr(CLAP_EXT_PARAMS),
        vtable: &clap_plugin_params {
            count: Some(Self::count),
            get_info: Some(Self::get_info),
            get_value: Some(Self::get_value),
            value_to_text: Some(Self::value_to_text),
            text_to_value: Some(Self::text_to_value),
            flush: Some(Self::flush),
        } as *const _ as *const _,
    }];
}

impl<P: Plugin> ClapParams<P> {
    unsafe extern "C" fn count(clap_plugin: *const clap_plugin) -> u32 {
        let Some(wrapper) = (unsafe { ClapPluginWrapper::from_raw(clap_plugin) }) else {
            return 0;
        };
        let Some(params) = wrapper.extensions.get::<Params<P>>() else {
            return 0;
        };
        let plugin = Params::<P>::get(wrapper.plugin.as_ref());
        (params.count)(plugin) as u32
    }

    unsafe extern "C" fn get_info(
        clap_plugin: *const clap_plugin,
        index: u32,
        clap_param_info: *mut clap_param_info,
    ) -> bool {
        let Some(clap_param_info) = (unsafe { clap_param_info.as_mut() }) else {
            return false;
        };
        let Some(wrapper) = (unsafe { ClapPluginWrapper::from_raw(clap_plugin) }) else {
            return false;
        };
        let Some(params) = wrapper.extensions.get::<Params<P>>() else {
            return false;
        };
        let plugin = Params::<P>::get(wrapper.plugin.as_ref());
        let Some(p) = (params.get)(plugin, index as usize) else {
            return false;
        };
        let identifier = p.symbol.downgrade();
        let id = identifier.into();
        let info = ClapParamInfo {
            id,
            param: &p,
            cookie: null_mut(),
        };
        *clap_param_info = info.into();
        true
    }

    unsafe extern "C" fn get_value(
        plugin: *const clap_plugin,
        param_id: clap_id,
        value: *mut f64,
    ) -> bool {
        let Some(value) = (unsafe { value.as_mut() }) else {
            return false;
        };
        let Some(wrapper) = (unsafe { ClapPluginWrapper::from_raw(plugin) }) else {
            return false;
        };
        let Some(params) = wrapper.extensions.get::<Params<P>>() else {
            return false;
        };
        let Some(entry) = wrapper.entities.get(&param_id.into()) else {
            return false;
        };
        let plugin = Params::<P>::get(wrapper.plugin.as_ref());
        let Some(p) = (params.get)(plugin, entry.index) else {
            return false;
        };
        *value = p.value.get();
        true
    }

    unsafe extern "C" fn value_to_text(
        clap_plugin: *const clap_plugin,
        param_id: clap_id,
        value: f64,
        display: *mut i8,
        size: u32,
    ) -> bool {
        if display.is_null() {
            return false;
        }
        let buffer = unsafe { core::slice::from_raw_parts_mut(display as *mut u8, size as usize) };
        let mut message = NullTermMessage::new(buffer);
        let Some(wrapper) = (unsafe { ClapPluginWrapper::from_raw(clap_plugin) }) else {
            return false;
        };
        let Some(params) = wrapper.extensions.get::<Params<P>>() else {
            return false;
        };
        let Some(entity) = wrapper.entities.get(&param_id.into()) else {
            return false;
        };
        let plugin = Params::<P>::get(wrapper.plugin.as_ref());
        let Some(p) = (params.get)(plugin, entity.index) else {
            return false;
        };
        (p.value_to_text)(&mut message, value)
    }

    unsafe extern "C" fn text_to_value(
        clap_plugin: *const clap_plugin,
        param_id: clap_id,
        display: *const i8,
        value: *mut f64,
    ) -> bool {
        let Some(value) = (unsafe { value.as_mut() }) else {
            return false;
        };
        if display.is_null() {
            return false;
        }
        let Some(text) = (unsafe { CStr::from_ptr(display).to_str().ok() }) else {
            return false;
        };
        let Some(wrapper) = (unsafe { ClapPluginWrapper::from_raw(clap_plugin) }) else {
            return false;
        };
        let Some(params) = wrapper.extensions.get::<Params<P>>() else {
            return false;
        };
        let Some(entity) = wrapper.entities.get(&param_id.into()) else {
            return false;
        };
        let plugin = Params::<P>::get(wrapper.plugin.as_ref());
        let Some(p) = (params.get)(plugin, entity.index) else {
            return false;
        };
        let Some(v) = (p.text_to_value)(text) else {
            return false;
        };
        *value = v;
        true
    }

    unsafe extern "C" fn flush(
        clap_plugin: *const clap_plugin,
        input_events: *const clap_input_events,
        output_events: *const clap_output_events,
    ) {
        let Some(wrapper) = (unsafe { ClapPluginWrapper::from_raw_mut(clap_plugin) }) else {
            return;
        };
        let Some(input_events) = (unsafe { input_events.as_ref() }) else {
            return;
        };
        let Some(output_events) = (unsafe { output_events.as_ref() }) else {
            return;
        };
        let Some(params) = wrapper.extensions.get::<Params<P>>() else {
            return;
        };
        let input_iter_option = unsafe { ClapHostEventIter::new(input_events, &wrapper.entities) };
        let Some(mut input_iter) = input_iter_option else {
            return;
        };
        let Some(mut sender) = ClapEventSender::new(output_events, &mut wrapper.sent_events) else {
            return;
        };
        let plugin = Params::<P>::get_mut(wrapper.plugin.as_mut());
        (params.flush)(plugin, &mut input_iter, &mut sender);
    }
}

impl<'d> Into<clap_param_info> for ClapParamInfo<'d> {
    fn into(self) -> clap_param_info {
        let mut name = ['\0' as c_char; 256];
        for (i, b) in self.param.name.bytes().take(name.len() - 1).enumerate() {
            name[i] = b as c_char;
        }

        let mut module = ['\0' as c_char; 1024];
        let group = &self.param.group;

        let mut module_iter = group
            .prefix
            .bytes()
            .chain(if group.prefix.is_empty() { "" } else { "/" }.bytes())
            .chain(group.name.bytes());

        for (i, b) in module_iter.by_ref().take(module.len() - 1).enumerate() {
            module[i] = b as c_char;
        }

        clap_param_info {
            id: self.id.get(),
            name,
            module,
            default_value: self.param.value.get_default(),
            min_value: self.param.value.get_min(),
            max_value: self.param.value.get_max(),
            flags: ClapParamFlags::from(self.param).bits(),
            cookie: self.cookie as *mut c_void,
        }
    }
}

const PARAM_FLAG_MAPPING: &[(ParamFlags, ClapParamFlags)] = &[
    (ParamFlags::PERIODIC, ClapParamFlags::PERIODIC),
    (ParamFlags::HIDDEN, ClapParamFlags::HIDDEN),
    (ParamFlags::READONLY, ClapParamFlags::READONLY),
    (ParamFlags::AUTOMATABLE, ClapParamFlags::AUTOMATABLE),
    (
        ParamFlags::AUTOMATABLE_PER_NOTE_ID,
        ClapParamFlags::AUTOMATABLE_PER_NOTE_ID,
    ),
    (
        ParamFlags::AUTOMATABLE_PER_KEY,
        ClapParamFlags::AUTOMATABLE_PER_KEY,
    ),
    (
        ParamFlags::AUTOMATABLE_PER_CHANNEL,
        ClapParamFlags::AUTOMATABLE_PER_CHANNEL,
    ),
    (
        ParamFlags::AUTOMATABLE_PER_PORT,
        ClapParamFlags::AUTOMATABLE_PER_PORT,
    ),
    (ParamFlags::MODULATABLE, ClapParamFlags::MODULATABLE),
    (
        ParamFlags::MODULATABLE_PER_NOTE_ID,
        ClapParamFlags::MODULATABLE_PER_NOTE_ID,
    ),
    (
        ParamFlags::MODULATABLE_PER_KEY,
        ClapParamFlags::MODULATABLE_PER_KEY,
    ),
    (
        ParamFlags::MODULATABLE_PER_CHANNEL,
        ClapParamFlags::MODULATABLE_PER_CHANNEL,
    ),
    (
        ParamFlags::MODULATABLE_PER_PORT,
        ClapParamFlags::MODULATABLE_PER_PORT,
    ),
    (
        ParamFlags::REQUIRES_PROCESS,
        ClapParamFlags::REQUIRES_PROCESS,
    ),
];

impl<'p> From<&Param> for ClapParamFlags {
    fn from(value: &Param) -> Self {
        let mut flags = PARAM_FLAG_MAPPING
            .iter()
            .filter(|(f, _)| value.flags.contains(*f))
            .fold(ClapParamFlags::empty(), |acc, (_, c)| acc | *c);

        match &value.value.get_interpolation() {
            ParamInterpolation::Continuous => {}
            ParamInterpolation::Stepped => {
                flags |= ClapParamFlags::STEPPED;
            }
            ParamInterpolation::Bypass => {
                flags |= ClapParamFlags::STEPPED | ClapParamFlags::IS_BYPASS;
            }
            ParamInterpolation::Enum(_) => {
                flags |= ClapParamFlags::STEPPED | ClapParamFlags::ENUM;
            }
        }

        flags
    }
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct ClapParamFlags: u32 {
        // param is stepped — integer values only
        const STEPPED  = CLAP_PARAM_IS_STEPPED;
        // param is enum — values are labeled
        const ENUM     = CLAP_PARAM_IS_ENUM;
        // param value wraps around — e.g. pan, phase
        const PERIODIC = CLAP_PARAM_IS_PERIODIC;
        // param hidden from user — internal use
        const HIDDEN   = CLAP_PARAM_IS_HIDDEN;
        // param cannot be changed by user or automation
        const READONLY = CLAP_PARAM_IS_READONLY;
        // param is the plugin's bypass
        const IS_BYPASS = CLAP_PARAM_IS_BYPASS;
        // param can be automated
        const AUTOMATABLE             = CLAP_PARAM_IS_AUTOMATABLE;
        // param can be automated per note id (polyphonic)
        const AUTOMATABLE_PER_NOTE_ID = CLAP_PARAM_IS_AUTOMATABLE_PER_NOTE_ID;
        // param can be automated per key
        const AUTOMATABLE_PER_KEY     = CLAP_PARAM_IS_AUTOMATABLE_PER_KEY;
        // param can be automated per MIDI channel
        const AUTOMATABLE_PER_CHANNEL = CLAP_PARAM_IS_AUTOMATABLE_PER_CHANNEL;
        // param can be automated per note port
        const AUTOMATABLE_PER_PORT    = CLAP_PARAM_IS_AUTOMATABLE_PER_PORT;
        // param supports modulation signal
        const MODULATABLE             = CLAP_PARAM_IS_MODULATABLE;
        // param supports per note id modulation (polyphonic)
        const MODULATABLE_PER_NOTE_ID = CLAP_PARAM_IS_MODULATABLE_PER_NOTE_ID;
        // param supports per key modulation
        const MODULATABLE_PER_KEY     = CLAP_PARAM_IS_MODULATABLE_PER_KEY;
        // param supports per channel modulation
        const MODULATABLE_PER_CHANNEL = CLAP_PARAM_IS_MODULATABLE_PER_CHANNEL;
        // param supports per port modulation
        const MODULATABLE_PER_PORT    = CLAP_PARAM_IS_MODULATABLE_PER_PORT;
        // param needs process() to be called to update value
        const REQUIRES_PROCESS        = CLAP_PARAM_REQUIRES_PROCESS;
    }
}
