use clap_sys::events::{
    CLAP_EVENT_MIDI, CLAP_EVENT_NOTE_CHOKE, CLAP_EVENT_NOTE_END, CLAP_EVENT_NOTE_EXPRESSION,
    CLAP_EVENT_NOTE_OFF, CLAP_EVENT_NOTE_ON, CLAP_NOTE_EXPRESSION_BRIGHTNESS,
    CLAP_NOTE_EXPRESSION_EXPRESSION, CLAP_NOTE_EXPRESSION_PAN, CLAP_NOTE_EXPRESSION_PRESSURE,
    CLAP_NOTE_EXPRESSION_TUNING, CLAP_NOTE_EXPRESSION_VIBRATO, CLAP_NOTE_EXPRESSION_VOLUME,
    clap_event_midi, clap_event_note, clap_event_note_expression,
};

use doze_common::midi::channel_event::{
    CHANNEL_KEY_PRESSURE, CONTROL_CHANGE, EVENT_TYPE_MASK, MIDI_CHANNEL_MASK, NOTE_OFF, NOTE_ON,
    PITCH_BEND_CHANGE, POLYPHONIC_KEY_PRESSURE, PROGRAM_CHANGE,
};

use doze_plugin::events::{
    Event, HostEvent, HostMidiEvent, MidiNote, MidiPort, NoteExpression, PluginEvent,
    PluginMidiEvent,
};

use super::{CLAP_NOTE_WILDCARD, ClapEvent, ClapEventFlags, make_header};

// System realtime — not a channel event, not in midi-consts
const MIDI_CLOCK: u8 = 0xF8;

const MIDI_MAX_VALUE: f64 = 127.0;
const MIDI_MAX_VALUE_F32: f32 = 127.0;
const MIDI_PITCH_BEND_CENTER: i16 = 8192;
const MIDI_PITCH_BEND_MSB_SHIFT: u8 = 7;

// --- Shared helpers ---

fn midi_note_from_clap_note(e: &clap_event_note) -> MidiNote {
    MidiNote {
        port: e.port_index as u8,
        channel: e.channel as u8,
        key: e.key as u8,
        note_id: (e.note_id != CLAP_NOTE_WILDCARD).then_some(e.note_id as u32),
    }
}

fn midi_note_from_clap_note_expression(e: &clap_event_note_expression) -> MidiNote {
    MidiNote {
        port: e.port_index as u8,
        channel: e.channel as u8,
        key: e.key as u8,
        note_id: (e.note_id != CLAP_NOTE_WILDCARD).then_some(e.note_id as u32),
    }
}

fn note_expression_id(expression: &NoteExpression) -> i32 {
    match expression {
        NoteExpression::Volume => CLAP_NOTE_EXPRESSION_VOLUME,
        NoteExpression::Pan => CLAP_NOTE_EXPRESSION_PAN,
        NoteExpression::Tuning => CLAP_NOTE_EXPRESSION_TUNING,
        NoteExpression::Vibrato => CLAP_NOTE_EXPRESSION_VIBRATO,
        NoteExpression::Expression => CLAP_NOTE_EXPRESSION_EXPRESSION,
        NoteExpression::Brightness => CLAP_NOTE_EXPRESSION_BRIGHTNESS,
        NoteExpression::Pressure => CLAP_NOTE_EXPRESSION_PRESSURE,
        NoteExpression::Unknown(id) => *id as i32,
    }
}

fn clap_expression_id_to_note_expression(id: i32) -> NoteExpression {
    match id {
        CLAP_NOTE_EXPRESSION_VOLUME => NoteExpression::Volume,
        CLAP_NOTE_EXPRESSION_PAN => NoteExpression::Pan,
        CLAP_NOTE_EXPRESSION_TUNING => NoteExpression::Tuning,
        CLAP_NOTE_EXPRESSION_VIBRATO => NoteExpression::Vibrato,
        CLAP_NOTE_EXPRESSION_EXPRESSION => NoteExpression::Expression,
        CLAP_NOTE_EXPRESSION_BRIGHTNESS => NoteExpression::Brightness,
        CLAP_NOTE_EXPRESSION_PRESSURE => NoteExpression::Pressure,
        id => NoteExpression::Unknown(id as u32),
    }
}

// --- CLAP → Host ---

pub fn clap_note_to_host_event<'r>(e: &clap_event_note) -> Option<Event<HostEvent<'r>>> {
    let note = midi_note_from_clap_note(e);
    let midi_event = match e.header.type_ as u16 {
        CLAP_EVENT_NOTE_ON => HostMidiEvent::NoteOn {
            note,
            velocity: e.velocity,
        },
        CLAP_EVENT_NOTE_OFF => HostMidiEvent::NoteOff {
            note,
            velocity: e.velocity,
        },
        CLAP_EVENT_NOTE_CHOKE => HostMidiEvent::NoteChoke { note },
        _ => return None,
    };
    Some(Event {
        sample_offset: e.header.time,
        flags: ClapEventFlags::from_bits_truncate(e.header.flags).into(),
        event: HostEvent::Midi(midi_event),
    })
}

pub fn clap_note_expression_to_host_event<'r>(
    e: &clap_event_note_expression,
) -> Option<Event<HostEvent<'r>>> {
    Some(Event {
        sample_offset: e.header.time,
        flags: ClapEventFlags::from_bits_truncate(e.header.flags).into(),
        event: HostEvent::Midi(HostMidiEvent::NoteExpression {
            note: midi_note_from_clap_note_expression(e),
            expression: clap_expression_id_to_note_expression(e.expression_id),
            value: e.value,
        }),
    })
}

pub fn clap_midi_to_host_event<'r>(e: &clap_event_midi) -> Option<Event<HostEvent<'r>>> {
    Some(Event {
        sample_offset: e.header.time,
        flags: ClapEventFlags::from_bits_truncate(e.header.flags).into(),
        event: HostEvent::Midi(clap_midi_bytes_to_host_midi_event(e)?),
    })
}

fn clap_midi_bytes_to_host_midi_event(e: &clap_event_midi) -> Option<HostMidiEvent> {
    let status = e.data[0];
    let data1 = e.data[1];
    let data2 = e.data[2];
    let port = e.port_index as u8;

    if status == MIDI_CLOCK {
        return Some(HostMidiEvent::Clock { port });
    }

    let channel = status & MIDI_CHANNEL_MASK;
    let event = match status & EVENT_TYPE_MASK {
        NOTE_ON if data2 > 0 => HostMidiEvent::NoteOn {
            note: MidiNote {
                port,
                channel,
                key: data1,
                note_id: None,
            },
            velocity: data2 as f64 / MIDI_MAX_VALUE,
        },
        NOTE_OFF | NOTE_ON => HostMidiEvent::NoteOff {
            note: MidiNote {
                port,
                channel,
                key: data1,
                note_id: None,
            },
            velocity: data2 as f64 / MIDI_MAX_VALUE,
        },
        PITCH_BEND_CHANGE => HostMidiEvent::PitchBend {
            port: MidiPort { port, channel },
            value: {
                let raw = ((data2 as u16) << MIDI_PITCH_BEND_MSB_SHIFT) | (data1 as u16);
                (raw as f32 - MIDI_PITCH_BEND_CENTER as f32) / MIDI_PITCH_BEND_CENTER as f32
            },
        },
        POLYPHONIC_KEY_PRESSURE => HostMidiEvent::Pressure {
            note: MidiNote {
                port,
                channel,
                key: data1,
                note_id: None,
            },
            value: data2 as f32 / MIDI_MAX_VALUE_F32,
        },
        CHANNEL_KEY_PRESSURE => HostMidiEvent::ChannelPressure {
            port: MidiPort { port, channel },
            value: data1 as f32 / MIDI_MAX_VALUE_F32,
        },
        PROGRAM_CHANGE => HostMidiEvent::ProgramChange {
            port: MidiPort { port, channel },
            program: data1,
        },
        CONTROL_CHANGE => HostMidiEvent::ControlChange {
            port: MidiPort { port, channel },
            control_change: data1,
            value: data2,
        },
        _ => return None,
    };
    Some(event)
}

// --- Plugin → CLAP ---

pub fn midi_event_to_clap(event: &Event<PluginEvent<'_>>) -> Option<ClapEvent> {
    let PluginEvent::Midi(midi) = &event.event else {
        return None;
    };
    match midi {
        PluginMidiEvent::NoteOn { .. }
        | PluginMidiEvent::NoteOff { .. }
        | PluginMidiEvent::NoteEnd { .. } => {
            clap_note_from_plugin_event(event, midi).map(ClapEvent::Note)
        }
        PluginMidiEvent::NoteExpression { .. } => {
            clap_note_expression_from_plugin_event(event, midi).map(ClapEvent::NoteExpression)
        }
        _ => clap_midi_bytes_from_plugin_event(event, midi).map(ClapEvent::Midi),
    }
}

fn clap_note_from_plugin_event(
    event: &Event<PluginEvent<'_>>,
    midi: &PluginMidiEvent,
) -> Option<clap_event_note> {
    let (note, velocity, type_) = match midi {
        PluginMidiEvent::NoteOn { note, velocity } => (note, *velocity, CLAP_EVENT_NOTE_ON),
        PluginMidiEvent::NoteOff { note, velocity } => (note, *velocity, CLAP_EVENT_NOTE_OFF),
        PluginMidiEvent::NoteEnd { note } => (note, 0.0, CLAP_EVENT_NOTE_END),
        _ => return None,
    };
    Some(clap_event_note {
        header: make_header::<clap_event_note>(type_, event.sample_offset, event.flags.into()),
        note_id: note.note_id.map(|v| v as i32).unwrap_or(CLAP_NOTE_WILDCARD),
        port_index: note.port as i16,
        channel: note.channel as i16,
        key: note.key as i16,
        velocity,
    })
}

fn clap_note_expression_from_plugin_event(
    event: &Event<PluginEvent<'_>>,
    midi: &PluginMidiEvent,
) -> Option<clap_event_note_expression> {
    let PluginMidiEvent::NoteExpression {
        note,
        expression,
        value,
    } = midi
    else {
        return None;
    };
    Some(clap_event_note_expression {
        header: make_header::<clap_event_note_expression>(
            CLAP_EVENT_NOTE_EXPRESSION,
            event.sample_offset,
            event.flags.into(),
        ),
        expression_id: note_expression_id(expression),
        note_id: note.note_id.map(|v| v as i32).unwrap_or(CLAP_NOTE_WILDCARD),
        port_index: note.port as i16,
        channel: note.channel as i16,
        key: note.key as i16,
        value: *value,
    })
}

fn clap_midi_bytes_from_plugin_event(
    event: &Event<PluginEvent<'_>>,
    midi: &PluginMidiEvent,
) -> Option<clap_event_midi> {
    let (port_index, data) = match midi {
        PluginMidiEvent::ControlChange {
            port,
            control_change,
            value,
        } => (
            port.port,
            [CONTROL_CHANGE | port.channel, *control_change, *value],
        ),
        PluginMidiEvent::PitchBend { port, value } => {
            let raw =
                (*value * MIDI_PITCH_BEND_CENTER as f32 + MIDI_PITCH_BEND_CENTER as f32) as u16;
            let lsb = (raw & 0x7F) as u8;
            let msb = ((raw >> MIDI_PITCH_BEND_MSB_SHIFT) & 0x7F) as u8;
            (port.port, [PITCH_BEND_CHANGE | port.channel, lsb, msb])
        }
        PluginMidiEvent::Pressure { note, value } => (
            note.port,
            [
                POLYPHONIC_KEY_PRESSURE | note.channel,
                note.key,
                (*value * MIDI_MAX_VALUE_F32) as u8,
            ],
        ),
        PluginMidiEvent::ChannelPressure { port, value } => (
            port.port,
            [
                CHANNEL_KEY_PRESSURE | port.channel,
                (*value * MIDI_MAX_VALUE_F32) as u8,
                0,
            ],
        ),
        PluginMidiEvent::ProgramChange { port, program } => {
            (port.port, [PROGRAM_CHANGE | port.channel, *program, 0])
        }
        PluginMidiEvent::Clock { port } => (*port, [MIDI_CLOCK, 0, 0]),
        _ => return None,
    };
    Some(clap_event_midi {
        header: make_header::<clap_event_midi>(
            CLAP_EVENT_MIDI,
            event.sample_offset,
            event.flags.into(),
        ),
        port_index: port_index as u16,
        data,
    })
}
