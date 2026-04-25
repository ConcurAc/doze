[![Crates.io](https://img.shields.io/crates/v/doze)](https://crates.io/crates/doze_clap)

# doze_clap

To export a clap plugin, implement `doze_plugin::Entry` to on a type and pass it to macro `doze_clap::export`.

```rust
use doze_plugin::prelude::*;

struct MyEntry;

impl<A: PluginApi> Entry<A> for MyEntry {
    fn init(_path: Option<&Path>) -> Option<PluginFactoryBuilder<A>> {
        todo!();
    }
}

doze_clap::export!(MyEntry);
```

## Status

See [TODO.md](TODO.md) for what is implemented and what is planned.
