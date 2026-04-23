# Doze

A lightweight Rust framework for building audio plugins. Write plugin logic once and export to any supported format.

Currently supports **CLAP** to a minimal working standard. VST3 and a first-party DSP crate are planned.

## Design Goals

- Format-agnostic plugin API: one codebase, multiple targets
- Minimal runtime footprint
- Modular crate ecosystem
- Broad platform support

## Crates

| Crate | Description |
|---|---|
| `doze_common` | Shared types and utilities |
| `doze_plugin` | Middleware plugin API |
| `doze_clap` | CLAP implementation |

## Getting Started

The CLAP support requires the Rust nightly compiler for `min_specialization`. 
To try a minimal working example, run the following.

```sh
cargo build -p gain --release
```

This will build a (e.g. `libgain.so` on linux) which can be renamed to `gain.clap` and used in a compatible DAW.
Tested with REAPER on linux.

See `examples/gain` for the minimal plugin implementation.

A Nix flake is provided for reproducible environments.

## Status

Doze is in active early development. The API will change before 1.0.0. See [TODO.md](TODO.md) for what is implemented and what is planned.

## License

Apache License 2.0. See `LICENSE`.
