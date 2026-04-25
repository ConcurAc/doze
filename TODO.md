# To Do

## Crates
- [`doze_plugin`](crates/doze_plugin/TODO.md)

### doze_dsp

- [ ] processing graph base
- [ ] compile time static pipelines
- [ ] dynamic pipelines
- [ ] time domain processing
- [ ] frequency domain processing
- [ ] sample accurate processing
- [ ] sample accurate event updating

## Backends
- [`doze_clap`](backends/doze_clap/TODO.md)

---

## Design Constraints

- Real-time audio paths must be allocation-free and lock-free
- Zero unsafe code outside C interop
- One crate per clear responsibility
- Correctness before optimisation
