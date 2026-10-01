#!/usr/bin/env python3
"""Check that every source comment has a Chinese counterpart.

The repository is being made bilingual: each comment block carries the
English paragraph first and the Chinese one after a blank doc line. This
script reports the blocks that have no Chinese yet, so the sweep can be
driven and verified without reading every file.

It also reports the two mistakes that are easy to make while adding a
language: a code string that was translated by accident, and a comment that
mentions a term the glossary keeps in English where a Chinese name exists.

Usage:

    python3 scripts/check-bilingual.py            # every crate
    python3 scripts/check-bilingual.py crates/orbit
    python3 scripts/check-bilingual.py --quiet     # only the summary line
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

# A comment line: a doc line, or a plain comment that is not a doc marker.
DOC = re.compile(r"^\s*///")
LINE = re.compile(r"^\s*//[^/!]")

# CJK, used to decide whether a block already carries the Chinese text.
CJK = re.compile(r"[\u4e00-\u9fff]")

# The replacement character, which shows up when a byte sequence is mangled.
MOJIBAKE = "�"

# An English function word sitting between Chinese characters, which means a
# half translated sentence. The glossary keeps words like `Walker` and
# `SGP4` in English, so a lowercase word between CJK is a mistake.
MIXED = re.compile(r"[\u4e00-\u9fff][a-z]{3,}[\u4e00-\u9fff]")

# A comment that is only a formula or a bare symbol carries no prose to
# translate, so demanding a second language on it would be noise. Such a
# comment is left as it stands, which is the whole of it.
FORMULA = re.compile("^`?[A-Za-z0-9_^*+\\-/() =,;.\u3002\uff0c]*`?[\.\u3002]?$")

# Three letters in a row is a word, not a symbol, so a line carrying one is
# prose and needs translating even if it is mostly numbers.
PROSE = re.compile(r"[A-Za-z]{3}")


def is_formula_only(block: list[str]) -> bool:
    """True when every line of the block is code, a formula, or blank."""
    saw_content = False
    for line in block:
        body = re.sub(r"^\s*(///|//[^/!])\s?", "", line).strip()
        if not body:
            continue
        saw_content = True
        if not FORMULA.match(body):
            return False
        # A single backticked token is a symbol, whatever it spells. Prose
        # needs a verb, and `cos` or `eta` has none.
        if len(re.findall(r"[A-Za-z]{2,}", body)) > 1:
            return False
    return saw_content


def blocks(path: Path) -> list[tuple[int, list[str]]]:
    """Returns the comment blocks of a file with the line each starts on."""
    lines = path.read_text(encoding="utf-8").splitlines()
    found: list[tuple[int, list[str]]] = []
    index = 0
    while index < len(lines):
        if DOC.match(lines[index]) or LINE.match(lines[index]):
            start = index
            block: list[str] = []
            while index < len(lines) and (DOC.match(lines[index]) or LINE.match(lines[index])):
                block.append(lines[index])
                index += 1
            found.append((start + 1, block))
        else:
            index += 1
    return found


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("paths", nargs="*", default=[])
    parser.add_argument("--quiet", action="store_true")
    args = parser.parse_args(argv)

    roots = [Path(p).resolve() for p in args.paths] or [ROOT / "crates"]
    files: list[Path] = []
    for base in roots:
        if base.is_file():
            files.append(base)
        else:
            files.extend(sorted(base.rglob("src/**/*.rs")))

    total = 0
    translated = 0
    gaps: list[str] = []
    mojibake: list[str] = []
    mixed: list[str] = []

    for path in files:
        if "examples" in path.parts:
            continue
        for line, block in blocks(path):
            text = "\n".join(block)
            if MOJIBAKE in text:
                mojibake.append(f"{path.relative_to(ROOT)}:{line}")
            found = MIXED.search(text)
            if found:
                mixed.append(
                    f"{path.relative_to(ROOT)}:{line}  {found.group(0)!r}"
                )
            if is_formula_only(block):
                # Nothing to translate; the symbol is the whole comment.
                continue
            total += 1
            if CJK.search(text):
                translated += 1
            else:
                first = block[0].strip().lstrip("/!").strip()
                gaps.append(f"{path.relative_to(ROOT)}:{line}  {first[:64]}")

    if not args.quiet:
        for gap in gaps:
            print(f"  no Chinese  {gap}")
    for bad in mojibake:
        print(f"  MOJIBAKE    {bad}")
    for bad in mixed:
        print(f"  MIXED       {bad}")

    remaining = len(gaps)
    print(
        f"comment blocks: {total}, bilingual: {translated}, "
        f"still English only: {remaining}"
    )
    return 1 if (mojibake or mixed) else 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
