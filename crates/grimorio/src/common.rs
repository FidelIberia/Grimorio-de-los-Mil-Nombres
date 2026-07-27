//! Shared hashing and narrative echo for all whisper voices.

/// Active locale for wisdom fragments and generation epithets.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Default, serde::Serialize, serde::Deserialize,
)]
pub enum Language {
    #[default]
    English,
    Spanish,
    Russian,
}

impl Language {
    /// Stable token for logs and checkpoints.
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::English => "en",
            Self::Spanish => "es",
            Self::Russian => "ru",
        }
    }

    /// Parse CLI tokens (`english`, `spanish`, `russian`, `rusian`).
    #[must_use]
    pub fn from_cli_flag(flag: &str) -> Option<Self> {
        match flag {
            "english" | "en" => Some(Self::English),
            "spanish" | "es" | "espanol" => Some(Self::Spanish),
            "russian" | "rusian" | "ru" => Some(Self::Russian),
            _ => None,
        }
    }
}

/// Compact digest of internal history used to bias whisper selection.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct NarrativeEcho {
    /// Dominant tone index in the host's tone table order (e.g. GlyphTone::ALL).
    pub dominant_tone_idx: u8,
    /// Lattice scar mass (or `0` if no lattice).
    pub scar_mass: u32,
    /// FNV-1a of last rare-event token (or `0`).
    pub rare_event_token: u64,
    /// Quantized minimum viability across last report (`255` = healthy).
    pub dying_viability_quant: u8,
    /// Ghost-breath mass from glyph archaeology.
    pub fossil_absence_mass: u32,
    /// Observer attention digest.
    pub attention_xor: u64,
    /// Soul attunement quantized (`0..255`) from the host's soul state.
    pub soul_attunement_quant: u8,
}

/// FNV-1a 64 over bytes (stable tiny hasher for event names).
#[must_use]
pub fn fnv1a64(data: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in data {
        h ^= *b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

/// Mixes a scalar into a running hash.
#[must_use]
pub fn mix_u64(base: u64, value: u64) -> u64 {
    base.wrapping_add(value.rotate_left(17))
}

#[must_use]
pub fn mix_echo(base: u64, echo: &NarrativeEcho) -> u64 {
    let mut x = base;
    x ^= (echo.dominant_tone_idx as u64).rotate_left(17);
    x ^= (echo.scar_mass as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    x ^= echo.rare_event_token.rotate_left(11);
    x ^= (echo.dying_viability_quant as u64).wrapping_mul(0x85eb_ca6b);
    x ^= (echo.fossil_absence_mass as u64).rotate_left(23);
    x ^= echo.attention_xor;
    x ^= (echo.soul_attunement_quant as u64).wrapping_mul(0x517c_c1b7_2722_0a95);
    x
}

/// Quantizes `f32` in `[0,1]` (or any finite value) to `0..=255`.
#[must_use]
pub fn quantize01(value: f32) -> u8 {
    if !value.is_finite() {
        return 0;
    }
    (value.clamp(0.0, 1.0) * 255.0) as u8
}
