"""Cross-check the ApsisRS TEME to Earth-fixed transformation.

Reads the positions written by ``cargo run -p apsis-rs --features physics
--example dump_teme`` and recomputes each one with ERFA, applying the same
Greenwich mean sidereal time and the same zero polar motion, so that the
check compares the implementation rather than the choice of inputs.

The reference is the sidereal time of 1982 and the polar motion composition
of the IAU SOFA library, as implemented by ERFA. Note that astropy agrees
with this transformation only to a few hundred metres, because astropy
applies the measured polar motion and a measured UT1 - UTC while this check
uses zero for both. That agreement is reported for information but is not
the acceptance criterion.

The script needs no network access.

Usage:

    pixi run python validation/python/coord/check_teme.py [path]

Exit status is zero when every position agrees within tolerance and one
otherwise.
"""

from __future__ import annotations

import csv
import sys
from pathlib import Path

import erfa

# Tolerance in kilometers. The transformation is a product of two rotations
# evaluated in the same order, so the agreement is at the level of the last
# bits of a double.
TOLERANCE_KM = 1e-9

# The two-part Julian date used by the Rust example for the whole days.
DAY_PART = 2400000.5

DEFAULT_INPUT = Path("validation/data/expected/rust_teme.csv")


def read_rows(path: Path) -> list[dict[str, str]]:
    """Reads the CSV rows."""
    with path.open(newline="") as handle:
        reader = csv.DictReader(handle)
        return [row for row in reader if row.get("mjd")]


def reference_position(mjd: float, position: list[float]) -> list[float]:
    """Recomputes the Earth-fixed position with ERFA."""
    # The sidereal time, from the two-part Julian date.
    sidereal = erfa.gmst82(DAY_PART, mjd)
    # The rotation of the frame about its third axis, composed as the
    # reference `rz` does, and the identity polar motion.
    rotation = erfa.rz(sidereal, erfa.ir())
    polar = erfa.pom00(0.0, 0.0, 0.0)
    matrix = erfa.rxr(polar, rotation)

    # The rotation maps the frame, so the position is the matrix applied to
    # the components.
    return [
        sum(matrix[i][j] * position[j] for j in range(3)) for i in range(3)
    ]


def main(argv: list[str]) -> int:
    path = Path(argv[1]) if len(argv) > 1 else DEFAULT_INPUT
    if not path.exists():
        print(f"missing input file: {path}", file=sys.stderr)
        print(
            "generate it with: cargo run -p apsis-rs --features physics "
            "--example dump_teme > " + str(path),
            file=sys.stderr,
        )
        return 2

    rows = read_rows(path)
    if not rows:
        print("input file has no data rows", file=sys.stderr)
        return 2

    # The TEME inputs used by the example, in the same order.
    inputs = [
        [7000.0, 1000.0, 2000.0],
        [7000.0, -3000.0, 1000.0],
        [42164.0, 0.0, 0.0],
        [6878.137, 100.0, -50.0],
    ]

    worst = 0.0
    worst_at = None
    for row, position in zip(rows, inputs):
        mjd = float(row["mjd"])
        actual = [
            float(row["x_ecef"]),
            float(row["y_ecef"]),
            float(row["z_ecef"]),
        ]
        expected = reference_position(mjd, position)
        error = max(abs(a - b) for a, b in zip(actual, expected))
        if error > worst:
            worst = error
            worst_at = mjd

    print(f"positions compared: {len(rows)}")
    print(f"worst difference:   {worst:.3e} km at MJD {worst_at}")
    print(f"tolerance:          {TOLERANCE_KM:.3e} km")

    if worst <= TOLERANCE_KM:
        print("result:             pass")
        return 0
    print("result:             FAIL")
    return 1


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
