"""Cross-check the ApsisRS two-line element parser against an independent reader.

Reads the element fields written by ``cargo run -p apsis-rs --features orbit
--example dump_tle`` and reparses the same element sets with a column indexed
reader written from the NORAD format, then compares every field. The
checksum of each line is recomputed from its definition, so a field placed in
the wrong column or a sign convention that was dropped shows up as a
disagreement.

The format is positional, with the first character of a line at column zero:

* line 1: satellite number in columns 2 to 6, classification in column 7,
  international designator in columns 9 to 16, epoch year and fraction of a
  day in columns 18 to 32, the first derivative of the mean motion divided
  by two in columns 33 to 43, the second derivative divided by six in columns
  44 to 52, the drag term in columns 53 to 61, the element set number in
  columns 62 to 67, and the checksum in column 68;
* line 2: inclination in columns 8 to 16, right ascension in columns 17 to
  25, eccentricity in columns 26 to 33 as an implied leading decimal,
  argument of perigee in columns 34 to 42, mean anomaly in columns 43 to 51,
  mean motion in columns 52 to 63, the revolution number in columns 63 to 68,
  and the checksum in column 68.

A dash counts as one and every digit counts as its own value, summed over the
first sixty eight columns and reduced modulo ten.

The script needs no network access.

Usage:

    pixi run python validation/python/orbit/check_tle.py [path]

Exit status is zero when every field agrees and one otherwise.
"""

from __future__ import annotations

import csv
import sys
from pathlib import Path

# The reader works on the same characters the parser saw, so the two must
# agree to the last digit. The tolerance is therefore exact, not
# approximate, and only guards the exponent fields against a difference in
# how the two languages round a very small value.
ABSOLUTE_TOLERANCE = 1e-12
EXPONENT_TOLERANCE = 1e-18

DEFAULT_INPUT = Path("validation/data/expected/rust_tle.csv")

# The element sets, in the order the example writes them. The lines are the
# same ones the example parses.
ELEMENT_SETS = {
    "vanguard_1": (
        "1 00005U 58002B   00179.78495062  .00000023  00000-0  28098-4 0  4753",
        "2 00005  34.2682 348.7242 1859667 331.7664  19.3264 10.82419157413667",
    ),
    "iss_2008": (
        "1 25544U 98067A   08264.51782528 -.00002182  00000-0 -11606-4 0  2927",
        "2 25544  51.6416 247.4627 0006703 130.5360 325.0288 15.72125391563537",
    ),
    "teme_94_deep": (
        "1 27651U 03004A   06185.99374200  .00000109  00000-0  19909-4 0   445",
        "2 27651   7.1743 182.9443 7867251 269.0386  18.6601  1.03461646  9884",
    ),
    "lrs_4_deep": (
        "1 09880U 77021A   06174.85710247  .00000090  00000-0  10000-3 0  7322",
        "2 09880  64.1526 313.1296 6546925 103.4683 305.3980  2.00754276 13047",
    ),
    "circular_implied_zero": (
        "1 20500U 90001A   90001.00000000  .00000000  00000-0  00000-0 0    00",
        "2 20500  82.0000 100.0000 0000000  90.0000  90.0000 14.90000000    13",
    ),
    "retrograde": (
        "1 20600U 90002B   12345.50000000 -.00001234  00000-0  12345-6 0  9991",
        "2 20600 120.0000 200.0000 0010000  30.0000 330.0000 15.00000000 12341",
    ),
    "two_digit_year_1999": (
        "1 10000U 99001A   99365.00000000  .00000000  00000-0  00000-0 0    05",
        "2 10000  45.0000  10.0000 0001000   0.0000   0.0000 14.00000000    09",
    ),
    "with_name_line": (
        "1 12345U 80001C   20001.00000000  .00010000  00000-0  00000-0 0  9998",
        "2 12345  60.0000  30.0000 0002000 100.0000 200.0000 12.00000000 12344",
    ),
}


def checksum(line: str) -> int:
    """Returns the modulo ten checksum of a line from its definition."""
    total = 0
    for character in line[:68]:
        if character == "-":
            total += 1
        elif character.isdigit():
            total += int(character)
    return total % 10


def field(line: str, start: int, end: int) -> str:
    """Returns a slice of a line."""
    return line[start:end]


def parse_exponential(text: str) -> float:
    """Reads a NORAD assumed decimal point exponential.

    The second derivative of the mean motion and the drag term are stored
    with the decimal point assumed before the first digit and the exponent
    written without an ``E``: ``-11606-4`` is ``-0.11606e-4``. A leading
    ``+`` or ``-`` before the mantissa is the sign of the value, and the sign
    before the exponent is the sign of the exponent.
    """
    text = text.strip()
    if not text:
        return 0.0
    # The exponent is introduced by the last sign, once a mantissa digit is
    # in hand, so scan from the end.
    index = len(text) - 1
    while index > 0 and text[index] not in "+-":
        index -= 1
    mantissa = text[:index].strip()
    exponent = text[index:]
    if mantissa.startswith("-"):
        sign = -1.0
        mantissa = mantissa[1:]
    elif mantissa.startswith("+"):
        sign = 1.0
        mantissa = mantissa[1:]
    else:
        sign = 1.0
    if not mantissa:
        mantissa = "0"
    if not exponent:
        return 0.0
    return sign * float("0." + mantissa) * 10.0 ** int(exponent)


def parse_line_one(line: str) -> dict:
    """Parses the first line by column index."""
    epoch = field(line, 18, 32).strip()
    year = int(epoch[:2])
    fraction = float(epoch[2:])
    # The two digit year expands with the catalogue convention: 57 to 99 is
    # the twentieth century and 00 to 56 the twenty first.
    century = 1900 if year >= 57 else 2000
    return {
        "satellite_number": int(field(line, 2, 7)),
        "classification": field(line, 7, 8).strip(),
        "international_designator": field(line, 9, 17).strip(),
        "epoch_year": century + year,
        "epoch_fraction": fraction,
        "mean_motion_dot": float(field(line, 33, 43)),
        "mean_motion_double_dot": parse_exponential(field(line, 44, 52)),
        "bstar": parse_exponential(field(line, 53, 61)),
        "element_set_number": int(field(line, 65, 68)),
        "checksum": checksum(line),
    }


def parse_line_two(line: str) -> dict:
    """Parses the second line by column index."""
    # The eccentricity is stored as a leading decimal: the column holds the
    # fractional digits and the decimal point is implied before the first.
    eccentricity = float("0." + field(line, 26, 33).strip())
    return {
        "inclination_deg": float(field(line, 8, 16)),
        "right_ascension_deg": float(field(line, 17, 25)),
        "eccentricity": eccentricity,
        "argument_of_perigee_deg": float(field(line, 34, 42)),
        "mean_anomaly_deg": float(field(line, 43, 51)),
        "mean_motion": float(field(line, 52, 63)),
        "revolution_number": int(field(line, 63, 68)),
        "checksum": checksum(line),
    }


def main(argv: list[str]) -> int:
    path = Path(argv[1]) if len(argv) > 1 else DEFAULT_INPUT
    if not path.exists():
        print(f"missing input file: {path}", file=sys.stderr)
        print(
            "generate it with: cargo run -p apsis-rs --features orbit "
            "--example dump_tle > " + str(path),
            file=sys.stderr,
        )
        return 2

    with path.open(newline="") as handle:
        rows = list(csv.DictReader(handle))
    if not rows:
        print("input file has no data rows", file=sys.stderr)
        return 2

    failures: list[str] = []
    compared = 0

    for row in rows:
        name = row["name"]
        if name not in ELEMENT_SETS:
            failures.append(f"{name}: not in the reference set")
            continue
        first, second = ELEMENT_SETS[name]
        one = parse_line_one(first)
        two = parse_line_two(second)

        def check(field_name: str, expected, actual, tolerance=ABSOLUTE_TOLERANCE) -> None:
            if isinstance(expected, float):
                if abs(expected - actual) > tolerance:
                    failures.append(f"{name}.{field_name}: {actual} against {expected}")
            elif expected != actual:
                failures.append(f"{name}.{field_name}: {actual!r} against {expected!r}")

        check("satellite_number", one["satellite_number"], int(row["satellite_number"]))
        check("classification", one["classification"], row["classification"])
        check(
            "international_designator",
            one["international_designator"],
            row["international_designator"],
        )
        check("epoch_year", one["epoch_year"], int(row["epoch_year"]))
        check(
            "mean_motion_dot",
            one["mean_motion_dot"],
            float(row["e_mean_motion_dot"]),
            EXPONENT_TOLERANCE,
        )
        check(
            "mean_motion_double_dot",
            one["mean_motion_double_dot"],
            float(row["e_mean_motion_double_dot"]),
            EXPONENT_TOLERANCE,
        )
        check("bstar", one["bstar"], float(row["bstar"]), EXPONENT_TOLERANCE)
        check(
            "element_set_number",
            one["element_set_number"],
            int(row["element_set_number"]),
        )
        check("inclination_deg", two["inclination_deg"], float(row["inclination_deg"]))
        check(
            "right_ascension_deg",
            two["right_ascension_deg"],
            float(row["right_ascension_deg"]),
        )
        check("eccentricity", two["eccentricity"], float(row["eccentricity"]))
        check(
            "argument_of_perigee_deg",
            two["argument_of_perigee_deg"],
            float(row["argument_of_perigee_deg"]),
        )
        check("mean_anomaly_deg", two["mean_anomaly_deg"], float(row["mean_anomaly_deg"]))
        check("mean_motion", two["mean_motion"], float(row["mean_motion"]))
        check(
            "revolution_number",
            two["revolution_number"],
            int(row["revolution_number"]),
        )
        check("line1_checksum", one["checksum"], int(row["line1_checksum"]))
        check("line2_checksum", two["checksum"], int(row["line2_checksum"]))
        compared += 1

    print(f"element sets compared: {compared}")
    print(f"fields per set:        17")
    for failure in failures:
        print(f"  {failure}")
    if failures:
        print("result:                FAIL")
        return 1
    print("result:                pass")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
