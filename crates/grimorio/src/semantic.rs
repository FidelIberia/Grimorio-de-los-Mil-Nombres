//! Semantic filters for epithet assembly — stem class, modifier compatibility, redundancy.
//!
//! Lexical semantics live in locale TOML (`archetype_markers`, `stem_group_markers`, …).
//! Rust only implements generic marker inference ([`infer_marker_keys`], [`keys_conflict`]).

use std::collections::HashSet;

use super::grammar::{AgentEntry, InflectedWord, StemEntry, VerbalState};
use super::locale::{AdjectiveStemRule, ArchetypeCompatRule, SemanticTables, VerbAgentRule};

/// Broad class of the head noun phrase.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StemClass {
    Object,
    Person,
    Divine,
    Creature,
    Abstract,
}

/// Role of an inflected piece in the lexicon.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WordKind {
    ObjectAdj,
    LivingTrait,
    CharacterEpithet,
}

/// Where a word is being attached in the forge pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PickSlot {
    Title,
    PrologueAdj,
    Modifier,
    Curse,
}

/// Agreement + semantics for the chosen stem.
#[derive(Debug, Clone)]
pub struct SemanticContext {
    pub class: StemClass,
    pub tags: HashSet<String>,
    pub groups: HashSet<String>,
    pub motifs: HashSet<String>,
    pub themes: HashSet<String>,
}

/// Tracks modifiers already committed in the epilogue.
#[derive(Debug, Clone, Default)]
pub struct ModifierState {
    pub used_groups: HashSet<String>,
    pub used_tags: HashSet<String>,
    /// Agreed surface forms already placed (blocks prologue/epilogue duplicates).
    pub used_surfaces: HashSet<String>,
    pub count: usize,
}

pub const MAX_EPILOGUE_MODIFIERS: usize = 2;

/// How a participle relates to the head noun.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerbalKind {
    /// Sealed, profaned, marked — fine on relics and objects.
    ObjectFate,
    /// Torn, burned, crushed — physical state of things or bodies.
    Physical,
    /// Betrayed, sworn, awakened — needs a conscious subject.
    AgentExperience,
}

/// Where to place a verbal phrase relative to the core name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerbalPlacement {
    /// `Sellado por X, Reliquia` — dramatic opener for living heads.
    CommaPrologue,
    /// `Reliquia sellada por X` — natural for Spanish/Russian objects.
    PostNominal,
}

impl SemanticContext {
    #[must_use]
    pub fn from_stem(stem: &StemEntry, semantic: &SemanticTables) -> Self {
        let class = classify_stem(stem, semantic);
        let mut tags: HashSet<String> = stem.tags.iter().map(|t| normalize_tag(t)).collect();
        tags.insert(class_tag(class).to_string());
        let mut groups: HashSet<String> = stem.groups.iter().map(|g| normalize_tag(g)).collect();
        groups.extend(infer_groups_from_text(&stem.text, semantic));
        tags.extend(infer_archetype_tags(&stem.text, semantic));
        let motifs = infer_motifs(&stem.text, semantic);
        let themes = infer_marker_keys(&stem.text, &semantic.themes);
        Self {
            class,
            tags,
            groups,
            motifs,
            themes,
        }
    }
}

/// Whether a verbal phrase includes an explicit agent (linker or Russian instrumental).
#[must_use]
pub fn phrase_has_verbal_agent(
    language: super::common::Language,
    phrase: &str,
    semantic: &SemanticTables,
) -> bool {
    if semantic
        .verbal_agent_linkers
        .iter()
        .map(|l| normalize_tag(l))
        .any(|l| !l.is_empty() && phrase.contains(l.as_str()))
    {
        return true;
    }
    language == super::common::Language::Russian
        && russian_participle_has_instrumental_agent(phrase, semantic)
}

/// Participles that imply an agent must not appear without one in the surface phrase.
#[must_use]
pub fn verbal_phrase_grammatical(
    language: super::common::Language,
    phrase: &str,
    semantic: &SemanticTables,
) -> bool {
    let lower = phrase.trim().to_lowercase();
    if lower.is_empty() {
        return false;
    }
    if surface_matches_any_marker(&lower, &semantic.banned_verbal_surface_markers)
        || surface_matches_any_marker(&lower, &semantic.banned_indefinite_markers)
        || surface_matches_any_marker(&lower, &semantic.banned_agent_markers)
    {
        return false;
    }
    if participle_requires_agent_surface(phrase, semantic)
        && !phrase_has_verbal_agent(language, phrase, semantic)
    {
        return false;
    }
    true
}

/// Patronymic suffix is incompatible with the current verbal / qualifier layout.
#[must_use]
pub fn patron_grammatically_allowed(
    language: super::common::Language,
    qualifier: Option<&str>,
    prologue: Option<&str>,
    prologue_is_verbal: bool,
    post_nominal_verbal: Option<&str>,
    semantic: &SemanticTables,
) -> bool {
    !patron_blocked_by_qualifier(qualifier, semantic)
        && !patron_blocked_by_verbal_attribution(
            language,
            prologue,
            prologue_is_verbal,
            post_nominal_verbal,
            semantic,
        )
}

/// Verbal attribution (`por alguien`, `by the spiral`) and patronymic (`de Miguel`) do not combine.
#[must_use]
pub fn patron_blocked_by_verbal_attribution(
    language: super::common::Language,
    prologue: Option<&str>,
    prologue_is_verbal: bool,
    post_nominal_verbal: Option<&str>,
    semantic: &SemanticTables,
) -> bool {
    if let Some(p) = post_nominal_verbal {
        if phrase_has_verbal_agent(language, p, semantic) {
            return true;
        }
        if participle_requires_agent_surface(p, semantic) {
            return true;
        }
    }
    if prologue_is_verbal {
        if let Some(p) = prologue {
            if phrase_has_verbal_agent(language, p, semantic) {
                return true;
            }
            if participle_requires_agent_surface(p, semantic) {
                return true;
            }
        }
    }
    false
}

fn russian_participle_has_instrumental_agent(phrase: &str, semantic: &SemanticTables) -> bool {
    let words: Vec<&str> = phrase.split_whitespace().collect();
    if words.len() < 2 || !super::surface::russian_looks_like_participle(words[0]) {
        return false;
    }
    let agent = words.last().unwrap_or(&"");
    let lower = agent.to_lowercase();
    if semantic.russian_instrumental_agent_suffixes.is_empty() {
        return super::surface::russian_participle_has_agent_phrase(phrase);
    }
    semantic
        .russian_instrumental_agent_suffixes
        .iter()
        .map(|s| normalize_tag(s))
        .any(|s| !s.is_empty() && lower.ends_with(s.as_str()))
}

/// Participles that only make sense with an agent (`tocada por X`, not `tocada de X`).
#[must_use]
pub fn verbal_state_requires_agent(state: &VerbalState, semantic: &SemanticTables) -> bool {
    if state.requires_agent {
        return true;
    }
    let modes = infer_verb_tags(state, semantic);
    if semantic
        .agent_required_verb_tags
        .iter()
        .map(|t| normalize_tag(t))
        .any(|t| !t.is_empty() && modes.contains(&t))
    {
        return true;
    }
    infer_agent_experience_participle(state, semantic) || infer_physical_participle(state, semantic)
}

fn participle_requires_agent_surface(participle: &str, semantic: &SemanticTables) -> bool {
    let lower = participle.to_lowercase();
    surface_matches_any_marker(&lower, &semantic.agent_required_participle_markers)
        || surface_matches_any_marker(&lower, &semantic.verbal_physical_markers)
        || surface_matches_any_marker(&lower, &semantic.verbal_agent_experience_markers)
}

fn surface_matches_any_marker(lower: &str, markers: &[String]) -> bool {
    markers
        .iter()
        .map(|m| normalize_tag(m))
        .any(|m| !m.is_empty() && lower.contains(m.as_str()))
}

/// Qualifiers that attribute origin (`del reinado`, `of the throne`) clash with patronymic `de X`.
#[must_use]
pub fn qualifier_is_possessive_origin(qualifier: &str, semantic: &SemanticTables) -> bool {
    let q = qualifier.trim().to_ascii_lowercase();
    if semantic
        .possessive_origin_qualifier_prefixes
        .iter()
        .map(|p| normalize_tag(p))
        .any(|p| !p.is_empty() && q.starts_with(p.as_str()))
    {
        return true;
    }
    surface_matches_any_marker(&q, &semantic.possessive_origin_qualifier_markers)
}

/// Patronymic suffix (`de Modi`, `of Miguel`) must not follow a genitive qualifier.
#[must_use]
pub fn patron_blocked_by_qualifier(qualifier: Option<&str>, semantic: &SemanticTables) -> bool {
    qualifier.is_some_and(|q| qualifier_is_possessive_origin(q, semantic))
}

/// Qualifier after a `por …` verbal reads as modifying the agent, not the relic (`… por X de Y`).
#[must_use]
pub fn qualifier_conflicts_with_verbal_agent(
    language: super::common::Language,
    qualifier: &str,
    post_nominal_verbal: Option<&str>,
    semantic: &SemanticTables,
) -> bool {
    let Some(verbal) = post_nominal_verbal else {
        return false;
    };
    if !phrase_has_verbal_agent(language, verbal, semantic) {
        return false;
    }
    let agent_text = verbal_agent_segment(language, verbal);
    let q_themes = infer_marker_keys(qualifier, &semantic.themes);
    let a_themes = infer_marker_keys(agent_text, &semantic.themes);
    if !q_themes.is_disjoint(&a_themes) {
        return true;
    }
    let q_motifs = infer_marker_keys(qualifier, &semantic.motif_markers);
    let a_motifs = infer_marker_keys(agent_text, &semantic.motif_markers);
    !q_motifs.is_disjoint(&a_motifs)
}

fn verbal_agent_segment(language: super::common::Language, verbal: &str) -> &str {
    match language {
        super::common::Language::Spanish => verbal
            .split_once(" por ")
            .map(|(_, agent)| agent)
            .unwrap_or(verbal),
        super::common::Language::English => verbal
            .split_once(" by ")
            .map(|(_, agent)| agent)
            .unwrap_or(verbal),
        super::common::Language::Russian => verbal.split_whitespace().last().unwrap_or(verbal),
    }
}

#[must_use]
pub fn title_allowed(ctx: &SemanticContext) -> bool {
    matches!(
        ctx.class,
        StemClass::Person | StemClass::Divine | StemClass::Creature
    )
}

#[must_use]
pub fn allows_verbal_state(
    state: &VerbalState,
    ctx: &SemanticContext,
    semantic: &SemanticTables,
) -> bool {
    let kind = effective_verbal_kind(state, semantic);
    verbal_kind_allowed_on_stem(kind, ctx)
        && verbal_requires_satisfied(state, ctx)
        && archetype_allows_verbal(state, ctx, semantic)
}

#[must_use]
pub fn verbal_placement(ctx: &SemanticContext) -> VerbalPlacement {
    match ctx.class {
        StemClass::Object | StemClass::Abstract => VerbalPlacement::PostNominal,
        StemClass::Person | StemClass::Divine | StemClass::Creature => {
            VerbalPlacement::CommaPrologue
        }
    }
}

/// Extra constraints when attaching a verbal agent (`por X`, `by X`, …).
#[derive(Debug, Clone, Default)]
pub struct VerbalForgeContext {
    pub has_patron: bool,
    pub verb_tags: HashSet<String>,
}

/// Plural unique relics (`Tronos vacíos`) stay generic; `de Janaka` needs the singular stem.
#[must_use]
pub fn stem_allows_patron(stem: &StemEntry) -> bool {
    !(stem.unique && stem.n == "p")
}

/// Coherence weight for lexical pieces against the chosen stem context.
/// Higher means "more likely", but all candidates can still appear.
#[must_use]
pub fn text_coherence_weight(ctx: &SemanticContext, text: &str, semantic: &SemanticTables) -> u64 {
    let motifs = infer_motifs(text, semantic);
    if motifs.is_empty() {
        return 10;
    }
    if ctx.motifs.is_empty() {
        return 12;
    }
    let overlap = motifs.iter().filter(|m| ctx.motifs.contains(*m)).count() as u64;
    if overlap > 0 {
        return 14 + (overlap * 18);
    }
    if keys_conflict(&ctx.motifs, &motifs, &semantic.motif_conflicts) {
        return 2;
    }
    6
}

/// Deterministic weighted pick for candidate lists.
#[must_use]
pub fn weighted_pick(seed: u64, slot: u32, weights: &[u64]) -> usize {
    if weights.is_empty() {
        return 0;
    }
    let total = weights
        .iter()
        .fold(0_u64, |acc, w| acc.saturating_add((*w).max(1)));
    if total == 0 {
        return 0;
    }
    let mixed = seed
        .wrapping_mul(0x9E37_79B9_7F4A_7C15)
        .wrapping_add((slot as u64).wrapping_mul(0x517c_c1b7_2722_0a95));
    let mut target = (mixed >> 16) % total;
    for (i, w) in weights.iter().enumerate() {
        let w = (*w).max(1);
        if target < w {
            return i;
        }
        target -= w;
    }
    weights.len().saturating_sub(1)
}

#[must_use]
pub fn allows_verbal_agent(
    agent: &AgentEntry,
    forge: &VerbalForgeContext,
    semantic: &SemanticTables,
) -> bool {
    if is_banned_agent(agent, semantic) {
        return false;
    }
    if forge.has_patron && agent.indefinite {
        return false;
    }
    if is_abstract_agent(agent, semantic) {
        return false;
    }
    !verb_agent_hard_forbidden(agent, forge, semantic)
}

fn verb_agent_hard_forbidden(
    agent: &AgentEntry,
    forge: &VerbalForgeContext,
    semantic: &SemanticTables,
) -> bool {
    if forge.verb_tags.is_empty() {
        return false;
    }
    let agent_classes = agent_classes_for(agent, semantic);
    if agent_classes.is_empty() {
        return false;
    }
    semantic.verb_agent_rules.iter().any(|rule| {
        if !rule.hard || !rule_applies_to_verb(rule, &forge.verb_tags) {
            return false;
        }
        rule.forbid_agent_classes
            .iter()
            .map(|c| normalize_tag(c))
            .any(|c| !c.is_empty() && agent_classes.contains(&c))
    })
}

/// Soft weight for verb↔agent pairing (never zero — poetic license preserved).
#[must_use]
pub fn verbal_agent_coherence_weight(
    agent: &AgentEntry,
    verb_tags: &HashSet<String>,
    semantic: &SemanticTables,
) -> u64 {
    if verb_tags.is_empty() {
        return 12;
    }
    let agent_classes = agent_classes_for(agent, semantic);
    if agent_classes.is_empty() {
        return 10;
    }
    let mut weight = 12_u64;
    for rule in &semantic.verb_agent_rules {
        if !rule_applies_to_verb(rule, verb_tags) {
            continue;
        }
        let forbid: HashSet<String> = rule
            .forbid_agent_classes
            .iter()
            .map(|c| normalize_tag(c))
            .collect();
        if forbid.iter().any(|c| agent_classes.contains(c)) {
            weight = weight.min(4);
        }
        let allow: HashSet<String> = rule
            .allow_agent_classes
            .iter()
            .map(|c| normalize_tag(c))
            .collect();
        if allow.iter().any(|c| agent_classes.contains(c)) {
            weight = weight.saturating_add(14);
        }
    }
    weight
}

/// Soft alignment between stem themes and qualifier/patron text.
#[must_use]
pub fn theme_coherence_weight(ctx: &SemanticContext, text: &str, semantic: &SemanticTables) -> u64 {
    let text_themes = infer_marker_keys(text, &semantic.themes);
    if ctx.themes.is_empty() || text_themes.is_empty() {
        return 10;
    }
    let overlap = text_themes
        .iter()
        .filter(|t| ctx.themes.contains(*t))
        .count() as u64;
    if overlap > 0 {
        10 + overlap * 8
    } else {
        7
    }
}

fn is_banned_agent(agent: &AgentEntry, semantic: &SemanticTables) -> bool {
    is_banned_indefinite(agent, semantic) || is_banned_agent_text(agent, semantic)
}

fn is_banned_indefinite(agent: &AgentEntry, semantic: &SemanticTables) -> bool {
    if !agent.indefinite {
        return false;
    }
    is_banned_agent_text(agent, semantic)
}

fn is_banned_agent_text(agent: &AgentEntry, semantic: &SemanticTables) -> bool {
    let lower = agent.text.to_lowercase();
    if semantic
        .banned_agent_markers
        .iter()
        .map(|m| normalize_tag(m))
        .any(|m| !m.is_empty() && lower.contains(m.as_str()))
    {
        return true;
    }
    semantic
        .banned_indefinite_markers
        .iter()
        .map(|m| normalize_tag(m))
        .any(|m| !m.is_empty() && lower.contains(m.as_str()))
}

fn is_abstract_agent(agent: &AgentEntry, semantic: &SemanticTables) -> bool {
    if agent_can_act_poetically(agent, semantic) {
        return false;
    }
    if let Some(ref k) = agent.kind {
        if normalize_tag(k) == "abstract" {
            return true;
        }
    }
    agent_classes_for(agent, semantic).contains("abstract")
}

/// Agents tagged in [`SemanticTables::poetic_actor_tags`] may attribute relics in mythic voice.
fn agent_can_act_poetically(agent: &AgentEntry, semantic: &SemanticTables) -> bool {
    agent.tags.iter().any(|t| {
        let n = normalize_tag(t);
        semantic
            .poetic_actor_tags
            .iter()
            .map(|p| normalize_tag(p))
            .any(|p| !p.is_empty() && p == n)
    })
}

/// True when the assembled name would be only a bare stem (no descriptor layer).
#[must_use]
pub fn output_needs_descriptor(
    qualifier: Option<&str>,
    patron: Option<&str>,
    title: Option<&str>,
    prologue: Option<&str>,
    post_nominal_verbal: Option<&str>,
    epilogue_modifiers: &[String],
    epilogue_curse: Option<&str>,
) -> bool {
    qualifier.is_none()
        && patron.is_none()
        && title.is_none()
        && prologue.is_none()
        && post_nominal_verbal.is_none()
        && epilogue_modifiers.is_empty()
        && epilogue_curse.is_none()
}

#[must_use]
pub fn infer_verb_tags(state: &VerbalState, semantic: &SemanticTables) -> HashSet<String> {
    let mut tags: HashSet<String> = state
        .verb_tags
        .iter()
        .map(|t| normalize_tag(t))
        .filter(|t| !t.is_empty())
        .collect();
    if tags.is_empty() {
        tags.extend(infer_marker_keys(
            &verbal_probe(state),
            &semantic.verb_tag_markers,
        ));
    }
    tags
}

#[must_use]
pub fn allows_word(
    word: &InflectedWord,
    ctx: &SemanticContext,
    slot: PickSlot,
    modifier_state: Option<&ModifierState>,
    semantic_rules: &SemanticTables,
) -> bool {
    let kind = effective_kind(word, slot, semantic_rules);
    if !kind_allowed_on_stem(kind, ctx) {
        return false;
    }
    if !requires_satisfied(word, ctx) {
        return false;
    }
    if word_forbids_stem_tags(word, ctx) {
        return false;
    }
    if group_collides(word, ctx, modifier_state) {
        return false;
    }
    if let Some(state) = modifier_state {
        if modifier_tag_collides(word, state, semantic_rules) {
            return false;
        }
    }
    if !archetype_allows_word(word, ctx, semantic_rules) {
        return false;
    }
    if !adjective_stem_allowed(word, ctx, slot, semantic_rules) {
        return false;
    }
    true
}

pub fn register_modifier(state: &mut ModifierState, word: &InflectedWord, surface: Option<&str>) {
    state.count += 1;
    if let Some(s) = surface {
        state.used_surfaces.insert(normalize_surface(s));
    }
    if let Some(ref g) = word.group {
        state.used_groups.insert(normalize_tag(g));
    }
    for t in &word.tags {
        state.used_tags.insert(normalize_tag(t));
    }
}

fn normalize_surface(surface: &str) -> String {
    surface.trim().to_lowercase()
}

#[must_use]
pub fn surface_already_used(surface: &str, state: &ModifierState) -> bool {
    state.used_surfaces.contains(&normalize_surface(surface))
}

/// Qualifier already present in the head surface (`Fragmentos del vacío` + `del vacío`).
#[must_use]
pub fn qualifier_redundant_with_stem(stem: &str, qualifier: &str) -> bool {
    let stem_n = normalize_surface(stem);
    let q_n = normalize_surface(qualifier);
    if q_n.is_empty() {
        return false;
    }
    stem_n.ends_with(&q_n) || stem_n.contains(&format!(" {q_n}"))
}

/// Modifier or curse already expressed on the head (`Проклятый замок` + `проклятый`).
#[must_use]
pub fn surface_redundant_with_stem(stem: &str, surface: &str) -> bool {
    if qualifier_redundant_with_stem(stem, surface) {
        return true;
    }
    let stem_n = normalize_surface(stem);
    let surf_n = normalize_surface(surface);
    if surf_n.is_empty() {
        return false;
    }
    stem_n
        .split_whitespace()
        .any(|w| w == surf_n || w.starts_with(surf_n.as_str()))
}

fn classify_stem(stem: &StemEntry, semantic: &SemanticTables) -> StemClass {
    if let Some(ref s) = stem.semantic {
        return parse_stem_class(s);
    }
    for tag in &stem.tags {
        if let Some(class) = class_from_tag(tag) {
            return class;
        }
    }
    infer_class_from_text(&stem.text, semantic)
}

fn parse_stem_class(s: &str) -> StemClass {
    match normalize_tag(s).as_str() {
        "person" | "human" => StemClass::Person,
        "divine" | "god" => StemClass::Divine,
        "creature" | "beast" | "monster" => StemClass::Creature,
        "abstract" => StemClass::Abstract,
        _ => StemClass::Object,
    }
}

fn class_from_tag(tag: &str) -> Option<StemClass> {
    match normalize_tag(tag).as_str() {
        "person" | "human" => Some(StemClass::Person),
        "divine" | "god" => Some(StemClass::Divine),
        "creature" | "beast" | "monster" => Some(StemClass::Creature),
        "abstract" => Some(StemClass::Abstract),
        "object" => Some(StemClass::Object),
        _ => None,
    }
}

fn class_tag(class: StemClass) -> &'static str {
    match class {
        StemClass::Object => "object",
        StemClass::Person => "person",
        StemClass::Divine => "divine",
        StemClass::Creature => "creature",
        StemClass::Abstract => "abstract",
    }
}

/// Untagged stems: class from [`SemanticTables::stem_class_markers`] only.
fn infer_class_from_text(text: &str, semantic: &SemanticTables) -> StemClass {
    const PRIORITY: &[&str] = &["divine", "person", "creature", "abstract", "object"];
    let keys = infer_marker_keys(text, &semantic.stem_class_markers);
    for key in PRIORITY {
        if keys.contains(*key) {
            return parse_stem_class(key);
        }
    }
    StemClass::Object
}

fn effective_kind(word: &InflectedWord, slot: PickSlot, semantic: &SemanticTables) -> WordKind {
    if slot == PickSlot::Title {
        return WordKind::CharacterEpithet;
    }
    if let Some(ref k) = word.kind {
        return parse_word_kind(k);
    }
    if infer_living_trait_lemma(word, semantic) {
        WordKind::LivingTrait
    } else {
        WordKind::ObjectAdj
    }
}

fn parse_word_kind(s: &str) -> WordKind {
    match normalize_tag(s).as_str() {
        "living_trait" | "living" | "mind" | "character_trait" => WordKind::LivingTrait,
        "character_epithet" | "epithet" | "title" => WordKind::CharacterEpithet,
        _ => WordKind::ObjectAdj,
    }
}

fn kind_allowed_on_stem(kind: WordKind, ctx: &SemanticContext) -> bool {
    match kind {
        WordKind::CharacterEpithet => title_allowed(ctx),
        WordKind::LivingTrait => matches!(
            ctx.class,
            StemClass::Person | StemClass::Divine | StemClass::Creature
        ),
        WordKind::ObjectAdj => true,
    }
}

fn requires_satisfied(word: &InflectedWord, ctx: &SemanticContext) -> bool {
    if word.requires.is_empty() {
        return true;
    }
    word.requires.iter().any(|req| {
        let n = normalize_tag(req);
        ctx.tags.contains(&n) || tag_matches_class(&n, ctx.class)
    })
}

fn tag_matches_class(tag: &str, class: StemClass) -> bool {
    class_from_tag(tag) == Some(class)
}

fn group_collides(
    word: &InflectedWord,
    ctx: &SemanticContext,
    modifier_state: Option<&ModifierState>,
) -> bool {
    let Some(ref g) = word.group else {
        return false;
    };
    let g = normalize_tag(g);
    if ctx.groups.contains(&g) {
        return true;
    }
    modifier_state
        .map(|s| s.used_groups.contains(&g))
        .unwrap_or(false)
}

fn modifier_tag_collides(
    word: &InflectedWord,
    state: &ModifierState,
    semantic_rules: &SemanticTables,
) -> bool {
    for f in &word.forbids {
        if state.used_tags.contains(&normalize_tag(f)) {
            return true;
        }
    }
    for t in &word.tags {
        let nt = normalize_tag(t);
        if tag_conflicts_with_set(&nt, &state.used_tags, semantic_rules) {
            return true;
        }
    }
    false
}

fn tag_conflicts_with_set(
    tag: &str,
    used: &HashSet<String>,
    semantic_rules: &SemanticTables,
) -> bool {
    for pair in &semantic_rules.tag_conflicts {
        let a = normalize_tag(&pair.a);
        let b = normalize_tag(&pair.b);
        if (tag == a && used.contains(&b)) || (tag == b && used.contains(&a)) {
            return true;
        }
    }
    false
}

fn adjective_stem_allowed(
    word: &InflectedWord,
    ctx: &SemanticContext,
    slot: PickSlot,
    semantic_rules: &SemanticTables,
) -> bool {
    if semantic_rules.adjective_stem_rules.is_empty() || slot == PickSlot::Title {
        return true;
    }
    let kind = effective_kind(word, slot, semantic_rules);
    let word_tags: HashSet<String> = word
        .tags
        .iter()
        .map(|t| normalize_tag(t))
        .filter(|t| !t.is_empty())
        .collect();
    let mut tags = word_tags;
    if matches!(kind, WordKind::LivingTrait) {
        tags.insert("living_trait".to_string());
    }
    for rule in &semantic_rules.adjective_stem_rules {
        if !stem_matches_adjective_rule(ctx, rule) {
            continue;
        }
        if rule
            .forbid_kinds
            .iter()
            .map(|k| parse_word_kind(k))
            .any(|k| k == kind)
        {
            return false;
        }
        if rule
            .forbid_tags
            .iter()
            .map(|t| normalize_tag(t))
            .any(|t| !t.is_empty() && tags.contains(&t))
        {
            return false;
        }
    }
    true
}

fn stem_matches_adjective_rule(ctx: &SemanticContext, rule: &AdjectiveStemRule) -> bool {
    if rule
        .stem_tags
        .iter()
        .map(|t| normalize_tag(t))
        .any(|t| ctx.tags.contains(&t))
    {
        return true;
    }
    rule.stem_classes
        .iter()
        .map(|c| normalize_tag(c))
        .any(|c| tag_matches_class(&c, ctx.class) || ctx.tags.contains(&c))
}

fn archetype_allows_word(
    word: &InflectedWord,
    ctx: &SemanticContext,
    semantic_rules: &SemanticTables,
) -> bool {
    let mut word_tags: HashSet<String> = word
        .tags
        .iter()
        .map(|t| normalize_tag(t))
        .filter(|t| !t.is_empty())
        .collect();
    if matches!(
        effective_kind(word, PickSlot::Modifier, semantic_rules),
        WordKind::LivingTrait
    ) {
        word_tags.insert("living_trait".to_string());
    }
    if word_tags.is_empty() {
        return true;
    }
    for rule in &semantic_rules.archetype_compat {
        if !ctx.tags.contains(&normalize_tag(&rule.archetype)) {
            continue;
        }
        if rule_forbids_word(rule, &word_tags) {
            return false;
        }
        if !rule.allows.is_empty() && !rule_allows_word(rule, &word_tags) {
            return false;
        }
    }
    true
}

fn rule_forbids_word(rule: &ArchetypeCompatRule, word_tags: &HashSet<String>) -> bool {
    rule.forbids
        .iter()
        .map(|f| normalize_tag(f))
        .any(|f| !f.is_empty() && word_tags.contains(&f))
}

fn rule_allows_word(rule: &ArchetypeCompatRule, word_tags: &HashSet<String>) -> bool {
    rule.allows
        .iter()
        .map(|a| normalize_tag(a))
        .any(|a| !a.is_empty() && word_tags.contains(&a))
}

/// Extra pick weight when a tagged adjective matches an archetype [`ArchetypeCompatRule::allows`] list.
#[must_use]
pub fn archetype_profile_weight(
    word: &InflectedWord,
    ctx: &SemanticContext,
    semantic_rules: &SemanticTables,
) -> u64 {
    let word_tags: HashSet<String> = word
        .tags
        .iter()
        .map(|t| normalize_tag(t))
        .filter(|t| !t.is_empty())
        .collect();
    if word_tags.is_empty() {
        return 0;
    }
    let mut boost = 0_u64;
    for rule in &semantic_rules.archetype_compat {
        if !ctx.tags.contains(&normalize_tag(&rule.archetype)) {
            continue;
        }
        if rule_allows_word(rule, &word_tags) {
            boost = boost.saturating_add(14);
        }
    }
    boost
}

fn agent_classes_for(agent: &AgentEntry, semantic: &SemanticTables) -> HashSet<String> {
    let mut classes: HashSet<String> = agent
        .tags
        .iter()
        .map(|t| normalize_tag(t))
        .filter(|t| !t.is_empty())
        .collect();
    classes.extend(infer_marker_keys(
        &agent.text.to_lowercase(),
        &semantic.agent_classes,
    ));
    classes
}

fn rule_applies_to_verb(rule: &VerbAgentRule, verb_tags: &HashSet<String>) -> bool {
    if rule.verb_tags.is_empty() {
        return false;
    }
    rule.verb_tags
        .iter()
        .map(|t| normalize_tag(t))
        .any(|t| !t.is_empty() && verb_tags.contains(&t))
}

fn infer_groups_from_text(text: &str, semantic: &SemanticTables) -> HashSet<String> {
    infer_marker_keys(text, &semantic.stem_group_markers)
}

fn infer_archetype_tags(text: &str, semantic: &SemanticTables) -> HashSet<String> {
    infer_marker_keys(text, &semantic.archetype_markers)
}

fn infer_living_trait_lemma(word: &InflectedWord, semantic: &SemanticTables) -> bool {
    let probe = word_probe(word);
    semantic
        .living_trait_markers
        .iter()
        .map(|m| normalize_tag(m))
        .any(|m| !m.is_empty() && probe.contains(m.as_str()))
}

fn word_probe(word: &InflectedWord) -> String {
    word.ms
        .as_deref()
        .or(word.fs.as_deref())
        .or(word.invariant.as_deref())
        .unwrap_or("")
        .to_lowercase()
}

fn word_forbids_stem_tags(word: &InflectedWord, ctx: &SemanticContext) -> bool {
    word.forbids
        .iter()
        .any(|f| ctx.tags.contains(&normalize_tag(f)))
}

fn infer_motifs(text: &str, semantic: &SemanticTables) -> HashSet<String> {
    infer_marker_keys(text, &semantic.motif_markers)
}

fn infer_marker_keys(text: &str, sets: &[super::locale::SemanticMarkerSet]) -> HashSet<String> {
    let lower = text.to_lowercase();
    let mut keys = HashSet::new();
    for entry in sets {
        let key = normalize_tag(entry.key.as_str());
        if key.is_empty() {
            continue;
        }
        if entry
            .markers
            .iter()
            .map(|m| normalize_tag(m))
            .any(|m| !m.is_empty() && lower.contains(m.as_str()))
        {
            keys.insert(key);
        }
    }
    keys
}

fn keys_conflict(
    a: &HashSet<String>,
    b: &HashSet<String>,
    pairs: &[super::locale::SemanticTagConflict],
) -> bool {
    pairs.iter().any(|pair| {
        let pa = normalize_tag(&pair.a);
        let pb = normalize_tag(&pair.b);
        (a.contains(&pa) && b.contains(&pb)) || (a.contains(&pb) && b.contains(&pa))
    })
}

fn effective_verbal_kind(state: &VerbalState, semantic: &SemanticTables) -> VerbalKind {
    if let Some(ref k) = state.kind {
        return parse_verbal_kind(k);
    }
    if infer_agent_experience_participle(state, semantic) {
        VerbalKind::AgentExperience
    } else if infer_physical_participle(state, semantic) {
        VerbalKind::Physical
    } else {
        VerbalKind::ObjectFate
    }
}

fn parse_verbal_kind(s: &str) -> VerbalKind {
    match normalize_tag(s).as_str() {
        "agent_experience" | "agent" | "living" | "mind" => VerbalKind::AgentExperience,
        "physical" | "harm" | "damage" => VerbalKind::Physical,
        _ => VerbalKind::ObjectFate,
    }
}

fn verbal_kind_allowed_on_stem(kind: VerbalKind, ctx: &SemanticContext) -> bool {
    match kind {
        VerbalKind::AgentExperience => matches!(
            ctx.class,
            StemClass::Person | StemClass::Divine | StemClass::Creature
        ),
        VerbalKind::ObjectFate | VerbalKind::Physical => true,
    }
}

fn verbal_requires_satisfied(state: &VerbalState, ctx: &SemanticContext) -> bool {
    if state.requires.is_empty() {
        return true;
    }
    state.requires.iter().any(|req| {
        let n = normalize_tag(req);
        ctx.tags.contains(&n) || tag_matches_class(&n, ctx.class)
    })
}

/// Blocks participles whose damage mode clashes with the head archetype (e.g. `tear` on mirrors).
fn archetype_allows_verbal(
    state: &VerbalState,
    ctx: &SemanticContext,
    semantic: &SemanticTables,
) -> bool {
    let modes = infer_verb_tags(state, semantic);
    if modes.is_empty() {
        return true;
    }
    for rule in &semantic.archetype_verbal_compat {
        if !ctx.tags.contains(&normalize_tag(&rule.archetype)) {
            continue;
        }
        if rule
            .forbids
            .iter()
            .map(|f| normalize_tag(f))
            .any(|f| !f.is_empty() && modes.contains(&f))
        {
            return false;
        }
    }
    true
}

fn infer_agent_experience_participle(state: &VerbalState, semantic: &SemanticTables) -> bool {
    let probe = verbal_probe(state);
    semantic
        .verbal_agent_experience_markers
        .iter()
        .map(|m| normalize_tag(m))
        .any(|m| !m.is_empty() && probe.contains(m.as_str()))
}

fn infer_physical_participle(state: &VerbalState, semantic: &SemanticTables) -> bool {
    let probe = verbal_probe(state);
    semantic
        .verbal_physical_markers
        .iter()
        .map(|m| normalize_tag(m))
        .any(|m| !m.is_empty() && probe.contains(m.as_str()))
}

fn verbal_probe(state: &VerbalState) -> String {
    state
        .ms
        .as_deref()
        .or(state.fs.as_deref())
        .or(state.mp.as_deref())
        .or(state.fp.as_deref())
        .or(state.invariant.as_deref())
        .or(state.ns.as_deref())
        .or(state.np.as_deref())
        .unwrap_or("")
        .to_lowercase()
}

fn normalize_tag(s: &str) -> String {
    s.trim().to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::super::grammar::{AgentEntry, AgreementKey, Gender, Number, VerbalState};
    use super::super::locale::{AdjectiveStemRule, SemanticMarkerSet, SemanticTagConflict};
    use super::*;

    fn semantic_rules() -> SemanticTables {
        SemanticTables {
            motif_markers: vec![
                SemanticMarkerSet {
                    key: "abyss".to_string(),
                    markers: vec!["abismo".to_string(), "abyss".to_string()],
                },
                SemanticMarkerSet {
                    key: "fire".to_string(),
                    markers: vec!["llama".to_string(), "burn".to_string()],
                },
                SemanticMarkerSet {
                    key: "cold".to_string(),
                    markers: vec!["helad".to_string(), "cold".to_string()],
                },
            ],
            motif_conflicts: vec![
                SemanticTagConflict {
                    a: "fire".to_string(),
                    b: "cold".to_string(),
                },
                SemanticTagConflict {
                    a: "abyss".to_string(),
                    b: "celestial".to_string(),
                },
            ],
            archetype_markers: vec![
                SemanticMarkerSet {
                    key: "edge".to_string(),
                    markers: vec![
                        "aguja".to_string(),
                        "needle".to_string(),
                        "filamento".to_string(),
                        "filament".to_string(),
                    ],
                },
                SemanticMarkerSet {
                    key: "mirror".to_string(),
                    markers: vec!["espejo".to_string(), "mirror".to_string()],
                },
            ],
            stem_group_markers: vec![
                SemanticMarkerSet {
                    key: "damage".to_string(),
                    markers: vec!["roto".to_string(), "rota".to_string(), "broken".to_string()],
                },
                SemanticMarkerSet {
                    key: "emptiness".to_string(),
                    markers: vec!["vacío".to_string(), "empty".to_string()],
                },
            ],
            stem_class_markers: vec![SemanticMarkerSet {
                key: "person".to_string(),
                markers: vec!["rey ".to_string()],
            }],
            tag_conflicts: vec![
                SemanticTagConflict {
                    a: "fire_active".to_string(),
                    b: "fire_done".to_string(),
                },
                SemanticTagConflict {
                    a: "incorporeo".to_string(),
                    b: "errante".to_string(),
                },
            ],
            archetype_compat: vec![
                super::super::locale::ArchetypeCompatRule {
                    archetype: "edge".to_string(),
                    forbids: vec![
                        "hollow_volume".to_string(),
                        "organic".to_string(),
                        "wither".to_string(),
                    ],
                    allows: vec!["metal".to_string()],
                },
                super::super::locale::ArchetypeCompatRule {
                    archetype: "mirror".to_string(),
                    forbids: vec!["organic".to_string(), "wither".to_string()],
                    allows: Vec::new(),
                },
            ],
            archetype_verbal_compat: vec![
                super::super::locale::ArchetypeCompatRule {
                    archetype: "mirror".to_string(),
                    forbids: vec!["tear".to_string(), "drown".to_string()],
                    allows: Vec::new(),
                },
                super::super::locale::ArchetypeCompatRule {
                    archetype: "edge".to_string(),
                    forbids: vec!["tear".to_string(), "drown".to_string()],
                    allows: Vec::new(),
                },
                super::super::locale::ArchetypeCompatRule {
                    archetype: "object".to_string(),
                    forbids: vec!["drown".to_string()],
                    allows: Vec::new(),
                },
            ],
            verb_tag_markers: vec![
                SemanticMarkerSet {
                    key: "tear".to_string(),
                    markers: vec!["desgarr".to_string()],
                },
                SemanticMarkerSet {
                    key: "seal".to_string(),
                    markers: vec!["sellad".to_string()],
                },
                SemanticMarkerSet {
                    key: "drown".to_string(),
                    markers: vec!["ahogad".to_string()],
                },
                SemanticMarkerSet {
                    key: "devour".to_string(),
                    markers: vec!["devorad".to_string()],
                },
            ],
            agent_required_verb_tags: vec!["touch".to_string()],
            agent_required_participle_markers: vec!["tocad".to_string()],
            banned_indefinite_markers: vec![
                "algo".to_string(),
                "alguien".to_string(),
                "quien sabe".to_string(),
                "something".to_string(),
                "someone".to_string(),
            ],
            verbal_agent_experience_markers: vec![
                "traicion".to_string(),
                "betray".to_string(),
                "предан".to_string(),
            ],
            verbal_physical_markers: vec![
                "quemad".to_string(),
                "burn".to_string(),
                "enterr".to_string(),
                "desgarr".to_string(),
            ],
            verbal_agent_linkers: vec![" por ".to_string(), " by ".to_string()],
            adjective_stem_rules: vec![AdjectiveStemRule {
                stem_tags: vec!["object".to_string(), "relic".to_string()],
                forbid_kinds: vec!["living_trait".to_string()],
                ..AdjectiveStemRule::default()
            }],
            ..SemanticTables::default()
        }
    }

    fn stem(text: &str, semantic: Option<&str>, groups: &[&str]) -> StemEntry {
        StemEntry {
            text: text.to_string(),
            g: "m".to_string(),
            n: "s".to_string(),
            tags: Vec::new(),
            semantic: semantic.map(str::to_string),
            groups: groups.iter().map(|s| (*s).to_string()).collect(),
            unique: false,
            family: "test".to_string(),
        }
    }

    #[test]
    fn object_stem_rejects_character_title() {
        let rules = semantic_rules();
        let ctx = SemanticContext::from_stem(&stem("Trono vacío", None, &[]), &rules);
        assert_eq!(ctx.class, StemClass::Object);
        assert!(!title_allowed(&ctx));
        assert!(ctx.groups.contains("emptiness"));
    }

    #[test]
    fn damaged_stem_blocks_damage_group_modifier() {
        let rules = semantic_rules();
        let ctx = SemanticContext::from_stem(&stem("Campana rota", None, &[]), &rules);
        assert!(ctx.groups.contains("damage"));
        let fracturado = InflectedWord {
            ms: Some("fracturada".to_string()),
            group: Some("damage".to_string()),
            ..InflectedWord::default()
        };
        assert!(!allows_word(
            &fracturado,
            &ctx,
            PickSlot::Modifier,
            Some(&ModifierState::default()),
            &rules,
        ));
    }

    #[test]
    fn living_trait_blocked_on_object() {
        let rules = semantic_rules();
        let ctx = SemanticContext::from_stem(&stem("Bastión", None, &[]), &rules);
        let loco = InflectedWord {
            ms: Some("loco".to_string()),
            kind: Some("living_trait".to_string()),
            ..InflectedWord::default()
        };
        assert!(!allows_word(&loco, &ctx, PickSlot::Modifier, None, &rules));
    }

    #[test]
    fn adjective_stem_rule_blocks_trait_on_relic_tag() {
        let rules = semantic_rules();
        let mut entry = stem("Реликвия", None, &[]);
        entry.tags = vec!["relic".to_string(), "object".to_string()];
        let ctx = SemanticContext::from_stem(&entry, &rules);
        let greedy = InflectedWord {
            ms: Some("алчный".to_string()),
            kind: Some("living_trait".to_string()),
            ..InflectedWord::default()
        };
        assert!(!allows_word(
            &greedy,
            &ctx,
            PickSlot::Modifier,
            None,
            &rules
        ));
        let keen = InflectedWord {
            invariant: Some("sharp".to_string()),
            tags: vec!["sharp".to_string()],
            ..InflectedWord::default()
        };
        assert!(allows_word(&keen, &ctx, PickSlot::Modifier, None, &rules));
    }

    #[test]
    fn person_stem_allows_title() {
        let rules = semantic_rules();
        let ctx = SemanticContext::from_stem(&stem("Rey sin rostro", Some("person"), &[]), &rules);
        assert!(title_allowed(&ctx));
    }

    #[test]
    fn traicionado_blocked_on_object() {
        let rules = semantic_rules();
        let ctx = SemanticContext::from_stem(&stem("Espejo roto", None, &["damage"]), &rules);
        let v = VerbalState {
            ms: Some("traicionado".to_string()),
            kind: Some("agent_experience".to_string()),
            ..VerbalState::default()
        };
        assert!(!allows_verbal_state(&v, &ctx, &rules));
    }

    #[test]
    fn tear_verbal_blocked_on_mirror_stem() {
        let rules = semantic_rules();
        let ctx = SemanticContext::from_stem(&stem("Espejos rotos", None, &["damage"]), &rules);
        assert!(ctx.tags.contains("mirror"));
        let desgarrado = VerbalState {
            ms: Some("desgarrados".to_string()),
            verb_tags: vec!["tear".to_string()],
            ..VerbalState::default()
        };
        assert!(!allows_verbal_state(&desgarrado, &ctx, &rules));
        let sellado = VerbalState {
            ms: Some("sellados".to_string()),
            verb_tags: vec!["seal".to_string()],
            ..VerbalState::default()
        };
        assert!(allows_verbal_state(&sellado, &ctx, &rules));
    }

    #[test]
    fn flay_verbal_blocked_on_mirror_allowed_on_spirit() {
        let rules = semantic_rules();
        let mirror = SemanticContext::from_stem(&stem("Espejos rotos", None, &["damage"]), &rules);
        let spirit = SemanticContext::from_stem(
            &StemEntry {
                text: "Espíritus".to_string(),
                g: "m".to_string(),
                n: "p".to_string(),
                tags: vec!["creature".to_string(), "person".to_string()],
                semantic: None,
                groups: Vec::new(),
                unique: false,
                family: "spirit".to_string(),
            },
            &rules,
        );
        assert!(mirror.tags.contains("mirror"));
        assert!(spirit.tags.contains("creature"));
        assert_eq!(spirit.class, StemClass::Creature);
        let desollado = VerbalState {
            mp: Some("desollados".to_string()),
            verb_tags: vec!["flay".to_string()],
            requires: vec![
                "corpse".to_string(),
                "person".to_string(),
                "creature".to_string(),
            ],
            kind: Some("agent_experience".to_string()),
            ..VerbalState::default()
        };
        assert!(!allows_verbal_state(&desollado, &mirror, &rules));
        assert!(allows_verbal_state(&desollado, &spirit, &rules));
    }

    #[test]
    fn tear_verbal_allowed_on_corpse_stem() {
        let rules = semantic_rules();
        let ctx = SemanticContext::from_stem(&stem("Cadáver", None, &[]), &rules);
        let desgarrado = VerbalState {
            ms: Some("desgarrado".to_string()),
            verb_tags: vec!["tear".to_string()],
            ..VerbalState::default()
        };
        assert!(allows_verbal_state(&desgarrado, &ctx, &rules));
    }

    #[test]
    fn sellado_allowed_on_object_post_nominal() {
        let rules = semantic_rules();
        let ctx = SemanticContext::from_stem(&stem("Espejo roto", None, &[]), &rules);
        let v = VerbalState {
            ms: Some("sellado".to_string()),
            ..VerbalState::default()
        };
        assert!(allows_verbal_state(&v, &ctx, &rules));
        assert_eq!(verbal_placement(&ctx), VerbalPlacement::PostNominal);
    }

    #[test]
    fn plural_unique_stem_rejects_patron() {
        let throne = StemEntry {
            text: "Tronos vacíos".to_string(),
            g: "m".to_string(),
            n: "p".to_string(),
            tags: vec!["object".to_string()],
            semantic: None,
            groups: vec!["emptiness".to_string()],
            unique: true,
            family: "empty_throne".to_string(),
        };
        assert!(!stem_allows_patron(&throne));
        let single = StemEntry {
            n: "s".to_string(),
            text: "Trono vacío".to_string(),
            unique: true,
            ..throne.clone()
        };
        assert!(stem_allows_patron(&single));
    }

    #[test]
    fn afilado_rejected_on_vessel_stem() {
        let rules = semantic_rules();
        let ctx = SemanticContext::from_stem(
            &StemEntry {
                text: "Cálices".to_string(),
                g: "m".to_string(),
                n: "p".to_string(),
                tags: vec!["object".to_string(), "vessel".to_string()],
                semantic: None,
                groups: Vec::new(),
                unique: true,
                family: "chalice".to_string(),
            },
            &rules,
        );
        let afilado = InflectedWord {
            ms: Some("afilados".to_string()),
            requires: vec!["weapon".to_string(), "edge".to_string()],
            ..InflectedWord::default()
        };
        assert!(!allows_word(
            &afilado,
            &ctx,
            PickSlot::Modifier,
            None,
            &rules,
        ));
        let quemado = InflectedWord {
            ms: Some("quemados".to_string()),
            ..InflectedWord::default()
        };
        assert!(allows_word(
            &quemado,
            &ctx,
            PickSlot::Modifier,
            None,
            &rules
        ));
    }

    #[test]
    fn despiadado_rejected_on_filament_stem() {
        let rules = semantic_rules();
        let ctx = SemanticContext::from_stem(&stem("Filamentos", None, &[]), &rules);
        assert!(ctx.tags.contains("edge"));
        let despiadado = InflectedWord {
            mp: Some("despiadados".to_string()),
            kind: Some("living_trait".to_string()),
            ..InflectedWord::default()
        };
        assert!(!allows_word(
            &despiadado,
            &ctx,
            PickSlot::Modifier,
            None,
            &rules,
        ));
    }

    #[test]
    fn marchito_rejected_on_mirror_stem() {
        let rules = semantic_rules();
        let ctx = SemanticContext::from_stem(&stem("Espejos rotos", None, &["damage"]), &rules);
        assert!(ctx.tags.contains("mirror"));
        let marchito = InflectedWord {
            mp: Some("marchitos".to_string()),
            tags: vec!["organic".to_string(), "wither".to_string()],
            ..InflectedWord::default()
        };
        assert!(!allows_word(
            &marchito,
            &ctx,
            PickSlot::Modifier,
            None,
            &rules,
        ));
    }

    #[test]
    fn drown_and_devour_blocked_on_orb_stem() {
        let rules = semantic_rules();
        let ctx = SemanticContext::from_stem(&stem("Orbe", None, &[]), &rules);
        assert!(ctx.tags.contains("object"));
        let ahogado = VerbalState {
            ms: Some("ahogado".to_string()),
            verb_tags: vec!["drown".to_string()],
            ..VerbalState::default()
        };
        assert!(!allows_verbal_state(&ahogado, &ctx, &rules));
        let devorado = VerbalState {
            ms: Some("devorado".to_string()),
            verb_tags: vec!["devour".to_string()],
            requires: vec![
                "corpse".to_string(),
                "person".to_string(),
                "creature".to_string(),
            ],
            ..VerbalState::default()
        };
        assert!(!allows_verbal_state(&devorado, &ctx, &rules));
    }

    #[test]
    fn devour_allowed_on_corpse_stem() {
        let rules = semantic_rules();
        let ctx = SemanticContext::from_stem(
            &StemEntry {
                text: "Cadáver".to_string(),
                g: "m".to_string(),
                n: "s".to_string(),
                tags: vec!["corpse".to_string()],
                semantic: None,
                groups: Vec::new(),
                unique: false,
                family: "corpse".to_string(),
            },
            &rules,
        );
        let devorado = VerbalState {
            ms: Some("devorado".to_string()),
            verb_tags: vec!["devour".to_string()],
            requires: vec!["corpse".to_string(), "person".to_string()],
            ..VerbalState::default()
        };
        assert!(allows_verbal_state(&devorado, &ctx, &rules));
    }

    #[test]
    fn drown_verbal_blocked_on_filament_stem() {
        let rules = semantic_rules();
        let ctx = SemanticContext::from_stem(&stem("Filamentos", None, &[]), &rules);
        assert!(ctx.tags.contains("edge"));
        let ahogado = VerbalState {
            mp: Some("ahogados".to_string()),
            verb_tags: vec!["drown".to_string()],
            ..VerbalState::default()
        };
        assert!(!allows_verbal_state(&ahogado, &ctx, &rules));
        let profanado = VerbalState {
            mp: Some("profanados".to_string()),
            ..VerbalState::default()
        };
        assert!(allows_verbal_state(&profanado, &ctx, &rules));
    }

    #[test]
    fn tenaz_rejected_on_icon() {
        let rules = semantic_rules();
        let ctx = SemanticContext::from_stem(
            &StemEntry {
                text: "Icono".to_string(),
                g: "m".to_string(),
                n: "s".to_string(),
                tags: vec!["object".to_string(), "icon".to_string()],
                semantic: None,
                groups: Vec::new(),
                unique: false,
                family: "icon".to_string(),
            },
            &rules,
        );
        let tenaz = InflectedWord {
            ms: Some("tenaz".to_string()),
            kind: Some("living_trait".to_string()),
            ..InflectedWord::default()
        };
        assert!(!allows_word(&tenaz, &ctx, PickSlot::Modifier, None, &rules));
    }

    #[test]
    fn qualifier_conflicts_when_agent_shares_naming_theme() {
        let mut rules = semantic_rules();
        rules.themes.push(SemanticMarkerSet {
            key: "naming".to_string(),
            markers: vec!["nombr".to_string(), "nombre".to_string()],
        });
        assert!(qualifier_conflicts_with_verbal_agent(
            super::super::common::Language::Spanish,
            "de nombres olvidados",
            Some("quemado por el primer nombre"),
            &rules,
        ));
        assert!(!qualifier_conflicts_with_verbal_agent(
            super::super::common::Language::Spanish,
            "del abismo",
            Some("quemado por las Moiras"),
            &rules,
        ));
    }

    #[test]
    fn clock_cannot_seal_or_tear() {
        let mut rules = semantic_rules();
        rules.agent_classes = vec![
            SemanticMarkerSet {
                key: "mundane".to_string(),
                markers: vec!["reloj".to_string(), "clock".to_string()],
            },
            SemanticMarkerSet {
                key: "time".to_string(),
                markers: vec!["tiempo".to_string(), "destino".to_string()],
            },
        ];
        rules.verb_agent_rules = vec![super::super::locale::VerbAgentRule {
            verb_tags: vec!["seal".to_string(), "tear".to_string()],
            forbid_agent_classes: vec!["mundane".to_string()],
            allow_agent_classes: Vec::new(),
            hard: true,
        }];
        let forge_seal = VerbalForgeContext {
            verb_tags: HashSet::from([normalize_tag("seal")]),
            ..VerbalForgeContext::default()
        };
        let forge_tear = VerbalForgeContext {
            verb_tags: HashSet::from([normalize_tag("tear")]),
            ..VerbalForgeContext::default()
        };
        let reloj = AgentEntry {
            text: "el reloj".to_string(),
            linker: "por".to_string(),
            tags: vec!["mundane".to_string()],
            ..AgentEntry::default()
        };
        let tiempo = AgentEntry {
            text: "el tiempo".to_string(),
            linker: "por".to_string(),
            tags: vec!["time".to_string()],
            ..AgentEntry::default()
        };
        assert!(!allows_verbal_agent(&reloj, &forge_seal, &rules));
        assert!(!allows_verbal_agent(&reloj, &forge_tear, &rules));
        assert!(allows_verbal_agent(&tiempo, &forge_seal, &rules));
    }

    #[test]
    fn vague_agent_primer_nombre_rejected() {
        let mut rules = semantic_rules();
        rules.banned_agent_markers = vec!["primer nombre".to_string()];
        let agent = AgentEntry {
            text: "el primer nombre".to_string(),
            linker: "por".to_string(),
            ..AgentEntry::default()
        };
        assert!(!allows_verbal_agent(
            &agent,
            &VerbalForgeContext::default(),
            &rules,
        ));
    }

    #[test]
    fn verbal_phrase_requires_explicit_agent() {
        let mut rules = semantic_rules();
        rules.verbal_physical_markers = vec!["desgarr".to_string()];
        rules.verbal_agent_linkers = vec![" por ".to_string()];
        assert!(!verbal_phrase_grammatical(
            super::super::common::Language::Spanish,
            "desgarrados",
            &rules,
        ));
        assert!(verbal_phrase_grammatical(
            super::super::common::Language::Spanish,
            "desgarrados por Modi",
            &rules,
        ));
        assert!(!verbal_phrase_grammatical(
            super::super::common::Language::Spanish,
            "desgarrados de Modi",
            &rules,
        ));
    }

    #[test]
    fn russian_verbal_rejects_vague_and_genitive_only_agent() {
        let mut rules = semantic_rules();
        rules.verbal_physical_markers = vec!["закован".to_string()];
        rules.banned_verbal_surface_markers = vec!["чем-то".to_string()];
        rules.russian_instrumental_agent_suffixes = vec!["ом".to_string(), "ой".to_string()];
        assert!(!verbal_phrase_grammatical(
            super::super::common::Language::Russian,
            "закованные чем-то",
            &rules,
        ));
        assert!(!verbal_phrase_grammatical(
            super::super::common::Language::Russian,
            "закованные Кракена",
            &rules,
        ));
        assert!(verbal_phrase_grammatical(
            super::super::common::Language::Russian,
            "закованные Кракеном",
            &rules,
        ));
    }

    #[test]
    fn patron_blocked_when_verbal_has_agent() {
        use super::super::common::Language;
        let rules = semantic_rules();
        assert!(patron_blocked_by_verbal_attribution(
            Language::Spanish,
            None,
            false,
            Some("enterrada por quien sabe quién"),
            &rules,
        ));
        assert!(patron_blocked_by_verbal_attribution(
            Language::Spanish,
            None,
            false,
            Some("tocada"),
            &rules,
        ));
        assert!(patron_blocked_by_verbal_attribution(
            Language::Spanish,
            None,
            false,
            Some("desgarrados"),
            &rules,
        ));
        assert!(!patron_blocked_by_verbal_attribution(
            Language::Spanish,
            None,
            false,
            Some("sellado"),
            &rules,
        ));
    }

    #[test]
    fn patron_blocked_with_genitive_qualifier() {
        let mut es = semantic_rules();
        es.possessive_origin_qualifier_prefixes =
            vec!["del ".into(), "de la ".into(), "de ".into()];
        assert!(patron_blocked_by_qualifier(Some("del reinado caído"), &es,));
        let mut en = semantic_rules();
        en.possessive_origin_qualifier_prefixes = vec!["of ".into()];
        assert!(patron_blocked_by_qualifier(
            Some("of the Fallen Throne"),
            &en,
        ));
        let mut ru = semantic_rules();
        ru.possessive_origin_qualifier_markers = vec!["павшего".into()];
        assert!(patron_blocked_by_qualifier(Some("павшего трона"), &ru,));
        assert!(!patron_blocked_by_qualifier(None, &es));
    }

    #[test]
    fn physical_verbal_requires_named_agent() {
        let rules = semantic_rules();
        let ctx = SemanticContext::from_stem(&stem("Orbe fracturado", None, &[]), &rules);
        let desgarrado = VerbalState {
            mp: Some("desgarrados".to_string()),
            verb_tags: vec!["tear".to_string()],
            ..VerbalState::default()
        };
        assert!(verbal_state_requires_agent(&desgarrado, &rules));
        let phrase = super::super::verbal::forge_verbal_phrase(
            super::super::common::Language::Spanish,
            AgreementKey {
                gender: Gender::M,
                number: Number::Plural,
            },
            42,
            7,
            &[desgarrado],
            &[],
            &[super::super::grammar::ProperName::Plain("Modi".to_string())],
            "por",
            &ctx,
            &rules,
            &VerbalForgeContext::default(),
            &super::super::verbal::VerbalAttachPolicy::default(),
            |_, _| 999,
        );
        assert_eq!(phrase.as_deref(), Some("desgarrados por Modi"));
    }

    #[test]
    fn touch_verbal_requires_agent_in_forge() {
        let mut rules = semantic_rules();
        rules.agent_required_verb_tags = vec!["touch".to_string()];
        let ctx = SemanticContext::from_stem(&stem("Llave de la cripta", None, &[]), &rules);
        let tocado = VerbalState {
            fs: Some("tocada".to_string()),
            verb_tags: vec!["touch".to_string()],
            requires_agent: true,
            ..VerbalState::default()
        };
        assert!(verbal_state_requires_agent(&tocado, &rules));
        let phrase = super::super::verbal::forge_verbal_phrase(
            super::super::common::Language::Spanish,
            AgreementKey {
                gender: Gender::F,
                number: Number::Singular,
            },
            42,
            7,
            &[tocado],
            &[],
            &[super::super::grammar::ProperName::Plain(
                "Anubis".to_string(),
            )],
            "por",
            &ctx,
            &rules,
            &VerbalForgeContext::default(),
            &super::super::verbal::VerbalAttachPolicy::default(),
            |_, _| 999,
        );
        assert_eq!(phrase.as_deref(), Some("tocada por Anubis"));
    }

    #[test]
    fn motif_overlap_gets_higher_weight() {
        let rules = semantic_rules();
        let ctx = SemanticContext::from_stem(&stem("Campana del abismo", None, &[]), &rules);
        let abyss = text_coherence_weight(&ctx, "del abismo", &rules);
        let fire = text_coherence_weight(&ctx, "de la llama roja", &rules);
        assert!(abyss > fire);
    }

    #[test]
    fn abstract_agent_and_algo_rejected() {
        let rules = semantic_rules();
        let forge = VerbalForgeContext {
            has_patron: true,
            ..VerbalForgeContext::default()
        };
        let juicio = AgentEntry {
            text: "el juicio".to_string(),
            linker: "por".to_string(),
            kind: Some("abstract".to_string()),
            ..AgentEntry::default()
        };
        assert!(!allows_verbal_agent(&juicio, &forge, &rules));
        let algo = AgentEntry {
            text: "algo".to_string(),
            indefinite: true,
            linker: "por".to_string(),
            ..AgentEntry::default()
        };
        assert!(!allows_verbal_agent(
            &algo,
            &VerbalForgeContext::default(),
            &rules
        ));
        let quien = AgentEntry {
            text: "quien sabe quién".to_string(),
            indefinite: true,
            linker: "por".to_string(),
            ..AgentEntry::default()
        };
        assert!(!allows_verbal_agent(&quien, &forge, &rules));
    }

    #[test]
    fn surface_redundant_when_modifier_repeats_stem_lead() {
        assert!(surface_redundant_with_stem("Проклятый кандал", "проклятый"));
        assert!(!surface_redundant_with_stem("Дикий осколок", "алчный"));
    }

    #[test]
    fn qualifier_redundant_when_stem_already_contains_phrase() {
        assert!(qualifier_redundant_with_stem(
            "Fragmentos del vacío",
            "del vacío"
        ));
        assert!(qualifier_redundant_with_stem(
            "Llave del abismo",
            "del abismo"
        ));
        assert!(!qualifier_redundant_with_stem("Garra", "del vacío"));
        assert!(!qualifier_redundant_with_stem("Trono vacío", "del vacío"));
    }

    #[test]
    fn reserved_surface_blocks_duplicate_modifier_pick() {
        let mut state = ModifierState::default();
        let estrellada = InflectedWord {
            fs: Some("estrellada".to_string()),
            ..InflectedWord::default()
        };
        register_modifier(&mut state, &estrellada, Some("estrellada"));
        assert!(surface_already_used("estrellada", &state));
        assert!(surface_already_used("Estrellada", &state));
    }

    #[test]
    fn fire_active_and_done_conflict() {
        let rules = semantic_rules();
        let ctx = SemanticContext::from_stem(&stem("Filamento", None, &[]), &rules);
        let ardiente = InflectedWord {
            ms: Some("ardiente".to_string()),
            tags: vec!["fire_active".to_string()],
            ..InflectedWord::default()
        };
        let mut state = ModifierState::default();
        register_modifier(&mut state, &ardiente, None);
        let quemado = InflectedWord {
            ms: Some("quemado".to_string()),
            tags: vec!["fire_done".to_string()],
            ..InflectedWord::default()
        };
        assert!(!allows_word(
            &quemado,
            &ctx,
            PickSlot::Modifier,
            Some(&state),
            &rules,
        ));
    }

    #[test]
    fn archetype_allows_whitelist() {
        let rules = semantic_rules();
        let ctx = SemanticContext::from_stem(
            &StemEntry {
                text: "Agujas".to_string(),
                g: "f".to_string(),
                n: "p".to_string(),
                tags: vec!["edge".to_string()],
                semantic: None,
                groups: Vec::new(),
                unique: false,
                family: "needle".to_string(),
            },
            &rules,
        );
        let metal = InflectedWord {
            ms: Some("oxidada".to_string()),
            tags: vec!["metal".to_string()],
            ..InflectedWord::default()
        };
        assert!(allows_word(&metal, &ctx, PickSlot::Modifier, None, &rules));
        let fire_done = InflectedWord {
            ms: Some("quemada".to_string()),
            tags: vec!["fire_done".to_string()],
            ..InflectedWord::default()
        };
        assert!(!allows_word(
            &fire_done,
            &ctx,
            PickSlot::Modifier,
            None,
            &rules,
        ));
    }

    #[test]
    fn hollow_blocked_on_edge_stem() {
        let rules = semantic_rules();
        let ctx = SemanticContext::from_stem(
            &StemEntry {
                text: "Agujas".to_string(),
                g: "f".to_string(),
                n: "p".to_string(),
                tags: vec!["edge".to_string()],
                semantic: None,
                groups: Vec::new(),
                unique: false,
                family: "needle".to_string(),
            },
            &rules,
        );
        let hueco = InflectedWord {
            ms: Some("huecas".to_string()),
            tags: vec!["hollow_volume".to_string()],
            ..InflectedWord::default()
        };
        assert!(!allows_word(&hueco, &ctx, PickSlot::Modifier, None, &rules));
    }

    #[test]
    fn traicionado_comma_prologue_on_person() {
        let rules = semantic_rules();
        let ctx = SemanticContext::from_stem(&stem("Rey caído", Some("person"), &[]), &rules);
        let v = VerbalState {
            ms: Some("traicionado".to_string()),
            kind: Some("agent_experience".to_string()),
            ..VerbalState::default()
        };
        assert!(allows_verbal_state(&v, &ctx, &rules));
        assert_eq!(verbal_placement(&ctx), VerbalPlacement::CommaPrologue);
    }

    #[test]
    fn incorporeal_and_wandering_conflict() {
        let incorp = InflectedWord {
            ms: Some("incorpóreo".to_string()),
            tags: vec!["incorporeo".to_string()],
            ..InflectedWord::default()
        };
        let mut state = ModifierState::default();
        register_modifier(&mut state, &incorp, None);
        let errante = InflectedWord {
            ms: Some("errante".to_string()),
            tags: vec!["errante".to_string()],
            ..InflectedWord::default()
        };
        let rules = semantic_rules();
        assert!(!allows_word(
            &errante,
            &SemanticContext::from_stem(&stem("Orbe", None, &[]), &rules),
            PickSlot::Modifier,
            Some(&state),
            &rules,
        ));
    }
}
