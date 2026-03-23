use clap_sys::events::{
    CLAP_EVENT_PARAM_GESTURE_BEGIN, CLAP_EVENT_PARAM_GESTURE_END, CLAP_EVENT_PARAM_VALUE,
    clap_event_param_gesture, clap_event_param_mod, clap_event_param_value,
};
use doze_plugin::{
    events::{Event, HostEvent, HostParamEvent, NoteContext, PluginEvent, PluginParamEvent},
    plugin::EntityRegistry,
};

use super::{CLAP_NOTE_WILDCARD, ClapEvent, ClapEventFlags, make_header};
use crate::ClapId;

// --- Shared helpers ---

fn note_context_from_clap(note_id: i32, port_index: i16, channel: i16, key: i16) -> NoteContext {
    NoteContext {
        note_id: (note_id != CLAP_NOTE_WILDCARD).then_some(note_id as u32),
        port: (port_index != CLAP_NOTE_WILDCARD as i16).then_some(port_index as u16),
        channel: (channel != CLAP_NOTE_WILDCARD as i16).then_some(channel as u8),
        key: (key != CLAP_NOTE_WILDCARD as i16).then_some(key as u8),
    }
}

fn note_context_to_clap(context: &NoteContext) -> (i32, i16, i16, i16) {
    (
        context
            .note_id
            .map(|v| v as i32)
            .unwrap_or(CLAP_NOTE_WILDCARD),
        context
            .port
            .map(|v| v as i16)
            .unwrap_or(CLAP_NOTE_WILDCARD as i16),
        context
            .channel
            .map(|v| v as i16)
            .unwrap_or(CLAP_NOTE_WILDCARD as i16),
        context
            .key
            .map(|v| v as i16)
            .unwrap_or(CLAP_NOTE_WILDCARD as i16),
    )
}

// --- CLAP → Host ---

pub fn clap_param_value_to_host_event<'r>(
    e: &clap_event_param_value,
    registry: &'r EntityRegistry<ClapId>,
) -> Option<Event<HostEvent<'r>>> {
    let id = registry.get(&e.param_id.into())?.identifier.downgrade();
    let context = note_context_from_clap(e.note_id, e.port_index, e.channel, e.key);
    Some(Event {
        sample_offset: e.header.time,
        flags: ClapEventFlags::from_bits_truncate(e.header.flags).into(),
        event: HostEvent::Param(HostParamEvent::Value {
            id,
            value: e.value,
            context,
        }),
    })
}

pub fn clap_param_mod_to_host_event<'r>(
    e: &clap_event_param_mod,
    registry: &'r EntityRegistry<ClapId>,
) -> Option<Event<HostEvent<'r>>> {
    let id = registry.get(&e.param_id.into())?.identifier.downgrade();
    let context = note_context_from_clap(e.note_id, e.port_index, e.channel, e.key);
    Some(Event {
        sample_offset: e.header.time,
        flags: ClapEventFlags::from_bits_truncate(e.header.flags).into(),
        event: HostEvent::Param(HostParamEvent::Mod {
            id,
            amount: e.amount,
            context,
        }),
    })
}

// --- Plugin → CLAP ---

pub fn param_event_to_clap(event: &Event<PluginEvent<'_>>) -> Option<ClapEvent> {
    match &event.event {
        PluginEvent::Param(PluginParamEvent::Value { id, value, context }) => {
            let (note_id, port_index, channel, key) = note_context_to_clap(context);
            Some(ClapEvent::ParamValue(clap_event_param_value {
                header: make_header::<clap_event_param_value>(
                    CLAP_EVENT_PARAM_VALUE,
                    event.sample_offset,
                    event.flags.into(),
                ),
                param_id: ClapId::from(*id).get(),
                cookie: std::ptr::null_mut(),
                note_id,
                port_index,
                channel,
                key,
                value: *value,
            }))
        }
        PluginEvent::Param(PluginParamEvent::GestureBegin { id }) => {
            Some(ClapEvent::ParamGesture(clap_event_param_gesture {
                header: make_header::<clap_event_param_gesture>(
                    CLAP_EVENT_PARAM_GESTURE_BEGIN,
                    event.sample_offset,
                    event.flags.into(),
                ),
                param_id: ClapId::from(*id).get(),
            }))
        }
        PluginEvent::Param(PluginParamEvent::GestureEnd { id }) => {
            Some(ClapEvent::ParamGesture(clap_event_param_gesture {
                header: make_header::<clap_event_param_gesture>(
                    CLAP_EVENT_PARAM_GESTURE_END,
                    event.sample_offset,
                    event.flags.into(),
                ),
                param_id: ClapId::from(*id).get(),
            }))
        }
        _ => None,
    }
}
