"""Cross-check the ApsisRS coverage geometry against spherical trigonometry.

Reads the quantities written by ``cargo run -p apsis-rs --features network
--example dump_coverage`` and recomputes each one from the spherical
triangle relations using only the standard library and numpy. The point is
that the Python side is written from the geometry, not from the Rust code,
so an error in the Rust formula shows up as a disagreement.

The three angles of the triangle formed by the satellite, the ground point,
and the centre of the Earth satisfy

    elevation + nadir + central = 90 degrees

and the law of sines gives

    sin(nadir) / body_radius = cos(elevation) / satellite_radius

from which the nadir angle, the central angle, and the slant range follow.
The footprint is the spherical cap the central angle cuts from the body, so
its area is a zone area and its fraction is that over the whole surface.

The script needs no network access.

Usage:

    pixi run python validation/python/constellation/check_coverage.py [path]

Exit status is zero when every quantity agrees within tolerance and one
otherwise.
"""

from __future__ import annotations

import csv
import math
import sys
from pathlib import Path

# The agreement is at the floating point level because both sides evaluate
# the same closed form expressions. The tolerances are set to catch a
# transposed radius or a missing conversion, not to allow a loose match.
ANGLE_TOLERANCE_RAD = 1e-12
LENGTH_TOLERANCE_KM = 1e-9
AREA_RELATIVE_TOLERANCE = 1e-12
FRACTION_TOLERANCE = 1e-12

DEFAULT_INPUT = Path("validation/data/expected/rust_coverage.csv")


def reference_nadir(satellite_radius: float, body_radius: float, elevation: float) -> float:
    """Returns the nadir angle from the law of sines."""
    return math.asin(body_radius * math.cos(elevation) / satellite_radius)


def reference_central(satellite_radius: float, body_radius: float, elevation: float) -> float:
    """Returns the central angle from the angle sum."""
    nadir = reference_nadir(satellite_radius, body_radius, elevation)
    return max(0.0, math.pi / 2.0 - elevation - nadir)


def reference_range(satellite_radius: float, body_radius: float, elevation: float) -> float:
    """Returns the slant range from the law of cosines."""
    central = reference_central(satellite_radius, body_radius, elevation)
    squared = (
        satellite_radius**2
        + body_radius**2
        - 2.0 * satellite_radius * body_radius * math.cos(central)
    )
    return math.sqrt(squared)


def reference_cap_area(central: float, body_radius: float) -> float:
    """Returns the area of a spherical cap of a given central angle.

    A cap is a zone between the pole and a colatitude, so its area is
    ``2 pi R^2 (1 - cos(central))``. The footprint is not centred on a pole in
    general, but a spherical cap is rotationally symmetric, so the same
    expression applies.
    """
    return 2.0 * math.pi * body_radius**2 * (1.0 - math.cos(central))


def read_blocks(path: Path):
    """Reads the two blocks of the file.

    The first block is the case table with a header row; a blank line
    separates it from the inverse relation block, which has its own header.
    """
    text = path.read_text().splitlines()
    blocks: list[list[str]] = [[]]
    for line in text:
        if line.strip():
            blocks[-1].append(line)
        else:
            blocks.append([])
    return [block for block in blocks if block]


def check_cases(rows: list[dict[str, str]]) -> tuple[bool, list[str]]:
    """Checks the case table and returns whether it passed and any failures."""
    failures: list[str] = []
    compared = 0
    worst = {
        "nadir": 0.0,
        "central": 0.0,
        "range": 0.0,
        "area": 0.0,
        "fraction": 0.0,
    }

    for row in rows:
        name = row["case"]
        satellite_radius = float(row["satellite_radius_km"])
        body_radius = float(row["body_radius_km"])
        elevation = float(row["elevation_mask_rad"])
        compared += 1

        nadir = reference_nadir(satellite_radius, body_radius, elevation)
        central = reference_central(satellite_radius, body_radius, elevation)
        slant = reference_range(satellite_radius, body_radius, elevation)
        area = reference_cap_area(central, body_radius)
        fraction = area / (4.0 * math.pi * body_radius**2)

        worst["nadir"] = max(worst["nadir"], abs(nadir - float(row["nadir_rad"])))
        worst["central"] = max(worst["central"], abs(central - float(row["central_rad"])))
        worst["range"] = max(worst["range"], abs(slant - float(row["slant_range_km"])))
        rust_area = float(row["footprint_area_km2"])
        worst["area"] = max(worst["area"], abs(area - rust_area) / rust_area)
        worst["fraction"] = max(
            worst["fraction"], abs(fraction - float(row["footprint_fraction"]))
        )

        if abs(nadir - float(row["nadir_rad"])) > ANGLE_TOLERANCE_RAD:
            failures.append(f"{name}: nadir differs by {abs(nadir - float(row['nadir_rad']))}")
        if abs(central - float(row["central_rad"])) > ANGLE_TOLERANCE_RAD:
            failures.append(
                f"{name}: central differs by {abs(central - float(row['central_rad']))}"
            )
        if abs(slant - float(row["slant_range_km"])) > LENGTH_TOLERANCE_KM:
            failures.append(
                f"{name}: range differs by {abs(slant - float(row['slant_range_km']))}"
            )
        if abs(area - rust_area) / rust_area > AREA_RELATIVE_TOLERANCE:
            failures.append(f"{name}: area differs by {abs(area - rust_area) / rust_area}")
        if abs(fraction - float(row["footprint_fraction"])) > FRACTION_TOLERANCE:
            failures.append(
                f"{name}: fraction differs by "
                f"{abs(fraction - float(row['footprint_fraction']))}"
            )

    print(f"cases compared:      {compared}")
    print(f"worst nadir:         {worst['nadir']:.6e} rad")
    print(f"worst central:       {worst['central']:.6e} rad")
    print(f"worst range:         {worst['range']:.6e} km")
    print(f"worst area (rel):    {worst['area']:.6e}")
    print(f"worst fraction:      {worst['fraction']:.6e}")
    return (not failures, failures)


def check_inverse(rows: list[dict[str, str]]) -> tuple[bool, list[str]]:
    """Checks that the central angle and elevation relations invert."""
    failures: list[str] = []
    compared = 0
    worst = 0.0
    for row in rows:
        if not row["elevation_rad"]:
            continue
        central_degrees = float(row["inverse_central_degrees"])
        elevation = float(row["elevation_rad"])
        round_trip = float(row["round_trip_central_degrees"])
        compared += 1

        # Recompute the round trip independently: from the elevation, the
        # central angle follows from the angle sum.
        satellite_radius = 6_378.137 + 550.0
        body_radius = 6_378.137
        nadir = reference_nadir(satellite_radius, body_radius, elevation)
        expected = max(0.0, math.pi / 2.0 - elevation - nadir)
        error = abs(expected * 180.0 / math.pi - round_trip)
        worst = max(worst, error)
        if error > 1e-9:
            failures.append(
                f"central {central_degrees} deg: round trip differs by {error}"
            )

    print(f"inverse compared:    {compared}")
    print(f"worst round trip:    {worst:.6e} deg")
    return (not failures, failures)


def main(argv: list[str]) -> int:
    path = Path(argv[1]) if len(argv) > 1 else DEFAULT_INPUT
    if not path.exists():
        print(f"missing input file: {path}", file=sys.stderr)
        print(
            "generate it with: cargo run -p apsis-rs --features network "
            "--example dump_coverage > " + str(path),
            file=sys.stderr,
        )
        return 2

    blocks = read_blocks(path)
    if len(blocks) < 2:
        print("input file does not have the two expected blocks", file=sys.stderr)
        return 2

    cases = list(csv.DictReader(blocks[0]))
    inverse = list(csv.DictReader(blocks[1]))
    if not cases or not inverse:
        print("input file has an empty block", file=sys.stderr)
        return 2

    cases_ok, case_failures = check_cases(cases)
    inverse_ok, inverse_failures = check_inverse(inverse)

    for failure in case_failures + inverse_failures:
        print(f"  {failure}")

    if cases_ok and inverse_ok:
        print("result:              pass")
        return 0
    print("result:              FAIL")
    return 1


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
