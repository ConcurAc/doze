use std::{
    ffi::{CStr, c_void},
    marker::PhantomData,
    ptr::null_mut,
};

use clap_sys::{
    events::{clap_input_events, clap_output_events},
    ext::params::{
        CLAP_EXT_PARAMS, CLAP_PARAM_IS_AUTOMATABLE, CLAP_PARAM_IS_AUTOMATABLE_PER_CHANNEL,
        CLAP_PARAM_IS_AUTOMATABLE_PER_KEY, CLAP_PARAM_IS_AUTOMATABLE_PER_NOTE_ID,
        CLAP_PARAM_IS_AUTOMATABLE_PER_PORT, CLAP_PARAM_IS_BYPASS, CLAP_PARAM_IS_ENUM,
        CLAP_PARAM_IS_HIDDEN, CLAP_PARAM_IS_MODULATABLE, CLAP_PARAM_IS_MODULATABLE_PER_CHANNEL,
        CLAP_PARAM_IS_MODULATABLE_PER_KEY, CLAP_PARAM_IS_MODULATABLE_PER_NOTE_ID,
        CLAP_PARAM_IS_MODULATABLE_PER_PORT, CLAP_PARAM_IS_PERIODIC, CLAP_PARAM_IS_READONLY,
        CLAP_PARAM_IS_STEPPED, CLAP_PARAM_REQUIRES_PROCESS, clap_param_info, clap_plugin_params,
    },
    id::clap_id,
    plugin::clap_plugin,
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
        let option: Option<u32> = (move || unsafe {
            let wrapper = ClapPluginWrapper::from_raw(clap_plugin)?;
            let params = wrapper.extensions.get::<Params<P>>()?;

            let plugin = Params::<P>::get(wrapper.plugin.as_ref());

            Some((params.count)(plugin) as u32)
        })();

        option.unwrap_or(0)
    }

    unsafe extern "C" fn get_info(
        clap_plugin: *const clap_plugin,
        index: u32,
        clap_param_info: *mut clap_param_info,
    ) -> bool {
        let option: Option<()> = (move || unsafe {
            let clap_param_info = clap_param_info.as_mut()?;

            let wrapper = ClapPluginWrapper::from_raw(clap_plugin)?;
            let params = wrapper.extensions.get::<Params<P>>()?;

            let plugin = Params::<P>::get(wrapper.plugin.as_ref());

            (params.get)(plugin, index as usize).map(|p| {
                let identifier = p.symbol.downgrade();

                let id = identifier.into();

                let info = ClapParamInfo {
                    id,
                    param: &p,
                    cookie: null_mut(), // optimisation not implemented yet
                };
                *clap_param_info = info.into();
            })
        })();

        option.is_some()
    }

    unsafe extern "C" fn get_value(
        plugin: *const clap_plugin,
        param_id: clap_id,
        value: *mut f64,
    ) -> bool {
        let option: Option<()> = (move || unsafe {
            let value = value.as_mut()?;

            let wrapper = ClapPluginWrapper::from_raw(plugin)?;
            let params = wrapper.extensions.get::<Params<P>>()?;

            let entry = wrapper.entities.get(&param_id.into())?;

            let plugin = Params::<P>::get(wrapper.plugin.as_ref());

            (params.get)(plugin, entry.index).map(|p| *value = p.value.get())
        })();

        option.is_some()
    }

    unsafe extern "C" fn value_to_text(
        clap_plugin: *const clap_plugin,
        param_id: clap_id,
        value: f64,
        display: *mut i8,
        size: u32,
    ) -> bool {
        let option: Option<bool> = (move || unsafe {
            if display.is_null() {
                return None;
            }

            let buffer = core::slice::from_raw_parts_mut(display as *mut u8, size as usize);
            let mut message = NullTermMessage::new(buffer);

            let wrapper = ClapPluginWrapper::from_raw(clap_plugin)?;
            let params = wrapper.extensions.get::<Params<P>>()?;
            let entity = wrapper.entities.get(&param_id.into())?;
            let plugin = Params::<P>::get(wrapper.plugin.as_ref());

            (params.get)(plugin, entity.index).map(|p| (p.value_to_text)(&mut message, value))
        })();

        option.unwrap_or_default()
    }

    unsafe extern "C" fn text_to_value(
        clap_plugin: *const clap_plugin,
        param_id: clap_id,
        display: *const i8,
        value: *mut f64,
    ) -> bool {
        let option: Option<()> = (move || unsafe {
            let value = value.as_mut()?;

            if display.is_null() {
                return None;
            }
            let text = CStr::from_ptr(display).to_str().ok()?;

            let wrapper = ClapPluginWrapper::from_raw(clap_plugin)?;
            let params = wrapper.extensions.get::<Params<P>>()?;
            let entity = wrapper.entities.get(&param_id.into())?;
            let plugin = Params::<P>::get(wrapper.plugin.as_ref());

            (params.get)(plugin, entity.index)
                .and_then(|p| (p.text_to_value)(text))
                .map(|v| *value = v)
        })();

        option.is_some()
    }

    unsafe extern "C" fn flush(
        clap_plugin: *const clap_plugin,
        input_events: *const clap_input_events,
        output_events: *const clap_output_events,
    ) {
        let _: Option<()> = (move || unsafe {
            let wrapper = ClapPluginWrapper::from_raw_mut(clap_plugin)?;

            let input_events = input_events.as_ref()?;
            let output_events = output_events.as_ref()?;

            let params = wrapper.extensions.get::<Params<P>>()?;

            let mut input_iter = ClapHostEventIter::new(input_events, &wrapper.entities)?;

            let mut sender = ClapEventSender::new(output_events, &mut wrapper.sent_events)?;

            let plugin = Params::<P>::get_mut(wrapper.plugin.as_mut());

            (params.flush)(plugin, &mut input_iter, &mut sender);
            Some(())
        })();
    }
}

impl<'d> Into<clap_param_info> for ClapParamInfo<'d> {
    fn into(self) -> clap_param_info {
        let mut name = [0i8; 256];
        for (i, b) in self.param.name.bytes().take(255).enumerate() {
            name[i] = b as i8;
        }
        let mut module = [0i8; 1024];
        let group = &self.param.group;
        for (i, b) in group
            .prefix
            .bytes()
            .chain([b'/'].into_iter())
            .chain(group.name.bytes())
            .take(1023)
            .enumerate()
        {
            module[i] = b as i8;
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
