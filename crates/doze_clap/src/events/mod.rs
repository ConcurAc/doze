pub mod iter;
pub mod midi;
pub mod param;
pub mod transport;

use clap_sys::events::{
    CLAP_CORE_EVENT_SPACE_ID, CLAP_EVENT_DONT_RECORD, CLAP_EVENT_IS_LIVE, CLAP_EVENT_MIDI,
    CLAP_EVENT_NOTE_CHOKE, CLAP_EVENT_NOTE_END, CLAP_EVENT_NOTE_EXPRESSION, CLAP_EVENT_NOTE_OFF,
    CLAP_EVENT_NOTE_ON, CLAP_EVENT_PARAM_GESTURE_BEGIN, CLAP_EVENT_PARAM_GESTURE_END,
    CLAP_EVENT_PARAM_MOD, CLAP_EVENT_PARAM_VALUE, CLAP_EVENT_TRANSPORT, clap_event_header,
    clap_event_midi, clap_event_note, clap_event_note_expression, clap_event_param_gesture,
    clap_event_param_mod, clap_event_param_value, clap_event_transport, clap_output_events,
};

use doze_common::storage::alloc::History;

use doze_plugin::{
    events::{Event, EventFlags, EventSender, HostEvent, PluginEvent},
    plugin::EntityRegistry,
};

use crate::ClapId;

pub(super) const CLAP_NOTE_WILDCARD: i32 = -1;

// --- Shared header constructor ---

pub(super) fn make_header<T>(type_: u16, time: u32, flags: ClapEventFlags) -> clap_event_header {
    clap_event_header {
        size: size_of::<T>() as u32,
        time,
        space_id: CLAP_CORE_EVENT_SPACE_ID,
        type_,
        flags: flags.bits(),
    }
}

// --- CLAP event enums ---

pub enum ClapEvent {
    ParamValue(clap_event_param_value),
    ParamMod(clap_event_param_mod),
    ParamGesture(clap_event_param_gesture),
    Note(clap_event_note),
    NoteExpression(clap_event_note_expression),
    Midi(clap_event_midi),
    Transport(clap_event_transport),
}

pub enum ClapEventBorrow<'e> {
    ParamValue(&'e clap_event_param_value),
    ParamMod(&'e clap_event_param_mod),
    ParamGesture(&'e clap_event_param_gesture),
    Note(&'e clap_event_note),
    NoteExpression(&'e clap_event_note_expression),
    Midi(&'e clap_event_midi),
    Transport(&'e clap_event_transport),
}

impl ClapEvent {
    pub fn header(&self) -> *const clap_event_header {
        match self {
            ClapEvent::ParamValue(e) => &e.header,
            ClapEvent::ParamMod(e) => &e.header,
            ClapEvent::ParamGesture(e) => &e.header,
            ClapEvent::Note(e) => &e.header,
            ClapEvent::NoteExpression(e) => &e.header,
            ClapEvent::Midi(e) => &e.header,
            ClapEvent::Transport(e) => &e.header,
        }
    }
}

impl ClapEventBorrow<'_> {
    pub unsafe fn from_header(header: &clap_event_header) -> Option<Self> {
        let ptr = header as *const _;
        unsafe {
            match header.type_ as u16 {
                CLAP_EVENT_PARAM_VALUE => Some(Self::ParamValue(
                    (ptr as *const clap_event_param_value).as_ref()?,
                )),
                CLAP_EVENT_PARAM_MOD => Some(Self::ParamMod(
                    (ptr as *const clap_event_param_mod).as_ref()?,
                )),
                CLAP_EVENT_PARAM_GESTURE_BEGIN | CLAP_EVENT_PARAM_GESTURE_END => Some(
                    Self::ParamGesture((ptr as *const clap_event_param_gesture).as_ref()?),
                ),
                CLAP_EVENT_NOTE_ON
                | CLAP_EVENT_NOTE_OFF
                | CLAP_EVENT_NOTE_CHOKE
                | CLAP_EVENT_NOTE_END => {
                    Some(Self::Note((ptr as *const clap_event_note).as_ref()?))
                }
                CLAP_EVENT_NOTE_EXPRESSION => Some(Self::NoteExpression(
                    (ptr as *const clap_event_note_expression).as_ref()?,
                )),
                CLAP_EVENT_MIDI => Some(Self::Midi((ptr as *const clap_event_midi).as_ref()?)),
                CLAP_EVENT_TRANSPORT => Some(Self::Transport(
                    (ptr as *const clap_event_transport).as_ref()?,
                )),
                _ => None,
            }
        }
    }
}

impl<'r> ClapEventBorrow<'_> {
    pub fn into_host_event(
        self,
        registry: &'r EntityRegistry<ClapId>,
    ) -> Option<Event<HostEvent<'r>>> {
        match self {
            ClapEventBorrow::ParamValue(e) => param::clap_param_value_to_host_event(e, registry),
            ClapEventBorrow::ParamMod(e) => param::clap_param_mod_to_host_event(e, registry),
            ClapEventBorrow::ParamGesture(_) => None,
            ClapEventBorrow::Note(e) => midi::clap_note_to_host_event(e),
            ClapEventBorrow::NoteExpression(e) => midi::clap_note_expression_to_host_event(e),
            ClapEventBorrow::Midi(e) => midi::clap_midi_to_host_event(e),
            ClapEventBorrow::Transport(e) => Some(transport::clap_transport_to_host_event(e)),
        }
    }
}

// --- Plugin → CLAP dispatch ---

pub fn plugin_event_to_clap_event(event: &Event<PluginEvent<'_>>) -> Option<ClapEvent> {
    param::param_event_to_clap(event).or_else(|| midi::midi_event_to_clap(event))
}

// --- Flags ---

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct ClapEventFlags: u32 {
        const IS_LIVE     = CLAP_EVENT_IS_LIVE;
        const DONT_RECORD = CLAP_EVENT_DONT_RECORD;
    }
}

const CLAP_EVENT_FLAG_MAPPING: &[(EventFlags, ClapEventFlags)] = &[
    (EventFlags::IS_LIVE, ClapEventFlags::IS_LIVE),
    (EventFlags::DONT_RECORD, ClapEventFlags::DONT_RECORD),
];

impl From<ClapEventFlags> for EventFlags {
    fn from(value: ClapEventFlags) -> Self {
        CLAP_EVENT_FLAG_MAPPING
            .iter()
            .filter(|(_, c)| value.contains(*c))
            .fold(EventFlags::empty(), |acc, (f, _)| acc | *f)
    }
}

impl From<EventFlags> for ClapEventFlags {
    fn from(value: EventFlags) -> Self {
        CLAP_EVENT_FLAG_MAPPING
            .iter()
            .filter(|(f, _)| value.contains(*f))
            .fold(ClapEventFlags::empty(), |acc, (_, c)| acc | *c)
    }
}

// --- ClapEventSender ---

pub struct ClapEventSender<'h, 'p> {
    buffer: &'p mut History<ClapEvent>,
    clap_output_events: &'h clap_output_events,
    tx: &'h unsafe extern "C" fn(*const clap_output_events, *const clap_event_header) -> bool,
}

impl<'h, 'p> EventSender for ClapEventSender<'h, 'p> {
    type Event = Event<PluginEvent<'p>>;
    fn send(&mut self, event: Self::Event) -> Option<()> {
        self.buffer.write(plugin_event_to_clap_event(&event)?);
        let event = self.buffer.last()?;
        unsafe { (self.tx)(self.clap_output_events, event.header()) }.then_some(())
    }
}

impl<'h, 'p> ClapEventSender<'h, 'p> {
    pub fn new(
        output_events: &'h clap_output_events,
        buffer: &'p mut History<ClapEvent>,
    ) -> Option<Self> {
        Some(Self {
            buffer,
            clap_output_events: output_events,
            tx: output_events.try_push.as_ref()?,
        })
    }
}
