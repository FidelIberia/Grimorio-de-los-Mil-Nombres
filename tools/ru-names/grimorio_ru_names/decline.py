"""Russian literary declension for mythic nominatives."""

from __future__ import annotations


def decline(
    name: str,
    override: dict[str, tuple[str, str]] | None = None,
) -> tuple[str, str]:
    """Return `(patron_genitive, verbal_instrumental)` for *name*."""
    if override and name in override:
        return override[name]
    if "-" in name:
        return name, name
    if name.endswith("ия"):
        stem = name[:-2]
        return f"{stem}ии", f"{stem}ией"
    if name.endswith("а"):
        stem = name[:-1]
        return f"{stem}ы", f"{stem}ой"
    if name.endswith("я"):
        return f"{name[:-1]}и", f"{name[:-1]}ей"
    if name.endswith("ь"):
        return f"{name[:-1]}я", f"{name[:-1]}ем"
    if name.endswith("й"):
        return f"{name[:-1]}я", f"{name[:-1]}ем"
    if name.endswith("о"):
        return f"{name[:-1]}а", f"{name[:-1]}ом"
    if name.endswith(("р", "н", "с")):
        return f"{name}а", f"{name}ом"
    return f"{name}а", f"{name}ом"
