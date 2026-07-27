//! Host-agnostic standout axes for generation epithets.

/// Trait axes the epithet forge reads (no host evolution types).
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct StandoutPortrait {
    /// Human-readable identifier provided by the host.
    pub label: String,
    /// Current generation counter.
    pub generation: u32,
    /// Current fitness value.
    pub fitness: f32,
    /// Computed viability score in `[0.0, 1.0]`.
    pub viability: f32,
    /// Vitality axis (optional; unused by forge today, kept for host parity).
    #[serde(default)]
    pub vitality: Option<f32>,
    /// Resonance axis snapshot.
    #[serde(default)]
    pub resonance: Option<f32>,
    /// Effective mutation pressure.
    #[serde(default)]
    pub mutation_pressure: Option<f32>,
    /// Symbolic pattern density.
    #[serde(default)]
    pub symbolic_density: Option<f32>,
    /// Archive depth / lattice scar memory proxy.
    #[serde(default)]
    pub memory_depth: Option<f32>,
    /// Shadow pull — low harmony, high ritual tension.
    #[serde(default)]
    pub shadow_pull: Option<f32>,
    /// Emergent mythic role token (optional).
    #[serde(default)]
    pub myth: Option<String>,
}
