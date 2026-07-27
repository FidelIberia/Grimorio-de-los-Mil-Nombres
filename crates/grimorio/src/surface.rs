//! Language-specific epithet surface rules (order, caps, titles).

use super::common::Language;
use super::grammar::{AgreementKey, Gender, Number};

/// Whether a comma-separated segment looks like a verbal prologue (`… por …`, `… by …`).
#[cfg(test)]
#[must_use]
fn segment_is_verbal(language: Language, segment: &str) -> bool {
    match language {
        Language::Spanish => segment.contains(" por "),
        Language::English => segment.contains(" by "),
        Language::Russian => russian_segment_is_verbal(segment),
    }
}

/// Russian participle phrase includes an agent (`запечатанная тенью`, `раздавленные титаном`).
#[must_use]
pub fn russian_participle_has_agent_phrase(phrase: &str) -> bool {
    let words: Vec<&str> = phrase.split_whitespace().collect();
    words.len() >= 2 && russian_looks_like_participle(words[0])
}

/// Russian: participle + instrumental agent (`запечатанная тенью`, `призванный кем-то`).
#[cfg(test)]
fn russian_segment_is_verbal(segment: &str) -> bool {
    if segment.contains(" кем-то") || segment.contains(" чем-то") {
        return true;
    }
    let words: Vec<&str> = segment.split_whitespace().collect();
    if words.len() < 2 {
        return false;
    }
    russian_looks_like_participle(words[0])
        && russian_instrumental_agent(words.last().unwrap_or(&""))
}

#[must_use]
pub fn russian_looks_like_participle(word: &str) -> bool {
    word.ends_with("нный")
        || word.ends_with("нная")
        || word.ends_with("нные")
        || word.ends_with("ённый")
        || word.ends_with("ённая")
        || word.ends_with("ённые")
        || word.ends_with("енный")
        || word.ends_with("енная")
        || word.ends_with("енные")
        || word.ends_with("тый")
        || word.ends_with("тая")
        || word.ends_with("тые")
        || word.ends_with("чённый")
        || word.ends_with("чённая")
        || word.ends_with("чённые")
        || word.ends_with("меченый")
        || word.ends_with("меченая")
        || word.ends_with("меченые")
}

#[cfg(test)]
fn russian_instrumental_agent(word: &str) -> bool {
    word.ends_with("ой")
        || word.ends_with("ом")
        || word.ends_with("ём")
        || word.ends_with("ем")
        || word.ends_with("ами")
        || word.ends_with("ями")
        || word.ends_with("ою")
        || word.ends_with("ьмой")
        || word.ends_with("нею")
        || word.ends_with("тенью")
        || word.ends_with("бездной")
        || word.ends_with("тьмой")
        || word.ends_with("орками")
        || word.ends_with("богами")
        || word.ends_with("титаном")
        || word.ends_with("титанами")
        || word.ends_with("Мойрами")
        || word.ends_with("судом")
        || word.ends_with("морем")
        || word.ends_with("бурей")
        || word.ends_with("змеёй")
        || word.ends_with("мёртвыми")
}

/// English and Russian stack adjectives before the noun; Spanish uses post-nominal order.
#[must_use]
pub fn adjectives_precede_noun(language: Language) -> bool {
    matches!(language, Language::English | Language::Russian)
}

/// Default caps on the name phrase after a verbal prologue.
#[must_use]
pub fn format_name_phrase_caps(language: Language, phrase: &str) -> String {
    match language {
        Language::English => sentence_case_phrase(phrase),
        Language::Spanish => capitalize_first(phrase),
        Language::Russian => russian_sentence_case_phrase(phrase),
    }
}

#[must_use]
pub fn sentence_case_phrase(s: &str) -> String {
    capitalize_first(&s.to_lowercase())
}

/// Literary Russian: first word capitalized; following words lowercased.
/// After a comma, later segments are fully lowercased (`Запечатанная тенью, реликвия …`).
#[must_use]
pub fn russian_sentence_case_phrase(s: &str) -> String {
    if !s.contains(", ") {
        return russian_sentence_case_segment(s);
    }
    let mut parts: Vec<String> = s.split(", ").map(str::to_string).collect();
    if let Some(first) = parts.first_mut() {
        *first = russian_sentence_case_segment(first);
    }
    for seg in parts.iter_mut().skip(1) {
        *seg = seg.to_lowercase();
    }
    parts.join(", ")
}

fn russian_sentence_case_segment(s: &str) -> String {
    let words: Vec<&str> = s.split_whitespace().collect();
    if words.is_empty() {
        return String::new();
    }
    let mut out = capitalize_first(words[0]);
    for w in words.iter().skip(1) {
        out.push(' ');
        out.push_str(&w.to_lowercase());
    }
    out
}

/// Glued lead adjective + name (`Ancient Maul`, `Древний молот`).
#[must_use]
pub fn format_glued_prologue(language: Language, lead: &str, name_phrase: &str) -> String {
    let glued = format!("{lead} {name_phrase}");
    match language {
        Language::English => title_case_epithet(&glued),
        Language::Spanish => title_case_epithet(&glued),
        Language::Russian => russian_sentence_case_phrase(&glued),
    }
}

#[must_use]
pub fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

#[must_use]
pub fn title_case_epithet(s: &str) -> String {
    s.split_whitespace()
        .map(capitalize_first)
        .collect::<Vec<_>>()
        .join(" ")
}

/// Title appositive after a full noun phrase (drops articles in es/en).
#[must_use]
pub fn title_for_name_phrase(language: Language, key: AgreementKey, title: &str) -> String {
    let trimmed = title.trim();
    match language {
        Language::Spanish => strip_prefixes(trimmed, spanish_title_articles(key)),
        Language::English => strip_prefixes(trimmed, &["the ", "The ", "a ", "A ", "an ", "An "]),
        Language::Russian => strip_prefixes(trimmed, russian_title_prefixes(key)),
    }
}

fn spanish_title_articles(key: AgreementKey) -> &'static [&'static str] {
    match (key.gender, key.number) {
        (Gender::F, Number::Singular) => &["la ", "La "],
        (Gender::F, Number::Plural) => &["las ", "Las "],
        (Gender::M, Number::Plural) => &["los ", "Los "],
        _ => &["el ", "El ", "los ", "Los "],
    }
}

fn russian_title_prefixes(key: AgreementKey) -> &'static [&'static str] {
    match (key.gender, key.number) {
        (Gender::F, Number::Singular) => &["та ", "Та "],
        (Gender::F, Number::Plural) => &["те ", "Те "],
        (Gender::M, Number::Plural) => &["те ", "Те "],
        _ => &["тот ", "Тот ", "те ", "Те "],
    }
}

fn strip_prefixes(text: &str, prefixes: &[&str]) -> String {
    for prefix in prefixes {
        if let Some(rest) = text.strip_prefix(prefix) {
            return rest.to_string();
        }
    }
    text.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::epithet::forge_sample;
    use crate::grammar::AgreementKey;

    #[test]
    fn epithet_uses_verbal_state_at_most_once() {
        for language in [Language::Spanish, Language::English, Language::Russian] {
            for index in 0..80 {
                for seed in [1_u64, 77, 9001, 424_242] {
                    let name = forge_sample(language, index, seed);
                    let verbal_hits = name
                        .split(", ")
                        .filter(|seg| segment_is_verbal(language, seg))
                        .count();
                    assert!(
                        verbal_hits <= 1,
                        "expected at most one verbal segment for {language:?}, got {verbal_hits} in: {name}"
                    );
                }
            }
        }
    }

    #[test]
    fn russian_sentence_case_lowercases_after_first_word() {
        assert_eq!(
            russian_sentence_case_phrase("Кровавые Чёрные свечи"),
            "Кровавые чёрные свечи"
        );
        assert_eq!(
            russian_sentence_case_phrase("Запечатанная тенью, Реликвия бездны, проклятая"),
            "Запечатанная тенью, реликвия бездны, проклятая"
        );
    }

    #[test]
    fn english_strips_the_from_title() {
        assert_eq!(
            title_for_name_phrase(
                Language::English,
                AgreementKey::from_tags("m", "s"),
                "the Nameless"
            ),
            "Nameless"
        );
    }

    #[test]
    fn russian_strips_demonstrative_from_title() {
        let key = AgreementKey::from_tags("f", "s");
        assert_eq!(
            title_for_name_phrase(Language::Russian, key, "та безымянная"),
            "безымянная"
        );
    }
}
