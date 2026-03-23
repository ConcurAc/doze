use std::marker::PhantomData;

use clap_sys::{
    audio_buffer::clap_audio_buffer,
    process::{
        CLAP_PROCESS_CONTINUE, CLAP_PROCESS_CONTINUE_IF_NOT_QUIET, CLAP_PROCESS_ERROR,
        CLAP_PROCESS_SLEEP, CLAP_PROCESS_TAIL, clap_process, clap_process_status,
    },
};

use doze_plugin::{
    events::{Event, HostEvent},
    process::{AudioBuffer, AudioBufferError, AudioBus, Process, Status},
};

use crate::events::ClapEventSender;

pub struct ClapProcess<'h, 'p> {
    frames: usize,
    steady_time: i64,

    audio_inputs: ClapAudioInput<'h>,
    audio_outputs: ClapAudioOutput<'h>,

    events: &'h mut dyn Iterator<Item = Event<HostEvent<'h>>>,
    sender: ClapEventSender<'h, 'p>,
}

impl<'h, 'p> Into<Process<'h, 'p>> for &'p mut ClapProcess<'h, 'p> {
    fn into(self) -> Process<'h, 'p> {
        Process {
            frames: self.frames,
            steady_time: self.steady_time,
            audio_inputs: &self.audio_inputs,
            audio_outputs: &self.audio_outputs,
            events: &mut self.events,
            sender: &mut self.sender,
        }
    }
}

impl<'h, 'p> ClapProcess<'h, 'p> {
    pub fn new(
        clap_process: &'h clap_process,
        events: &'h mut dyn Iterator<Item = Event<HostEvent<'h>>>,
        sender: ClapEventSender<'h, 'p>,
    ) -> Option<Self> {
        let frames = clap_process.frames_count as usize;
        Some(Self {
            frames,
            steady_time: clap_process.steady_time,
            audio_inputs: ClapAudioInput::from(clap_process),
            audio_outputs: ClapAudioOutput::from(clap_process),
            events,
            sender,
        })
    }
}

pub struct ClapAudioBus<'h, D> {
    buffers: &'h [clap_audio_buffer],
    frames: usize,

    _direction: PhantomData<D>,
}

impl<'h, D> AudioBus<'h> for ClapAudioBus<'_, D> {
    fn get_f32_buffer(&self, index: usize) -> Result<AudioBuffer<'h, f32>, AudioBufferError> {
        let Some(buffer) = self.buffers.get(index) else {
            return Err(AudioBufferError::NotFound);
        };

        if buffer.data32.is_null() {
            return if buffer.data64.is_null() {
                Err(AudioBufferError::Null)
            } else {
                Err(AudioBufferError::IsF64)
            };
        }

        Ok(unsafe {
            AudioBuffer::new(
                core::slice::from_raw_parts_mut(buffer.data32, buffer.channel_count as usize),
                self.frames,
                buffer.constant_mask,
                buffer.latency,
            )
        })
    }
    fn get_f64_buffer(&self, index: usize) -> Result<AudioBuffer<'h, f64>, AudioBufferError> {
        let Some(buffer) = self.buffers.get(index) else {
            return Err(AudioBufferError::NotFound);
        };

        if buffer.data64.is_null() {
            return if buffer.data32.is_null() {
                Err(AudioBufferError::Null)
            } else {
                Err(AudioBufferError::IsF32)
            };
        }

        Ok(unsafe {
            AudioBuffer::new(
                core::slice::from_raw_parts_mut(buffer.data64, buffer.channel_count as usize),
                self.frames,
                buffer.constant_mask,
                buffer.latency,
            )
        })
    }
}

pub type ClapAudioInput<'p> = ClapAudioBus<'p, Input>;

pub struct Input;

impl<'p> From<&'p clap_process> for ClapAudioBus<'p, Input> {
    fn from(process: &'p clap_process) -> Self {
        Self {
            buffers: unsafe {
                core::slice::from_raw_parts(
                    process.audio_inputs,
                    process.audio_inputs_count as usize,
                )
            },
            frames: process.frames_count as usize,
            _direction: PhantomData,
        }
    }
}

pub type ClapAudioOutput<'h> = ClapAudioBus<'h, Output>;

pub struct Output;

impl<'h> From<&'h clap_process> for ClapAudioOutput<'h> {
    fn from(process: &'h clap_process) -> Self {
        Self {
            buffers: unsafe {
                core::slice::from_raw_parts(
                    process.audio_outputs,
                    process.audio_outputs_count as usize,
                )
            },
            frames: process.frames_count as usize,
            _direction: PhantomData,
        }
    }
}

pub fn status_to_clap(status: Status) -> clap_process_status {
    match status {
        Status::Continue => CLAP_PROCESS_CONTINUE,
        Status::ContinueIfNotQuiet => CLAP_PROCESS_CONTINUE_IF_NOT_QUIET,
        Status::Tail => CLAP_PROCESS_TAIL,
        Status::Sleep => CLAP_PROCESS_SLEEP,
        Status::Error => CLAP_PROCESS_ERROR,
    }
}
