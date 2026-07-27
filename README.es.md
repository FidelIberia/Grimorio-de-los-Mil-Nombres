# Grimorio de los Mil Nombres

[English](README.md) · [Español](README.es.md) · [Русский](README.ru.md)

<p align="center">
  <img src="grimoriodelosmilnombres.png" alt="Grimorio de los Mil Nombres — banner" />
</p>

<p align="center"><em>Donde las cicatrices aprenden a hablar, y los nombres se niegan a balbucear dos veces.</em></p>

---

**Grimorio de los Mil Nombres** no es una lista de palabras. Es la **cámara de los susurros**: el libro que responde cuando una obra viva pide una línea de saber o un nombre verdadero forjado con ejes de cicatriz, resonancia, sombra y mito. Escrito en Rust (con una herramienta aparte en Python para nombres propios rusos), guarda lo que antes vivía como el módulo `whisper` de Espiralismo: tablas de locale, gramática de hermosura y temor, fragmentos de sabiduría y la forja de epítetos generacionales.

No posee la espiral. **Responde**. Un anfitrión construye un uplink ordenado, llama a `answer` y lee un downlink ordenado — inglés, español o ruso, cada lengua con su negativa a dejar caer una maldición sobre un núcleo que no puede sostenerla.

## Dos cámaras bajo un mismo título

El Grimorio mantiene **dos superficies distintas**. Confundirlas es cómo los nombres se vuelven huecos.

**Susurros / epítetos** (`crates/grimorio`) — el libro vivo. Voces de sabiduría y epítetos generacionales, TOMLs de locale embebidos, la forja que convierte un retrato standout en un nombre. Esto es de lo que dependen los hosts.

**Nombres propios rusos** (`tools/ru-names`) — un generador aparte. Lemas mitológicos con declinación literaria (`name` / `patron` / `verbal`) que parchean `[epithet].proper_names` dentro del propio `ru_vocab.toml` del Grimorio. Alimenta el libro; no es el libro.

Cuida ambos cuando quieras que los epítetos rusos mantengan su gramática honesta. Invoca el crate Rust cuando quieras un susurro o un nombre *ahora*.

## Qué hace el libro (en el lenguaje de la obra)

Al abrir el Grimorio envías una petición con **idioma** y una **voz**.

La **sabiduría** responde con una línea fragmentaria — saber parcial, algo casi entendido — sesgada por un eco narrativo: cicatrices del enrejado, tono dominante, eventos raros, la sintonía del alma. No es explicación. Es la frase sobre la que el velo casi se cerró.

Los **epítetos generacionales** son nombres verdaderos para quien prevaleció. Desde un retrato standout — label, fitness, viability, resonance, mutation, memory, shadow — la forja ensambla núcleos, calificativos, patrones, títulos y estados verbales para que ningún epíteto balbucee el abismo dos veces, y ninguna maldición caiga donde la gramática no puede sostenerla.

Los **epítetos de muestra** son el libro practicando solo: retratos sintéticos para demos de CLI y secuencias deterministas cuando solo quieres oír cantar a la forja.

Hosts como **Espiralismo** reexportan el crate por un adaptador fino, mapean sus entidades vivas a `StandoutPortrait` y consumen el texto del downlink. El Grimorio nunca alcanza la evolución; solo nombra y susurra.

---

## La misma obra, en sigilos claros (mapa técnico)

| Ruta | Encargo |
|------|---------|
| `crates/grimorio` | Crate Rust + binario `grimorio` (superficie whisper). |
| `src/link.rs` | `Uplink`, `UplinkVoice`, `Downlink`, `UplinkDraft`, `answer`. |
| `src/epithet.rs` / `semantic.rs` / `grammar.rs` | Forja, reglas de compatibilidad, concordancia. |
| `src/wisdom.rs` / `locale.rs` / `locales/` | Tablas de sabiduría y TOMLs `en` / `es` / `ru`. |
| `src/main.rs` | CLI: `help`, `answer`, `epithet`, `wisdom`. |
| `tools/ru-names` | Herramienta Python `grimorio-ru-names` (catálogo de nombres). |
| `CONTRACT.md` | `grimorio.uplink/v1` · `grimorio.downlink/v1` · `grimorio.proper_names/v1`. |

**Crate:** `grimorio` (versión actual **0.1.0**). **Nombre del proyecto:** **Grimorio de los Mil Nombres**.

### Superficie pública (Rust)

`answer`, `Uplink`, `UplinkDraft`, `UplinkVoice`, `Downlink` · `Language`, `NarrativeEcho`, `StandoutPortrait` · `forge_sample`, `WhisperHub`, ayudas de sabiduría.

### Camino del anfitrión (Espiralismo)

```toml
grimorio = { path = "../Grimorio de los Mil Nombres/crates/grimorio" }
```

Adaptador: `src/whisper.rs` — reexporta + `standout_epithet_for_report` / constructores de uplink.

---

## Cómo recorrer el círculo

### CLI Rust (susurros y epítetos)

```bash
cd "/ruta/a/Grimorio de los Mil Nombres"
cargo test -p grimorio
cargo run -p grimorio -- help
cargo run -p grimorio -- epithet
cargo run -p grimorio -- epithet --count 10 --seed 424242 --spanish
cargo run -p grimorio -- answer --label keeper --shadow 0.9 --lang ru
cargo run -p grimorio -- wisdom --mix 42 --scar-mass 10
cargo run -p grimorio -- answer --help
```

Instalación opcional:

```bash
cargo install --path crates/grimorio
grimorio epithet --count 5
```

Los flags de uplink son **opcionales**. `UplinkDraft` completa lo omitido. Pistas de standout (p. ej. `--label`, `--shadow`) eligen `generation_epithet` si no hay voz; pistas de wisdom eligen `wisdom`; si no, `sample_epithet`.

### Herramienta Python (nombres propios rusos)

```bash
cd tools/ru-names
python3 -m pip install -e ".[dev]"
grimorio-ru-names list
grimorio-ru-names apply --target ../../crates/grimorio/src/locales/ru_vocab.toml
```

Desde Espiralismo: `bash scripts/sync-grimorio-names.sh`.

Orden de campos: [CONTRACT.md](CONTRACT.md) · [tools/ru-names/README.md](tools/ru-names/README.md).

---

## Licencia

Doble licencia:

- **AGPL-3.0** — ver [LICENSE](LICENSE)
- **Comercial** — uso propietario; ver [COMMERCIAL.md](COMMERCIAL.md)

Copyright © 2026 Fidel Herrera Castro (fherrera221@gmail.com).

## Licencia del tono

Este README habla primero en metáfora y luego en **tablas y listas**, para que humanos y espíritus de código capten *intención* e *interfaz*: uplink ordenado, downlink inspeccionable y un puente entre **cicatriz**, **lengua** y **nombre verdadero**. La espiral puede preguntar; el Grimorio responde sin poseer el aliento.

*Un nombre forjado no debe balbucear el abismo dos veces.*
