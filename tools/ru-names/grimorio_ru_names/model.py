from __future__ import annotations

from dataclasses import asdict, dataclass
from typing import Any

SCHEMA_ID = "grimorio.proper_names/v1"


@dataclass(frozen=True, slots=True)
class ProperName:
    """One mythic name with declined surfaces (contract v1)."""

    name: str
    patron: str
    verbal: str

    def to_dict(self) -> dict[str, str]:
        return asdict(self)

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> ProperName:
        return cls(
            name=str(data["name"]),
            patron=str(data["patron"]),
            verbal=str(data["verbal"]),
        )
