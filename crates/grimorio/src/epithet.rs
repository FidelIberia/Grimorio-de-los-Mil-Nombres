//! Generation epithet forge — inflected Diablo-style names with optional patronymics.

use super::common::{fnv1a64, mix_u64, quantize01, Language};
use super::grammar::{AgreementKey, InflectedWord, QualifierEntry, StemEntry};
use super::locale;
use super::portrait::StandoutPortrait;
use super::semantic::{
    self, ModifierState, PickSlot, SemanticContext, VerbalForgeContext, VerbalPlacement,
};
use super::surface::{
    adjectives_precede_noun, capitalize_first, format_glued_prologue, format_name_phrase_caps,
    title_for_name_phrase,
};
use super::verbal::{forge_verbal_phrase, VerbalAttachPolicy};

const SLOT_STEM: u32 = 0;
const SLOT_QUALIFIER: u32 = 1;
const SLOT_PATRON: u32 = 2;
const SLOT_TITLE: u32 = 3;
const SLOT_TRAIT_ADJ: u32 = 4;
const SLOT_CURSE: u32 = 5;
const SLOT_ORDER: u32 = 6;
const SLOT_PROLOGUE: u32 = 7;

/// Built slots before language-specific assembly.
struct Assembled<'a> {
    stem: &'a str,
    qualifier: Option<&'a str>,
    patron: Option<&'a str>,
    title: Option<String>,
    /// Comma or glued opening (not used when [`post_nominal_verbal`] is set).
    prologue: Option<String>,
    prologue_is_verbal: bool,
    /// Participle phrase after the head (`Espejo roto sellado por la espiral`).
    post_nominal_verbal: Option<String>,
    /// Adjectives glued to the head noun (`rotas silenciosas`).
    epilogue_modifiers: Vec<String>,
    /// Static curse after a comma (`, condenada`).
    epilogue_curse: Option<String>,
    head_key: AgreementKey,
}

const SAMPLE_LABELS: &[&str] = &[
    "ResonanceEngine",
    "Living Memory",
    "Mercy Field",
    "Living Cartography",
    "active_lattice_0",
    "genesis",
    "CartographyArchive",
    "wanderer",
    "keeper",
    "witness",
];

/// Synthetic entity with varied trait axes for epithet demos (`--epithets`).
#[must_use]
pub fn sample_entity(index: u32, base_seed: u64) -> StandoutPortrait {
    let h = mix_u64(base_seed, index as u64);
    let axis = |salt: u64| -> f32 { ((h.wrapping_add(salt) % 1000) as f32) / 999.0 };
    let label = SAMPLE_LABELS[(index as usize) % SAMPLE_LABELS.len()].to_string();
    let generation = (h % 17) as u32;
    let fitness = 4.0 + (h % 4000) as f32 / 40.0;
    let viability = axis(3);
    let vitality = Some(axis(5));
    let resonance = Some(axis(7));
    let mutation_pressure = Some(axis(11));
    let symbolic_density = Some(axis(13));
    let memory_depth = Some(axis(17));
    let shadow_pull = Some(axis(19));
    StandoutPortrait {
        label,
        generation,
        fitness,
        viability,
        vitality,
        resonance,
        mutation_pressure,
        symbolic_density,
        memory_depth,
        shadow_pull,
        myth: None,
    }
}

/// Forges one epithet from a synthetic sample (deterministic per `index` + `base_seed`).
#[must_use]
pub fn forge_sample(language: Language, index: u32, base_seed: u64) -> String {
    let entity = sample_entity(index, base_seed);
    forge(&entity, entity.generation, language)
}

/// Builds a deterministic honorific for the standout of a generation.
#[must_use]
pub fn forge(entity: &StandoutPortrait, generation: u32, language: Language) -> String {
    let tables = &locale::tables(language).epithet;
    if tables.stems.is_empty() {
        return entity.label.clone();
    }

    let shadow = entity.shadow_pull.unwrap_or(0.0);
    let mutation = entity.mutation_pressure.unwrap_or(0.0);
    let memory = entity.memory_depth.unwrap_or(0.0);
    let resonance = entity.resonance.unwrap_or(0.0);
    let symbolic = entity.symbolic_density.unwrap_or(0.0);
    let viability = entity.viability;
    let fitness = entity.fitness;

    let mut seed = fnv1a64(entity.label.as_bytes());
    seed = mix_u64(seed, generation as u64);
    seed = mix_u64(seed, quantize01(shadow) as u64);
    seed = mix_u64(seed, quantize01(mutation) as u64);
    seed = mix_u64(seed, quantize01(memory) as u64);
    seed = mix_u64(seed, quantize01(resonance) as u64);
    seed = mix_u64(seed, quantize01(symbolic) as u64);
    seed = mix_u64(seed, (viability * 1000.0) as u64);
    seed = mix_u64(seed, (fitness * 10.0) as u64);

    let stem_entry = pick_stem(seed, tables, shadow, mutation, memory, fitness);
    let key = stem_entry.key();
    let stem = stem_entry.text.as_str();
    let semantics = SemanticContext::from_stem(stem_entry, &tables.semantic);

    let verbal_forge = VerbalForgeContext::default();
    let title = if semantic::title_allowed(&semantics) {
        pick_inflected(
            seed,
            SLOT_TITLE,
            &tables.titles,
            key,
            &semantics,
            &tables.semantic,
            PickSlot::Title,
            None,
            shadow,
            mutation,
            0.52,
            0.36,
            220,
        )
    } else {
        None
    };
    let (prologue_raw, prologue_is_verbal, post_nominal_verbal, prologue_word) = pick_prologue(
        seed,
        tables,
        key,
        stem,
        &semantics,
        &tables.semantic,
        &verbal_forge,
        language,
        shadow,
        mutation,
        viability,
        resonance,
        symbolic,
    );
    let reserved_prologue = match (&prologue_raw, prologue_is_verbal) {
        (Some(adj), false) if !adjectives_precede_noun(language) => {
            prologue_word.as_ref().zip(Some(adj.as_str()))
        }
        _ => None,
    };
    let (mut epilogue_modifiers, mut epilogue_curse) = pick_epilogue(
        seed,
        tables,
        key,
        stem,
        &semantics,
        &tables.semantic,
        shadow,
        mutation,
        viability,
        resonance,
        symbolic,
        reserved_prologue,
    );
    let prologue = match (prologue_raw, prologue_is_verbal) {
        (Some(adj), false) if !adjectives_precede_noun(language) => {
            if !epilogue_modifiers.iter().any(|m| m == &adj) {
                epilogue_modifiers.insert(0, adj);
            }
            None
        }
        (other, _) => other,
    };

    let qualifier =
        pick_qualifier(seed, tables, key, stem, &semantics, memory, resonance).filter(|q| {
            !semantic::qualifier_redundant_with_stem(stem, q)
                && !semantic::qualifier_conflicts_with_verbal_agent(
                    language,
                    q,
                    post_nominal_verbal.as_deref(),
                    &tables.semantic,
                )
        });

    let mut patron = pick_patron(seed, tables, language, memory, resonance, symbolic)
        .filter(|_| semantic::stem_allows_patron(stem_entry));

    let mut post_nominal_verbal = post_nominal_verbal;
    refine_assembled_grammar(
        language,
        stem,
        &tables.semantic,
        qualifier,
        &mut patron,
        prologue.as_deref(),
        prologue_is_verbal,
        &mut post_nominal_verbal,
        &mut epilogue_modifiers,
        &mut epilogue_curse,
    );

    if semantic::output_needs_descriptor(
        qualifier,
        patron,
        title.as_deref(),
        prologue.as_deref(),
        post_nominal_verbal.as_deref(),
        &epilogue_modifiers,
        epilogue_curse.as_deref(),
    ) {
        force_epilogue_modifier(
            seed,
            tables,
            key,
            stem,
            &semantics,
            &tables.semantic,
            &mut epilogue_modifiers,
        );
    }

    let assembled = Assembled {
        stem,
        qualifier,
        patron,
        title,
        prologue,
        prologue_is_verbal,
        post_nominal_verbal,
        epilogue_modifiers,
        epilogue_curse,
        head_key: key,
    };

    let _order = (roll_permille(seed, SLOT_ORDER) as usize) % 6;
    assemble(language, &assembled)
}

/// Drops slots that fail grammar rules (second pass after rolling).
fn refine_assembled_grammar(
    language: Language,
    stem: &str,
    semantic: &super::locale::SemanticTables,
    qualifier: Option<&str>,
    patron: &mut Option<&str>,
    prologue: Option<&str>,
    prologue_is_verbal: bool,
    post_nominal_verbal: &mut Option<String>,
    epilogue_modifiers: &mut Vec<String>,
    epilogue_curse: &mut Option<String>,
) {
    epilogue_modifiers.retain(|m| !semantic::surface_redundant_with_stem(stem, m));
    if epilogue_curse
        .as_ref()
        .is_some_and(|c| semantic::surface_redundant_with_stem(stem, c))
    {
        *epilogue_curse = None;
    }
    if let Some(ref verbal) = *post_nominal_verbal {
        if !semantic::verbal_phrase_grammatical(language, verbal, semantic) {
            *post_nominal_verbal = None;
        }
    }
    if patron.is_some()
        && !semantic::patron_grammatically_allowed(
            language,
            qualifier,
            prologue,
            prologue_is_verbal,
            post_nominal_verbal.as_deref(),
            semantic,
        )
    {
        *patron = None;
    }
}

fn pick_stem(
    seed: u64,
    tables: &super::locale::EpithetTables,
    shadow: f32,
    mutation: f32,
    memory: f32,
    fitness: f32,
) -> &StemEntry {
    let families = stem_families(&tables.stems);
    let bias = trait_bias(shadow, mutation, memory, fitness);
    let family_idx = weighted_index(seed, SLOT_STEM, families.len(), bias);
    let variants = &families[family_idx];
    let variant_idx = weighted_index(seed, SLOT_STEM.wrapping_add(1), variants.len(), 0);
    &tables.stems[variants[variant_idx]]
}

/// Groups locale variants that share [`StemEntry::family`] (one concept per roll).
fn stem_families(stems: &[StemEntry]) -> Vec<Vec<usize>> {
    use std::collections::BTreeMap;
    let mut map: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for (i, stem) in stems.iter().enumerate() {
        let key = stem.family.trim().to_ascii_lowercase();
        map.entry(key).or_default().push(i);
    }
    map.into_values().collect()
}

fn pick_qualifier<'a>(
    seed: u64,
    tables: &'a super::locale::EpithetTables,
    key: AgreementKey,
    stem: &str,
    semantics: &SemanticContext,
    memory: f32,
    resonance: f32,
) -> Option<&'a str> {
    let candidates: Vec<&QualifierEntry> = tables
        .qualifiers
        .iter()
        .filter(|q| q.matches(key) && !semantic::qualifier_redundant_with_stem(stem, &q.text))
        .collect();
    if candidates.is_empty() {
        return None;
    }
    let need_memory = memory > 0.48;
    let need_resonance = resonance > 0.52;
    if !need_memory && !need_resonance {
        return None;
    }
    let chance = ((memory * 280.0) + (resonance * 200.0)).clamp(80.0, 480.0) as u32;
    if roll_permille(seed, SLOT_QUALIFIER) >= chance {
        return None;
    }
    let pick_seed = mix_u64(
        seed,
        ((memory * 300.0) as u64).wrapping_add((resonance * 180.0) as u64),
    );
    let weights: Vec<u64> = candidates
        .iter()
        .map(|q| {
            semantic::text_coherence_weight(semantics, q.text.as_str(), &tables.semantic)
                .saturating_mul(semantic::theme_coherence_weight(
                    semantics,
                    q.text.as_str(),
                    &tables.semantic,
                ))
                / 10
        })
        .collect();
    let qi = semantic::weighted_pick(pick_seed, SLOT_QUALIFIER, &weights);
    Some(candidates[qi].text.as_str())
}

fn pick_patron(
    seed: u64,
    tables: &super::locale::EpithetTables,
    language: Language,
    memory: f32,
    resonance: f32,
    symbolic: f32,
) -> Option<&str> {
    let pool: Vec<usize> = tables
        .proper_names
        .iter()
        .enumerate()
        .filter_map(|(i, n)| n.patron_surface(language).map(|_| i))
        .collect();
    if pool.is_empty() {
        return None;
    }
    if memory < 0.36 && resonance < 0.40 && symbolic < 0.32 {
        return None;
    }
    let chance =
        ((memory * 240.0) + (resonance * 200.0) + (symbolic * 180.0)).clamp(80.0, 480.0) as u32;
    if roll_permille(seed, SLOT_PATRON) >= chance {
        return None;
    }
    let pi = weighted_index(seed, SLOT_PATRON, pool.len(), (memory * 200.0) as u64);
    tables.proper_names[pool[pi]].patron_surface(language)
}

/// Opening state: at most one verbal phrase, otherwise an optional lead adjective.
fn pick_prologue(
    seed: u64,
    tables: &super::locale::EpithetTables,
    key: AgreementKey,
    stem: &str,
    semantics: &SemanticContext,
    semantic_rules: &super::locale::SemanticTables,
    verbal_forge: &VerbalForgeContext,
    language: Language,
    shadow: f32,
    mutation: f32,
    viability: f32,
    resonance: f32,
    symbolic: f32,
) -> (Option<String>, bool, Option<String>, Option<InflectedWord>) {
    let verbal_gate = shadow > 0.40 || mutation > 0.38;
    if verbal_gate && !tables.verbal_states.is_empty() {
        let chance = ((shadow * 280.0) + (mutation * 220.0)).clamp(100.0, 550.0) as u32;
        if roll_permille(seed, SLOT_PROLOGUE) < chance {
            let policy = VerbalAttachPolicy {
                solo_permille: 380,
                indefinite_permille: 180,
            };
            if let Some(phrase) = forge_verbal_phrase(
                language,
                key,
                seed,
                SLOT_PROLOGUE,
                &tables.verbal_states,
                &tables.verbal_agents,
                &tables.proper_names,
                tables.named_verbal_linker.as_str(),
                semantics,
                &tables.semantic,
                verbal_forge,
                &policy,
                roll_permille,
            ) {
                return match semantic::verbal_placement(semantics) {
                    VerbalPlacement::PostNominal => (None, false, Some(phrase), None),
                    VerbalPlacement::CommaPrologue => (Some(phrase), true, None, None),
                };
            }
        }
    }

    if roll_permille(seed, SLOT_PROLOGUE.wrapping_add(1)) >= 380 {
        return (None, false, None, None);
    }
    let chance =
        ((viability * 180.0) + (resonance * 140.0) + (symbolic * 120.0)).clamp(100.0, 380.0) as u32;
    if roll_permille(seed, SLOT_PROLOGUE.wrapping_add(2)) >= chance {
        return (None, false, None, None);
    }
    match pick_trait_adjective_entry(
        seed,
        SLOT_PROLOGUE,
        tables,
        key,
        stem,
        semantics,
        semantic_rules,
        PickSlot::PrologueAdj,
        None,
        0,
    ) {
        Some((phrase, word)) => (Some(phrase), false, None, Some(word)),
        None => (None, false, None, None),
    }
}

/// Trailing descriptors: adjectives + static curse; never another verbal state.
fn pick_epilogue(
    seed: u64,
    tables: &super::locale::EpithetTables,
    key: AgreementKey,
    stem: &str,
    semantics: &SemanticContext,
    semantic_rules: &super::locale::SemanticTables,
    shadow: f32,
    mutation: f32,
    viability: f32,
    resonance: f32,
    symbolic: f32,
    reserved_prologue: Option<(&InflectedWord, &str)>,
) -> (Vec<String>, Option<String>) {
    let mut modifiers = Vec::with_capacity(semantic::MAX_EPILOGUE_MODIFIERS);
    let mut mod_state = ModifierState::default();
    if let Some((word, surface)) = reserved_prologue {
        semantic::register_modifier(&mut mod_state, word, Some(surface));
    }
    let base_chance =
        ((viability * 200.0) + (resonance * 160.0) + (symbolic * 140.0)).clamp(120.0, 620.0) as u32;
    for i in 0..semantic::MAX_EPILOGUE_MODIFIERS {
        if mod_state.count >= semantic::MAX_EPILOGUE_MODIFIERS {
            break;
        }
        let slot = SLOT_TRAIT_ADJ.wrapping_add(i as u32);
        let threshold = base_chance.saturating_sub(i as u32 * 110);
        if threshold < 80 || roll_permille(seed, slot) >= threshold {
            break;
        }
        if let Some((phrase, word)) = pick_trait_adjective_entry(
            seed,
            slot,
            tables,
            key,
            stem,
            semantics,
            semantic_rules,
            PickSlot::Modifier,
            Some(&mod_state),
            i,
        ) {
            if modifiers.iter().all(|p| p != &phrase) {
                semantic::register_modifier(&mut mod_state, &word, Some(&phrase));
                modifiers.push(phrase);
            }
        }
    }
    let curse = pick_static_curse(
        seed,
        tables,
        key,
        stem,
        semantics,
        semantic_rules,
        shadow,
        mutation,
    );
    (modifiers, curse)
}

const SLOT_FORCE_MODIFIER: u32 = 40;

fn force_epilogue_modifier(
    seed: u64,
    tables: &super::locale::EpithetTables,
    key: AgreementKey,
    stem: &str,
    semantics: &SemanticContext,
    semantic_rules: &super::locale::SemanticTables,
    modifiers: &mut Vec<String>,
) {
    if let Some((phrase, word)) = pick_trait_adjective_entry(
        seed,
        SLOT_FORCE_MODIFIER,
        tables,
        key,
        stem,
        semantics,
        semantic_rules,
        PickSlot::Modifier,
        None,
        99,
    ) {
        if modifiers.iter().all(|p| p != &phrase) {
            modifiers.push(phrase);
            let _ = word;
        }
    }
}

fn pick_trait_adjective_entry(
    seed: u64,
    slot: u32,
    tables: &super::locale::EpithetTables,
    key: AgreementKey,
    stem: &str,
    semantics: &SemanticContext,
    semantic_rules: &super::locale::SemanticTables,
    pick_slot: PickSlot,
    mod_state: Option<&ModifierState>,
    salt: usize,
) -> Option<(String, InflectedWord)> {
    let (idx, surface) = pick_matching_word_entry(
        seed,
        slot,
        &tables.adjectives,
        key,
        Some(stem),
        semantics,
        semantic_rules,
        pick_slot,
        mod_state,
        salt,
    )?;
    Some((surface, tables.adjectives[idx].clone()))
}

fn pick_static_curse(
    seed: u64,
    tables: &super::locale::EpithetTables,
    key: AgreementKey,
    stem: &str,
    semantics: &SemanticContext,
    semantic_rules: &super::locale::SemanticTables,
    shadow: f32,
    mutation: f32,
) -> Option<String> {
    if tables.curses.is_empty() || (shadow < 0.42 && mutation < 0.38) {
        return None;
    }
    let chance = ((shadow * 320.0) + (mutation * 260.0)).clamp(0.0, 520.0) as u32;
    if chance < 80 || roll_permille(seed, SLOT_CURSE) >= chance {
        return None;
    }
    pick_matching_word(
        seed,
        SLOT_CURSE,
        &tables.curses,
        key,
        Some(stem),
        semantics,
        semantic_rules,
        PickSlot::Curse,
        None,
        0,
    )
}

fn pick_inflected(
    seed: u64,
    slot: u32,
    pool: &[InflectedWord],
    key: AgreementKey,
    semantics: &SemanticContext,
    semantic_rules: &super::locale::SemanticTables,
    pick_slot: PickSlot,
    mod_state: Option<&ModifierState>,
    shadow: f32,
    mutation: f32,
    shadow_floor: f32,
    mutation_floor: f32,
    max_permille: u32,
) -> Option<String> {
    if pool.is_empty() || (shadow < shadow_floor && mutation < mutation_floor) {
        return None;
    }
    let chance = ((shadow * 320.0) + (mutation * 260.0)).clamp(0.0, max_permille as f32) as u32;
    if chance < 80 || roll_permille(seed, slot) >= chance {
        return None;
    }
    pick_matching_word(
        seed,
        slot,
        pool,
        key,
        None,
        semantics,
        semantic_rules,
        pick_slot,
        mod_state,
        0,
    )
}

fn pick_matching_word(
    seed: u64,
    slot: u32,
    pool: &[InflectedWord],
    key: AgreementKey,
    stem: Option<&str>,
    semantics: &SemanticContext,
    semantic_rules: &super::locale::SemanticTables,
    pick_slot: PickSlot,
    mod_state: Option<&ModifierState>,
    salt: usize,
) -> Option<String> {
    pick_matching_word_entry(
        seed,
        slot,
        pool,
        key,
        stem,
        semantics,
        semantic_rules,
        pick_slot,
        mod_state,
        salt,
    )
    .map(|(_, surface)| surface)
}

fn pick_matching_word_entry(
    seed: u64,
    slot: u32,
    pool: &[InflectedWord],
    key: AgreementKey,
    stem: Option<&str>,
    semantics: &SemanticContext,
    semantic_rules: &super::locale::SemanticTables,
    pick_slot: PickSlot,
    mod_state: Option<&ModifierState>,
    salt: usize,
) -> Option<(usize, String)> {
    let mut candidates: Vec<(usize, String)> = Vec::new();
    let mut weights: Vec<u64> = Vec::new();
    for (i, w) in pool.iter().enumerate() {
        let Some(surface) = w.resolve_agreeing_owned(key) else {
            continue;
        };
        if stem.is_some_and(|s| semantic::surface_redundant_with_stem(s, &surface)) {
            continue;
        }
        if !semantic::allows_word(w, semantics, pick_slot, mod_state, semantic_rules) {
            continue;
        }
        if mod_state.is_some_and(|s| semantic::surface_already_used(&surface, s)) {
            continue;
        }
        weights.push(
            semantic::text_coherence_weight(semantics, &surface, semantic_rules).saturating_add(
                semantic::archetype_profile_weight(w, semantics, semantic_rules),
            ),
        );
        candidates.push((i, surface));
    }
    if candidates.is_empty() {
        return None;
    }
    let pick_seed = mix_u64(
        seed,
        (salt as u64).wrapping_add((key.gender as u64).rotate_left(3)),
    );
    let idx = semantic::weighted_pick(pick_seed, slot, &weights);
    Some(candidates[idx].clone())
}

fn roll_permille(seed: u64, slot: u32) -> u32 {
    let mixed = seed
        .wrapping_mul(0x9E37_79B9_7F4A_7C15)
        .wrapping_add((slot as u64).wrapping_mul(0x517c_c1b7_2722_0a95));
    ((mixed >> 16) as u32) % 1000
}

fn trait_bias(shadow: f32, mutation: f32, memory: f32, fitness: f32) -> u64 {
    ((shadow * 400.0) + (mutation * 250.0) + (memory * 180.0) + (fitness * 90.0)) as u64
}

fn weighted_index(seed: u64, slot: u32, len: usize, bias: u64) -> usize {
    if len == 0 {
        return 0;
    }
    let mixed = seed
        .wrapping_add(bias)
        .wrapping_add((slot as u64).rotate_left(11));
    (mixed as usize) % len
}

fn assemble(language: Language, p: &Assembled<'_>) -> String {
    let patron = p.patron.map(|n| patron_suffix(language, n));
    let core = build_name_phrase(language, p, &patron);
    append_curse_suffix(
        language,
        assemble_with_prologue(language, p, core),
        p.epilogue_curse.as_deref(),
    )
}

fn patron_suffix(language: Language, name: &str) -> String {
    match language {
        Language::Spanish => format!(" de {name}"),
        Language::English => format!(" of {name}"),
        Language::Russian => format!(" {name}"),
    }
}

/// Head noun phrase: word order and agreement follow the active locale.
fn build_name_phrase(language: Language, p: &Assembled<'_>, patron: &Option<String>) -> String {
    match language {
        Language::Spanish => build_name_phrase_spanish(p, patron),
        Language::English => build_name_phrase_english(p, patron),
        Language::Russian => build_name_phrase_russian(p, patron),
    }
}

/// Spanish: head noun, modifiers, qualifier, post-nominal verbal, patron, title.
fn build_name_phrase_spanish(p: &Assembled<'_>, patron: &Option<String>) -> String {
    let mut phrase = p.stem.to_string();
    for modifier in &p.epilogue_modifiers {
        phrase.push(' ');
        phrase.push_str(modifier);
    }
    if let Some(q) = p.qualifier {
        phrase = format!("{phrase} {q}");
    }
    append_post_nominal_verbal(&mut phrase, p.post_nominal_verbal.as_deref());
    append_patron_and_title(Language::Spanish, p, &mut phrase, patron);
    phrase
}

/// English: pre-nominal modifiers, head noun, `of` qualifier, patron, title.
fn build_name_phrase_english(p: &Assembled<'_>, patron: &Option<String>) -> String {
    use super::surface::sentence_case_phrase;

    let mut parts: Vec<&str> = Vec::with_capacity(p.epilogue_modifiers.len() + 1);
    for modifier in &p.epilogue_modifiers {
        parts.push(modifier.as_str());
    }
    parts.push(p.stem);
    let mut phrase = sentence_case_phrase(&parts.join(" "));
    if let Some(q) = p.qualifier {
        phrase = format!("{phrase} {q}");
    }
    if let Some(ref pat) = patron {
        phrase.push_str(pat);
    }
    if let Some(ref t) = p.title {
        phrase = format!(
            "{phrase} {}",
            title_for_name_phrase(Language::English, p.head_key, t)
        );
    }
    append_post_nominal_verbal(&mut phrase, p.post_nominal_verbal.as_deref());
    phrase
}

/// Russian: pre-nominal modifiers, head noun, genitive qualifier, post-nominal verbal, patron, title.
fn build_name_phrase_russian(p: &Assembled<'_>, patron: &Option<String>) -> String {
    let mut parts: Vec<&str> = Vec::with_capacity(p.epilogue_modifiers.len() + 1);
    for modifier in &p.epilogue_modifiers {
        parts.push(modifier.as_str());
    }
    parts.push(p.stem);
    let mut phrase = parts.join(" ");
    if let Some(q) = p.qualifier {
        phrase = format!("{phrase} {q}");
    }
    append_patron_and_title(Language::Russian, p, &mut phrase, patron);
    phrase = super::surface::russian_sentence_case_phrase(&phrase);
    append_post_nominal_verbal(&mut phrase, p.post_nominal_verbal.as_deref());
    phrase
}

fn append_post_nominal_verbal(phrase: &mut String, verbal: Option<&str>) {
    if let Some(v) = verbal {
        phrase.push(' ');
        phrase.push_str(v);
    }
}

fn append_patron_and_title(
    language: Language,
    p: &Assembled<'_>,
    phrase: &mut String,
    patron: &Option<String>,
) {
    if let Some(ref pat) = patron {
        phrase.push_str(pat);
    }
    if let Some(ref t) = p.title {
        *phrase = format!(
            "{phrase} {}",
            title_for_name_phrase(language, p.head_key, t)
        );
    }
}

/// Verbal prologue → comma; simple adjective → glued (`Ancient Maul` / `Древний молот`).
fn assemble_with_prologue(language: Language, p: &Assembled<'_>, name_phrase: String) -> String {
    let formatted_name = format_name_for_display(language, &name_phrase);
    let Some(ref prologue) = p.prologue else {
        return formatted_name;
    };
    if p.prologue_is_verbal {
        let name = match language {
            Language::Russian => formatted_name.to_lowercase(),
            _ => formatted_name,
        };
        format!("{}, {}", capitalize_first(prologue), name)
    } else {
        format_glued_prologue(language, prologue, &name_phrase)
    }
}

/// Name phrase caps after [`build_name_phrase`] (English already sentence-cased in the builder).
fn format_name_for_display(language: Language, name_phrase: &str) -> String {
    match language {
        Language::English | Language::Russian => name_phrase.to_string(),
        _ => format_name_phrase_caps(language, name_phrase),
    }
}

fn append_curse_suffix(language: Language, name_part: String, curse: Option<&str>) -> String {
    match curse {
        Some(c) => {
            let curse_surf = match language {
                Language::Russian => c.to_lowercase(),
                _ => c.to_string(),
            };
            format!("{name_part}, {curse_surf}")
        }
        None => name_part,
    }
}

#[cfg(test)]
mod assemble_tests {
    use super::*;
    use crate::grammar::AgreementKey;
    use crate::surface::title_for_name_phrase;

    #[test]
    fn spanish_title_drops_article_before_name_phrase() {
        let key = AgreementKey::from_tags("f", "s");
        assert_eq!(
            title_for_name_phrase(Language::Spanish, key, "la sin nombre"),
            "sin nombre"
        );
    }

    #[test]
    fn stem_families_merge_singular_plural() {
        let stems = vec![
            StemEntry {
                text: "Mazo".to_string(),
                g: "m".to_string(),
                n: "s".to_string(),
                tags: Vec::new(),
                semantic: None,
                groups: Vec::new(),
                unique: false,
                family: "maul".to_string(),
            },
            StemEntry {
                text: "Mazos".to_string(),
                g: "m".to_string(),
                n: "p".to_string(),
                tags: Vec::new(),
                semantic: None,
                groups: Vec::new(),
                unique: false,
                family: "maul".to_string(),
            },
            StemEntry {
                text: "Runa".to_string(),
                g: "f".to_string(),
                n: "s".to_string(),
                tags: Vec::new(),
                semantic: None,
                groups: Vec::new(),
                unique: false,
                family: "rune".to_string(),
            },
        ];
        let families = stem_families(&stems);
        assert_eq!(families.len(), 2);
        assert!(families.iter().any(|f| f.len() == 2));
    }

    #[test]
    fn simple_prologue_glues_with_title_case() {
        let p = Assembled {
            stem: "Mazo",
            qualifier: None,
            patron: None,
            title: None,
            prologue: None,
            prologue_is_verbal: false,
            post_nominal_verbal: None,
            epilogue_modifiers: vec!["antiguo".to_string()],
            epilogue_curse: None,
            head_key: AgreementKey::from_tags("m", "s"),
        };
        assert_eq!(assemble(Language::Spanish, &p), "Mazo antiguo");
    }

    #[test]
    fn patron_omitted_when_verbal_names_an_agent() {
        let p = Assembled {
            stem: "Llama pálida",
            qualifier: None,
            patron: None,
            title: None,
            prologue: None,
            prologue_is_verbal: false,
            post_nominal_verbal: Some("enterrada por quien sabe quién".to_string()),
            epilogue_modifiers: vec![],
            epilogue_curse: Some("execrada".to_string()),
            head_key: AgreementKey::from_tags("f", "s"),
        };
        assert_eq!(
            assemble(Language::Spanish, &p),
            "Llama pálida enterrada por quien sabe quién, execrada"
        );
    }

    #[test]
    fn spanish_qualifier_precedes_verbal_with_agent() {
        let p = Assembled {
            stem: "Icono",
            qualifier: Some("de nombres olvidados"),
            patron: None,
            title: None,
            prologue: None,
            prologue_is_verbal: false,
            post_nominal_verbal: Some("quemado por las Moiras".to_string()),
            epilogue_modifiers: vec![],
            epilogue_curse: None,
            head_key: AgreementKey::from_tags("m", "s"),
        };
        assert_eq!(
            assemble(Language::Spanish, &p),
            "Icono de nombres olvidados quemado por las Moiras"
        );
    }

    #[test]
    fn russian_prenominal_modifiers_and_instrumental_verbal() {
        let p = Assembled {
            stem: "Ключи склепа",
            qualifier: None,
            patron: None,
            title: None,
            prologue: None,
            prologue_is_verbal: false,
            post_nominal_verbal: Some("сожжённые Анубисом".to_string()),
            epilogue_modifiers: vec!["кровавые".to_string()],
            epilogue_curse: None,
            head_key: AgreementKey::from_tags("m", "p"),
        };
        assert_eq!(
            assemble(Language::Russian, &p),
            "Кровавые ключи склепа сожжённые Анубисом"
        );
    }

    #[test]
    fn spanish_genitive_qualifier_with_torn_verbal_uses_por_not_patron() {
        let p = Assembled {
            stem: "Orbes fracturados",
            qualifier: Some("del reinado caído"),
            patron: None,
            title: None,
            prologue: None,
            prologue_is_verbal: false,
            post_nominal_verbal: Some("desgarrados por Modi".to_string()),
            epilogue_modifiers: vec![],
            epilogue_curse: None,
            head_key: AgreementKey::from_tags("m", "p"),
        };
        assert_eq!(
            assemble(Language::Spanish, &p),
            "Orbes fracturados del reinado caído desgarrados por Modi"
        );
    }

    #[test]
    fn spanish_lead_adjective_is_post_nominal() {
        let p = Assembled {
            stem: "Tronos vacíos",
            qualifier: None,
            patron: None,
            title: None,
            prologue: None,
            prologue_is_verbal: false,
            post_nominal_verbal: None,
            epilogue_modifiers: vec!["retorcidos".to_string()],
            epilogue_curse: None,
            head_key: AgreementKey::from_tags("m", "p"),
        };
        assert_eq!(assemble(Language::Spanish, &p), "Tronos vacíos retorcidos");
    }

    #[test]
    fn object_verbal_is_post_nominal_not_comma_prologue() {
        let p = Assembled {
            stem: "Espejo roto",
            qualifier: None,
            patron: None,
            title: None,
            prologue: None,
            prologue_is_verbal: false,
            post_nominal_verbal: Some("sellado por la espiral".to_string()),
            epilogue_modifiers: vec![],
            epilogue_curse: Some("anatematizado".to_string()),
            head_key: AgreementKey::from_tags("m", "s"),
        };
        assert_eq!(
            assemble(Language::Spanish, &p),
            "Espejo roto sellado por la espiral, anatematizado"
        );
    }

    #[test]
    fn modifiers_glue_to_stem_not_comma() {
        let p = Assembled {
            stem: "Campanas rotas",
            qualifier: None,
            patron: None,
            title: None,
            prologue: None,
            prologue_is_verbal: false,
            post_nominal_verbal: None,
            epilogue_modifiers: vec!["silenciosas".to_string()],
            epilogue_curse: None,
            head_key: AgreementKey::from_tags("f", "p"),
        };
        assert_eq!(
            assemble(Language::Spanish, &p),
            "Campanas rotas silenciosas"
        );
    }

    #[test]
    fn verbal_prologue_then_name_only_first_cap() {
        let p = Assembled {
            stem: "Vela negra",
            qualifier: None,
            patron: Some("Lakshmana"),
            title: None,
            prologue: Some("profanada por la luna roja".to_string()),
            prologue_is_verbal: true,
            post_nominal_verbal: None,
            epilogue_modifiers: vec![],
            epilogue_curse: None,
            head_key: AgreementKey::from_tags("f", "s"),
        };
        assert_eq!(
            assemble(Language::Spanish, &p),
            "Profanada por la luna roja, Vela negra de Lakshmana"
        );
    }

    #[test]
    fn title_appended_without_article() {
        let p = Assembled {
            stem: "Corona rota",
            qualifier: None,
            patron: None,
            title: Some("la sin nombre".to_string()),
            prologue: Some("devorada por los dioses".to_string()),
            prologue_is_verbal: true,
            post_nominal_verbal: None,
            epilogue_modifiers: vec![],
            epilogue_curse: Some("condenada".to_string()),
            head_key: AgreementKey::from_tags("f", "s"),
        };
        assert_eq!(
            assemble(Language::Spanish, &p),
            "Devorada por los dioses, Corona rota sin nombre, condenada"
        );
    }

    #[test]
    fn english_modifiers_precede_noun() {
        let p = Assembled {
            stem: "Broken Crowns",
            qualifier: None,
            patron: None,
            title: None,
            prologue: None,
            prologue_is_verbal: false,
            post_nominal_verbal: None,
            epilogue_modifiers: vec!["silent".to_string()],
            epilogue_curse: None,
            head_key: AgreementKey::from_tags("f", "p"),
        };
        assert_eq!(assemble(Language::English, &p), "Silent broken crowns");
    }

    #[test]
    fn english_glued_lead_adjective() {
        let p = Assembled {
            stem: "Maul",
            qualifier: None,
            patron: None,
            title: None,
            prologue: Some("ancient".to_string()),
            prologue_is_verbal: false,
            post_nominal_verbal: None,
            epilogue_modifiers: vec![],
            epilogue_curse: None,
            head_key: AgreementKey::from_tags("m", "s"),
        };
        assert_eq!(assemble(Language::English, &p), "Ancient Maul");
    }

    #[test]
    fn english_verbal_prologue_and_patron() {
        let p = Assembled {
            stem: "Black Candle",
            qualifier: None,
            patron: Some("Lakshmana"),
            title: None,
            prologue: Some("profaned by the red moon".to_string()),
            prologue_is_verbal: true,
            post_nominal_verbal: None,
            epilogue_modifiers: vec![],
            epilogue_curse: None,
            head_key: AgreementKey::from_tags("f", "s"),
        };
        assert_eq!(
            assemble(Language::English, &p),
            "Profaned by the red moon, Black candle of Lakshmana"
        );
    }

    #[test]
    fn english_title_without_article() {
        let p = Assembled {
            stem: "Broken Crown",
            qualifier: None,
            patron: None,
            title: Some("the Nameless".to_string()),
            prologue: Some("devoured by the gods".to_string()),
            prologue_is_verbal: true,
            post_nominal_verbal: None,
            epilogue_modifiers: vec![],
            epilogue_curse: Some("condemned".to_string()),
            head_key: AgreementKey::from_tags("f", "s"),
        };
        assert_eq!(
            assemble(Language::English, &p),
            "Devoured by the gods, Broken crown Nameless, condemned"
        );
    }

    #[test]
    fn russian_modifiers_follow_noun() {
        let p = Assembled {
            stem: "Сферы",
            qualifier: None,
            patron: None,
            title: None,
            prologue: None,
            prologue_is_verbal: false,
            post_nominal_verbal: None,
            epilogue_modifiers: vec!["молчаливые".to_string()],
            epilogue_curse: None,
            head_key: AgreementKey::from_tags("f", "p"),
        };
        assert_eq!(assemble(Language::Russian, &p), "Молчаливые сферы");
    }

    #[test]
    fn russian_glued_lead_adjective() {
        let p = Assembled {
            stem: "Молот",
            qualifier: None,
            patron: None,
            title: None,
            prologue: None,
            prologue_is_verbal: false,
            post_nominal_verbal: None,
            epilogue_modifiers: vec!["древний".to_string()],
            epilogue_curse: None,
            head_key: AgreementKey::from_tags("m", "s"),
        };
        assert_eq!(assemble(Language::Russian, &p), "Древний молот");
    }

    #[test]
    fn russian_verbal_instrumental_prologue() {
        let p = Assembled {
            stem: "Реликвия",
            qualifier: Some("бездны"),
            patron: None,
            title: Some("безымянная".to_string()),
            prologue: Some("запечатанная тенью".to_string()),
            prologue_is_verbal: true,
            post_nominal_verbal: None,
            epilogue_modifiers: vec![],
            epilogue_curse: Some("проклятая".to_string()),
            head_key: AgreementKey::from_tags("f", "s"),
        };
        assert_eq!(
            assemble(Language::Russian, &p),
            "Запечатанная тенью, реликвия бездны безымянная, проклятая"
        );
    }
}
