//! Composes participles with optional agents (`sellado` + `por` + `la sombra`).

use super::common::Language;
use super::grammar::{AgentEntry, AgreementKey, ProperName, VerbalState};
use super::locale::SemanticTables;
use super::semantic::{self, SemanticContext, VerbalForgeContext};

/// Permille thresholds for agent attachment (tuned per call site).
pub struct VerbalAttachPolicy {
    /// Skip agent entirely (participle only).
    pub solo_permille: u32,
    /// Pick from [`AgentEntry::indefinite`] pool.
    pub indefinite_permille: u32,
}

impl Default for VerbalAttachPolicy {
    fn default() -> Self {
        Self {
            solo_permille: 320,
            indefinite_permille: 140,
        }
    }
}

/// Builds `participle` or `participle linker agent` for the active locale.
#[must_use]
pub fn compose_verbal(language: Language, participle: &str, agent: Option<&AgentEntry>) -> String {
    let Some(agent) = agent else {
        return participle.to_string();
    };
    let linker = agent.linker.trim();
    let _ = language;
    if linker.is_empty() {
        format!("{participle} {}", agent.text)
    } else {
        format!("{participle} {linker} {}", agent.text)
    }
}

/// Builds a concrete agent entry from an epithet `proper_names` string.
#[must_use]
pub fn named_verbal_agent(name: &str, linker: &str) -> AgentEntry {
    AgentEntry {
        text: name.to_string(),
        linker: linker.to_string(),
        tags: vec!["named".to_string()],
        ..AgentEntry::default()
    }
}

/// Resolves a participle and optionally attaches an agent (semantically filtered).
#[must_use]
pub fn forge_verbal_phrase(
    language: Language,
    key: AgreementKey,
    seed: u64,
    slot: u32,
    states: &[VerbalState],
    agents: &[AgentEntry],
    proper_names: &[ProperName],
    named_verbal_linker: &str,
    semantics: &SemanticContext,
    semantic_rules: &SemanticTables,
    forge: &VerbalForgeContext,
    policy: &VerbalAttachPolicy,
    roll: fn(u64, u32) -> u32,
) -> Option<String> {
    let candidates: Vec<(usize, &str)> = states
        .iter()
        .enumerate()
        .filter_map(|(i, s)| {
            let surface = s.participle(key)?;
            if semantic::allows_verbal_state(s, semantics, semantic_rules) {
                Some((i, surface))
            } else {
                None
            }
        })
        .collect();
    if candidates.is_empty() {
        return None;
    }
    let weights: Vec<u64> = candidates
        .iter()
        .map(|(_, participle)| {
            semantic::text_coherence_weight(semantics, participle, semantic_rules)
        })
        .collect();
    let idx = semantic::weighted_pick(seed, slot, &weights);
    let (state_idx, participle) = candidates[idx];
    let state = &states[state_idx];
    let verb_tags = semantic::infer_verb_tags(state, semantic_rules);
    let mut forge_with_verb = forge.clone();
    forge_with_verb.verb_tags = verb_tags;

    let needs_agent = semantic::verbal_state_requires_agent(state, semantic_rules);
    let named_agents: Vec<AgentEntry> = proper_names
        .iter()
        .filter_map(|name| {
            let text = name.verbal_surface(language)?;
            Some(named_verbal_agent(text, named_verbal_linker))
        })
        .collect();

    let agent = if needs_agent {
        pick_agent(
            seed,
            slot.wrapping_add(21),
            agents,
            &named_agents,
            false,
            &forge_with_verb,
            semantic_rules,
        )
        .or_else(|| {
            pick_agent(
                seed,
                slot.wrapping_add(22),
                agents,
                &named_agents,
                true,
                &forge_with_verb,
                semantic_rules,
            )
        })
    } else {
        let r = roll(seed, slot.wrapping_add(20));
        if r < policy.solo_permille {
            return Some(participle.to_string());
        }
        if r < policy.solo_permille + policy.indefinite_permille {
            pick_agent(
                seed,
                slot.wrapping_add(21),
                agents,
                &named_agents,
                true,
                &forge_with_verb,
                semantic_rules,
            )
        } else {
            pick_agent(
                seed,
                slot.wrapping_add(22),
                agents,
                &named_agents,
                false,
                &forge_with_verb,
                semantic_rules,
            )
        }
    };

    if needs_agent && agent.is_none() {
        return None;
    }

    let phrase = compose_verbal(language, participle, agent);
    if semantic::verbal_phrase_grammatical(language, &phrase, semantic_rules) {
        Some(phrase)
    } else {
        None
    }
}

fn pick_agent<'a>(
    seed: u64,
    slot: u32,
    agents: &'a [AgentEntry],
    named_agents: &'a [AgentEntry],
    indefinite: bool,
    forge: &VerbalForgeContext,
    semantic_rules: &SemanticTables,
) -> Option<&'a AgentEntry> {
    let pool: Vec<&AgentEntry> = agents
        .iter()
        .chain(named_agents.iter())
        .filter(|a| {
            a.indefinite == indefinite && semantic::allows_verbal_agent(a, forge, semantic_rules)
        })
        .collect();
    if pool.is_empty() {
        return None;
    }
    let weights: Vec<u64> = pool
        .iter()
        .map(|a| semantic::verbal_agent_coherence_weight(a, &forge.verb_tags, semantic_rules))
        .collect();
    let idx = semantic::weighted_pick(seed, slot, &weights);
    Some(pool[idx])
}
