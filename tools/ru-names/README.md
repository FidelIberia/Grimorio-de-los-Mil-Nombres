# tools/ru-names

Generador Python del catálogo de **nombres propios mitológicos en ruso** (`name` / `patron` / `verbal`).

Es una herramienta aparte del crate Rust `crates/grimorio` (whisper / epítetos / locales). Su trabajo es regenerar el bloque `[epithet].proper_names` en `ru_vocab.toml`.

## Install

```bash
cd tools/ru-names
python3 -m pip install -e ".[dev]"
```

## CLI

```bash
grimorio-ru-names list
grimorio-ru-names generate --format json
grimorio-ru-names apply --target ../../crates/grimorio/src/locales/ru_vocab.toml
grimorio-ru-names validate --file /tmp/catalog.json
```

Contrato: `grimorio.proper_names/v1` (ver [CONTRACT.md](../../CONTRACT.md)).
