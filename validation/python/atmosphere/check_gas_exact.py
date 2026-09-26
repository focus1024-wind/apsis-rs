"""Cross-check the ApsisRS exact slant path against ITU-R P.676-12.

Reads the values written by ``cargo run -p apsis-rs --features rf --example
dump_gas_exact`` and recomputes each one with the ``itur`` package, which
implements the exact method of the recommendation directly. The exact method
integrates the specific attenuation through a layered standard atmosphere and
traces the refracted ray, so it is the form to use at low elevation.

The script needs no network access.

Usage:

    pixi run python validation/python/atmosphere/check_gas_exact.py [path]

Exit status is zero when every value agrees within tolerance and one
otherwise.
"""

from __future__ import annotations

import csv
import sys
from pathlib import Path

from itur.models.itu676 import _ITU676_12_

# The reference atmosphere, matching the Rust side.
PRESSURE_HPA = 1013.25
WATER_VAPOUR_DENSITY = 7.5
TEMPERATURE_K = 288.15

# The implementation reproduces the reference to about nine significant
# digits, so this allows for the rounding of the recorded values.
RELATIVE_TOLERANCE = 1e-7

DEFAULT_INPUT = Path("validation/data/expected/rust_gas_exact.csv")


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
            "--example dump_gas_exact > " + str(path),
            file=sys.stderr,
        )
        return 2

    rows = read_rows(path)
    if not rows:
        print("input file has no data rows", file=sys.stderr)
        return 2

    model = _ITU676_12_()
    worst = 0.0
    worst_at = None

    for row in rows:
        frequency = float(row["frequency_ghz"])
        elevation = float(row["elevation_deg"])
        reference = model.gaseous_attenuation_slant_path(
            frequency, elevation, WATER_VAPOUR_DENSITY, PRESSURE_HPA, TEMPERATURE_K,
            mode="exact",
        )
        reference = float(reference.value if hasattr(reference, "value") else reference)

        actual = float(row["attenuation_db"])
        relative = abs(actual - reference) / max(abs(reference), 1e-12)
        if relative > worst:
            worst = relative
            worst_at = (frequency, elevation)

    print(f"scenarios compared: {len(rows)}")
    print(f"worst relative:     {worst:.3e} at {worst_at}")
    print(f"tolerance:          {RELATIVE_TOLERANCE:.3e} relative")

    if worst <= RELATIVE_TOLERANCE:
        print("result:             pass")
        return 0
    print("result:             FAIL")
    return 1


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
