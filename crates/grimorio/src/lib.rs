//! Grimorio de los Mil Nombres — fragmentary lore and generation epithets.
//!
//! Host contract: build an ordered [`Uplink`], call [`answer`], read [`Downlink`].
//!
//! - [`UplinkVoice::Wisdom`] — partial lore.
//! - [`UplinkVoice::GenerationEpithet`] — Diablo 2–style names from trait axes.
//! - [`UplinkVoice::SampleEpithet`] — CLI / demo samples.
//!
//! Locales: `locales/en.toml`, `locales/es.toml`, `locales/ru.toml`.

mod common;
mod epithet;
mod grammar;
mod link;
mod locale;
mod portrait;
mod semantic;
mod surface;
mod traits;
mod verbal;
mod wisdom;

pub use grammar::{
    AgentEntry, AgreementKey, Gender, InflectedWord, Number, ProperName, QualifierEntry, StemEntry,
    VerbalState,
};

pub use common::{fnv1a64, mix_echo, mix_u64, quantize01, Language, NarrativeEcho};
pub use epithet::forge as forge_generation_epithet;
pub use epithet::{forge_sample, sample_entity};
pub use link::{
    answer, Downlink, Uplink, UplinkDraft, UplinkVoice, VoiceKind, DOWNLINK_SCHEMA, UPLINK_SCHEMA,
};
pub use portrait::StandoutPortrait;
pub use traits::{
    GenerationEpithetVoice, WhisperHub, WhisperKind, WhisperRequest, WhisperVoice, WisdomVoice,
};
pub use wisdom::pick as pick_wisdom;

/// Deterministic English fragment (stable for tests and legacy callers).
#[must_use]
pub fn pick_whisper(mix: u64) -> &'static str {
    wisdom::pick_english_fragment(mix, &NarrativeEcho::default())
}

/// Deterministic fragment with echo bias (English table, stable `&str` for legacy callers).
#[must_use]
pub fn pick_narrative_whisper(base_mix: u64, echo: &NarrativeEcho) -> &'static str {
    wisdom::pick_english_fragment(base_mix, echo)
}

/// Localized wisdom line (legacy helper; prefer [`answer`] + [`Uplink::wisdom`]).
#[must_use]
pub fn pick_narrative_whisper_localized(
    language: Language,
    base_mix: u64,
    echo: &NarrativeEcho,
) -> String {
    answer(&Uplink::wisdom(language, base_mix, echo.clone())).text
}

/// Standout epithet for a portrait (legacy helper; prefer [`answer`] + [`Uplink::generation_epithet`]).
#[must_use]
pub fn standout_epithet(
    entity: &StandoutPortrait,
    generation: u32,
    language: Language,
) -> Option<String> {
    let down = answer(&Uplink::generation_epithet(
        language,
        generation,
        entity.clone(),
    ));
    if down.empty {
        None
    } else {
        Some(down.text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pick_whisper_is_deterministic() {
        assert_eq!(pick_whisper(0xC0FFEE), pick_whisper(0xC0FFEE));
    }

    #[test]
    fn pick_whisper_covers_table() {
        let n = locale::tables(Language::English).wisdom.fragments.len();
        for i in 0..n {
            let line = pick_whisper(i as u64);
            assert!(!line.is_empty());
        }
    }

    #[test]
    fn narrative_echo_biases_index_without_panic() {
        let echo = NarrativeEcho {
            dominant_tone_idx: 3,
            scar_mass: 42,
            rare_event_token: 0xBEEF,
            dying_viability_quant: 200,
            fossil_absence_mass: 1200,
            attention_xor: 0xC0DED00D,
            soul_attunement_quant: 180,
        };
        let a = pick_narrative_whisper(0x1111, &NarrativeEcho::default());
        let b = pick_narrative_whisper(0x1111, &echo);
        assert!(!a.is_empty() && !b.is_empty());
    }

    #[test]
    fn epithet_omits_origin_and_curse_when_traits_are_low() {
        let entity = StandoutPortrait {
            label: "quiet_lattice".to_string(),
            generation: 1,
            fitness: 5.0,
            viability: 0.7,
            vitality: Some(0.5),
            resonance: Some(0.2),
            mutation_pressure: Some(0.15),
            symbolic_density: Some(0.25),
            memory_depth: Some(0.1),
            shadow_pull: Some(0.12),
            myth: None,
        };
        let name = forge_generation_epithet(&entity, 1, Language::Spanish);
        assert!(!name.is_empty());
        assert!(!name.contains("maldito"));
        assert!(!name.contains("hechizado"));
        assert!(!name.contains("del abismo"));
        assert!(!name.contains("de la desesperación"));
    }

    #[test]
    fn epithet_forge_is_deterministic_and_non_empty() {
        let entity = StandoutPortrait {
            label: "ResonanceEngine".to_string(),
            generation: 4,
            fitness: 31.5,
            viability: 0.62,
            vitality: Some(0.71),
            resonance: Some(0.55),
            mutation_pressure: Some(0.48),
            symbolic_density: Some(0.33),
            memory_depth: Some(0.8),
            shadow_pull: Some(0.77),
            myth: Some("keeper".to_string()),
        };
        let a = forge_generation_epithet(&entity, 4, Language::Spanish);
        let b = forge_generation_epithet(&entity, 4, Language::Spanish);
        assert_eq!(a, b);
        assert!(!a.is_empty());
    }

    #[test]
    fn verbal_phrase_splits_participle_and_agent() {
        use super::grammar::{AgentEntry, VerbalState};
        use super::verbal::compose_verbal;

        let key = AgreementKey::from_tags("m", "s");
        let state = VerbalState {
            ms: Some("sellado".to_string()),
            fs: Some("sellada".to_string()),
            ..VerbalState::default()
        };
        let participle = state.participle(key).unwrap();
        let agent = AgentEntry {
            text: "la sombra".to_string(),
            g: Some("f".to_string()),
            n: Some("s".to_string()),
            linker: "por".to_string(),
            indefinite: false,
            ..AgentEntry::default()
        };
        assert_eq!(
            compose_verbal(Language::Spanish, participle, Some(&agent)),
            "sellado por la sombra"
        );
        assert_eq!(
            compose_verbal(Language::Spanish, participle, None),
            "sellado"
        );
    }

    #[test]
    fn localized_wisdom_differs_by_language() {
        let echo = NarrativeEcho::default();
        let en = pick_wisdom(Language::English, 42, &echo);
        let es = pick_wisdom(Language::Spanish, 42, &echo);
        assert_ne!(en, es);
    }

    #[test]
    fn host_path_uses_uplink_downlink() {
        let up = Uplink::sample_epithet(Language::Russian, 0, 99);
        let down = answer(&up);
        assert_eq!(down.schema, DOWNLINK_SCHEMA);
        assert_eq!(up.schema, UPLINK_SCHEMA);
        assert!(!down.empty);
    }
}
