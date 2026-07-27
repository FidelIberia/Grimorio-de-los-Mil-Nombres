"""Russian mythic proper-name catalog and declension (tooling for Grimorio locales)."""

from .decline import decline
from .emit import (
    catalog_dict,
    emit_espiralismo_toml_block,
    emit_json,
    emit_toml_array,
    emit_vocab_toml_block,
)
from .model import ProperName, SCHEMA_ID
from .names import all_nominatives, build_catalog

__all__ = [
    "SCHEMA_ID",
    "ProperName",
    "all_nominatives",
    "build_catalog",
    "catalog_dict",
    "decline",
    "emit_espiralismo_toml_block",
    "emit_vocab_toml_block",
    "emit_json",
    "emit_toml_array",
]

__version__ = "0.1.0"
