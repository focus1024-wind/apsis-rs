#!/usr/bin/env python3
"""Generate the Rust source for the IAU 2000 complementary terms.

The complementary terms are the small periodic additions to the equation of
the equinoxes that the nutation in longitude does not contain. They come
from the same theory as the nutation and are needed for the equation of the
equinoxes to be consistent with the nutation series.

The reference is the ERFA library, a derivative of the IAU SOFA library.

Fetch the reference with:

    curl -O https://raw.githubusercontent.com/liberfa/erfa/master/src/eect00.c

Usage:

    python validation/tools/generate_eect_table.py path/to/eect00.c
"""

from __future__ import annotations

import re
import sys
from pathlib import Path


def extract_array(source: str, name: str) -> str:
    """Returns the body of a C array initializer."""
    start = source.index(name + "[] = {")
    brace = source.index("{", start)
    depth = 0
    index = brace
    while index < len(source):
        if source[index] == "{":
            depth += 1
        elif source[index] == "}":
            depth -= 1
            if depth == 0:
                break
        index += 1
    return source[brace : index + 1]


def parse_terms(body: str) -> list[tuple[str, str]]:
    """Returns the multiplier and coefficient groups of each term."""
    body = re.sub(r"/\*.*?\*/", "", body, flags=re.S)
    return re.findall(r"\{\{([^{}]*)\},\s*([^}]*)\}", body)


def emit(name: str, doc: str, terms: list[tuple[str, str]]) -> None:
    """Prints one Rust constant."""
    print(doc)
    print(f"pub static {name}: [ComplementaryTerm; {len(terms)}] = [")
    for multipliers, coefficients in terms:
        arguments = ", ".join(value.strip() for value in multipliers.split(","))
        values = ", ".join(value.strip() for value in coefficients.split(","))
        print(
            f"    ComplementaryTerm {{ argument: [{arguments}], "
            f"coefficients: [{values}] }},"
        )
    print("];")
    print()


def main(argv: list[str]) -> int:
    if len(argv) < 2:
        print(__doc__, file=sys.stderr)
        return 2
    source = Path(argv[1]).read_text()
    constant = parse_terms(extract_array(source, "e0"))
    linear = parse_terms(extract_array(source, "e1"))

    print("//! The IAU 2000 complementary terms of the equation of the equinoxes.")
    print("//!")
    print("//! The table is generated from the reference implementation by")
    print("//! `validation/tools/generate_eect_table.py` and is reproduced without")
    print("//! change.")
    print("//!")
    print("//! Source: the MHB2000 model, as distributed with the IAU SOFA library")
    print("//! and implemented by ERFA.")
    print()
    print("use super::ComplementaryTerm;")
    print()
    emit(
        "CONSTANT_TERMS",
        "/// The terms that do not depend on time.\n///\n/// The eight multipliers apply to the arguments `l`, `l'`, `F`, `D`, `Om`,\n/// the mean longitude of Venus, the mean longitude of the Earth, and the\n/// general accumulated precession, in that order.",
        constant,
    )
    emit(
        "LINEAR_TERMS",
        "/// The terms whose amplitude grows linearly with time.\n///\n/// The arguments are the same as for the constant terms.",
        linear,
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
