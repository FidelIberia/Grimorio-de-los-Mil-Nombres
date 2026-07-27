"""Patch Grimorio vocab files (`crates/grimorio/src/locales/ru_vocab.toml`)."""

from __future__ import annotations

import re
from pathlib import Path

from .emit import emit_vocab_toml_block
from .names import build_catalog


EPITHET_PROPER_NAMES_RE = re.compile(
    r"\[epithet\]\nproper_names = \[[\s\S]*?\]\n?",
)


def apply_to_vocab(target: Path) -> int:
    """Replace `[epithet] proper_names = [...]` in *target*. Returns name count."""
    path = Path(target)
    text = path.read_text(encoding="utf-8")
    catalog = build_catalog()
    replacement = emit_vocab_toml_block(catalog)
    new_text, n = EPITHET_PROPER_NAMES_RE.subn(replacement, text, count=1)
    if n != 1:
        raise SystemExit(
            f"proper_names block not found in {path} (replacements={n})"
        )
    path.write_text(new_text, encoding="utf-8")
    return len(catalog)
