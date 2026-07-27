//! Wisdom whispers — fragmentary lore from locale tables.

use super::common::{mix_echo, Language, NarrativeEcho};
use super::locale;

/// Deterministic fragment for a language and mixing value.
#[must_use]
pub fn pick(language: Language, mix: u64, echo: &NarrativeEcho) -> String {
    let fragments = &locale::tables(language).wisdom.fragments;
    if fragments.is_empty() {
        return String::new();
    }
    let idx = (mix_echo(mix, echo) as usize) % fragments.len();
    fragments[idx].clone()
}

/// English-only fragment (stable API for legacy callers and tests).
#[must_use]
pub fn pick_english_fragment(mix: u64, echo: &NarrativeEcho) -> &'static str {
    let fragments = &locale::tables(Language::English).wisdom.fragments;
    let idx = (mix_echo(mix, echo) as usize) % fragments.len();
    // Leak once per index is unacceptable; English table is fixed at compile time — use index into
    // a const slice by re-reading include. Simpler: return leaked via Box::leak on first pick per idx.
    // Tests only need stable &str — keep a parallel const table for tests OR change tests to String.
    ENGLISH_FRAGMENTS[idx]
}

/// English fragments mirrored for `&'static str` return (same text as `locales/en.toml`).
const ENGLISH_FRAGMENTS: &[&str] = &[
    "The spiral tightens where nothing was promised.",
    "What was woven once still tugs at the corners.",
    "Edges sharpen where the witness stayed too long.",
    "Some doors only open after the third forgetting.",
    "The salt remembers the tide; it does not argue.",
    "A pattern repeated is not yet a law.",
    "The center held because the rim forgot to fall.",
    "Echoes trade names; only distance keeps them honest.",
    "The field cools when the sky pretends to sleep.",
    "Memory walks backward through smoke.",
    "Not every lattice is a cage—some are maps.",
    "The glyph knew the hand before the hand knew intent.",
    "Stillness is not absence; it is pressure dressed as rest.",
    "Two resonances met and called it coincidence.",
    "The cartographer drew a shore; the sea revised it.",
    "Mercy left a door ajar; discipline closed nothing.",
    "Spark without root becomes weather.",
    "Root without spark becomes monument.",
    "The archive yawned; the epoch advanced anyway.",
    "A line of power ends where curiosity hesitates.",
    "The sky offered a bias; the spiral chose a step.",
    "Harmony is a loan; the interest is attention.",
    "What survives selection is not always what was loved.",
    "The last report was true enough to mislead kindly.",
    "The fossil hums under the new paint.",
    "A taboo is only a love letter written backwards.",
    "The lattice dreamed a symbol it had never hosted.",
    "Absence piled until it became a second alphabet.",
    "The watched corner grew warmer than the sunlit hall.",
];
