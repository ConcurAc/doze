use std::{
    ffi::{CStr, c_void},
    marker::PhantomData,
};

use clap_sys::{
    ext::{
        ambisonic::CLAP_PORT_AMBISONIC,
        audio_ports::{
            CLAP_AUDIO_PORT_IS_MAIN, CLAP_AUDIO_PORT_PREFERS_64BITS,
            CLAP_AUDIO_PORT_REQUIRES_COMMON_SAMPLE_SIZE, CLAP_AUDIO_PORT_SUPPORTS_64BITS,
            CLAP_EXT_AUDIO_PORTS, CLAP_PORT_MONO, CLAP_PORT_STEREO, clap_audio_port_info,
            clap_plugin_audio_ports,
        },
        surround::CLAP_PORT_SURROUND,
    },
    id::CLAP_INVALID_ID,
    plugin::clap_plugin,
};

use doze_common::identifier::WeakIdentifier;
use doze_plugin::{
    extensions::{
        PluginAccess,
        audio_ports::{AudioPortFlags, AudioPorts, PortDirection, PortGroup},
    },
    plugin::Plugin,
    prelude::AudioPortDescriptor,
};

use crate::{
    ClapId,
    extensions::{ClapExtensionDescriptor, ClapExtensions, ClapExtern, ClapPayload},
    plugin::ClapPluginWrapper,
};

impl<P: Plugin> ClapPayload<AudioPorts<P>> for ClapExtern {
    fn get() -> ClapExtensions {
        ClapAudioPorts::<P>::CLAP_EXTENSION
    }
}

struct ClapAudioPorts<P: Plugin>(PhantomData<P>);

impl<P: Plugin> ClapAudioPorts<P> {
    const CLAP_EXTENSION: ClapExtensions = &[ClapExtensionDescriptor {
        id: WeakIdentifier::from_cstr(CLAP_EXT_AUDIO_PORTS),
        vtable: &clap_plugin_audio_ports {
            get: Some(Self::get),
            count: Some(Self::count),
        } as *const _ as *const c_void,
    }];
}

impl<P: Plugin> ClapAudioPorts<P> {
    unsafe extern "C" fn count(clap_plugin: *const clap_plugin, is_input: bool) -> u32 {
        let Some(wrapper) = (unsafe { ClapPluginWrapper::from_raw(clap_plugin) }) else {
            return 0;
        };

        let Some(audio_ports) = wrapper.extensions.get::<AudioPorts<P>>() else {
            return 0;
        };

        let direction = if is_input {
            PortDirection::Input
        } else {
            PortDirection::Output
        };

        let plugin = AudioPorts::<P>::get(wrapper.plugin.as_ref());

        (audio_ports.count)(plugin, direction) as u32
    }

    unsafe extern "C" fn get(
        clap_plugin: *const clap_plugin,
        index: u32,
        is_input: bool,
        clap_audio_port_info: *mut clap_audio_port_info,
    ) -> bool {
        let Some(wrapper) = (unsafe { ClapPluginWrapper::from_raw(clap_plugin) }) else {
            return false;
        };

        let Some(audio_ports) = wrapper.extensions.get::<AudioPorts<P>>() else {
            return false;
        };

        let Some(clap_audio_port_info) = (unsafe { clap_audio_port_info.as_mut() }) else {
            return false;
        };

        let direction = if is_input {
            PortDirection::Input
        } else {
            PortDirection::Output
        };

        let index = index as usize;

        let plugin = AudioPorts::<P>::get(wrapper.plugin.as_ref());

        (audio_ports.get)(plugin, direction, index)
            .map(|p| {
                let identifier = p.symbol.downgrade();
                let id = identifier.into();

                let in_place_pair = audio_ports
                    .in_place_pairs
                    .map(|f| (f)(plugin, identifier).map(|p| p.into()))
                    .flatten();

                let info = ClapAudioPortInfo {
                    id,
                    descriptor: p,
                    in_place_pair,
                };
                *clap_audio_port_info = info.into();
            })
            .is_some()
    }
}

pub struct ClapAudioPortInfo<'d> {
    pub id: ClapId,
    pub descriptor: &'d AudioPortDescriptor,
    pub in_place_pair: Option<ClapId>,
}

impl<'d> Into<clap_audio_port_info> for ClapAudioPortInfo<'d> {
    fn into(self) -> clap_audio_port_info {
        let mut name = [0; 256];
        for (i, b) in self.descriptor.name.bytes().take(255).enumerate() {
            name[i] = b as i8;
        }
        clap_audio_port_info {
            id: self.id.get(),
            name,
            channel_count: self.descriptor.group.channel_count(),
            port_type: port_group_as_clap(&self.descriptor.group).as_ptr(),
            flags: ClapAudioPortFlags::from(self.descriptor.flags).bits(),
            in_place_pair: self
                .in_place_pair
                .map(|i| i.get())
                .unwrap_or(CLAP_INVALID_ID),
        }
    }
}

impl From<AudioPortFlags> for ClapAudioPortFlags {
    fn from(value: AudioPortFlags) -> Self {
        const MAPPING: &[(AudioPortFlags, ClapAudioPortFlags)] = &[
            (AudioPortFlags::IS_MAIN, ClapAudioPortFlags::IS_MAIN),
            (
                AudioPortFlags::SUPPORTS_64BIT,
                ClapAudioPortFlags::SUPPORTS_64BIT,
            ),
            (
                AudioPortFlags::PREFERS_64BIT,
                ClapAudioPortFlags::PREFERS_64BIT,
            ),
            (
                AudioPortFlags::REQUIRES_COMMON_SAMPLE_SIZE,
                ClapAudioPortFlags::REQUIRES_COMMON_SAMPLE_SIZE,
            ),
        ];
        MAPPING
            .iter()
            .filter(|(f, _)| value.contains(*f))
            .fold(ClapAudioPortFlags::empty(), |acc, (_, c)| acc | *c)
    }
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct ClapAudioPortFlags: u32 {
        const IS_MAIN = CLAP_AUDIO_PORT_IS_MAIN;
        const SUPPORTS_64BIT = CLAP_AUDIO_PORT_SUPPORTS_64BITS;
        const PREFERS_64BIT = CLAP_AUDIO_PORT_PREFERS_64BITS;
        const REQUIRES_COMMON_SAMPLE_SIZE = CLAP_AUDIO_PORT_REQUIRES_COMMON_SAMPLE_SIZE;
    }
}

const MONO: &'static [u8] = CLAP_PORT_MONO.to_bytes();
const STEREO: &'static [u8] = CLAP_PORT_STEREO.to_bytes();
const SURROUND: &'static [u8] = CLAP_PORT_SURROUND.to_bytes();
const AMBISONIC: &'static [u8] = CLAP_PORT_AMBISONIC.to_bytes();

pub fn port_group_from_clap(port_type: &CStr, channel_count: u32) -> Option<PortGroup> {
    match port_type.to_bytes() {
        MONO => Some(PortGroup::Mono),
        STEREO => Some(PortGroup::Stereo),
        SURROUND => Some(PortGroup::Surround { channel_count }),
        AMBISONIC => {
            if channel_count < 4 {
                return None;
            }
            let order = (channel_count as f32).sqrt() as u32 - 1;
            if (order + 1) * (order + 1) != channel_count {
                return None;
            }
            Some(PortGroup::Ambisonic { order })
        }
        _ => None,
    }
}

pub fn port_group_as_clap(pg: &PortGroup) -> &'static CStr {
    match pg {
        PortGroup::Mono => CLAP_PORT_MONO,
        PortGroup::Left | PortGroup::Right => CLAP_PORT_MONO,
        PortGroup::Mid | PortGroup::Side => CLAP_PORT_MONO,
        PortGroup::Stereo | PortGroup::MidSide => CLAP_PORT_STEREO,
        PortGroup::Surround { channel_count: _ } => CLAP_PORT_SURROUND,
        PortGroup::Ambisonic { order: _ } => CLAP_PORT_AMBISONIC,
        _ => CLAP_PORT_SURROUND,
    }
}
