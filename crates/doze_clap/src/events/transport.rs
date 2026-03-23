use clap_sys::{
    events::clap_event_transport,
    fixedpoint::{CLAP_BEATTIME_FACTOR, CLAP_SECTIME_FACTOR},
};
use doze_plugin::events::{
    BeatsTimeline, Event, HostEvent, SecondsTimeline, Tempo, TimeSignature, TransportEvent,
    TransportFlags,
};

use super::ClapEventFlags;

const BEATTIME_FACTOR: f64 = CLAP_BEATTIME_FACTOR as f64;
const SECTIME_FACTOR: f64 = CLAP_SECTIME_FACTOR as f64;

// --- CLAP → Host ---

pub fn clap_transport_to_host_event<'r>(e: &clap_event_transport) -> Event<HostEvent<'r>> {
    Event {
        sample_offset: e.header.time,
        flags: ClapEventFlags::from_bits_truncate(e.header.flags).into(),
        event: HostEvent::Transport(clap_transport_to_transport_event(e)),
    }
}

fn clap_transport_to_transport_event(e: &clap_event_transport) -> TransportEvent {
    let flags = ClapTransportFlags::from_bits_truncate(e.flags);
    TransportEvent {
        sample_offset: e.header.time,
        flags: flags.into(),
        bpm: flags
            .contains(ClapTransportFlags::HAS_TEMPO)
            .then_some(Tempo {
                bpm: e.tempo,
                bpm_increment: e.tempo_inc,
            }),
        beats: flags
            .contains(ClapTransportFlags::HAS_BEATS_TIMELINE)
            .then_some(BeatsTimeline {
                song_position: e.song_pos_beats as f64 / BEATTIME_FACTOR,
                bar_start: e.bar_start as f64 / BEATTIME_FACTOR,
                bar_number: e.bar_number,
                loop_start: e.loop_start_beats as f64 / BEATTIME_FACTOR,
                loop_end: e.loop_end_beats as f64 / BEATTIME_FACTOR,
            }),
        seconds: flags
            .contains(ClapTransportFlags::HAS_SECONDS_TIMELINE)
            .then_some(SecondsTimeline {
                song_pos: e.song_pos_seconds as f64 / SECTIME_FACTOR,
                loop_start: e.loop_start_seconds as f64 / SECTIME_FACTOR,
                loop_end: e.loop_end_seconds as f64 / SECTIME_FACTOR,
            }),
        time_signature: flags
            .contains(ClapTransportFlags::HAS_TIME_SIGNATURE)
            .then_some(TimeSignature {
                numerator: e.tsig_num,
                denominator: e.tsig_denom,
            }),
    }
}

// --- Flags ---

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct ClapTransportFlags: u32 {
        const PLAYING              = clap_sys::events::CLAP_TRANSPORT_IS_PLAYING;
        const RECORDING            = clap_sys::events::CLAP_TRANSPORT_IS_RECORDING;
        const LOOP_ACTIVE          = clap_sys::events::CLAP_TRANSPORT_IS_LOOP_ACTIVE;
        const WITHIN_PRE_ROLL      = clap_sys::events::CLAP_TRANSPORT_IS_WITHIN_PRE_ROLL;
        const HAS_TEMPO            = clap_sys::events::CLAP_TRANSPORT_HAS_TEMPO;
        const HAS_BEATS_TIMELINE   = clap_sys::events::CLAP_TRANSPORT_HAS_BEATS_TIMELINE;
        const HAS_SECONDS_TIMELINE = clap_sys::events::CLAP_TRANSPORT_HAS_SECONDS_TIMELINE;
        const HAS_TIME_SIGNATURE   = clap_sys::events::CLAP_TRANSPORT_HAS_TIME_SIGNATURE;
    }
}

const TRANSPORT_MAPPING: &[(TransportFlags, ClapTransportFlags)] = &[
    (TransportFlags::PLAYING, ClapTransportFlags::PLAYING),
    (TransportFlags::RECORDING, ClapTransportFlags::RECORDING),
    (TransportFlags::LOOP_ACTIVE, ClapTransportFlags::LOOP_ACTIVE),
    (
        TransportFlags::WITHIN_PRE_ROLL,
        ClapTransportFlags::WITHIN_PRE_ROLL,
    ),
];

impl From<ClapTransportFlags> for TransportFlags {
    fn from(value: ClapTransportFlags) -> Self {
        TRANSPORT_MAPPING
            .iter()
            .filter(|(_, c)| value.contains(*c))
            .fold(TransportFlags::empty(), |acc, (f, _)| acc | *f)
    }
}

impl From<TransportFlags> for ClapTransportFlags {
    fn from(value: TransportFlags) -> Self {
        TRANSPORT_MAPPING
            .iter()
            .filter(|(f, _)| value.contains(*f))
            .fold(ClapTransportFlags::empty(), |acc, (_, c)| acc | *c)
    }
}
