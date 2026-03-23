use doze_common::identifier::WeakIdentifier;

#[derive(Debug, Clone)]
pub struct Event<E> {
    pub sample_offset: u32,
    pub flags: EventFlags,
    pub event: E,
}

#[derive(Debug, Clone)]
pub enum HostEvent<'i> {
    Param(HostParamEvent<'i>),
    Midi(HostMidiEvent),
    Transport(TransportEvent),
}

#[derive(Debug, Clone)]
pub enum PluginEvent<'i> {
    Param(PluginParamEvent<'i>),
    Midi(PluginMidiEvent),
}

#[derive(Debug, Clone)]
pub enum HostParamEvent<'i> {
    Value {
        id: WeakIdentifier<'i>,
        value: f64,
        context: NoteContext,
    },
    Mod {
        id: WeakIdentifier<'i>,
        amount: f64,
        context: NoteContext,
    },
}

#[derive(Debug, Clone)]
pub enum PluginParamEvent<'i> {
    Value {
        id: WeakIdentifier<'i>,
        value: f64,
        context: NoteContext,
    },
    GestureBegin {
        id: WeakIdentifier<'i>,
    },
    GestureEnd {
        id: WeakIdentifier<'i>,
    },
}

#[derive(Debug, Clone, Copy)]
pub struct NoteContext {
    pub note_id: Option<u32>,
    pub port: Option<u16>,
    pub channel: Option<u8>,
    pub key: Option<u8>,
}

#[derive(Debug, Clone)]
pub enum HostMidiEvent {
    NoteOn {
        note: MidiNote,
        velocity: f64,
    },
    NoteOff {
        note: MidiNote,
        velocity: f64,
    },
    NoteChoke {
        note: MidiNote,
    },
    NoteExpression {
        note: MidiNote,
        expression: NoteExpression,
        value: f64,
    },
    ControlChange {
        port: MidiPort,
        control_change: u8,
        value: u8,
    },
    PitchBend {
        port: MidiPort,
        value: f32,
    },
    Pressure {
        note: MidiNote,
        value: f32,
    },
    ChannelPressure {
        port: MidiPort,
        value: f32,
    },
    ProgramChange {
        port: MidiPort,
        program: u8,
    },
    Clock {
        port: u8,
    },
}

#[derive(Debug, Clone)]
pub enum PluginMidiEvent {
    NoteOn {
        note: MidiNote,
        velocity: f64,
    },
    NoteOff {
        note: MidiNote,
        velocity: f64,
    },
    NoteEnd {
        note: MidiNote,
    },
    NoteExpression {
        note: MidiNote,
        expression: NoteExpression,
        value: f64,
    },
    ControlChange {
        port: MidiPort,
        control_change: u8,
        value: u8,
    },
    PitchBend {
        port: MidiPort,
        value: f32,
    },
    Pressure {
        note: MidiNote,
        value: f32,
    },
    ChannelPressure {
        port: MidiPort,
        value: f32,
    },
    ProgramChange {
        port: MidiPort,
        program: u8,
    },
    Clock {
        port: u8,
    },
}

#[derive(Debug, Clone, Copy)]
pub struct MidiPort {
    pub port: u8,
    pub channel: u8,
}

#[derive(Debug, Clone)]
pub struct MidiNote {
    pub port: u8,
    pub channel: u8,
    pub key: u8,
    pub note_id: Option<u32>,
}

#[derive(Debug, Clone, Copy)]
pub enum NoteExpression {
    Volume,
    Pan,
    Tuning,
    Vibrato,
    Expression,
    Brightness,
    Pressure,
    Unknown(u32),
}

#[derive(Debug, Clone)]
pub struct TransportEvent {
    pub sample_offset: u32,
    pub flags: TransportFlags,
    pub bpm: Option<Tempo>,
    pub beats: Option<BeatsTimeline>,
    pub seconds: Option<SecondsTimeline>,
    pub time_signature: Option<TimeSignature>,
}

#[derive(Debug, Clone)]
pub struct Tempo {
    pub bpm: f64,
    pub bpm_increment: f64,
}

#[derive(Debug, Clone)]
pub struct BeatsTimeline {
    pub song_position: f64,
    pub bar_start: f64,
    pub bar_number: i32,
    pub loop_start: f64,
    pub loop_end: f64,
}

#[derive(Debug, Clone)]
pub struct SecondsTimeline {
    pub song_pos: f64,
    pub loop_start: f64,
    pub loop_end: f64,
}

#[derive(Debug, Clone)]
pub struct TimeSignature {
    pub numerator: u16,
    pub denominator: u16,
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct TransportFlags: u32 {
        const PLAYING         = 1 << 0;
        const RECORDING       = 1 << 1;
        const LOOP_ACTIVE     = 1 << 2;
        const WITHIN_PRE_ROLL = 1 << 3;
    }
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct EventFlags: u32 {
        const IS_LIVE       = 1 << 0;
        const DONT_RECORD   = 1 << 1;
    }
}

pub trait EventSender {
    type Event;
    fn send(&mut self, event: Self::Event) -> Option<()>;
}
