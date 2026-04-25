[![Crates.io](https://img.shields.io/crates/v/doze)](https://crates.io/crates/doze_plugin)

# doze_plugin

Middleware abstraction layer for building audio plugins, independent of any specific plugin format.

## Overview

`doze_plugin` defines the core traits and types used to implement audio plugins within the doze
framework. It is format-agnostic — the same plugin implementation can target CLAP, VST, or any
other format via a format-specific crate (e.g. `doze_clap`).

## Core Traits

### `Plugin`

The main trait to implement for your plugin. Provides lifecycle and processing callbacks.

```rust
impl Plugin for MyPlugin {
    fn activate(&mut self, sample_rate: f64, min_frames: u32, max_frames: u32) -> bool { .. }
    fn start_processing(&mut self) -> bool { .. }
    fn process(&mut self, state: Process) -> Status { .. }
}
```

### `Entry<A: PluginApi>`

The factory entry point. Construct your plugin descriptors, extensions, and builders here,
then return a `PluginFactoryBuilder` to register them with the host.

```rust
impl<A: PluginApi> Entry<A> for MyEntry {
    fn init(path: Option<&Path>) -> Option<PluginFactoryBuilder<A>> { .. }
}
```

For fine-grained control of the Entry implementation across different formats you can create separate `impl` for each API.

```rust
impl Entry<doze_clap::Clap> for MyEntry {
    fn init(path: Option<&Path>) -> Option<PluginFactoryBuilder<doze_clap::Clap>> { .. }
}
```

## Extensions

Extensions are host-plugin interfaces declared at factory time and queried by the host at runtime.

### `AudioPorts`

Describes the plugin's audio port layout.

```rust
let audio_ports = AudioPorts::<MyPlugin> {
    count: |plugin, direction| { .. },
    get: |plugin, direction, index| { .. },
    in_place_pairs: None,
}
```

### `Params`

Exposes plugin parameters to the host for automation and modulation.

Parameters are indexed and must remain stable for the plugin's entire runtime — the host
identifies parameters by their `symbol`, not their index. `IndexMap` is recommended for
storage as it provides both stable ordering and symbol-based lookup.

```rust
let params = Params::<MyPlugin> {
    count: |plugin| { .. },
    get: |plugin, index| { .. },
    flush: |plugin, events, _output| { .. },
}
```

## Plugin Builder

Assemble your plugin with `PluginBuilder`, then register it with the factory.

```rust
let plugin = PluginBuilder::<A, MyPlugin>::default()
    .set_creator(|| Box::new(MyPlugin::new()))
    .set_descriptor(descriptor)
    .add_extension(audio_ports)
    .add_extension(params);

PluginFactoryBuilder::new().add_plugin(plugin.into())
```

## Exporting

Pass your `Entry` implementor to the format-specific export macro.

```rust
doze_clap::export!(MyEntry);
```

## Status

See [TODO.md](TODO.md) for what is implemented and what is planned.
