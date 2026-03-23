use doze_plugin::features::PluginFeature;

use clap_sys::plugin_features::{
    CLAP_PLUGIN_FEATURE_AMBISONIC, CLAP_PLUGIN_FEATURE_ANALYZER, CLAP_PLUGIN_FEATURE_AUDIO_EFFECT,
    CLAP_PLUGIN_FEATURE_CHORUS, CLAP_PLUGIN_FEATURE_COMPRESSOR, CLAP_PLUGIN_FEATURE_DEESSER,
    CLAP_PLUGIN_FEATURE_DELAY, CLAP_PLUGIN_FEATURE_DISTORTION, CLAP_PLUGIN_FEATURE_DRUM,
    CLAP_PLUGIN_FEATURE_DRUM_MACHINE, CLAP_PLUGIN_FEATURE_EQUALIZER, CLAP_PLUGIN_FEATURE_EXPANDER,
    CLAP_PLUGIN_FEATURE_FILTER, CLAP_PLUGIN_FEATURE_FLANGER, CLAP_PLUGIN_FEATURE_FREQUENCY_SHIFTER,
    CLAP_PLUGIN_FEATURE_GATE, CLAP_PLUGIN_FEATURE_GLITCH, CLAP_PLUGIN_FEATURE_GRANULAR,
    CLAP_PLUGIN_FEATURE_INSTRUMENT, CLAP_PLUGIN_FEATURE_LIMITER, CLAP_PLUGIN_FEATURE_MASTERING,
    CLAP_PLUGIN_FEATURE_MIXING, CLAP_PLUGIN_FEATURE_MONO, CLAP_PLUGIN_FEATURE_MULTI_EFFECTS,
    CLAP_PLUGIN_FEATURE_NOTE_DETECTOR, CLAP_PLUGIN_FEATURE_NOTE_EFFECT,
    CLAP_PLUGIN_FEATURE_PHASE_VOCODER, CLAP_PLUGIN_FEATURE_PHASER,
    CLAP_PLUGIN_FEATURE_PITCH_CORRECTION, CLAP_PLUGIN_FEATURE_PITCH_SHIFTER,
    CLAP_PLUGIN_FEATURE_RESTORATION, CLAP_PLUGIN_FEATURE_REVERB, CLAP_PLUGIN_FEATURE_SAMPLER,
    CLAP_PLUGIN_FEATURE_STEREO, CLAP_PLUGIN_FEATURE_SURROUND, CLAP_PLUGIN_FEATURE_SYNTHESIZER,
    CLAP_PLUGIN_FEATURE_TRANSIENT_SHAPER, CLAP_PLUGIN_FEATURE_TREMOLO, CLAP_PLUGIN_FEATURE_UTILITY,
};

use std::ffi::CStr;

const INSTRUMENT: &'static [u8] = CLAP_PLUGIN_FEATURE_INSTRUMENT.to_bytes();
const AUDIO_EFFECT: &'static [u8] = CLAP_PLUGIN_FEATURE_AUDIO_EFFECT.to_bytes();
const NOTE_DETECTOR: &[u8] = CLAP_PLUGIN_FEATURE_NOTE_DETECTOR.to_bytes();
const NOTE_EFFECT: &[u8] = CLAP_PLUGIN_FEATURE_NOTE_EFFECT.to_bytes();
const ANALYZER: &[u8] = CLAP_PLUGIN_FEATURE_ANALYZER.to_bytes();
const SYNTHESIZER: &[u8] = CLAP_PLUGIN_FEATURE_SYNTHESIZER.to_bytes();
const SAMPLER: &[u8] = CLAP_PLUGIN_FEATURE_SAMPLER.to_bytes();
const DRUM: &[u8] = CLAP_PLUGIN_FEATURE_DRUM.to_bytes();
const DRUM_MACHINE: &[u8] = CLAP_PLUGIN_FEATURE_DRUM_MACHINE.to_bytes();
const FILTER: &[u8] = CLAP_PLUGIN_FEATURE_FILTER.to_bytes();
const PHASER: &[u8] = CLAP_PLUGIN_FEATURE_PHASER.to_bytes();
const EQUALIZER: &[u8] = CLAP_PLUGIN_FEATURE_EQUALIZER.to_bytes();
const DE_ESSER: &[u8] = CLAP_PLUGIN_FEATURE_DEESSER.to_bytes();
const PHASE_VOCODER: &[u8] = CLAP_PLUGIN_FEATURE_PHASE_VOCODER.to_bytes();
const GRANULAR: &[u8] = CLAP_PLUGIN_FEATURE_GRANULAR.to_bytes();
const FREQUENCY_SHIFTER: &[u8] = CLAP_PLUGIN_FEATURE_FREQUENCY_SHIFTER.to_bytes();
const PITCH_SHIFTER: &[u8] = CLAP_PLUGIN_FEATURE_PITCH_SHIFTER.to_bytes();
const DISTORTION: &[u8] = CLAP_PLUGIN_FEATURE_DISTORTION.to_bytes();
const TRANSIENT_SHAPER: &[u8] = CLAP_PLUGIN_FEATURE_TRANSIENT_SHAPER.to_bytes();
const COMPRESSOR: &[u8] = CLAP_PLUGIN_FEATURE_COMPRESSOR.to_bytes();
const EXPANDER: &[u8] = CLAP_PLUGIN_FEATURE_EXPANDER.to_bytes();
const GATE: &[u8] = CLAP_PLUGIN_FEATURE_GATE.to_bytes();
const LIMITER: &[u8] = CLAP_PLUGIN_FEATURE_LIMITER.to_bytes();
const FLANGER: &[u8] = CLAP_PLUGIN_FEATURE_FLANGER.to_bytes();
const CHORUS: &[u8] = CLAP_PLUGIN_FEATURE_CHORUS.to_bytes();
const DELAY: &[u8] = CLAP_PLUGIN_FEATURE_DELAY.to_bytes();
const REVERB: &[u8] = CLAP_PLUGIN_FEATURE_REVERB.to_bytes();
const TREMOLO: &[u8] = CLAP_PLUGIN_FEATURE_TREMOLO.to_bytes();
const GLITCH: &[u8] = CLAP_PLUGIN_FEATURE_GLITCH.to_bytes();
const UTILITY: &[u8] = CLAP_PLUGIN_FEATURE_UTILITY.to_bytes();
const PITCH_CORRECTION: &[u8] = CLAP_PLUGIN_FEATURE_PITCH_CORRECTION.to_bytes();
const RESTORATION: &[u8] = CLAP_PLUGIN_FEATURE_RESTORATION.to_bytes();
const MULTI_EFFECTS: &[u8] = CLAP_PLUGIN_FEATURE_MULTI_EFFECTS.to_bytes();
const MIXING: &[u8] = CLAP_PLUGIN_FEATURE_MIXING.to_bytes();
const MASTERING: &[u8] = CLAP_PLUGIN_FEATURE_MASTERING.to_bytes();
const MONO: &[u8] = CLAP_PLUGIN_FEATURE_MONO.to_bytes();
const STEREO: &[u8] = CLAP_PLUGIN_FEATURE_STEREO.to_bytes();
const SURROUND: &[u8] = CLAP_PLUGIN_FEATURE_SURROUND.to_bytes();
const AMBISONIC: &[u8] = CLAP_PLUGIN_FEATURE_AMBISONIC.to_bytes();

pub fn feature_from_clap(feature: impl AsRef<CStr>) -> Option<PluginFeature> {
    match feature.as_ref().to_bytes() {
        INSTRUMENT => Some(PluginFeature::Instrument),
        AUDIO_EFFECT => Some(PluginFeature::AudioEffect),
        NOTE_DETECTOR => Some(PluginFeature::NoteDetector),
        NOTE_EFFECT => Some(PluginFeature::NoteEffect),
        ANALYZER => Some(PluginFeature::Analyzer),
        SYNTHESIZER => Some(PluginFeature::Synthesizer),
        SAMPLER => Some(PluginFeature::Sampler),
        DRUM => Some(PluginFeature::Drum),
        DRUM_MACHINE => Some(PluginFeature::DrumMachine),
        FILTER => Some(PluginFeature::Filter),
        PHASER => Some(PluginFeature::Phaser),
        EQUALIZER => Some(PluginFeature::Equalizer),
        DE_ESSER => Some(PluginFeature::Deesser),
        PHASE_VOCODER => Some(PluginFeature::PhaseVocoder),
        GRANULAR => Some(PluginFeature::Granular),
        FREQUENCY_SHIFTER => Some(PluginFeature::FrequencyShifter),
        PITCH_SHIFTER => Some(PluginFeature::PitchShifter),
        DISTORTION => Some(PluginFeature::Distortion),
        TRANSIENT_SHAPER => Some(PluginFeature::TransientShaper),
        COMPRESSOR => Some(PluginFeature::Compressor),
        EXPANDER => Some(PluginFeature::Expander),
        GATE => Some(PluginFeature::Gate),
        LIMITER => Some(PluginFeature::Limiter),
        FLANGER => Some(PluginFeature::Flanger),
        CHORUS => Some(PluginFeature::Chorus),
        DELAY => Some(PluginFeature::Delay),
        REVERB => Some(PluginFeature::Reverb),
        TREMOLO => Some(PluginFeature::Tremolo),
        GLITCH => Some(PluginFeature::Glitch),
        UTILITY => Some(PluginFeature::Utility),
        PITCH_CORRECTION => Some(PluginFeature::PitchCorrection),
        RESTORATION => Some(PluginFeature::Restoration),
        MULTI_EFFECTS => Some(PluginFeature::MultiEffects),
        MIXING => Some(PluginFeature::Mixing),
        MASTERING => Some(PluginFeature::Mastering),
        MONO => Some(PluginFeature::Mono),
        STEREO => Some(PluginFeature::Stereo),
        SURROUND => Some(PluginFeature::Surround),
        AMBISONIC => Some(PluginFeature::Ambisonic),
        _ => None,
    }
}

pub fn feature_as_clap(feature: &PluginFeature) -> Option<&'static CStr> {
    match feature {
        PluginFeature::Instrument => Some(CLAP_PLUGIN_FEATURE_INSTRUMENT),
        PluginFeature::AudioEffect => Some(CLAP_PLUGIN_FEATURE_AUDIO_EFFECT),
        PluginFeature::NoteDetector => Some(CLAP_PLUGIN_FEATURE_NOTE_DETECTOR),
        PluginFeature::NoteEffect => Some(CLAP_PLUGIN_FEATURE_NOTE_EFFECT),
        PluginFeature::Analyzer => Some(CLAP_PLUGIN_FEATURE_ANALYZER),
        PluginFeature::Synthesizer => Some(CLAP_PLUGIN_FEATURE_SYNTHESIZER),
        PluginFeature::Sampler => Some(CLAP_PLUGIN_FEATURE_SAMPLER),
        PluginFeature::Drum => Some(CLAP_PLUGIN_FEATURE_DRUM),
        PluginFeature::DrumMachine => Some(CLAP_PLUGIN_FEATURE_DRUM_MACHINE),
        PluginFeature::Filter => Some(CLAP_PLUGIN_FEATURE_FILTER),
        PluginFeature::Phaser => Some(CLAP_PLUGIN_FEATURE_PHASER),
        PluginFeature::Equalizer => Some(CLAP_PLUGIN_FEATURE_EQUALIZER),
        PluginFeature::Deesser => Some(CLAP_PLUGIN_FEATURE_DEESSER),
        PluginFeature::PhaseVocoder => Some(CLAP_PLUGIN_FEATURE_PHASE_VOCODER),
        PluginFeature::Granular => Some(CLAP_PLUGIN_FEATURE_GRANULAR),
        PluginFeature::FrequencyShifter => Some(CLAP_PLUGIN_FEATURE_FREQUENCY_SHIFTER),
        PluginFeature::PitchShifter => Some(CLAP_PLUGIN_FEATURE_PITCH_SHIFTER),
        PluginFeature::Distortion => Some(CLAP_PLUGIN_FEATURE_DISTORTION),
        PluginFeature::TransientShaper => Some(CLAP_PLUGIN_FEATURE_TRANSIENT_SHAPER),
        PluginFeature::Compressor => Some(CLAP_PLUGIN_FEATURE_COMPRESSOR),
        PluginFeature::Expander => Some(CLAP_PLUGIN_FEATURE_EXPANDER),
        PluginFeature::Gate => Some(CLAP_PLUGIN_FEATURE_GATE),
        PluginFeature::Limiter => Some(CLAP_PLUGIN_FEATURE_LIMITER),
        PluginFeature::Flanger => Some(CLAP_PLUGIN_FEATURE_FLANGER),
        PluginFeature::Chorus => Some(CLAP_PLUGIN_FEATURE_CHORUS),
        PluginFeature::Delay => Some(CLAP_PLUGIN_FEATURE_DELAY),
        PluginFeature::Reverb => Some(CLAP_PLUGIN_FEATURE_REVERB),
        PluginFeature::Tremolo => Some(CLAP_PLUGIN_FEATURE_TREMOLO),
        PluginFeature::Glitch => Some(CLAP_PLUGIN_FEATURE_GLITCH),
        PluginFeature::Utility => Some(CLAP_PLUGIN_FEATURE_UTILITY),
        PluginFeature::PitchCorrection => Some(CLAP_PLUGIN_FEATURE_PITCH_CORRECTION),
        PluginFeature::Restoration => Some(CLAP_PLUGIN_FEATURE_RESTORATION),
        PluginFeature::MultiEffects => Some(CLAP_PLUGIN_FEATURE_MULTI_EFFECTS),
        PluginFeature::Mixing => Some(CLAP_PLUGIN_FEATURE_MIXING),
        PluginFeature::Mastering => Some(CLAP_PLUGIN_FEATURE_MASTERING),
        PluginFeature::Mono => Some(CLAP_PLUGIN_FEATURE_MONO),
        PluginFeature::Stereo => Some(CLAP_PLUGIN_FEATURE_STEREO),
        PluginFeature::Surround => Some(CLAP_PLUGIN_FEATURE_SURROUND),
        PluginFeature::Ambisonic => Some(CLAP_PLUGIN_FEATURE_AMBISONIC),
    }
}
