//! Locale tables embedded at compile time (`locales/*.toml`).

use serde::Deserialize;

use super::grammar::{
    AgentEntry, InflectedWord, ProperName, QualifierEntry, StemEntry, VerbalState,
};
use super::Language;

#[derive(Debug, Clone, Deserialize)]
pub struct EpithetTables {
    pub stems: Vec<StemEntry>,
    pub qualifiers: Vec<QualifierEntry>,
    pub adjectives: Vec<InflectedWord>,
    #[serde(default)]
    pub titles: Vec<InflectedWord>,
    #[serde(default)]
    pub curses: Vec<InflectedWord>,
    #[serde(default)]
    pub verbal_states: Vec<VerbalState>,
    #[serde(default)]
    pub verbal_agents: Vec<AgentEntry>,
    #[serde(default)]
    pub proper_names: Vec<ProperName>,
    /// Linker between participle and a `proper_names` agent (`por`, `by`, …).
    #[serde(default)]
    pub named_verbal_linker: String,
    #[serde(default)]
    pub semantic: SemanticTables,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LocaleTables {
    pub wisdom: WisdomTables,
    pub epithet: EpithetTables,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WisdomTables {
    pub fragments: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct EpithetVocabOverlay {
    epithet: EpithetTables,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct SemanticTables {
    /// Coherence motifs inferred from free text (`abismo` -> `abyss`).
    #[serde(default)]
    pub motif_markers: Vec<SemanticMarkerSet>,
    /// Motif pairs that must not co-occur (`fire` vs `cold`).
    #[serde(default)]
    pub motif_conflicts: Vec<SemanticTagConflict>,
    /// Head-noun archetypes inferred from stem surface (`espejo` -> `mirror`).
    #[serde(default)]
    pub archetype_markers: Vec<SemanticMarkerSet>,
    /// Redundancy groups inferred from stem surface (`roto` -> `damage`).
    #[serde(default)]
    pub stem_group_markers: Vec<SemanticMarkerSet>,
    /// Stem class when `semantic` / class tags are absent (`rey` -> `person`).
    #[serde(default)]
    pub stem_class_markers: Vec<SemanticMarkerSet>,
    /// Thematic families for soft alignment (`funeral`, `fallen_royal`, …).
    #[serde(default)]
    pub themes: Vec<SemanticMarkerSet>,
    /// Participial verb classes (`seal`, `bury`, `drag`, …).
    #[serde(default)]
    pub verb_tag_markers: Vec<SemanticMarkerSet>,
    /// Agent classes (`storm`, `titan`, `void`, …).
    #[serde(default)]
    pub agent_classes: Vec<SemanticMarkerSet>,
    /// Agent-experience participles (`traicionado`, `betrayed`, ...).
    #[serde(default)]
    pub verbal_agent_experience_markers: Vec<String>,
    /// Physical participles (`quemado`, `crushed`, ...).
    #[serde(default)]
    pub verbal_physical_markers: Vec<String>,
    /// Mind/cruelty adjectives (`despiadado`, `pitiless`, …) — not for inanimate objects.
    #[serde(default)]
    pub living_trait_markers: Vec<String>,
    /// Tags that must not co-occur on modifiers (`fire_active` vs `fire_done`).
    #[serde(default)]
    pub tag_conflicts: Vec<SemanticTagConflict>,
    /// Per-archetype adjective tag allow/forbid (`edge` forbids `hollow_volume`).
    #[serde(default)]
    pub archetype_compat: Vec<ArchetypeCompatRule>,
    /// Stem tag/class → forbidden adjective kinds or tags (`object` forbids `living_trait`).
    #[serde(default)]
    pub adjective_stem_rules: Vec<AdjectiveStemRule>,
    /// Per-archetype participial damage modes (`mirror` forbids `tear`, …).
    #[serde(default)]
    pub archetype_verbal_compat: Vec<ArchetypeCompatRule>,
    /// Soft verb↔agent pairing (weight only — poetic exceptions stay possible).
    #[serde(default)]
    pub verb_agent_rules: Vec<VerbAgentRule>,
    /// Vague indefinite agents to reject (`alguien`, `someone`, …).
    #[serde(default)]
    pub banned_indefinite_markers: Vec<String>,
    /// Vague concrete-looking agents to reject (`el primer nombre`, …).
    #[serde(default)]
    pub banned_agent_markers: Vec<String>,
    /// Agent [`AgentEntry::tags`] that may attribute verbs poetically (`time`, `storm`, …).
    #[serde(default)]
    pub poetic_actor_tags: Vec<String>,
    /// [`VerbalState::verb_tags`] that must appear with `por` / `by` / instrumental agent.
    #[serde(default)]
    pub agent_required_verb_tags: Vec<String>,
    /// Surface fallbacks when checking dangling participles (`tocad`, `touch`, …).
    #[serde(default)]
    pub agent_required_participle_markers: Vec<String>,
    /// Qualifier phrase prefixes that attribute origin (`del `, `of `) — block patronymic suffix.
    #[serde(default)]
    pub possessive_origin_qualifier_prefixes: Vec<String>,
    /// Qualifier substrings when prefixes do not apply (e.g. bare Russian genitive).
    #[serde(default)]
    pub possessive_origin_qualifier_markers: Vec<String>,
    /// Substrings that mark an attached verbal agent (` por `, ` by `).
    #[serde(default)]
    pub verbal_agent_linkers: Vec<String>,
    /// Reject full verbal phrases containing these fragments (`чем-то`, `someone`, …).
    #[serde(default)]
    pub banned_verbal_surface_markers: Vec<String>,
    /// Russian: last-token suffixes for instrumental agents (`ом`, `ой`, `тенью`, …).
    #[serde(default)]
    pub russian_instrumental_agent_suffixes: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct SemanticTagConflict {
    pub a: String,
    pub b: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ArchetypeCompatRule {
    pub archetype: String,
    #[serde(default)]
    pub allows: Vec<String>,
    #[serde(default)]
    pub forbids: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct AdjectiveStemRule {
    /// Stem [`SemanticContext::tags`] that trigger this rule (`object`, `relic`, …).
    #[serde(default)]
    pub stem_tags: Vec<String>,
    /// Stem class keys (`object`, `person`, `abstract`, …) from [`StemClass`].
    #[serde(default)]
    pub stem_classes: Vec<String>,
    /// Reject adjectives whose [`WordKind`] matches (`living_trait`, …).
    #[serde(default)]
    pub forbid_kinds: Vec<String>,
    /// Reject adjectives carrying any of these tags.
    #[serde(default)]
    pub forbid_tags: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct VerbAgentRule {
    #[serde(default)]
    pub verb_tags: Vec<String>,
    #[serde(default)]
    pub forbid_agent_classes: Vec<String>,
    #[serde(default)]
    pub allow_agent_classes: Vec<String>,
    /// When true, [`forbid_agent_classes`] rejects the agent outright (not just lower weight).
    #[serde(default)]
    pub hard: bool,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct SemanticMarkerSet {
    pub key: String,
    #[serde(default)]
    pub markers: Vec<String>,
}

fn extend_epithet(base: &mut EpithetTables, extra: EpithetTables) {
    base.stems.extend(extra.stems);
    base.qualifiers.extend(extra.qualifiers);
    base.adjectives.extend(extra.adjectives);
    base.titles.extend(extra.titles);
    base.curses.extend(extra.curses);
    base.verbal_states.extend(extra.verbal_states);
    base.verbal_agents.extend(extra.verbal_agents);
    base.proper_names.extend(extra.proper_names);
    if !extra.named_verbal_linker.is_empty() {
        base.named_verbal_linker = extra.named_verbal_linker;
    }
    base.semantic
        .motif_markers
        .extend(extra.semantic.motif_markers);
    base.semantic
        .motif_conflicts
        .extend(extra.semantic.motif_conflicts);
    base.semantic
        .archetype_markers
        .extend(extra.semantic.archetype_markers);
    base.semantic
        .stem_group_markers
        .extend(extra.semantic.stem_group_markers);
    base.semantic
        .stem_class_markers
        .extend(extra.semantic.stem_class_markers);
    base.semantic
        .verbal_agent_experience_markers
        .extend(extra.semantic.verbal_agent_experience_markers);
    base.semantic
        .verbal_physical_markers
        .extend(extra.semantic.verbal_physical_markers);
    base.semantic
        .living_trait_markers
        .extend(extra.semantic.living_trait_markers);
    base.semantic.themes.extend(extra.semantic.themes);
    base.semantic
        .verb_tag_markers
        .extend(extra.semantic.verb_tag_markers);
    base.semantic
        .agent_classes
        .extend(extra.semantic.agent_classes);
    base.semantic
        .tag_conflicts
        .extend(extra.semantic.tag_conflicts);
    base.semantic
        .archetype_compat
        .extend(extra.semantic.archetype_compat);
    base.semantic
        .adjective_stem_rules
        .extend(extra.semantic.adjective_stem_rules);
    base.semantic
        .archetype_verbal_compat
        .extend(extra.semantic.archetype_verbal_compat);
    base.semantic
        .verb_agent_rules
        .extend(extra.semantic.verb_agent_rules);
    base.semantic
        .banned_indefinite_markers
        .extend(extra.semantic.banned_indefinite_markers);
    base.semantic
        .banned_agent_markers
        .extend(extra.semantic.banned_agent_markers);
    base.semantic
        .poetic_actor_tags
        .extend(extra.semantic.poetic_actor_tags);
    base.semantic
        .agent_required_verb_tags
        .extend(extra.semantic.agent_required_verb_tags);
    base.semantic
        .agent_required_participle_markers
        .extend(extra.semantic.agent_required_participle_markers);
    base.semantic
        .possessive_origin_qualifier_prefixes
        .extend(extra.semantic.possessive_origin_qualifier_prefixes);
    base.semantic
        .possessive_origin_qualifier_markers
        .extend(extra.semantic.possessive_origin_qualifier_markers);
    base.semantic
        .verbal_agent_linkers
        .extend(extra.semantic.verbal_agent_linkers);
    base.semantic
        .banned_verbal_surface_markers
        .extend(extra.semantic.banned_verbal_surface_markers);
    base.semantic
        .russian_instrumental_agent_suffixes
        .extend(extra.semantic.russian_instrumental_agent_suffixes);
}

fn parse_locale(base_toml: &str, vocab_toml: &str, label: &'static str) -> LocaleTables {
    let mut locale: LocaleTables =
        toml::from_str(base_toml).unwrap_or_else(|e| panic!("invalid whisper locale {label}: {e}"));
    let overlay: EpithetVocabOverlay =
        toml::from_str(vocab_toml).unwrap_or_else(|e| panic!("invalid whisper vocab {label}: {e}"));
    extend_epithet(&mut locale.epithet, overlay.epithet);
    validate_stem_families(&locale.epithet.stems, label);
    validate_named_verbal_linker(&locale.epithet, label);
    locale
}

fn validate_named_verbal_linker(epithet: &EpithetTables, label: &'static str) {
    if label == "ru" {
        return;
    }
    if epithet.named_verbal_linker.trim().is_empty() {
        panic!("locale {label}: epithet.named_verbal_linker must be set in TOML (e.g. por / by)");
    }
}

fn validate_stem_families(stems: &[StemEntry], label: &'static str) {
    for stem in stems {
        if stem.family.trim().is_empty() {
            panic!(
                "epithet stem {:?} in locale {label} missing family (English id, e.g. filament)",
                stem.text
            );
        }
    }
}

static EN: std::sync::OnceLock<LocaleTables> = std::sync::OnceLock::new();
static ES: std::sync::OnceLock<LocaleTables> = std::sync::OnceLock::new();
static RU: std::sync::OnceLock<LocaleTables> = std::sync::OnceLock::new();

/// Loaded tables for the active language.
#[must_use]
pub fn tables(language: Language) -> &'static LocaleTables {
    match language {
        Language::English => EN.get_or_init(|| {
            parse_locale(
                include_str!("locales/en.toml"),
                include_str!("locales/en_vocab.toml"),
                "en",
            )
        }),
        Language::Spanish => ES.get_or_init(|| {
            parse_locale(
                include_str!("locales/es.toml"),
                include_str!("locales/es_vocab.toml"),
                "es",
            )
        }),
        Language::Russian => RU.get_or_init(|| {
            parse_locale(
                include_str!("locales/ru.toml"),
                include_str!("locales/ru_vocab.toml"),
                "ru",
            )
        }),
    }
}
