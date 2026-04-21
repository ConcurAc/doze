use clap_sys::events::{clap_event_header, clap_input_events};

use doze_plugin::{
    events::{Event, HostEvent},
    plugin::EntityRegistry,
};

use crate::{ClapId, events::ClapEventBorrow};

// --- Raw iterator over clap_input_events ---

struct RawClapEventIter {
    ptr: *const clap_input_events,
    get_event: unsafe extern "C" fn(*const clap_input_events, u32) -> *const clap_event_header,
    index: u32,
    size: u32,
}

impl RawClapEventIter {
    unsafe fn new(events: &clap_input_events) -> Option<Self> {
        let get_size = events.size?;
        let get_event = events.get?;
        let ptr = events as *const _;
        let size = unsafe { (get_size)(ptr) };
        Some(Self {
            ptr,
            get_event,
            index: 0,
            size,
        })
    }

    unsafe fn next_header(&mut self) -> Option<&clap_event_header> {
        loop {
            if self.index >= self.size {
                return None;
            }
            let i = self.index;
            self.index += 1;
            if let Some(header) = unsafe { (self.get_event)(self.ptr, i).as_ref() } {
                return Some(header);
            }
        }
    }
}

// --- Host event iterator ---

pub struct ClapHostEventIter<'r> {
    raw: RawClapEventIter,
    entities: &'r EntityRegistry<ClapId>,
}

impl<'r> ClapHostEventIter<'r> {
    pub unsafe fn new(
        input_events: &clap_input_events,
        entities: &'r EntityRegistry<ClapId>,
    ) -> Option<Self> {
        Some(Self {
            raw: unsafe { RawClapEventIter::new(input_events)? },
            entities,
        })
    }
}

impl<'r> Iterator for ClapHostEventIter<'r> {
    type Item = Event<HostEvent>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let header = unsafe { self.raw.next_header()? };
            let event = unsafe { ClapEventBorrow::from_header(header) }
                .and_then(|e| e.into_host_event(self.entities));
            if let Some(event) = event {
                return Some(event);
            }
        }
    }
}
