"""Cross-check the ApsisRS SGP4 propagator against the reference library.

Reads the states written by ``cargo run -p apsis-rs --features orbit
--example dump_orbit``, propagates the same element sets with the ``sgp4``
reference implementation, and reports the largest position and velocity
difference for each element set.

The script needs no network access.

Usage:

    pixi run python validation/python/orbit/check_sgp4.py [path]

Exit status is zero when every state agrees within tolerance and one
otherwise.
"""

from __future__ import annotations

import csv
import sys
from pathlib import Path

from sgp4.earth_gravity import wgs72
from sgp4.model import Satrec, WGS72

# Tolerances for the cross-check.
#
# These are the limits the workspace commits to for propagated position and
# velocity. They are far above the observed agreement, which is at the
# floating point level, and far below the accuracy of the SGP4 theory
# itself, which is on the order of a kilometer per day of propagation.
POSITION_TOLERANCE_KM = 1e-3
VELOCITY_TOLERANCE_KMS = 1e-6

DEFAULT_INPUT = Path("validation/data/expected/rust_sgp4.csv")

# The element sets, in the same order as the Rust example. They are the
# historical test cases distributed with the reference implementation.
ELEMENT_SETS = [
    (
        "1 25544U 98067A   08264.51782528 -.00002182  00000-0 -11606-4 0  2927",
        "2 25544  51.6416 247.4627 0006703 130.5360 325.0288 15.72125391563537",
    ),
    (
        "1 00005U 58002B   00179.78495062  .00000023  00000-0  28098-4 0  4753",
        "2 00005  34.2682 348.7242 1859667 331.7664  19.3264 10.82419157413667",
    ),
    (
        "1 27651U 03004A   06185.99374200  .00000109  00000-0  19909-4 0   445",
        "2 27651   7.1743 182.9443 7867251 269.0386  18.6601  1.03461646  9884",
    ),
    (
        "1 09880U 77021A   06174.85710247  .00000090  00000-0  10000-3 0  7322",
        "2 09880  64.1526 313.1296 6546925 103.4683 305.3980  2.00754276 13047",
    ),
]


def read_rows(path: Path) -> list[dict[str, str]]:
    """Reads the Rust output rows."""
    with path.open(newline="") as handle:
        reader = csv.DictReader(handle)
        return [row for row in reader if row.get("set")]


def reference_state(satellite: Satrec, minutes: float):
    """Returns the reference position and velocity, or None on error."""
    error, position, velocity = satellite.sgp4_tsince(minutes)
    if error != 0:
        return None
    return position, velocity


def main(argv: list[str]) -> int:
    path = Path(argv[1]) if len(argv) > 1 else DEFAULT_INPUT
    if not path.exists():
        print(f"missing input file: {path}", file=sys.stderr)
        print(
            "generate it with: cargo run -p apsis-rs --features orbit "
            "--example dump_orbit > " + str(path),
            file=sys.stderr,
        )
        return 2

    rows = read_rows(path)
    if not rows:
        print("input file has no data rows", file=sys.stderr)
        return 2

    satellites = [
        Satrec.twoline2rv(first, second, WGS72) for first, second in ELEMENT_SETS
    ]

    worst_position = 0.0
    worst_position_at = None
    worst_velocity = 0.0
    worst_velocity_at = None
    compared = 0
    rust_errors = 0
    reference_errors = 0
    mismatched_errors = 0

    for row in rows:
        index = int(row["set"])
        minutes = float(row["minutes"])
        rust_error = row["error"] != "0"
        reference = reference_state(satellites[index], minutes)

        if rust_error and reference is None:
            # Both sides reject the instant, which is agreement.
            continue
        if rust_error or reference is None:
            mismatched_errors += 1
            if rust_error:
                rust_errors += 1
            else:
                reference_errors += 1
            continue

        position, velocity = reference
        rust_position = (float(row["x_km"]), float(row["y_km"]), float(row["z_km"]))
        rust_velocity = (
            float(row["vx_kms"]),
            float(row["vy_kms"]),
            float(row["vz_kms"]),
        )
        position_error = max(abs(a - b) for a, b in zip(rust_position, position))
        velocity_error = max(abs(a - b) for a, b in zip(rust_velocity, velocity))

        if position_error > worst_position:
            worst_position = position_error
            worst_position_at = (index, minutes)
        if velocity_error > worst_velocity:
            worst_velocity = velocity_error
            worst_velocity_at = (index, minutes)
        compared += 1

    print(f"states compared:    {compared}")
    print(f"rust only errors:   {rust_errors}")
    print(f"reference errors:   {reference_errors}")
    print(f"mismatched errors:  {mismatched_errors}")
    print(
        f"worst position:     {worst_position:.6e} km at (set, minutes) {worst_position_at}"
    )
    print(
        f"worst velocity:     {worst_velocity:.6e} km/s at (set, minutes) {worst_velocity_at}"
    )
    print(f"position tolerance: {POSITION_TOLERANCE_KM:.6e} km")
    print(f"velocity tolerance: {VELOCITY_TOLERANCE_KMS:.6e} km/s")

    if mismatched_errors > 0:
        print("result:             FAIL (error handling disagrees)")
        return 1
    if (
        compared > 0
        and worst_position <= POSITION_TOLERANCE_KM
        and worst_velocity <= VELOCITY_TOLERANCE_KMS
    ):
        print("result:             pass")
        return 0
    print("result:             FAIL")
    return 1


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
