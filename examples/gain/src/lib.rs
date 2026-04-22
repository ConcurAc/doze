use doze::prelude::*;

#[derive(Default)]
pub struct GainPlugin {
    input_ports: Vec<AudioPortDescriptor>,
    output_ports: Vec<AudioPortDescriptor>,
    params: Vec<Param>,
}

impl GainPlugin {
    fn handle_events(&mut self, events: impl Iterator<Item = Event<HostEvent>>) {
        for event in events {
            match event.event {
                HostEvent::Param(param) => match param {
                    HostParamEvent::Value {
                        index,
                        value,
                        context: _,
                    } => {
                        self.params[index].value.set(value);
                    }
                    HostParamEvent::Modulate {
                        index,
                        amount,
                        context: _,
                    } => {
                        self.params[index].value.modulate(amount);
                    }
                },
                _ => (),
            }
        }
    }
}

impl Plugin for GainPlugin {
    fn init(&mut self) {}
    fn reset(&mut self) {}
    fn activate(&mut self, _sample_rate: f64, _min_frames: u32, _max_frames: u32) -> bool {
        true
    }
    fn deactivate(&mut self) {}
    fn start_processing(&mut self) -> bool {
        true
    }
    fn stop_processing(&mut self) {}
    fn process(&mut self, state: Process) -> Status {
        self.handle_events(state.events);

        let inputs = state.audio_inputs;
        let outputs = state.audio_outputs;

        assert_eq!(inputs.count(), outputs.count());

        if let Some(gain) = self.params.first() {
            for i in 0..inputs.count() {
                let results = (inputs.get_f32_buffer(i), outputs.get_f32_buffer(i));
                if let (Ok(input), Ok(mut output)) = results {
                    apply_gain(&input, &mut output, gain.value.get());
                }

                let results = (inputs.get_f64_buffer(i), outputs.get_f64_buffer(i));
                if let (Ok(input), Ok(mut output)) = results {
                    apply_gain(&input, &mut output, gain.value.get());
                }
            }
        }

        Status::Continue
    }

    fn on_main_thread(&mut self) {}
}

fn apply_gain<'p, T: Sample>(
    input: &'p AudioBuffer<'p, T>,
    output: &'p mut AudioBuffer<'p, T>,
    gain: f64,
) {
    assert_eq!(input.count(), output.count());

    for (r, w) in input.iter_reader().zip(output.iter_writer()) {
        io::apply::<T, 128>(r, w, |s| s * f64::powf(10.0, gain * 0.05).as_primitive());
    }
}

pub struct GainEntry;

impl<A: PluginApi> Entry<A> for GainEntry {
    fn init(_path: Option<&Path>) -> Option<PluginFactoryBuilder<A>> {
        let builder = PluginFactoryBuilder::new().add_plugin(
            PluginBuilder::<A, GainPlugin>::default()
                .set_creator(|| {
                    let plugin = GainPlugin {
                        input_ports: vec![AudioPortDescriptor {
                            symbol: "i".into(),
                            name: "input".into(),
                            group: PortGroup::Stereo,
                            flags: AudioPortFlags::default(),
                        }],
                        output_ports: vec![AudioPortDescriptor {
                            symbol: "o".into(),
                            name: "output".into(),
                            group: PortGroup::Stereo,
                            flags: AudioPortFlags::default(),
                        }],
                        params: vec![Param {
                            symbol: "g".into(),
                            name: "gain".into(),
                            group: ParamGroup {
                                symbol: "g".into(),
                                name: "gain".into(),
                                prefix: "".into(),
                                port_group: PortGroup::Stereo,
                            },
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
                        }],
                    };
                    Box::new(plugin)
                })
                .set_descriptor(PluginDescriptor {
                    id: "com.example.gain".into(),
                    name: "Gain".into(),
                    vendor: "Example".into(),
                    version: "0.1.0".into(),
                    url: None,
                    manual_url: None,
                    support_url: None,
                    description: None,
                    features: vec![PluginFeature::AudioEffect, PluginFeature::Stereo].into(),
                })
                .add_extension(AudioPorts::<GainPlugin> {
                    count: |plugin, direction| match direction {
                        PortDirection::Input => plugin.input_ports.len(),
                        PortDirection::Output => plugin.output_ports.len(),
                    },
                    get: |plugin, direction, index| match direction {
                        PortDirection::Input => plugin.input_ports.get(index),
                        PortDirection::Output => plugin.output_ports.get(index),
                    },
                    in_place_pairs: None,
                })
                .add_extension(Params::<GainPlugin> {
                    count: |plugin| plugin.params.len(),
                    get: |plugin, index| plugin.params.get(index),
                    flush: |plugin, events, _output| plugin.handle_events(events),
                })
                .into(),
        );
        Some(builder)
    }

    fn deinit() {}
}

doze_clap::export!(GainEntry);
