use doze::{
    common::{collections::IndexMap, identifier::StrongIdentifier},
    prelude::*,
};

/// Gain plugin state struct.
///
/// This holds:
/// - input/output port definitions (audio routing metadata)
/// - parameter definitions (gain, etc.)
///
/// In this design, everything is stored manually rather than using
/// a higher-level parameter system.
pub struct GainPlugin {
    input_audio_ports: Vec<AudioPortDescriptor>,
    output_audio_ports: Vec<AudioPortDescriptor>,
    params: IndexMap<StrongIdentifier, Param>,
}

impl GainPlugin {
    fn new() -> Self {
        let input_audio_ports = vec![AudioPortDescriptor {
            symbol: "in".into(),
            name: "input".into(),
            group: PortGroup::Stereo,
            flags: AudioPortFlags::default(),
        }];

        let output_audio_ports = vec![AudioPortDescriptor {
            symbol: "out".into(),
            name: "output".into(),
            group: PortGroup::Stereo,
            flags: AudioPortFlags::default(),
        }];

        let params = vec![Param {
            symbol: "gain".into(),
            name: "gain".into(),
            group: ParamGroup::default(),
            flags: ParamFlags::empty(),
            value: ParamRange::Continuous {
                min: -60.0,
                max: 12.0,
                default: 0.0,
            }
            .into(),
            value_to_text: |writer, value| {
                let (value, unit) = ParamUnit::Decibels.scale(value);
                write!(writer, "{:.2} {}", value, unit).is_ok()
            },
            text_to_value: |text| ParamUnit::Decibels.parse(text),
        }];

        Self {
            input_audio_ports,
            output_audio_ports,
            params: params.into_iter().map(|p| (p.symbol.clone(), p)).collect(),
        }
    }
    /// Handles incoming host events (automation, modulation, etc.)
    ///
    /// The host sends parameter changes as events rather than directly
    /// modifying state, so we must decode and apply them manually.
    fn handle_events(&mut self, events: impl Iterator<Item = Event<HostEvent>>) {
        for event in events {
            match event.event {
                HostEvent::Param(param) => match param {
                    HostParamEvent::Value {
                        index,
                        value,
                        context: _,
                    } => {
                        if let Some((_, param)) = self.params.get_index_mut(index) {
                            param.value.set(value);
                        }
                    }
                    HostParamEvent::Modulate {
                        index,
                        amount,
                        context: _,
                    } => {
                        if let Some((_, param)) = self.params.get_index_mut(index) {
                            param.value.modulate(amount);
                        }
                    }
                },
                _ => (),
            }
        }
    }
}

impl Plugin for GainPlugin {
    fn activate(&mut self, _sample_rate: f64, _min_frames: u32, _max_frames: u32) -> bool {
        true
    }
    /// Called when processing starts (before audio callbacks begin).
    fn start_processing(&mut self) -> bool {
        true
    }
    /// Main audio processing callback (real-time thread).
    fn process(&mut self, state: Process) -> Status {
        // Apply any pending automation/modulation events first
        self.handle_events(state.events);

        let inputs = state.audio_inputs;
        let outputs = state.audio_outputs;

        debug_assert_eq!(inputs.count(), outputs.count());

        // Fetch gain parameter (first and only param in this plugin)
        // for multiple plugins IndexMap or phf are recommended for
        // param retrieval by symbol
        if let Some(gain) = self.params.get("gain".as_bytes()) {
            for i in 0..inputs.count() {
                // Try f32 buffer processing
                let results = (inputs.get_f32_buffer(i), outputs.get_f32_buffer(i));
                if let (Ok(input), Ok(mut output)) = results {
                    apply_gain(&input, &mut output, gain.value.get());
                }
                // Try f64 buffer processing
                let results = (inputs.get_f64_buffer(i), outputs.get_f64_buffer(i));
                if let (Ok(input), Ok(mut output)) = results {
                    apply_gain(&input, &mut output, gain.value.get());
                }
            }
        }

        Status::Continue
    }
}

/// Core DSP function: applies gain to an audio buffer.
///
/// Generic over sample type (`f32` or `f64`) so it works for both formats.
fn apply_gain<'p, T: Sample>(
    input: &'p AudioBuffer<'p, T>,
    output: &'p mut AudioBuffer<'p, T>,
    gain_db: f64,
) {
    debug_assert_eq!(input.count(), output.count());

    // Convert gain from dB to linear scale:
    let gain = f64::powf(10.0, gain_db * 0.05).as_primitive();

    for (r, w) in input.iter_reader().zip(output.iter_writer()) {
        io::apply::<T, 128>(r, w, |s| s * gain);
    }
}

/// Entry point for plugin factory system.
///
/// This builds the plugin instance and registers it with the host.
pub struct GainEntry;

impl<A: PluginApi> Entry<A> for GainEntry {
    fn init(_path: Option<&Path>) -> Option<PluginFactoryBuilder<A>> {
        let gain_descriptor = PluginDescriptor {
            id: "com.example.gain".into(),
            name: "Gain".into(),
            vendor: "Example".into(),
            version: "0.1.0".into(),
            url: None,
            manual_url: None,
            support_url: None,
            description: None,
            features: vec![PluginFeature::AudioEffect, PluginFeature::Stereo].into(),
        };

        let gain_audio_ports = AudioPorts::<GainPlugin> {
            count: |plugin, direction| match direction {
                PortDirection::Input => plugin.input_audio_ports.len(),
                PortDirection::Output => plugin.output_audio_ports.len(),
            },
            get: |plugin, direction, index| match direction {
                PortDirection::Input => plugin.input_audio_ports.get(index),
                PortDirection::Output => plugin.output_audio_ports.get(index),
            },
            in_place_pairs: None,
        };

        let gain_params = Params::<GainPlugin> {
            count: |plugin| plugin.params.len(),
            // Once the plugin reports a param to the host
            // it is expected to perist with the same index for the entire runtime.
            // Ordering does not affect host plugin identification. Params are remembered by
            // the host from their [`symbol`]
            get: |plugin, index| plugin.params.get_index(index).map(|(_, p)| p),
            flush: |plugin, events, _output| plugin.handle_events(events),
        };

        let gain_builder = PluginBuilder::<A, GainPlugin>::default()
            .set_creator(|| Box::new(GainPlugin::new()))
            .set_descriptor(gain_descriptor)
            .add_extension(gain_audio_ports)
            .add_extension(gain_params);

        Some(PluginFactoryBuilder::new().add_plugin(gain_builder.into()))
    }

    /// Called when plugin library is unloaded.
    fn deinit() {}
}

doze_clap::export!(GainEntry);
