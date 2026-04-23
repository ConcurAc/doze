# Doze Roadmap

> Rust audio plugin framework. CLAP primary target.

---

## CLAP Plugin Extensions

Plugin-side interfaces implemented by the plugin and queried by the host.

| Plugin | Host | Details |
|---|---|---|
| [x] | [ ] | `audio-ports` — define audio ports |
| [x] | [ ] | `params` — parameter management, value and modulation events |
| [ ] | [ ] | `state` — save and load plugin state, version-safe format, round-trip tests |
| [ ] | [ ] | `state-context` — state with preset/duplicate/project context |
| [ ] | [ ] | `note-ports` — polyphonic note support, full note expressions, MIDI 2.0 mapping |
| [ ] | [ ] | `latency` — report processing latency to host |
| [ ] | [ ] | `tail` — report processing tail length |
| [ ] | [ ] | `render` — realtime vs offline render mode |
| [ ] | [ ] | `voice-info` — voice count for polyphonic modulation |
| [ ] | [ ] | `gui` — generic GUI window lifecycle |
| [ ] | [ ] | `audio-ports-config` — pre-defined port configurations |
| [ ] | [ ] | `audio-ports-activation` — activate and deactivate individual audio ports |
| [ ] | [ ] | `configurable-audio-ports` — request plugin apply a given port configuration |
| [ ] | [ ] | `surround` — surround channel mapping inspection |
| [ ] | [ ] | `ambisonic` — ambisonic channel mapping inspection |
| [ ] | [ ] | `remote-controls` — bank of 8-knob controller mappings |
| [ ] | [ ] | `note-name` — named notes, useful for drum machines |
| [ ] | [ ] | `preset-load` — host-initiated preset loading |
| [ ] | [ ] | `param-indication` — physical controller and automation mapping info |
| [ ] | [ ] | `track-info` — track context provided by host |
| [ ] | [ ] | `context-menu` — exchange context menu entries with host |

## CLAP Host Extensions

Host-only interfaces with no plugin-side counterpart.

| Plugin | Host | Details |
|---|---|---|
| [ ] | [ ] | `log` — aggregate plugin logs via host |
| [ ] | [ ] | `thread-check` — validate current thread context |
| [ ] | [ ] | `thread-pool` — submit work to the host thread pool |
| [ ] | [ ] | `timer-support` — register periodic timer callbacks |
| [ ] | [ ] | `posix-fd-support` — register I/O handlers |
| [ ] | [ ] | `event-registry` — query supported event types from host |
| [ ] | [ ] | `transport-control` — plugin control of host transport (draft) |

---

## DSP Crate

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
