"""Cross-check the ApsisRS gaseous attenuation against ITU-R P.676-12.

Reads the values written by ``cargo run -p apsis-rs --features rf --example
dump_gas`` and recomputes each one with the ``itur`` package, which
implements the ITU-R recommendations directly, then reports the largest
difference in each column.

The ``itur`` package is a reference implementation of the recommendations
maintained as part of the ITU-R study programme, so it is an independent
check on the line tables and the formulae rather than a restatement of them.

The script needs no network access.

Usage:

    pixi run python validation/python/atmosphere/check_gas.py [path]

Exit status is zero when every value agrees within tolerance and one
otherwise.
"""

from __future__ import annotations

import csv
import sys
from pathlib import Path

import numpy as np
import astropy.units as u

from itur.models.itu676 import _ITU676_12_

# The reference atmosphere of the recommendation, matching the Rust side.
PRESSURE_HPA = 1013.25
WATER_VAPOUR_DENSITY = 7.5
TEMPERATURE_K = 288.15

# Tolerances.
#
# The specific attenuation spans several orders of magnitude, and the water
# vapour term at low frequencies is a small difference between larger
# contributions, so a pure relative test is too strict there. The check
# passes when either the relative difference is small or the absolute
# difference is negligible.
RELATIVE_TOLERANCE = 1e-8
ABSOLUTE_TOLERANCE = 1e-10

DEFAULT_INPUT = Path("validation/data/expected/rust_gas.csv")


def read_rows(path: Path) -> list[dict[str, str]]:
    """Reads the CSV rows."""
    with path.open(newline="") as handle:
        reader = csv.DictReader(handle)
        return [row for row in reader if row.get("frequency_ghz")]


def main(argv: list[str]) -> int:
    path = Path(argv[1]) if len(argv) > 1 else DEFAULT_INPUT
    if not path.exists():
        print(f"missing input file: {path}", file=sys.stderr)
        print(
            "generate it with: cargo run -p apsis-rs --features rf "
            "--example dump_gas > " + str(path),
            file=sys.stderr,
        )
        return 2

    rows = read_rows(path)
    if not rows:
        print("input file has no data rows", file=sys.stderr)
        return 2

    model = _ITU676_12_()

    columns = ["dry_db_per_km", "wet_db_per_km", "dry_height_km", "wet_height_km", "zenith_db"]
    worst = {column: 0.0 for column in columns}
    worst_absolute = {column: 0.0 for column in columns}
    worst_at = {column: None for column in columns}

    for row in rows:
        frequency = float(row["frequency_ghz"])
        dry = float(model.gamma0_exact(frequency, PRESSURE_HPA, WATER_VAPOUR_DENSITY, TEMPERATURE_K))
        wet = float(model.gammaw_exact(frequency, PRESSURE_HPA, WATER_VAPOUR_DENSITY, TEMPERATURE_K))
        heights = model.slant_inclined_path_equivalent_height(
            frequency, PRESSURE_HPA, WATER_VAPOUR_DENSITY, TEMPERATURE_K
        )
        dry_height = float(np.asarray(heights[0]).ravel()[0])
        wet_height = float(np.asarray(heights[1]).ravel()[0])
        zenith = dry * dry_height + wet * wet_height

        expected = {
            "dry_db_per_km": dry,
            "wet_db_per_km": wet,
            "dry_height_km": dry_height,
            "wet_height_km": wet_height,
            "zenith_db": zenith,
        }
        for column, reference in expected.items():
            actual = float(row[column])
            absolute = abs(actual - reference)
            relative = absolute / max(abs(reference), ABSOLUTE_TOLERANCE)
            # The column passes on either measure, so the reported figure is
            # the relative one and the acceptance test below also allows the
            # absolute difference to be negligible.
            if relative > worst[column]:
                worst[column] = relative
                worst_at[column] = frequency
            if absolute > worst_absolute[column]:
                worst_absolute[column] = absolute

    print(f"frequencies compared: {len(rows)}")
    for column in columns:
        print(
            f"{column:>15}: worst relative {worst[column]:.3e} "
            f"absolute {worst_absolute[column]:.3e} at {worst_at[column]} GHz"
        )
    print(
        f"{'tolerance':>15}:       {RELATIVE_TOLERANCE:.3e} relative or "
        f"{ABSOLUTE_TOLERANCE:.3e} absolute"
    )

    passes = all(
        worst[column] <= RELATIVE_TOLERANCE
        or worst_absolute[column] <= ABSOLUTE_TOLERANCE
        for column in columns
    )
    if passes:
        print("result:          pass")
        return 0
    print("result:          FAIL")
    return 1


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
