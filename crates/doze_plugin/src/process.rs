use doze_common::{
    io::{SliceReader, SliceWriter},
    sample::Sample,
};

use crate::events::{Event, EventSender, HostEvent, PluginEvent};

const MAX_CONSTANT_CHANNEL: usize = 64;

pub trait AudioBus<'h> {
    fn get_f32_buffer(&self, index: usize) -> Result<AudioBuffer<'h, f32>, AudioBufferError>;
    fn get_f64_buffer(&self, index: usize) -> Result<AudioBuffer<'h, f64>, AudioBufferError>;
}

pub struct AudioBuffer<'p, T: Sample> {
    channels: &'p [*mut T],
    frames: usize,
    constant_mask: u64,
    latency: u32,
}

impl<'p, T: Sample> AudioBuffer<'p, T> {
    pub fn get_reader(&self, channel: usize) -> Option<SliceReader<'p, T>> {
        let buffer = self
            .channels
            .get(channel)
            .map(|&d| unsafe { core::slice::from_raw_parts(d as *const _, self.frames) })?;

        Some(buffer.into())
    }
    pub fn get_writer(&mut self, channel: usize) -> Option<SliceWriter<'p, T>> {
        let buffer = self
            .channels
            .get(channel)
            .map(|&d| unsafe { core::slice::from_raw_parts_mut(d, self.frames) })?;

        Some(buffer.into())
    }
    pub fn set_constant(&mut self, channel: usize) {
        debug_assert!(channel < MAX_CONSTANT_CHANNEL);
        if channel < MAX_CONSTANT_CHANNEL {
            self.constant_mask |= 1 << channel;
        }
    }
    pub fn unset_constant(&mut self, channel: usize) {
        debug_assert!(channel < MAX_CONSTANT_CHANNEL);
        if channel < MAX_CONSTANT_CHANNEL {
            self.constant_mask |= 0 << channel;
        }
    }
    pub fn is_constant(&self, channel: usize) -> bool {
        debug_assert!(channel < MAX_CONSTANT_CHANNEL);
        if channel < MAX_CONSTANT_CHANNEL {
            self.constant_mask & (1 << channel) != 0
        } else {
            false
        }
    }
    pub fn set_latency(&mut self, latency: u32) {
        self.latency = latency;
    }
}

impl<'h, T: Sample> AudioBuffer<'h, T> {
    pub unsafe fn new(
        channels: &'h [*mut T],
        frames: usize,
        constant_mask: u64,
        latency: u32,
    ) -> Self {
        Self {
            channels,
            frames,
            constant_mask,
            latency,
        }
    }
}

pub enum AudioBufferError {
    NotFound,
    IsF32,
    IsF64,
    Null,
}

pub struct Process<'h, 'p> {
    pub frames: usize,
    pub steady_time: i64,

    pub audio_inputs: &'h dyn AudioBus<'h>,
    pub audio_outputs: &'h dyn AudioBus<'h>,

    pub events: &'h mut dyn Iterator<Item = Event<HostEvent<'h>>>,
    pub sender: &'h mut dyn EventSender<Event = Event<PluginEvent<'p>>>,
}

pub enum Status {
    Continue,
    ContinueIfNotQuiet,
    Tail,
    Sleep,
    Error,
}
