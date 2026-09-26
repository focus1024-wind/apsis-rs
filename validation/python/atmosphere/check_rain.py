"""Cross-check the ApsisRS rain coefficients against ITU-R P.838-3.

Reads the values written by ``cargo run -p apsis-rs --features rf --example
dump_rain`` and recomputes each one with the ``itur`` package, which
implements the recommendation directly.

The script needs no network access.

Usage:

    pixi run python validation/python/atmosphere/check_rain.py [path]

Exit status is zero when every value agrees within tolerance and one
otherwise.
"""

from __future__ import annotations

import csv
import sys
from pathlib import Path

from itur.models.itu838 import _ITU838_3_

# The implementation reproduces the reference to about twelve significant
# digits. The Rust output is written with twelve decimal places, which limits
# the comparison for the smallest coefficients, so the tolerance is set to
# one part in ten to the eighth.
RELATIVE_TOLERANCE = 1e-8

DEFAULT_INPUT = Path("validation/data/expected/rust_rain.csv")


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
            "--example dump_rain > " + str(path),
            file=sys.stderr,
        )
        return 2

    rows = read_rows(path)
    if not rows:
        print("input file has no data rows", file=sys.stderr)
        return 2

    model = _ITU838_3_()
    worst_k = 0.0
    worst_alpha = 0.0
    worst_at = None

    for row in rows:
        frequency = float(row["frequency_ghz"])
        elevation = float(row["elevation_deg"])
        polarisation = float(row["polarisation_deg"])
        k, alpha = model.rain_specific_attenuation_coefficients(
            frequency, elevation, polarisation
        )
        k = float(k)
        alpha = float(alpha)

        k_error = abs(float(row["k"]) - k) / k
        alpha_error = abs(float(row["alpha"]) - alpha) / abs(alpha)
        if k_error > worst_k or alpha_error > worst_alpha:
            worst_at = (frequency, elevation, polarisation)
            worst_k = max(worst_k, k_error)
            worst_alpha = max(worst_alpha, alpha_error)

    print(f"combinations compared: {len(rows)}")
    print(f"worst relative k:      {worst_k:.3e}")
    print(f"worst relative alpha:  {worst_alpha:.3e}")
    print(f"worst combination:     {worst_at}")
    print(f"tolerance:             {RELATIVE_TOLERANCE:.3e} relative")

    if worst_k <= RELATIVE_TOLERANCE and worst_alpha <= RELATIVE_TOLERANCE:
        print("result:                pass")
        return 0
    print("result:                FAIL")
    return 1


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
