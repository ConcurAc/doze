use doze_common::{
    io::{SliceReader, SliceWriter},
    sample::Sample,
};

use crate::events::{Event, EventSender, HostEvent, PluginEvent};

const MAX_CONSTANT_CHANNEL: usize = 64;

/// Audio processing state passed to the plugin's real-time callback.
///
/// This struct contains all the data needed for a single audio processing block:
/// - Audio input and output buffers
/// - Host events to process (parameter changes, MIDI, transport updates)
/// - An event sender for the plugin to communicate back to the host
///
/// # Lifetimes
/// - `'h`: Lifetime of host-provided data (valid only during this callback)
/// - `'p`: Lifetime of plugin-specific event data
///
/// # Real-Time Safety
/// This struct is designed for real-time use. Access to audio data is safe and
/// does not require allocation.
pub struct Process<'h, 'p> {
    /// Number of audio frames in this processing block.
    ///
    /// Assume between the `min_frames_count` and `max_frames_count`
    /// specified in `Plugin::activate()`.
    pub frames: usize,

    /// Monotonically increasing sample count from the host.
    ///
    /// Represents the steady time in samples since the session started.
    /// Continues incrementing even when transport is stopped, providing
    /// a reliable timeline for sample-accurate processing.
    pub steady_time: i64,

    /// Input audio buses (one per input port).
    ///
    /// Audio data can be provided by the host as either `f32` or `f64`.
    /// The ports are declared in the `AudioPorts` extension.
    pub audio_inputs: &'h dyn AudioBus<'h>,

    /// Output audio buses (one per output port).
    ///
    /// Write processed audio here.
    /// The ports are declared in the `AudioPorts` extension.
    pub audio_outputs: &'h dyn AudioBus<'h>,

    /// Iterator over incoming host events for this processing block.
    pub events: &'h mut dyn Iterator<Item = Event<HostEvent>>,

    /// Sender for plugin events back to the host.
    pub sender: &'h mut dyn EventSender<Event = Event<PluginEvent<'p>>>,
}

/// Processing status returned from `Plugin::process()`.
///
/// Indicates to the host how the plugin is doing and what to expect next.
///
/// # Typical Usage
/// Most plugins return `Status::Continue` to indicate normal operation.
/// Special cases like reverb tails use `Tail`, and unresponsive plugins
/// should return `Error`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// Normal processing. Plugin has consumed input and produced output.
    ///
    /// The host should call `process()` again with the next audio block.
    Continue,

    /// Processing normally, but optimize if output is silent.
    ///
    /// Plugin is producing output, but the host may skip calling `process()`
    /// if it determines the output would be silent anyway (host optimization).
    /// Useful for plugins that may have silence detection.
    ContinueIfNotQuiet,

    /// Plugin is outputting a processing tail (reverb decay, delay feedback, etc.).
    ///
    /// Plugin is producing output from internal state but does not need input.
    /// The host will continue calling `process()` with empty input buffers
    /// until the plugin returns `Sleep` or `Continue`.
    /// Useful for reverbs, delays, and other processors with internal feedback.
    Tail,

    /// Plugin has no more output to produce. Host may sleep the plugin.
    ///
    /// Plugin will not produce audio until the host calls `process()` again
    /// with real input (e.g., when the user presses a MIDI key on an instrument).
    /// Useful for instruments and plugins that can be idle between events.
    Sleep,

    /// An error occurred during processing.
    ///
    /// Plugin encountered an error (buffer overrun, NaN in DSP, etc.).
    /// The host may stop calling `process()` or log the error.
    /// This status indicates the plugin is in an error state.
    Error,
}

/// Trait for accessing audio input or output buses.
///
/// A bus contains one or more audio channels (e.g., stereo = 2 channels).
/// The plugin can query the bus for buffers in different sample formats
/// (f32 or f64).
///
/// # Implementation Notes
/// Typically implemented by the host's audio processing framework.
/// Plugins do not implement this trait directly.
pub trait AudioBus<'h> {
    /// Get the number of channels in this bus.
    ///
    /// # Returns
    /// Number of channels (audio ports). For stereo: 2, for mono: 1, etc.
    fn count(&self) -> usize;

    /// Get an f32 audio buffer for the given port index.
    ///
    /// # Arguments
    /// - `index`: Port index (0 <= `index` < `count()`)
    ///
    /// # Returns
    /// - `Ok(buffer)`: Valid audio buffer for this port
    /// - `Err(NotFound)`: Port index out of range
    /// - `Err(IsF64)`: This port uses `f64` format, not `f32`
    /// - `Err(Null)`: Buffer pointer is null (should not happen)
    fn get_f32_buffer(&self, index: usize) -> Result<AudioBuffer<'h, f32>, AudioBufferError>;

    /// Get an f64 audio buffer for the given port index.
    ///
    /// # Arguments
    /// - `index`: Port index (0 <= `index` < `count()`)
    ///
    /// # Returns
    /// - `Ok(buffer)`: Valid audio buffer for this port
    /// - `Err(NotFound)`: Port index out of range
    /// - `Err(IsF32)`: This port uses `f32` format, not `f64`
    /// - `Err(Null)`: Buffer pointer is null (should not happen)
    fn get_f64_buffer(&self, index: usize) -> Result<AudioBuffer<'h, f64>, AudioBufferError>;
}

/// Audio buffer for reading or writing sample data.
///
/// Represents audio data for one or more channels. Provides safe access
/// via readers/writers that prevent out-of-bounds access.
///
/// Generic over sample type to support both `f32` and `f64`.
///
/// # Lifetimes
/// - `'p`: Lifetime of the underlying audio memory (provided by the host)
pub struct AudioBuffer<'p, T: Sample> {
    channels: &'p [*mut T],
    frames: usize,
    constant_mask: u64,
    latency: u32,
}

impl<'p, T: Sample> AudioBuffer<'p, T> {
    /// Get the number of channels in this buffer.
    pub fn count(&self) -> usize {
        self.channels.len()
    }

    /// Get an immutable reader for a specific channel.
    ///
    /// # Arguments
    /// - `channel`: Channel index (0-based, must be < `count()`)
    ///
    /// # Returns
    /// - `Some(reader)`: Safe read-only access to the channel's samples
    /// - `None`: Channel index out of range
    pub fn get_reader(&self, channel: usize) -> Option<SliceReader<'p, T>> {
        let buffer = self
            .channels
            .get(channel)
            .map(|&d| unsafe { core::slice::from_raw_parts(d as *const _, self.frames) })?;

        Some(buffer.into())
    }

    /// Get a mutable writer for a specific channel.
    ///
    /// # Arguments
    /// - `channel`: Channel index (0-based, must be < `count()`)
    ///
    /// # Returns
    /// - `Some(writer)`: Safe write access to the channel's samples
    /// - `None`: Channel index out of range
    pub fn get_writer(&mut self, channel: usize) -> Option<SliceWriter<'p, T>> {
        let buffer = self
            .channels
            .get(channel)
            .map(|&d| unsafe { core::slice::from_raw_parts_mut(d, self.frames) })?;

        Some(buffer.into())
    }

    /// Iterate over all channels with immutable read access.
    ///
    /// # Returns
    /// An iterator of readers, one per channel. Readers may be None if
    /// a channel has a null pointer.
    pub fn iter_reader(&self) -> ReaderIter<'_, T> {
        ReaderIter {
            buffer: self,
            channel: 0,
        }
    }

    /// Iterate over all channels with mutable write access.
    ///
    /// # Returns
    /// An iterator of writers, one per channel. Writers may be None if
    /// a channel has a null pointer.
    pub fn iter_writer(&'p mut self) -> WriterIter<'p, T> {
        WriterIter {
            buffer: self,
            channel: 0,
        }
    }

    /// Mark a channel as constant (all samples are the same value).
    ///
    /// Used as an optimization hint: if all samples in a channel are identical,
    /// the host can avoid redundant processing. For example, if a reverb tail
    /// is silent on a channel, marking it constant as 0.0 allows the host to skip it.
    ///
    /// # Arguments
    /// - `channel`: Channel index (must be < MAX_CONSTANT_CHANNEL = 64)
    pub fn set_constant(&mut self, channel: usize) {
        debug_assert!(channel < MAX_CONSTANT_CHANNEL);
        if channel < MAX_CONSTANT_CHANNEL {
            self.constant_mask |= 1 << channel;
        }
    }

    /// Mark a channel as non-constant (samples vary).
    pub fn unset_constant(&mut self, channel: usize) {
        debug_assert!(channel < MAX_CONSTANT_CHANNEL);
        if channel < MAX_CONSTANT_CHANNEL {
            self.constant_mask |= 0 << channel;
        }
    }

    /// Check if a channel is marked as constant.
    ///
    /// # Returns
    /// `true` if the channel was marked constant via `set_constant()`, `false` otherwise.
    pub fn is_constant(&self, channel: usize) -> bool {
        debug_assert!(channel < MAX_CONSTANT_CHANNEL);
        if channel < MAX_CONSTANT_CHANNEL {
            self.constant_mask & (1 << channel) != 0
        } else {
            false
        }
    }

    /// Set the processing latency for this buffer (in samples).
    ///
    /// If this is an output buffer, this latency represents the delay
    /// introduced by the plugin's processing and should be reported to the host.
    /// The host uses this to compensate automation and other time-critical features.
    ///
    /// # Arguments
    /// - `latency`: Processing latency in samples (0 = no latency)
    pub fn set_latency(&mut self, latency: u32) {
        self.latency = latency;
    }
}

impl<'h, T: Sample> AudioBuffer<'h, T> {
    /// Create an audio buffer from raw components.
    ///
    /// # Safety
    /// - Each pointer in `channels` must point to valid audio data of `frames` samples
    /// - Pointers must remain valid for the buffer's lifetime ('h)
    /// - Pointers must be properly aligned for type `T`
    ///
    /// # Arguments
    /// - `channels`: Array of mutable pointers to audio data (one per channel)
    /// - `frames`: Number of samples in each channel
    /// - `constant_mask`: Bitfield indicating which channels are constant (bit = channel index)
    /// - `latency`: Processing latency in samples
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

/// Iterator over immutable channel readers.
///
/// Iterates through each channel and yields a reader for that channel.
/// Readers may be None if a channel has a null pointer.
pub struct ReaderIter<'b, T: Sample> {
    buffer: &'b AudioBuffer<'b, T>,
    channel: usize,
}

impl<'b, T: Sample> Iterator for ReaderIter<'b, T> {
    type Item = SliceReader<'b, T>;

    fn next(&mut self) -> Option<Self::Item> {
        let reader = self.buffer.get_reader(self.channel);
        self.channel += 1;
        reader
    }
}

/// Iterator over mutable channel writers.
///
/// Iterates through each channel and yields a writer for that channel.
/// Writers may be None if a channel has a null pointer.
pub struct WriterIter<'b, T: Sample> {
    buffer: &'b mut AudioBuffer<'b, T>,
    channel: usize,
}

impl<'b, T: Sample> Iterator for WriterIter<'b, T> {
    type Item = SliceWriter<'b, T>;

    fn next(&mut self) -> Option<Self::Item> {
        let writer = self.buffer.get_writer(self.channel);
        self.channel += 1;
        writer
    }
}

/// Error type for audio buffer access.
///
/// Returned when attempting to access an audio buffer in an unsupported format
/// or at an invalid index.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioBufferError {
    /// Port/channel index not found (out of bounds).
    NotFound,

    /// Requested f32 buffer but port uses f64 format.
    ///
    /// The plugin should try `get_f64_buffer()` instead.
    IsF32,

    /// Requested f64 buffer but port uses f32 format.
    ///
    /// The plugin should try `get_f32_buffer()` instead.
    IsF64,

    /// Buffer pointer is null (should not happen in normal operation).
    ///
    /// This indicates a problem with the host's buffer setup.
    Null,
}
