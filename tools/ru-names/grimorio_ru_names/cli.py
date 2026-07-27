"""CLI entry point: `grimorio-ru-names`."""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

from . import __version__
from .apply import apply_to_vocab
from .emit import emit_json, emit_toml_array
from .names import all_nominatives, build_catalog
from .validate import validate_file


def build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(
        prog="grimorio-ru-names",
        description="Russian mythic proper-name catalog & declension for Grimorio locales",
    )
    p.add_argument("--version", action="version", version=f"%(prog)s {__version__}")
    sub = p.add_subparsers(dest="cmd", required=True)

    sub.add_parser("list", help="Print nominative lemmas (one per line)")

    gen = sub.add_parser("generate", help="Emit full catalog (JSON or TOML array)")
    gen.add_argument(
        "--format",
        choices=("json", "toml"),
        default="json",
        help="Output format (default: json)",
    )
    gen.add_argument(
        "-o",
        "--output",
        type=Path,
        help="Write to file instead of stdout",
    )

    apply_p = sub.add_parser(
        "apply",
        help="Patch ru_vocab.toml [epithet].proper_names",
    )
    apply_p.add_argument(
        "--target",
        type=Path,
        required=True,
        help="Path to ru_vocab.toml (or any file with [epithet] proper_names)",
    )

    val = sub.add_parser("validate", help="Validate a grimorio.proper_names/v1 JSON file")
    val.add_argument("--file", type=Path, required=True)

    return p


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)

    if args.cmd == "list":
        for name in all_nominatives():
            print(name)
        return 0

    if args.cmd == "generate":
        text = emit_json() if args.format == "json" else emit_toml_array()
        if args.output:
            args.output.write_text(text, encoding="utf-8")
            print(f"Wrote {args.output} ({len(build_catalog())} names)", file=sys.stderr)
        else:
            sys.stdout.write(text)
        return 0

    if args.cmd == "apply":
        count = apply_to_vocab(args.target)
        print(f"Updated {args.target} ({count} names)")
        return 0

    if args.cmd == "validate":
        errors = validate_file(args.file)
        if errors:
            for err in errors:
                print(f"error: {err}", file=sys.stderr)
            return 1
        print(f"OK {args.file}")
        return 0

    return 2


if __name__ == "__main__":
    raise SystemExit(main())
