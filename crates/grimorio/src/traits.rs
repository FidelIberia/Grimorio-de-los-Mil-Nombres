//! Whisper voices — pluggable prose engines behind one request surface.
//!
//! Hosts should prefer [`crate::link::answer`] with an ordered [`crate::link::Uplink`].
//! [`WhisperHub::speak`] remains as a thin adapter over that link.

use super::common::{Language, NarrativeEcho};
use super::link::{answer, Uplink};
use super::portrait::StandoutPortrait;

/// Which voice answers a [`WhisperRequest`] / uplink voice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WhisperKind {
    /// Fragmentary one-line lore (partial wisdom).
    Wisdom,
    /// Diablo-style honorific for the generation standout.
    GenerationEpithet,
    /// Synthetic sample used by CLI / demos.
    SampleEpithet,
}

/// Input bundle for any whisper voice (legacy host helper).
#[derive(Debug, Clone)]
pub struct WhisperRequest<'a> {
    pub kind: WhisperKind,
    pub language: Language,
    pub mix: u64,
    pub echo: &'a NarrativeEcho,
    /// Standout entity for [`WhisperKind::GenerationEpithet`].
    pub standout: Option<&'a StandoutPortrait>,
    /// Generation index stamped on the epithet.
    pub generation: u32,
}

/// Routes requests through the uplink/downlink link.
#[derive(Debug, Clone, Copy, Default)]
pub struct WhisperHub;

impl WhisperHub {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Builds an [`Uplink`] from a legacy request and returns the downlink text.
    #[must_use]
    pub fn speak(&self, request: &WhisperRequest<'_>) -> String {
        self.answer_request(request).text
    }

    /// Same path as [`crate::link::answer`], via a legacy request shape.
    #[must_use]
    pub fn answer_request(&self, request: &WhisperRequest<'_>) -> crate::link::Downlink {
        let uplink = match request.kind {
            WhisperKind::Wisdom => {
                Uplink::wisdom(request.language, request.mix, request.echo.clone())
            }
            WhisperKind::GenerationEpithet => {
                let Some(standout) = request.standout else {
                    return crate::link::Downlink {
                        schema: crate::link::DOWNLINK_SCHEMA.to_string(),
                        language: request.language,
                        kind: WhisperKind::GenerationEpithet,
                        text: String::new(),
                        empty: true,
                    };
                };
                Uplink::generation_epithet(
                    request.language,
                    request.generation,
                    standout.clone(),
                )
            }
            WhisperKind::SampleEpithet => Uplink::sample_epithet(
                request.language,
                request.generation,
                request.mix,
            ),
        };
        answer(&uplink)
    }

    /// Preferred host entry: ordered uplink → ordered downlink.
    #[must_use]
    pub fn answer(&self, uplink: &Uplink) -> crate::link::Downlink {
        answer(uplink)
    }
}

/// Curated fragmentary lore (legacy marker type).
#[derive(Debug, Clone, Copy, Default)]
pub struct WisdomVoice;

/// Diablo 2–style name forge (legacy marker type).
#[derive(Debug, Clone, Copy, Default)]
pub struct GenerationEpithetVoice;

/// A whisper implementation (legacy trait; prefer [`crate::link::answer`]).
pub trait WhisperVoice {
    fn voice_id(&self) -> &'static str;
    fn speak(&self, request: &WhisperRequest<'_>) -> String;
}

impl WhisperVoice for WisdomVoice {
    fn voice_id(&self) -> &'static str {
        "wisdom"
    }

    fn speak(&self, request: &WhisperRequest<'_>) -> String {
        WhisperHub::new().speak(&WhisperRequest {
            kind: WhisperKind::Wisdom,
            ..request.clone()
        })
    }
}

impl WhisperVoice for GenerationEpithetVoice {
    fn voice_id(&self) -> &'static str {
        "generation_epithet"
    }

    fn speak(&self, request: &WhisperRequest<'_>) -> String {
        WhisperHub::new().speak(&WhisperRequest {
            kind: WhisperKind::GenerationEpithet,
            ..request.clone()
        })
    }
}
