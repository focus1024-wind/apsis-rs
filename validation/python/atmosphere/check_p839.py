"""Cross-check the ApsisRS rain height against ITU-R P.839-4.

Reads the values written by ``cargo run -p apsis-rs --features rf --example
dump_p839`` and recomputes each one with the ``itur`` package, which
implements the recommendation directly.

The reference implementation returns a not-a-number value outside the region
its bilinear interpolator covers, which happens near the poles and at the
seam. Those points are reported separately rather than counted as failures,
because the tabulated grid does cover them and the Rust model interpolates
from it.

The script needs no network access.

Usage:

    pixi run python validation/python/atmosphere/check_p839.py [path]

Exit status is zero when every compared value agrees within tolerance and one
otherwise.
"""

from __future__ import annotations

import csv
import math
import sys
from pathlib import Path

import numpy as np

from itur.models.itu839 import _ITU839_4_

# The table is stored in metres, so the interpolation reproduces the
# tabulated values to that resolution.
TOLERANCE_KM = 1e-9

DEFAULT_INPUT = Path("validation/data/expected/rust_p839.csv")


def read_rows(path: Path) -> list[dict[str, str]]:
    """Reads the CSV rows."""
    with path.open(newline="") as handle:
        reader = csv.DictReader(handle)
        return [row for row in reader if row.get("latitude_deg")]


def main(argv: list[str]) -> int:
    path = Path(argv[1]) if len(argv) > 1 else DEFAULT_INPUT
    if not path.exists():
        print(f"missing input file: {path}", file=sys.stderr)
        print(
            "generate it with: cargo run -p apsis-rs --features rf "
            "--example dump_p839 > " + str(path),
            file=sys.stderr,
        )
        return 2

    rows = read_rows(path)
    if not rows:
        print("input file has no data rows", file=sys.stderr)
        return 2

    model = _ITU839_4_()
    worst = 0.0
    worst_at = None
    compared = 0
    outside = 0

    for row in rows:
        latitude = float(row["latitude_deg"])
        longitude = float(row["longitude_deg"])
        reference = float(
            np.asarray(
                model.isoterm_0(
                    np.array([[latitude]]), np.array([[longitude]])
                )
            ).ravel()[0]
        )
        if math.isnan(reference):
            # The reference interpolator does not cover this point.
            outside += 1
            continue

        actual = float(row["isotherm_km"])
        error = abs(actual - reference)
        if error > worst:
            worst = error
            worst_at = (latitude, longitude)
        compared += 1

    print(f"points compared:      {compared}")
    print(f"outside the reference: {outside}")
    print(f"worst difference:     {worst:.3e} km at {worst_at}")
    print(f"tolerance:            {TOLERANCE_KM:.3e} km")

    if compared > 0 and worst <= TOLERANCE_KM:
        print("result:               pass")
        return 0
    print("result:               FAIL")
    return 1


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
