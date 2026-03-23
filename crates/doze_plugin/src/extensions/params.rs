use core::fmt::Write;

use doze_common::identifier::{StrongIdentifier, WeakIdentifier};

use super::{Extension, PluginAccess, RegistrySource, audio_ports::PortGroup};

use crate::{
    events::{Event, EventSender, HostEvent, PluginEvent},
    plugin::Plugin,
};

#[derive(Clone)]
pub struct Params<P: Plugin> {
    pub count: fn(&P) -> usize,
    pub get: for<'p> fn(&'p P, usize) -> Option<&'p Param>,
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

pub struct Param {
    pub symbol: StrongIdentifier,

    pub name: String,
    pub group: ParamGroup,

    pub flags: ParamFlags,

    pub value: ParamValue,

    pub value_to_text: fn(&mut dyn Write, f64) -> bool,
    pub text_to_value: fn(&str) -> Option<f64>,
}

pub struct ParamGroup {
    pub symbol: &'static str,
    pub name: &'static str,
    pub prefix: &'static str,
    pub port_group: PortGroup,
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct ParamFlags: u32 {
        const PERIODIC = 1 << 0;
        const HIDDEN   = 1 << 1;
        const READONLY = 1 << 2;

        const AUTOMATABLE             = 1 << 3;
        const AUTOMATABLE_PER_NOTE_ID = 1 << 4;
        const AUTOMATABLE_PER_KEY     = 1 << 5;
        const AUTOMATABLE_PER_CHANNEL = 1 << 6;
        const AUTOMATABLE_PER_PORT    = 1 << 7;
        const MODULATABLE             = 1 << 8;
        const MODULATABLE_PER_NOTE_ID = 1 << 9;
        const MODULATABLE_PER_KEY     = 1 << 10;
        const MODULATABLE_PER_CHANNEL = 1 << 11;
        const MODULATABLE_PER_PORT    = 1 << 12;
        const REQUIRES_PROCESS        = 1 << 13;

        const ENUMERATION      = 1 << 14;
        const CAUSES_ARTIFACTS = 1 << 15;
        const CHANGES_TEMPO    = 1 << 16;
    }
}

pub enum ParamRange {
    Continuous {
        min: f64,
        max: f64,
        default: f64,
    },
    Stepped {
        min: i32,
        max: i32,
        default: i32,
    },
    Bypass {
        default: bool,
    },
    Enum {
        variants: Vec<StrongIdentifier>,
        default: usize,
    },
}

pub struct ParamValue {
    value: f64,
    min: f64,
    max: f64,
    default: f64,
    interpolation: ParamInterpolation,
}

impl From<ParamRange> for ParamValue {
    fn from(range: ParamRange) -> Self {
        match range {
            ParamRange::Continuous { min, max, default } => {
                let default = default.clamp(min, max);
                Self {
                    value: default,
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

    #[inline]
    pub fn get(&self) -> f64 {
        self.value
    }

    #[inline]
    pub fn get_stepped(&self) -> i32 {
        self.value.round() as i32
    }

    #[inline]
    pub fn get_bypass(&self) -> bool {
        if self.value > 0.5 { true } else { false }
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

    // ── scale ────────────────────────────────────────────────────────────────

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

    // ── parse ────────────────────────────────────────────────────────────────

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

    // ── round-trip ───────────────────────────────────────────────────────────

    #[test]
    fn round_trip_hertz() {
        let (scaled, suffix) = ParamUnit::Hertz.scale(FREQ_KHZ);
        let parsed = ParamUnit::Hertz
            .parse(&format!("{}{}", scaled, suffix))
            .unwrap();
        assert!((parsed - FREQ_KHZ).abs() < FLOAT_EPSILON);
    }
}
