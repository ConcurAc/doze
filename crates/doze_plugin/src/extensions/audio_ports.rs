use doze_common::identifier::{StrongIdentifier, WeakIdentifier};

use crate::plugin::Plugin;

use super::{Extension, PluginAccess, RegistrySource};

/// Audio port discovery extension.
///
/// Allows the host to query the plugin for information about its audio input and output ports.
/// A port represents a group of audio channels (e.g., a stereo output = one port with 2 channels).
#[derive(Clone)]
pub struct AudioPorts<P: Plugin> {
    /// Query the number of ports in a given direction.
    pub count: fn(&P, PortDirection) -> usize,

    /// Get the descriptor for a specific port.
    ///
    /// Port indices must remain stable for the entire plugin session.
    pub get: fn(&P, PortDirection, usize) -> Option<&AudioPortDescriptor>,

    /// Query if two ports can process in-place (same memory buffer).
    ///
    /// If `Some(input_id)`, the output can be processed in-place with the given input,
    /// avoiding buffer copies when the plugin doesn't need to re-read the input.
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

/// Metadata describing a single audio port.
///
/// A port groups one or more audio channels. Ports are the primary unit of audio routing.
#[derive(Debug, Clone)]
pub struct AudioPortDescriptor {
    /// Stable symbolic identifier (e.g., "input", "output_main", "sidechain").
    /// Must remain stable across plugin versions to avoid breaking projects.
    pub symbol: StrongIdentifier,

    /// Human-readable display name for the host UI.
    pub name: String,

    /// Channel group configuration (mono, stereo, surround, ambisonics, etc.).
    pub group: PortGroup,

    /// Capability flags indicating sample format support and optimization hints.
    pub flags: AudioPortFlags,
}

bitflags::bitflags! {
    /// Flags describing audio port capabilities.
    #[derive(Debug, Default, Clone, Copy)]
    pub struct AudioPortFlags: u32 {
        /// This is the main audio port for this direction.
        const IS_MAIN = 1 << 0;

        /// Port supports 64-bit (f64) sample format.
        const SUPPORTS_64BIT = 1 << 1;

        /// Port prefers 64-bit (f64) sample format.
        const PREFERS_64BIT = 1 << 2;

        /// All channels in this port must use the same sample size.
        const REQUIRES_COMMON_SAMPLE_SIZE = 1 << 3;
    }
}

/// Direction of an audio port (input or output).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortDirection {
    /// Audio flows from host to plugin.
    Input,

    /// Audio flows from plugin to host.
    Output,
}

/// Channel configuration for an audio port.
///
/// Describes how many channels a port has and their layout.
#[derive(Debug, Default, Clone, Copy)]
pub enum PortGroup {
    /// Single audio channel (mono).
    #[default]
    Mono,

    /// Left channel of a stereo pair.
    Left,

    /// Right channel of a stereo pair.
    Right,

    /// Stereo pair (left + right).
    Stereo,

    /// Mid channel (for mid-side encoding).
    Mid,

    /// Side channel (for mid-side encoding).
    Side,

    /// Mid-side stereo pair.
    MidSide,

    /// Surround format with arbitrary number of channels.
    Surround {
        /// Number of channels in this surround configuration.
        channel_count: u32,
    },

    /// Ambisonic format with specified order (channels = (order + 1)²).
    Ambisonic {
        /// Ambisonic order (0, 1, 2, 3, etc.).
        order: u32,
    },

    /// Generic/unspecified channel layout.
    Generic,
}

impl PortGroup {
    /// Get the number of channels in this port group.
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
