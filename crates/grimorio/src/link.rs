//! Host ↔ Grimorio link: ordered [`Uplink`] in, ordered [`Downlink`] out.
//!
//! Schema ids: [`UPLINK_SCHEMA`], [`DOWNLINK_SCHEMA`].

use super::common::{Language, NarrativeEcho};
use super::epithet::{self, forge_sample};
use super::portrait::StandoutPortrait;
use super::traits::WhisperKind;
use super::wisdom;

/// Stable schema id for host → Grimorio envelopes.
pub const UPLINK_SCHEMA: &str = "grimorio.uplink/v1";

/// Stable schema id for Grimorio → host envelopes.
pub const DOWNLINK_SCHEMA: &str = "grimorio.downlink/v1";

/// Ordered uplink envelope. Field order is part of the contract.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Uplink {
    /// Must be [`UPLINK_SCHEMA`].
    pub schema: String,
    /// Locale for tables (`en` / `es` / `ru`).
    pub language: Language,
    /// Voice payload (tagged).
    pub voice: UplinkVoice,
}

/// Voice-specific uplink body. Serde tag: `kind`.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum UplinkVoice {
    /// Fragmentary lore line.
    Wisdom {
        /// Host mix seed (seed × epoch × ritual, etc.).
        mix: u64,
        /// Ordered narrative bias digest from the host.
        echo: NarrativeEcho,
    },
    /// Honorific for the generation standout.
    GenerationEpithet {
        /// Generation stamp on the epithet.
        generation: u32,
        /// Ordered trait axes for the standout (see [`StandoutPortrait`] field order).
        standout: StandoutPortrait,
    },
    /// Synthetic sample (CLI / demos).
    SampleEpithet {
        /// Sample index in the forge_sample sequence.
        index: u32,
        /// Deterministic mix seed.
        seed: u64,
    },
}

/// Ordered downlink envelope. Field order is part of the contract.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Downlink {
    /// Must be [`DOWNLINK_SCHEMA`].
    pub schema: String,
    /// Echo of the uplink language.
    pub language: Language,
    /// Which voice produced the text.
    pub kind: WhisperKind,
    /// Surface text (may be empty when [`Downlink::empty`] is true).
    pub text: String,
    /// True when no usable line was forged / selected.
    pub empty: bool,
}

impl Uplink {
    /// Build a wisdom uplink.
    #[must_use]
    pub fn wisdom(language: Language, mix: u64, echo: NarrativeEcho) -> Self {
        Self {
            schema: UPLINK_SCHEMA.to_string(),
            language,
            voice: UplinkVoice::Wisdom { mix, echo },
        }
    }

    /// Build a generation-epithet uplink.
    #[must_use]
    pub fn generation_epithet(
        language: Language,
        generation: u32,
        standout: StandoutPortrait,
    ) -> Self {
        Self {
            schema: UPLINK_SCHEMA.to_string(),
            language,
            voice: UplinkVoice::GenerationEpithet {
                generation,
                standout,
            },
        }
    }

    /// Build a sample-epithet uplink (CLI / demos).
    #[must_use]
    pub fn sample_epithet(language: Language, index: u32, seed: u64) -> Self {
        Self {
            schema: UPLINK_SCHEMA.to_string(),
            language,
            voice: UplinkVoice::SampleEpithet { index, seed },
        }
    }
}

impl Downlink {
    #[must_use]
    fn filled(language: Language, kind: WhisperKind, text: String) -> Self {
        let empty = text.is_empty();
        Self {
            schema: DOWNLINK_SCHEMA.to_string(),
            language,
            kind,
            text,
            empty,
        }
    }
}

/// Which voice a partial uplink should resolve to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VoiceKind {
    Wisdom,
    GenerationEpithet,
    #[default]
    SampleEpithet,
}

impl VoiceKind {
    /// Parse CLI / wire tokens.
    #[must_use]
    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "wisdom" | "w" => Some(Self::Wisdom),
            "generation_epithet" | "epithet" | "generation" | "g" => {
                Some(Self::GenerationEpithet)
            }
            "sample_epithet" | "sample" | "s" => Some(Self::SampleEpithet),
            _ => None,
        }
    }
}

/// Partial uplink: any field may be omitted; [`UplinkDraft::finish`] fills the rest.
#[derive(Clone, Debug, Default)]
pub struct UplinkDraft {
    pub language: Option<Language>,
    pub voice: Option<VoiceKind>,
    // Wisdom
    pub mix: Option<u64>,
    pub dominant_tone_idx: Option<u8>,
    pub scar_mass: Option<u32>,
    pub rare_event_token: Option<u64>,
    pub dying_viability_quant: Option<u8>,
    pub fossil_absence_mass: Option<u32>,
    pub attention_xor: Option<u64>,
    pub soul_attunement_quant: Option<u8>,
    // Generation epithet
    pub generation: Option<u32>,
    pub label: Option<String>,
    pub fitness: Option<f32>,
    pub viability: Option<f32>,
    pub vitality: Option<f32>,
    pub resonance: Option<f32>,
    pub mutation_pressure: Option<f32>,
    pub symbolic_density: Option<f32>,
    pub memory_depth: Option<f32>,
    pub shadow_pull: Option<f32>,
    pub myth: Option<String>,
    // Sample epithet
    pub index: Option<u32>,
    pub seed: Option<u64>,
}

impl UplinkDraft {
    /// True if any standout / generation field was set explicitly.
    #[must_use]
    pub fn has_standout_hints(&self) -> bool {
        self.generation.is_some()
            || self.label.is_some()
            || self.fitness.is_some()
            || self.viability.is_some()
            || self.vitality.is_some()
            || self.resonance.is_some()
            || self.mutation_pressure.is_some()
            || self.symbolic_density.is_some()
            || self.memory_depth.is_some()
            || self.shadow_pull.is_some()
            || self.myth.is_some()
    }

    /// True if any wisdom echo / mix field was set explicitly.
    #[must_use]
    pub fn has_wisdom_hints(&self) -> bool {
        self.mix.is_some()
            || self.dominant_tone_idx.is_some()
            || self.scar_mass.is_some()
            || self.rare_event_token.is_some()
            || self.dying_viability_quant.is_some()
            || self.fossil_absence_mass.is_some()
            || self.attention_xor.is_some()
            || self.soul_attunement_quant.is_some()
    }

    /// Resolve voice: explicit → standout hints → wisdom hints → sample.
    #[must_use]
    pub fn resolve_voice(&self) -> VoiceKind {
        if let Some(v) = self.voice {
            return v;
        }
        if self.has_standout_hints() {
            return VoiceKind::GenerationEpithet;
        }
        if self.has_wisdom_hints() {
            return VoiceKind::Wisdom;
        }
        VoiceKind::SampleEpithet
    }

    /// Fill omitted fields and build a complete [`Uplink`].
    #[must_use]
    pub fn finish(self) -> Uplink {
        let language = self.language.unwrap_or(Language::Spanish);
        let seed = self.seed.unwrap_or(0xC0FF_EE00_D15C_A11E);
        let voice = self.resolve_voice();
        match voice {
            VoiceKind::Wisdom => {
                let mix = self.mix.unwrap_or(seed);
                let echo = NarrativeEcho {
                    dominant_tone_idx: self.dominant_tone_idx.unwrap_or(0),
                    scar_mass: self.scar_mass.unwrap_or(0),
                    rare_event_token: self.rare_event_token.unwrap_or(0),
                    dying_viability_quant: self.dying_viability_quant.unwrap_or(255),
                    fossil_absence_mass: self.fossil_absence_mass.unwrap_or(0),
                    attention_xor: self.attention_xor.unwrap_or(0),
                    soul_attunement_quant: self.soul_attunement_quant.unwrap_or(128),
                };
                Uplink::wisdom(language, mix, echo)
            }
            VoiceKind::GenerationEpithet => {
                let base = epithet::sample_entity(self.index.unwrap_or(0), seed);
                let standout = StandoutPortrait {
                    label: self.label.unwrap_or(base.label),
                    generation: self.generation.unwrap_or(base.generation),
                    fitness: self.fitness.unwrap_or(base.fitness),
                    viability: self.viability.unwrap_or(base.viability),
                    vitality: self.vitality.or(base.vitality),
                    resonance: self.resonance.or(base.resonance),
                    mutation_pressure: self.mutation_pressure.or(base.mutation_pressure),
                    symbolic_density: self.symbolic_density.or(base.symbolic_density),
                    memory_depth: self.memory_depth.or(base.memory_depth),
                    shadow_pull: self.shadow_pull.or(base.shadow_pull),
                    myth: self.myth.or(base.myth),
                };
                let generation = self.generation.unwrap_or(standout.generation);
                Uplink::generation_epithet(language, generation, standout)
            }
            VoiceKind::SampleEpithet => {
                Uplink::sample_epithet(language, self.index.unwrap_or(0), seed)
            }
        }
    }
}

/// Sole host entry: consume an ordered [`Uplink`], return an ordered [`Downlink`].
#[must_use]
pub fn answer(uplink: &Uplink) -> Downlink {
    match &uplink.voice {
        UplinkVoice::Wisdom { mix, echo } => {
            let text = wisdom::pick(uplink.language, *mix, echo);
            Downlink::filled(uplink.language, WhisperKind::Wisdom, text)
        }
        UplinkVoice::GenerationEpithet {
            generation,
            standout,
        } => {
            let text = epithet::forge(standout, *generation, uplink.language);
            Downlink::filled(uplink.language, WhisperKind::GenerationEpithet, text)
        }
        UplinkVoice::SampleEpithet { index, seed } => {
            let text = forge_sample(uplink.language, *index, *seed);
            Downlink::filled(uplink.language, WhisperKind::SampleEpithet, text)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wisdom_uplink_round_trip_shape() {
        let up = Uplink::wisdom(Language::English, 7, NarrativeEcho::default());
        let down = answer(&up);
        assert_eq!(down.schema, DOWNLINK_SCHEMA);
        assert_eq!(down.kind, WhisperKind::Wisdom);
        assert!(!down.empty);
        assert!(!down.text.is_empty());
    }

    #[test]
    fn epithet_uplink_is_deterministic() {
        let standout = StandoutPortrait {
            label: "probe".into(),
            generation: 2,
            fitness: 10.0,
            viability: 0.5,
            vitality: Some(0.6),
            resonance: Some(0.4),
            mutation_pressure: Some(0.7),
            symbolic_density: Some(0.2),
            memory_depth: Some(0.9),
            shadow_pull: Some(0.8),
            myth: None,
        };
        let up = Uplink::generation_epithet(Language::Spanish, 2, standout);
        let a = answer(&up);
        let b = answer(&up);
        assert_eq!(a.text, b.text);
        assert_eq!(a.kind, WhisperKind::GenerationEpithet);
        assert!(!a.empty);
    }

    #[test]
    fn draft_fills_omitted_fields() {
        let up = UplinkDraft {
            language: Some(Language::Russian),
            label: Some("keeper".into()),
            shadow_pull: Some(0.9),
            seed: Some(7),
            ..UplinkDraft::default()
        }
        .finish();
        assert!(matches!(up.voice, UplinkVoice::GenerationEpithet { .. }));
        let down = answer(&up);
        assert!(!down.empty);
    }

    #[test]
    fn draft_defaults_to_sample() {
        let up = UplinkDraft {
            seed: Some(1),
            ..UplinkDraft::default()
        }
        .finish();
        assert!(matches!(up.voice, UplinkVoice::SampleEpithet { .. }));
    }
}
