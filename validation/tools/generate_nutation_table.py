#!/usr/bin/env python3
"""Generate the Rust source for the IAU 2000A nutation series.

The series is the MHB2000 model. It has a luni-solar part and a planetary
part, both long tables of integer argument multipliers and real
coefficients, so they are transcribed mechanically from the reference
rather than by hand.

The reference is the ERFA library, a derivative of the IAU SOFA library
maintained by the International Astronomical Union. Its source is the
authoritative statement of the series.

Field order is the one the reference uses, which is not the order the
coefficients are listed in the paper:

  luni-solar: multipliers l, l', F, D, Om; then sp, spt, cp, ce, cet, se
  planetary:  multipliers l, F, D, Om, Me, Ve, E, Ma, Ju, Sa, Ur, Ne, pa;
              then sp, cp, se, ce

The two series use different argument sets and different coefficient
groupings for the obliquity, so the generated Rust types keep them separate.

Fetch the reference with:

    curl -O https://raw.githubusercontent.com/liberfa/erfa/master/src/nut00a.c

Usage:

    python validation/tools/generate_nutation_table.py path/to/nut00a.c
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


def parse_terms(body: str) -> list[list[str]]:
    """Returns the numeric terms of an array of brace groups."""
    body = re.sub(r"/\*.*?\*/", "", body, flags=re.S)
    terms = []
    for group in re.findall(r"\{([^{}]*)\}", body):
        values = group.replace("\n", " ").strip()
        if values:
            terms.append([value.strip() for value in values.split(",")])
    return terms


def main(argv: list[str]) -> int:
    if len(argv) < 2:
        print(__doc__, file=sys.stderr)
        return 2
    source = Path(argv[1]).read_text()

    lunisolar = parse_terms(extract_array(source, "xls"))
    planetary = parse_terms(extract_array(source, "xpl"))

    if any(len(term) != 11 for term in lunisolar):
        print("unexpected luni-solar row width", file=sys.stderr)
        return 1
    if any(len(term) != 17 for term in planetary):
        print("unexpected planetary row width", file=sys.stderr)
        return 1

    print("//! The IAU 2000A nutation series.")
    print("//!")
    print("//! The table is generated from the reference implementation of the")
    print("//! model by `validation/tools/generate_nutation_table.py` and is")
    print("//! reproduced without change.")
    print("//!")
    print("//! The field order follows the reference, which is not the order the")
    print("//! coefficients are listed in the paper.")
    print("//!")
    print("//! Source: the MHB2000 model of Mathews, Herring and Buffett, adopted by")
    print("//! the International Astronomical Union in 2000 and distributed as part")
    print("//! of the IAU SOFA library, as implemented by ERFA.")
    print()
    print("use super::{PlanetaryTerm, SeriesTerm};")
    print()

    print("/// The luni-solar terms of the series.")
    print("///")
    print("/// The multipliers apply to the arguments `l`, `l'`, `F`, `D`, and `Om`")
    print("/// in that order. The coefficients are the longitude sine, its time")
    print("/// derivative, the longitude cosine, the obliquity cosine, its time")
    print("/// derivative, and the obliquity sine, in units of 0.1 microarcsecond.")
    print(f"pub static LUNI_SOLAR: [SeriesTerm; {len(lunisolar)}] = [")
    for term in lunisolar:
        multipliers = ", ".join(term[:5])
        coefficients = ", ".join(
            value if "." in value or "e" in value.lower() else value + ".0"
            for value in term[5:]
        )
        print(
            f"    SeriesTerm {{ argument: [{multipliers}], "
            f"coefficients: [{coefficients}] }},"
        )
    print("];")
    print()

    print("/// The planetary terms of the series.")
    print("///")
    print("/// The multipliers apply to the arguments `l`, `F`, `D`, `Om`, the mean")
    print("/// longitudes of Mercury through Uranus, the mean longitude of Neptune,")
    print("/// and the general accumulated precession, in that order. The")
    print("/// coefficients are the longitude sine, the longitude cosine, the")
    print("/// obliquity sine, and the obliquity cosine, in units of 0.1")
    print("/// microarcsecond.")
    print(f"pub static PLANETARY: [PlanetaryTerm; {len(planetary)}] = [")
    for term in planetary:
        multipliers = ", ".join(term[:13])
        coefficients = ", ".join(
            value if "." in value or "e" in value.lower() else value + ".0"
            for value in term[13:]
        )
        print(
            f"    PlanetaryTerm {{ argument: [{multipliers}], "
            f"coefficients: [{coefficients}] }},"
        )
    print("];")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
