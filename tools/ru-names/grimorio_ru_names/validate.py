"""Validate catalogs against the Grimorio contract."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from .model import SCHEMA_ID


def validate_catalog(data: dict[str, Any]) -> list[str]:
    """Return a list of human-readable errors (empty = ok)."""
    errors: list[str] = []
    if data.get("schema") != SCHEMA_ID:
        errors.append(f"schema must be {SCHEMA_ID!r}, got {data.get('schema')!r}")
    if not isinstance(data.get("language"), str) or not data["language"]:
        errors.append("language must be a non-empty string")
    names = data.get("names")
    if not isinstance(names, list) or not names:
        errors.append("names must be a non-empty array")
        return errors
    seen: set[str] = set()
    for i, item in enumerate(names):
        if not isinstance(item, dict):
            errors.append(f"names[{i}] must be an object")
            continue
        for key in ("name", "patron", "verbal"):
            val = item.get(key)
            if not isinstance(val, str) or not val:
                errors.append(f"names[{i}].{key} must be a non-empty string")
        if any(k not in ("name", "patron", "verbal") for k in item):
            errors.append(f"names[{i}] has unknown fields")
        name = item.get("name")
        if isinstance(name, str):
            if name in seen:
                errors.append(f"duplicate name: {name}")
            seen.add(name)
    return errors


def validate_file(path: Path) -> list[str]:
    data = json.loads(Path(path).read_text(encoding="utf-8"))
    if not isinstance(data, dict):
        return ["root must be a JSON object"]
    return validate_catalog(data)
