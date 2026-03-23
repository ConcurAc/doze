use doze_common::identifier::{StrongIdentifier, WeakIdentifier};

use crate::plugin::Plugin;

use super::{Extension, PluginAccess, RegistrySource};

#[derive(Clone)]
pub struct AudioPorts<P: Plugin> {
    pub count: fn(&P, PortDirection) -> usize,
    pub get: fn(&P, PortDirection, usize) -> Option<&AudioPortDescriptor>,
    pub in_place_pairs: Option<for<'i> fn(&'i P, WeakIdentifier) -> Option<WeakIdentifier<'i>>>,
}

impl<P: Plugin> Extension for AudioPorts<P> {
    fn as_registry_source(&self) -> Option<&dyn RegistrySource> {
        Some(self)
    }
}

impl<P: Plugin> PluginAccess<P> for AudioPorts<P> {}

impl<P: Plugin> RegistrySource for AudioPorts<P> {
    fn identifiers(&self, plugin: &dyn Plugin) -> Vec<StrongIdentifier> {
        let plugin = <Self as PluginAccess<P>>::get(plugin);
        let input_count = (self.count)(plugin, PortDirection::Input);
        let output_count = (self.count)(plugin, PortDirection::Output);
        let mut identifiers = Vec::with_capacity(input_count + output_count);
        for i in 0..input_count {
            if let Some(descriptor) = (self.get)(plugin, PortDirection::Input, i) {
                identifiers.push(descriptor.symbol.clone());
            };
        }
        for i in 0..output_count {
            if let Some(descriptor) = (self.get)(plugin, PortDirection::Output, i) {
                identifiers.push(descriptor.symbol.clone());
            };
        }
        identifiers
    }
}

#[derive(Debug)]
pub struct AudioPortDescriptor {
    pub symbol: StrongIdentifier,
    pub name: String,
    pub group: PortGroup,
    pub flags: AudioPortFlags,
}

bitflags::bitflags! {
    #[derive(Debug, Default, Clone, Copy)]
    pub struct AudioPortFlags: u32 {
        const IS_MAIN = 1 << 0;
        const SUPPORTS_64BIT = 1 << 1;
        const PREFERS_64BIT = 1 << 2;
        const REQUIRES_COMMON_SAMPLE_SIZE = 1 << 3;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortDirection {
    Input,
    Output,
}

#[derive(Debug, Default)]
pub enum PortGroup {
    #[default]
    Mono,

    Left,
    Right,
    Stereo,

    Mid,
    Side,
    MidSide,

    Surround {
        channel_count: u32,
    },
    Ambisonic {
        order: u32,
    },
    Generic,
}

impl PortGroup {
    pub fn channel_count(&self) -> u32 {
        match self {
            Self::Mono => 1,
            Self::Left | Self::Right => 1,
            Self::Mid | Self::Side => 1,
            Self::Stereo | Self::MidSide => 2,
            Self::Surround { channel_count } => *channel_count,
            Self::Ambisonic { order } => (order + 1) * (order + 1),
            Self::Generic => 0,
        }
    }
}
