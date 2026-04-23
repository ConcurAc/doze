use core::fmt::Write;

use doze_common::identifier::{StrongIdentifier, WeakIdentifier};

use super::{Extension, PluginAccess, RegistrySource};

use crate::{
    events::{Event, EventSender, HostEvent, PluginEvent},
    plugin::Plugin,
};

/// Parameter management extension for plugin automation and modulation.
///
/// Allows plugins to declare parameters that can be automated by the host,
/// modulated by MIDI/MPE controllers, and saved/restored with plugin state.
///
/// # Generic Parameters
/// - `P`: The plugin type implementing this extension
///
/// # Fields
/// - `count`: Function returning total number of parameters
/// - `get`: Function returning a parameter descriptor by index
/// - `flush`: Function processing incoming parameter events
///
/// # Important
/// Parameter indices must remain stable throughout the plugin session.
/// The host remembers parameters by their `symbol`, not index.
/// If indices change, host state becomes invalid.
#[derive(Clone)]
pub struct Params<P: Plugin> {
    /// Query the total number of parameters.
    ///
    /// # Returns
    /// Number of parameters (0 or more) exposed by this plugin.
    pub count: fn(&P) -> usize,

    /// Get the descriptor for a parameter by index.
    ///
    /// # Arguments
    /// - `plugin`: Reference to the plugin
    /// - `index`: Parameter index (0..count())
    ///
    /// # Returns
    /// - `Some(&param)`: Parameter descriptor at this index
    /// - `None`: Index out of range
    ///
    /// # Important
    /// Parameter index must remain stable for the entire plugin session.
    pub get: for<'p> fn(&'p P, usize) -> Option<&'p Param>,

    /// Process incoming parameter events from the host.
    ///
    /// Called when the host has parameter changes (automation, modulation, etc.)
    /// for the plugin to apply. This is the main entry point for parameter updates.
    ///
    /// # Arguments
    /// - `plugin`: Mutable reference to plugin for state updates
    /// - `events`: Iterator over incoming host events (parameter changes, etc.)
    /// - `sender`: Event sender for plugin → host communication
    ///
    /// This is called on the main thread and can block
    pub flush: fn(
        &mut P,
        &mut dyn Iterator<Item = Event<HostEvent>>,
        &mut dyn EventSender<Event = Event<PluginEvent>>,
    ),
}

impl<P: Plugin> Extension for Params<P> {
    fn as_registry_source(&self) -> Option<&dyn RegistrySource> {
        Some(self)
    }
}

impl<P: Plugin> PluginAccess<P> for Params<P> {}

impl<P: Plugin> RegistrySource for Params<P> {
    fn identifiers(&self, plugin: &dyn Plugin) -> Vec<StrongIdentifier> {
        let plugin = <Self as PluginAccess<P>>::get(plugin);
        let count = (self.count)(plugin);
        let mut identifiers = Vec::with_capacity(count);
        for i in 0..count {
            if let Some(param) = (self.get)(plugin, i) {
                identifiers.push(param.symbol.clone());
            }
        }
        identifiers
    }
}

/// Complete parameter definition and metadata.
///
/// Describes a single automatable parameter including its type, range, name,
/// display format, and current value. Used by the host to present parameter controls
/// and automation UI.
///
/// # Fields
/// - `symbol`: Stable identifier
/// - `name`: Display name for UI
/// - `group`: Parameter grouping for organization
/// - `flags`: Capability flags (automatable, modulatable, etc.)
/// - `value`: Current value and range information
/// - `value_to_text`: Convert normalized value to display string
/// - `text_to_value`: Parse display string back to value
pub struct Param {
    /// Stable unique identifier for this parameter.
    ///
    /// Used by the host to remember the parameter across sessions.
    /// Should be URL-safe and descriptive.
    ///
    /// **Important**: This must not change between plugin versions, as the host
    /// uses it to map automation data.
    pub symbol: StrongIdentifier,

    /// Human-readable display name for the parameter.
    ///
    /// Shown in the host's automation/parameter UI.
    pub name: String,

    /// Parameter grouping for UI organization.
    ///
    /// Allows organizing related parameters into sections.
    pub group: ParamGroup,

    /// Capability flags for this parameter.
    ///
    /// Indicates which automation/modulation modes are supported, whether the parameter
    /// is hidden, read-only, etc.
    pub flags: ParamFlags,

    /// Current value and range information.
    ///
    /// Stores the parameter's range (min/max/default) and current value + modulation state.
    pub value: ParamValue,

    /// Function to convert normalized value to display text.
    ///
    /// Called by the host to get the text representation of a parameter value.
    /// Examples: 0.5 → "0.0 dB", 0.0 → "-∞ dB"
    ///
    /// # Arguments
    /// - `writer`: Write the display string here
    /// - `normalized_value`: Normalized value (0..1 or special range)
    ///
    /// # Returns
    /// `true` if formatting succeeded, `false` on error.
    pub value_to_text: fn(&mut dyn Write, f64) -> bool,

    /// Function to parse display text back to normalized value.
    ///
    /// Called by the host when the user edits a parameter value as text.
    /// Examples: "6 dB" → 0.6, "1 kHz" → 0.1
    ///
    /// # Arguments
    /// - `text`: Text input from user
    ///
    /// # Returns
    /// - `Some(value)`: Successfully parsed to normalized value
    /// - `None`: Parsing failed
    pub text_to_value: fn(&str) -> Option<f64>,
}

/// Parameter grouping for UI organization and categorization.
///
/// Parameters can be organized into groups/sections in the UI to improve
/// usability (e.g., grouping all filter parameters together).
///
/// # Fields
/// - `symbol`: Identifier for this group (e.g., "filter")
/// - `name`: Display name (e.g., "Filter")
/// - `prefix`: Optional prefix for parameter names within this group
#[derive(Default, Clone, Copy)]
pub struct ParamGroup {
    /// Stable identifier for this parameter group.
    ///
    /// Used to group related parameters in the UI.
    pub symbol: &'static str,

    /// Display name for this parameter group.
    ///
    /// Shown as a section header in the parameter UI.
    pub name: &'static str,

    /// Optional prefix for parameter names within this group.
    ///
    /// If set, the host may prepend this to parameter display names.
    pub prefix: &'static str,
}

bitflags::bitflags! {
    /// Flags describing parameter capabilities and behavior.
    ///
    /// Used to communicate to the host which automation/modulation modes
    /// are supported and how to handle the parameter.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct ParamFlags: u32 {
        /// Parameter value wraps around (e.g., phase, pan).
        ///
        /// For periodic parameters, min and max are equivalent (e.g., 0° and 360°).
        const PERIODIC = 1 << 0;

        /// Parameter should not be displayed in the UI.
        ///
        /// Used for internal parameters that are not meant for user adjustment.
        const HIDDEN   = 1 << 1;

        /// Parameter is read-only (cannot be changed by the user).
        ///
        /// Used for parameters that report state but are not directly editable.
        const READONLY = 1 << 2;

        /// Parameter can be automated by the host (normal automation).
        const AUTOMATABLE             = 1 << 3;

        /// Parameter can be automated per-note (polyphonic automation by note ID).
        ///
        /// Used for parameters that can be set per polyphonic voice.
        const AUTOMATABLE_PER_NOTE_ID = 1 << 4;

        /// Parameter can be automated per-key (polyphonic by MIDI key number).
        ///
        /// Allows different values for different MIDI keys pressed simultaneously.
        const AUTOMATABLE_PER_KEY     = 1 << 5;

        /// Parameter can be automated per-channel (polyphonic by MIDI channel).
        const AUTOMATABLE_PER_CHANNEL = 1 << 6;

        /// Parameter can be automated per-port (different for each audio port).
        const AUTOMATABLE_PER_PORT    = 1 << 7;

        /// Parameter can be modulated (via MPE, LFO, envelopes, etc.).
        const MODULATABLE             = 1 << 8;

        /// Parameter can be modulated per-note (polyphonic modulation by note ID).
        const MODULATABLE_PER_NOTE_ID = 1 << 9;

        /// Parameter can be modulated per-key (polyphonic by MIDI key).
        const MODULATABLE_PER_KEY     = 1 << 10;

        /// Parameter can be modulated per-channel (polyphonic by MIDI channel).
        const MODULATABLE_PER_CHANNEL = 1 << 11;

        /// Parameter can be modulated per-port (different for each audio port).
        const MODULATABLE_PER_PORT    = 1 << 12;

        /// Parameter changes require the plugin to re-run the process() callback.
        ///
        /// Hint that changes to this parameter may affect audio output properties.
        const REQUIRES_PROCESS        = 1 << 13;

        /// Changing this parameter may cause audio artifacts (clicks, pops, etc.).
        ///
        /// Hint to the host that automation of this parameter should be smoothed.
        const CAUSES_ARTIFACTS = 1 << 15;

        /// Parameter changes affect the plugin's tempo/speed (for tempo-sync effects).
        ///
        /// Used for delay time, LFO rate, and other tempo-dependent parameters.
        const CHANGES_TEMPO    = 1 << 16;
    }
}

/// Parameter range specification.
///
/// Defines the type of parameter and its valid range/choices.
/// Converted to `ParamValue` for runtime use.
///
/// # Variants
/// - `Continuous`: Smooth floating-point range
/// - `Stepped`: Discrete integer values
/// - `Bypass`: Boolean on/off switch
/// - `Enum`: Enumeration with named choices
pub enum ParamRange {
    /// Continuous floating-point parameter.
    ///
    /// Used for smooth parameters like gain, frequency, etc.
    ///
    /// # Fields
    /// - `min`: Minimum value
    /// - `max`: Maximum value
    /// - `default`: Default value (must be between min and max)
    Continuous { min: f64, max: f64, default: f64 },

    /// Discrete integer-valued parameter.
    ///
    /// Used for parameters with discrete steps (voices, order, etc.).
    ///
    /// # Fields
    /// - `min`: Minimum value
    /// - `max`: Maximum value (inclusive)
    /// - `default`: Default value
    Stepped { min: i32, max: i32, default: i32 },

    /// Boolean bypass parameter (on/off).
    ///
    /// Special parameter for a simple on/off toggle, typically used for
    /// bypass switches where `true` = enabled/active and `false` = bypassed.
    ///
    /// # Fields
    /// - `default`: Initial state (true = enabled, false = bypassed)
    Bypass { default: bool },

    /// Enumeration with named choices.
    ///
    /// Parameter with a fixed set of named options
    ///
    /// # Fields
    /// - `variants`: List of symbolic names for each choice
    /// - `default`: Index of the default choice (0..variants.len())
    Enum {
        /// Named choices for this parameter.
        variants: Vec<StrongIdentifier>,
        /// Index of the default choice.
        default: usize,
    },
}

/// Runtime state and current value of a parameter.
///
/// Stores the actual parameter value, modulation depth, and range information.
/// Provides methods to update, query, and format the value.
///
/// Created from a `ParamRange` and managed by the plugin during processing.
pub struct ParamValue {
    /// Current normalized parameter value (0..1 or special range).
    value: f64,

    /// Current modulation depth applied to the value.
    modulation: f64,

    /// Minimum value for this parameter.
    min: f64,

    /// Maximum value for this parameter.
    max: f64,

    /// Default value for this parameter.
    default: f64,

    /// Interpolation mode for this parameter.
    interpolation: ParamInterpolation,
}

impl From<ParamRange> for ParamValue {
    fn from(range: ParamRange) -> Self {
        match range {
            ParamRange::Continuous { min, max, default } => {
                let default = default.clamp(min, max);
                Self {
                    value: default,
                    modulation: 0.0,
                    min,
                    max,
                    default,
                    interpolation: ParamInterpolation::Continuous,
                }
            }
            ParamRange::Stepped { min, max, default } => {
                let default = default.clamp(min, max) as f64;
                Self {
                    value: default,
                    modulation: 0.0,
                    min: min as f64,
                    max: max as f64,
                    default,
                    interpolation: ParamInterpolation::Stepped,
                }
            }
            ParamRange::Bypass { default } => {
                let default = if default { 1.0 } else { 0.0 };
                Self {
                    value: default,
                    modulation: 0.0,
                    min: 0.0,
                    max: 1.0,
                    default,
                    interpolation: ParamInterpolation::Bypass,
                }
            }
            ParamRange::Enum { variants, default } => {
                let min = 0;
                let max = variants.len().saturating_sub(1);
                let default = default.clamp(min, max) as f64;
                Self {
                    value: default,
                    modulation: 0.0,
                    min: min as f64,
                    max: max as f64,
                    default: default,
                    interpolation: ParamInterpolation::Enum(variants),
                }
            }
        }
    }
}

impl ParamValue {
    pub fn set(&mut self, value: f64) {
        let value = match self.interpolation {
            ParamInterpolation::Continuous => value,
            ParamInterpolation::Stepped => value.round(),
            ParamInterpolation::Bypass => value.round(),
            ParamInterpolation::Enum(..) => value.round(),
        };
        self.value = value.clamp(self.min, self.max);
    }

    pub fn modulate(&mut self, amount: f64) {
        self.modulation = amount;
    }

    #[inline]
    pub fn get(&self) -> f64 {
        (self.value + self.modulation).clamp(self.min, self.max)
    }

    #[inline]
    pub fn get_stepped(&self) -> i32 {
        self.get().round() as i32
    }

    #[inline]
    pub fn get_bypass(&self) -> bool {
        if self.get() > 0.5 { true } else { false }
    }

    #[inline]
    pub fn get_label<'p>(&'p self, index: usize) -> Option<WeakIdentifier<'p>> {
        if let ParamInterpolation::Enum(variants) = &self.interpolation {
            variants.get(index).map(|l| l.downgrade())
        } else {
            None
        }
    }

    #[inline]
    pub fn get_min(&self) -> f64 {
        self.min
    }

    #[inline]
    pub fn get_max(&self) -> f64 {
        self.max
    }

    #[inline]
    pub fn get_default(&self) -> f64 {
        self.default
    }

    #[inline]
    pub fn get_interpolation(&self) -> &ParamInterpolation {
        &self.interpolation
    }

    #[inline]
    pub fn is_continuous(&self) -> bool {
        matches!(self.interpolation, ParamInterpolation::Continuous)
    }

    #[inline]
    pub fn is_stepped(&self) -> bool {
        matches!(self.interpolation, ParamInterpolation::Stepped)
    }

    #[inline]
    pub fn is_bypass(&self) -> bool {
        matches!(self.interpolation, ParamInterpolation::Bypass)
    }

    #[inline]
    pub fn is_enum(&self) -> bool {
        matches!(self.interpolation, ParamInterpolation::Enum(_))
    }
}

#[derive(Debug, Default)]
pub enum ParamInterpolation {
    #[default]
    Continuous,
    Stepped,
    Bypass,
    Enum(Vec<StrongIdentifier>),
}

#[derive(Debug, Clone, Copy)]
pub enum ParamUnit {
    // Frequency
    Hertz,
    Kilohertz,
    Megahertz,
    // Amplitude
    Decibels,
    // Time
    Millis,
    Seconds,
    Minutes,
    Frames,
    // Musical
    Cents,
    Semitones,
    Octaves,
    Bpm,
    MidiNote,
    // Spatial
    Millimetres,
    Metres,
    Kilometres,
    Miles,
    Inches,
    // Other
    Percent,
    Degrees,
}

impl core::fmt::Display for ParamUnit {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.suffix())
    }
}

const HZ_SUFFIX: &str = "Hz";
const KHZ_SUFFIX: &str = "kHz";
const MHZ_SUFFIX: &str = "MHz";
const HZ_TO_KHZ: f64 = 1_000.0;
const HZ_TO_MHZ: f64 = 1_000_000.0;

const MS_SUFFIX: &str = "ms";
const S_SUFFIX: &str = "s";
const MIN_SUFFIX: &str = "min";
const MS_TO_S: f64 = 1_000.0;
const MS_TO_MIN: f64 = 60_000.0;
const S_TO_MIN: f64 = 60.0;

const MM_SUFFIX: &str = "mm";
const M_SUFFIX: &str = "m";
const KM_SUFFIX: &str = "km";
const MM_TO_M: f64 = 1_000.0;
const MM_TO_KM: f64 = 1_000_000.0;
const M_TO_KM: f64 = 1_000.0;
const M_TO_MM: f64 = 1_000.0;

const DB_SUFFIX: &str = "dB";
const SMP_SUFFIX: &str = "smp";
const CT_SUFFIX: &str = "ct";
const ST_SUFFIX: &str = "st";
const OCT_SUFFIX: &str = "oct";
const BPM_SUFFIX: &str = "BPM";
const NOTE_SUFFIX: &str = "note";
const MI_SUFFIX: &str = "mi";
const IN_SUFFIX: &str = "in";
const PC_SUFFIX: &str = "%";
const DEG_SUFFIX: &str = "°";

const FREQ_CANDIDATES: &[(&str, f64)] = &[
    (MHZ_SUFFIX, HZ_TO_MHZ),
    (KHZ_SUFFIX, HZ_TO_KHZ),
    (HZ_SUFFIX, 1.0),
];

const TIME_CANDIDATES: &[(&str, f64)] = &[
    (MIN_SUFFIX, MS_TO_MIN),
    (S_SUFFIX, MS_TO_S),
    (MS_SUFFIX, 1.0),
];

const DIST_CANDIDATES: &[(&str, f64)] =
    &[(KM_SUFFIX, MM_TO_KM), (M_SUFFIX, MM_TO_M), (MM_SUFFIX, 1.0)];

impl ParamUnit {
    pub fn suffix(&self) -> &'static str {
        match self {
            ParamUnit::Hertz => HZ_SUFFIX,
            ParamUnit::Kilohertz => KHZ_SUFFIX,
            ParamUnit::Megahertz => MHZ_SUFFIX,
            ParamUnit::Decibels => DB_SUFFIX,
            ParamUnit::Millis => MS_SUFFIX,
            ParamUnit::Seconds => S_SUFFIX,
            ParamUnit::Minutes => MIN_SUFFIX,
            ParamUnit::Frames => SMP_SUFFIX,
            ParamUnit::Cents => CT_SUFFIX,
            ParamUnit::Semitones => ST_SUFFIX,
            ParamUnit::Octaves => OCT_SUFFIX,
            ParamUnit::Bpm => BPM_SUFFIX,
            ParamUnit::MidiNote => NOTE_SUFFIX,
            ParamUnit::Millimetres => MM_SUFFIX,
            ParamUnit::Metres => M_SUFFIX,
            ParamUnit::Kilometres => KM_SUFFIX,
            ParamUnit::Miles => MI_SUFFIX,
            ParamUnit::Inches => IN_SUFFIX,
            ParamUnit::Percent => PC_SUFFIX,
            ParamUnit::Degrees => DEG_SUFFIX,
        }
    }

    pub fn scale(&self, value: f64) -> (f64, &'static str) {
        match self {
            ParamUnit::Hertz => {
                if value >= HZ_TO_MHZ {
                    (value / HZ_TO_MHZ, MHZ_SUFFIX)
                } else if value >= HZ_TO_KHZ {
                    (value / HZ_TO_KHZ, KHZ_SUFFIX)
                } else {
                    (value, HZ_SUFFIX)
                }
            }
            ParamUnit::Kilohertz => {
                if value >= HZ_TO_KHZ {
                    (value / HZ_TO_KHZ, MHZ_SUFFIX)
                } else {
                    (value, KHZ_SUFFIX)
                }
            }
            ParamUnit::Millis => {
                if value >= MS_TO_MIN {
                    (value / MS_TO_MIN, MIN_SUFFIX)
                } else if value >= MS_TO_S {
                    (value / MS_TO_S, S_SUFFIX)
                } else {
                    (value, MS_SUFFIX)
                }
            }
            ParamUnit::Seconds => {
                if value >= S_TO_MIN {
                    (value / S_TO_MIN, MIN_SUFFIX)
                } else {
                    (value, S_SUFFIX)
                }
            }
            ParamUnit::Metres => {
                if value >= M_TO_KM {
                    (value / M_TO_KM, KM_SUFFIX)
                } else if value < 1.0 / M_TO_MM {
                    (value * M_TO_MM, MM_SUFFIX)
                } else {
                    (value, M_SUFFIX)
                }
            }
            ParamUnit::Millimetres => {
                if value >= MM_TO_KM {
                    (value / MM_TO_KM, KM_SUFFIX)
                } else if value >= MM_TO_M {
                    (value / MM_TO_M, M_SUFFIX)
                } else {
                    (value, MM_SUFFIX)
                }
            }
            unit => (value, unit.suffix()),
        }
    }

    pub fn parse(&self, text: &str) -> Option<f64> {
        let text = text.trim();
        let candidates: &[(&str, f64)] = match self {
            ParamUnit::Hertz | ParamUnit::Kilohertz | ParamUnit::Megahertz => FREQ_CANDIDATES,
            ParamUnit::Millis | ParamUnit::Seconds | ParamUnit::Minutes => TIME_CANDIDATES,
            ParamUnit::Metres | ParamUnit::Kilometres | ParamUnit::Millimetres => DIST_CANDIDATES,
            _ => {
                return text
                    .trim_end_matches(self.suffix())
                    .trim()
                    .parse::<f64>()
                    .ok();
            }
        };
        let self_scale = candidates
            .iter()
            .find(|(s, _)| *s == self.suffix())
            .map(|(_, scale)| *scale)
            .unwrap_or(1.0);

        for (suffix, scale) in candidates {
            if let Some(numeric) = text.strip_suffix(suffix) {
                return numeric
                    .trim()
                    .parse::<f64>()
                    .ok()
                    .map(|v| v * scale / self_scale);
            }
        }
        text.parse::<f64>().ok().map(|v| v / self_scale)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FLOAT_EPSILON: f64 = 1e-15;

    // Frequency
    const FREQ_HZ: f64 = 440.0;
    const FREQ_KHZ: f64 = 4_000.0;
    const FREQ_MHZ: f64 = 2_000_000.0;
    const FREQ_KHZ_SCALED: f64 = 4.0;
    const FREQ_MHZ_SCALED: f64 = 2.0;
    const FREQ_KHZ_TO_MHZ: f64 = 2_000.0;

    // Time
    const TIME_S: f64 = 2_000.0;
    const TIME_MIN: f64 = 120_000.0;
    const TIME_S_IN_MIN: f64 = 120.0;
    const TIME_SCALED: f64 = 2.0;

    // Distance
    const DIST_M: f64 = 2_000.0;
    const DIST_KM: f64 = 2_000_000.0;
    const DIST_SCALED: f64 = 2.0;
    const DIST_M_SMALL: f64 = 0.0005;
    const DIST_M_SMALL_MM: f64 = 0.5;

    // Other
    const DB_VALUE: f64 = -6.0;

    fn assert_scale(unit: ParamUnit, input: f64, expected_v: f64, expected_s: &str) {
        let (v, s) = unit.scale(input);
        assert_eq!(s, expected_s);
        assert!((v - expected_v).abs() < FLOAT_EPSILON);
    }

    fn assert_parse(unit: ParamUnit, text: &str, expected: Option<f64>) {
        assert_eq!(unit.parse(text), expected);
    }

    #[test]
    fn scale_hertz_stays_hz() {
        assert_scale(ParamUnit::Hertz, FREQ_HZ, FREQ_HZ, "Hz");
    }
    #[test]
    fn scale_hertz_promotes_to_khz() {
        assert_scale(ParamUnit::Hertz, FREQ_KHZ, FREQ_KHZ_SCALED, "kHz");
    }
    #[test]
    fn scale_hertz_promotes_to_mhz() {
        assert_scale(ParamUnit::Hertz, FREQ_MHZ, FREQ_MHZ_SCALED, "MHz");
    }
    #[test]
    fn scale_khz_promotes_to_mhz() {
        assert_scale(
            ParamUnit::Kilohertz,
            FREQ_KHZ_TO_MHZ,
            FREQ_MHZ_SCALED,
            "MHz",
        );
    }
    #[test]
    fn scale_millis_promotes_to_s() {
        assert_scale(ParamUnit::Millis, TIME_S, TIME_SCALED, "s");
    }
    #[test]
    fn scale_millis_promotes_to_min() {
        assert_scale(ParamUnit::Millis, TIME_MIN, TIME_SCALED, "min");
    }
    #[test]
    fn scale_seconds_promotes_to_min() {
        assert_scale(ParamUnit::Seconds, TIME_S_IN_MIN, TIME_SCALED, "min");
    }
    #[test]
    fn scale_metres_promotes_to_km() {
        assert_scale(ParamUnit::Metres, DIST_M, DIST_SCALED, "km");
    }
    #[test]
    fn scale_metres_demotes_to_mm() {
        assert_scale(ParamUnit::Metres, DIST_M_SMALL, DIST_M_SMALL_MM, "mm");
    }
    #[test]
    fn scale_mm_promotes_to_m() {
        assert_scale(ParamUnit::Millimetres, DIST_M, DIST_SCALED, "m");
    }
    #[test]
    fn scale_mm_promotes_to_km() {
        assert_scale(ParamUnit::Millimetres, DIST_KM, DIST_SCALED, "km");
    }
    #[test]
    fn scale_non_scaling_unchanged() {
        assert_scale(ParamUnit::Decibels, DB_VALUE, DB_VALUE, "dB");
    }

    #[test]
    fn parse_hz_suffix() {
        assert_parse(ParamUnit::Hertz, "440 Hz", Some(FREQ_HZ));
    }
    #[test]
    fn parse_khz_suffix() {
        assert_parse(ParamUnit::Hertz, "4 kHz", Some(FREQ_KHZ));
    }
    #[test]
    fn parse_mhz_suffix() {
        assert_parse(ParamUnit::Hertz, "2 MHz", Some(FREQ_MHZ));
    }
    #[test]
    fn parse_s_suffix() {
        assert_parse(ParamUnit::Millis, "2 s", Some(TIME_S));
    }
    #[test]
    fn parse_min_suffix() {
        assert_parse(ParamUnit::Millis, "2 min", Some(TIME_MIN));
    }
    #[test]
    fn parse_m_suffix() {
        assert_parse(ParamUnit::Millimetres, "2 m", Some(DIST_M));
    }
    #[test]
    fn parse_km_suffix() {
        assert_parse(ParamUnit::Millimetres, "2 km", Some(DIST_KM));
    }
    #[test]
    fn parse_db_suffix() {
        assert_parse(ParamUnit::Decibels, "-6 dB", Some(DB_VALUE));
    }
    #[test]
    fn parse_trims_outer_whitespace() {
        assert_parse(ParamUnit::Hertz, "  440 Hz  ", Some(FREQ_HZ));
    }
    #[test]
    fn parse_trims_inner_whitespace() {
        assert_parse(ParamUnit::Hertz, "440 Hz", Some(FREQ_HZ));
    }
    #[test]
    fn parse_raw_number_fallback() {
        assert_parse(ParamUnit::Hertz, "440", Some(FREQ_HZ));
    }
    #[test]
    fn parse_invalid_returns_none() {
        assert_parse(ParamUnit::Hertz, "not a number", None);
    }
    #[test]
    fn parse_empty_returns_none() {
        assert_parse(ParamUnit::Hertz, "", None);
    }

    #[test]
    fn round_trip_hertz() {
        let (scaled, suffix) = ParamUnit::Hertz.scale(FREQ_KHZ);
        let parsed = ParamUnit::Hertz
            .parse(&format!("{}{}", scaled, suffix))
            .unwrap();
        assert!((parsed - FREQ_KHZ).abs() < FLOAT_EPSILON);
    }
}
