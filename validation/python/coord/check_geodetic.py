"""Cross-check the ApsisRS geodetic to Earth-fixed conversion.

Reads the grid written by ``cargo run -p apsis-rs --features physics
--example dump_geodesy`` and recomputes each position with astropy, then
reports the largest difference.

The script needs no network access. It reads the input from a file path
given on the command line, defaulting to the standard location, and writes a
report to standard output.

Usage:

    pixi run python validation/python/coord/check_geodetic.py [path]

Exit status is zero when every position agrees within the tolerance and one
otherwise.
"""

from __future__ import annotations

import csv
import sys
from pathlib import Path

import astropy.units as u
from astropy.coordinates import EarthLocation

# Position tolerance in meters. Astropy evaluates the same closed form, so
# the only difference should be floating point rounding.
TOLERANCE_M = 1e-6

DEFAULT_INPUT = Path("validation/data/expected/rust_geodetic.csv")


def rust_positions(path: Path) -> list[tuple[float, float, float, tuple[float, float, float]]]:
    """Reads the Rust output rows."""
    rows = []
    with path.open(newline="") as handle:
        reader = csv.DictReader(handle)
        for row in reader:
            rows.append(
                (
                    float(row["latitude_degrees"]),
                    float(row["longitude_degrees"]),
                    float(row["height_m"]),
                    (float(row["x_m"]), float(row["y_m"]), float(row["z_m"])),
                )
            )
    return rows


def astropy_position(latitude_deg: float, longitude_deg: float, height_m: float):
    """Recomputes a position with astropy."""
    location = EarthLocation.from_geodetic(
        lon=longitude_deg * u.deg,
        lat=latitude_deg * u.deg,
        height=height_m * u.m,
    )
    return (
        location.x.to_value(u.m),
        location.y.to_value(u.m),
        location.z.to_value(u.m),
    )


def main(argv: list[str]) -> int:
    path = Path(argv[1]) if len(argv) > 1 else DEFAULT_INPUT
    if not path.exists():
        print(f"missing input file: {path}", file=sys.stderr)
        print(
            "generate it with: cargo run -p apsis-rs --features physics "
            "--example dump_geodesy > " + str(path),
            file=sys.stderr,
        )
        return 2

    rows = rust_positions(path)
    if not rows:
        print("input file has no data rows", file=sys.stderr)
        return 2

    worst = 0.0
    worst_at = None
    for latitude, longitude, height, rust in rows:
        reference = astropy_position(latitude, longitude, height)
        error = max(abs(a - b) for a, b in zip(rust, reference))
        if error > worst:
            worst = error
            worst_at = (latitude, longitude, height)

    print(f"rows compared: {len(rows)}")
    print(f"worst error:   {worst:.3e} m at lat/lon/height {worst_at}")
    print(f"tolerance:     {TOLERANCE_M:.3e} m")

    if worst <= TOLERANCE_M:
        print("result:        pass")
        return 0
    print("result:        FAIL")
    return 1


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
