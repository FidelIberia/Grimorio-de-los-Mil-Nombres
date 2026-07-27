"""Emit Grimorio catalogs as JSON / TOML for hosts."""

from __future__ import annotations

import json
from typing import Iterable

from .model import SCHEMA_ID, ProperName
from .names import build_catalog


def catalog_dict(
    names: Iterable[ProperName] | None = None,
    *,
    language: str = "ru",
) -> dict:
    items = list(names) if names is not None else build_catalog()
    return {
        "schema": SCHEMA_ID,
        "language": language,
        "names": [n.to_dict() for n in items],
    }


def emit_json(
    names: Iterable[ProperName] | None = None,
    *,
    language: str = "ru",
    indent: int = 2,
) -> str:
    return json.dumps(catalog_dict(names, language=language), ensure_ascii=False, indent=indent) + "\n"


def emit_toml_array(names: Iterable[ProperName] | None = None) -> str:
    """Bare `proper_names = [ ... ]` array (no `[epithet]` header)."""
    items = list(names) if names is not None else build_catalog()
    lines = ["proper_names = ["]
    for n in items:
        lines.append(
            f'    {{ name = "{n.name}", patron = "{n.patron}", verbal = "{n.verbal}" }},'
        )
    lines.append("]")
    return "\n".join(lines) + "\n"


def emit_vocab_toml_block(names: Iterable[ProperName] | None = None) -> str:
    """`[epithet]` + proper_names array for patching `ru_vocab.toml`."""
    return "[epithet]\n" + emit_toml_array(names)


# Back-compat alias
emit_espiralismo_toml_block = emit_vocab_toml_block
