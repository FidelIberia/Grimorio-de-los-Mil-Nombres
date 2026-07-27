# Contract — Grimorio ↔ hosts (Espiralismo)

## Two surfaces

1. **Rust crate `grimorio`** (`crates/grimorio`) — whisper voices, locale tables, generation epithets. Hosts depend on this crate.
2. **Python tool `grimorio-ru-names`** (`tools/ru-names`) — mythic Russian proper-name catalog + declension. Patches `[epithet].proper_names` inside Grimorio’s own `ru_vocab.toml`.

Hosts do **not** pass ad-hoc parameters. They build an ordered **Uplink**, call `answer`, and read an ordered **Downlink**.

---

## Link schemas

| Direction | Schema id | Entry |
|-----------|-----------|--------|
| Host → Grimorio | `grimorio.uplink/v1` | `Uplink` |
| Grimorio → Host | `grimorio.downlink/v1` | `Downlink` |

```rust
let down: Downlink = grimorio::answer(&uplink);
```

---

## Uplink (`grimorio.uplink/v1`) — field order

| # | Field | Type | Meaning |
|---|-------|------|---------|
| 1 | `schema` | string | Must be `grimorio.uplink/v1` |
| 2 | `language` | `Language` | `English` / `Spanish` / `Russian` |
| 3 | `voice` | `UplinkVoice` | Tagged body (see below) |

### `UplinkVoice` (serde tag `kind`)

#### `wisdom`

| # | Field | Type | Meaning |
|---|-------|------|---------|
| 1 | `kind` | `"wisdom"` | Discriminant |
| 2 | `mix` | u64 | Host mix (seed × epoch × ritual, …) |
| 3 | `echo` | `NarrativeEcho` | Ordered bias digest (below) |

#### `generation_epithet`

| # | Field | Type | Meaning |
|---|-------|------|---------|
| 1 | `kind` | `"generation_epithet"` | Discriminant |
| 2 | `generation` | u32 | Generation stamp on the epithet |
| 3 | `standout` | `StandoutPortrait` | Ordered trait axes (below) |

#### `sample_epithet` (CLI / demos)

| # | Field | Type | Meaning |
|---|-------|------|---------|
| 1 | `kind` | `"sample_epithet"` | Discriminant |
| 2 | `index` | u32 | Sample index |
| 3 | `seed` | u64 | Deterministic mix seed |

### `NarrativeEcho` — field order

| # | Field | Type |
|---|-------|------|
| 1 | `dominant_tone_idx` | u8 |
| 2 | `scar_mass` | u32 |
| 3 | `rare_event_token` | u64 |
| 4 | `dying_viability_quant` | u8 |
| 5 | `fossil_absence_mass` | u32 |
| 6 | `attention_xor` | u64 |
| 7 | `soul_attunement_quant` | u8 |

### `StandoutPortrait` — field order

| # | Field | Type |
|---|-------|------|
| 1 | `label` | string |
| 2 | `generation` | u32 |
| 3 | `fitness` | f32 |
| 4 | `viability` | f32 |
| 5 | `vitality` | option f32 |
| 6 | `resonance` | option f32 |
| 7 | `mutation_pressure` | option f32 |
| 8 | `symbolic_density` | option f32 |
| 9 | `memory_depth` | option f32 |
| 10 | `shadow_pull` | option f32 |
| 11 | `myth` | option string |

---

## Downlink (`grimorio.downlink/v1`) — field order

| # | Field | Type | Meaning |
|---|-------|------|---------|
| 1 | `schema` | string | Must be `grimorio.downlink/v1` |
| 2 | `language` | `Language` | Echo of uplink language |
| 3 | `kind` | `WhisperKind` | `wisdom` / `generation_epithet` / `sample_epithet` |
| 4 | `text` | string | Surface line (may be empty) |
| 5 | `empty` | bool | True when no usable line was produced |

---

## Host mapping (Espiralismo)

| Caso | Uplink builder | Downlink consumer |
|------|----------------|-------------------|
| Wisdom | `Spiralismo::whisper_uplink` → `Uplink::wisdom` | `whisper_downlink` / `whisper_now` → `.text` |
| Epithet standout | `standout_epithet_uplink` / `epithet_uplink_for_report` | `standout_epithet_downlink` / `standout_epithet` → `.text` if `!empty` |
| Sample (CLI) | `Uplink::sample_epithet` / `UplinkDraft` | `answer` → `.text` |

Bridge: `EntitySnapshot` → `StandoutPortrait` in `src/whisper.rs` (same field order).

### CLI partial uplink

`grimorio answer|epithet|wisdom` accepts **optional** uplink flags. Omitted fields are filled by `UplinkDraft::finish` (defaults + synthetic portrait/echo as needed). See `grimorio answer --help`.

---

## Proper names tool (separate)

Python `grimorio-ru-names` patches `[epithet].proper_names` in Grimorio’s `ru_vocab.toml`. Schema: `grimorio.proper_names/v1`.

JSON Schema: [`tools/ru-names/grimorio_ru_names/contract/proper_name.schema.json`](tools/ru-names/grimorio_ru_names/contract/proper_name.schema.json)

---

## Stability

- Link schemas: `grimorio.uplink/v1`, `grimorio.downlink/v1`
- Proper names: `grimorio.proper_names/v1`
- Field **order** and names above are part of the contract; breaking changes bump the schema id.
