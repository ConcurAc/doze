# To Do

## doze_dsp

- [ ] processing graph base
- [ ] compile time static pipelines
- [ ] dynamic pipelines
- [ ] time domain processing
- [ ] frequency domain processing
- [ ] sample accurate processing
- [ ] sample accurate event updating

---

## Design Constraints

- Real-time audio paths must be allocation-free and lock-free
- Zero unsafe code outside C interop
- One crate per clear responsibility
- Correctness before optimisation
