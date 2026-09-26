"""Cross-check the ApsisRS IAU 2000/2006 Earth orientation against ERFA.

Reads the quantities written by ``cargo run -p apsis-rs --features physics
--example dump_iau2000`` and recomputes each one with ERFA, which is the
reference implementation distributed by the International Astronomical Union
through the SOFA library and its ERFA derivative.

The script needs no network access.

Usage:

    pixi run python validation/python/time/check_iau2000.py [path]

Exit status is zero when every quantity agrees within tolerance and one
otherwise.
"""

from __future__ import annotations

import csv
import sys
from pathlib import Path

import erfa

# Tolerances.
#
# The nutation, obliquity, and precession are reproduced at the level of the
# last bits of a double, so they are compared at 1e-14.
#
# The sidereal time and the matrices accumulate the double precision error of
# several rotations, so they are allowed more.
#
# Greenwich apparent sidereal time gets its own tolerance because ERFA's
# `gst06a` is built on the non-rotating origin while the Rust value is built
# on the equinox. The two formulations agree to about two microarcseconds,
# which is 1e-11 radians, and that is what is allowed here.
SCALAR_TOLERANCE = 1e-14
SIDEREAL_TOLERANCE = 1e-10
MATRIX_TOLERANCE = 1e-9


def tolerance_for(quantity: str) -> float:
    """Returns the tolerance that applies to a quantity."""
    if quantity == "gast":
        return SIDEREAL_TOLERANCE
    if quantity in ("bpn", "c2t"):
        return MATRIX_TOLERANCE
    return SCALAR_TOLERANCE

DEFAULT_INPUT = Path("validation/data/expected/rust_iau2000.csv")

# The Terrestrial Time offset from UTC, in seconds, matching the Rust side.
TT_MINUS_UTC_SECONDS = 32.184 + 37.0


def read_rows(path: Path) -> list[dict[str, str]]:
    """Reads the CSV rows."""
    with path.open(newline="") as handle:
        reader = csv.DictReader(handle)
        return [row for row in reader if row.get("mjd")]


def tt_julian_date(utc_mjd: float) -> float:
    """Returns the Terrestrial Time Julian date for a UTC Modified Julian Date."""
    return utc_mjd + 2400000.5 + TT_MINUS_UTC_SECONDS / 86400.0


def main(argv: list[str]) -> int:
    path = Path(argv[1]) if len(argv) > 1 else DEFAULT_INPUT
    if not path.exists():
        print(f"missing input file: {path}", file=sys.stderr)
        print(
            "generate it with: cargo run -p apsis-rs --features physics "
            "--example dump_iau2000 > " + str(path),
            file=sys.stderr,
        )
        return 2

    rows = read_rows(path)
    if not rows:
        print("input file has no data rows", file=sys.stderr)
        return 2

    worst: dict[str, float] = {}
    worst_at: dict[str, object] = {}
    compared = 0

    for row in rows:
        mjd = float(row["mjd"])
        quantity = row["quantity"]
        index = int(row["index"])
        actual = float(row["value"])

        tt = tt_julian_date(mjd)
        d1 = 2400000.5
        d2 = tt - d1

        if quantity == "obliquity":
            reference = erfa.obl06(d1, d2)
        elif quantity == "nutation_longitude_a":
            reference = erfa.nut00a(d1, d2)[0]
        elif quantity == "nutation_obliquity_a":
            reference = erfa.nut00a(d1, d2)[1]
        elif quantity == "nutation_longitude_06a":
            reference = erfa.nut06a(d1, d2)[0]
        elif quantity == "nutation_obliquity_06a":
            reference = erfa.nut06a(d1, d2)[1]
        elif quantity == "precession_gamma_bar":
            reference = erfa.pfw06(d1, d2)[0]
        elif quantity == "precession_phi_bar":
            reference = erfa.pfw06(d1, d2)[1]
        elif quantity == "precession_psi_bar":
            reference = erfa.pfw06(d1, d2)[2]
        elif quantity == "precession_epsilon":
            reference = erfa.pfw06(d1, d2)[3]
        elif quantity == "bpn":
            matrix = erfa.pnm06a(d1, d2)
            reference = matrix[index // 3][index % 3]
        elif quantity == "gast":
            reference = erfa.gst06a(d1, d2, d1, d2)
        elif quantity == "c2t":
            # The equinox based chain: the bias, precession and nutation,
            # then Greenwich apparent sidereal time, then the polar motion.
            matrix = erfa.c2t06a(d1, d2, d1, d2, 0.0, 0.0)
            reference = matrix[index // 3][index % 3]
        else:
            continue

        error = abs(actual - reference)
        if error > worst.get(quantity, 0.0):
            worst[quantity] = error
            worst_at[quantity] = (mjd, index)
        compared += 1

    print(f"values compared: {compared}")
    for quantity in sorted(worst):
        tolerance = tolerance_for(quantity)
        status = "ok" if worst[quantity] <= tolerance else "FAIL"
        print(
            f"  {quantity:26} worst {worst[quantity]:.3e} "
            f"(tolerance {tolerance:.0e}) at {worst_at[quantity]} {status}"
        )

    passed = all(
        value <= tolerance_for(quantity) for quantity, value in worst.items()
    )
    if passed and compared > 0:
        print("result: pass")
        return 0
    print("result: FAIL")
    return 1


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
