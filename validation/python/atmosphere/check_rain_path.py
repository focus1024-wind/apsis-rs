"""Cross-check the ApsisRS rain path against ITU-R P.618-13.

Reads the values written by ``cargo run -p apsis-rs --features rf --example
dump_rain_path`` and recomputes each one from the equations of the
recommendation. The specific attenuation coefficients come from the ``itur``
implementation of ITU-R P.838-3, and the path geometry and the two reduction
factors are transcribed from Recommendation P.618-13.

The script needs no network access.

Usage:

    pixi run python validation/python/atmosphere/check_rain_path.py [path]

Exit status is zero when every value agrees within tolerance and one
otherwise.
"""

from __future__ import annotations

import csv
import math
import sys
from pathlib import Path

from itur.models.itu838 import _ITU838_3_

# Tolerance on the path length and the attenuation, in their own units.
LENGTH_TOLERANCE_KM = 1e-9
ATTENUATION_TOLERANCE_DB = 1e-9

# The effective radius of the Earth used by the path geometry, in
# kilometers, from Recommendation P.618-13.
EFFECTIVE_EARTH_RADIUS_KM = 8500.0

# The rain layer height used by the Rust example, in kilometers.
RAIN_HEIGHT_KM = 3.0

DEFAULT_INPUT = Path("validation/data/expected/rust_rain_path.csv")


def read_rows(path: Path) -> list[dict[str, str]]:
    """Reads the CSV rows."""
    with path.open(newline="") as handle:
        reader = csv.DictReader(handle)
        return [row for row in reader if row.get("rain_rate_mm_per_h")]


def reference(row: dict[str, str], coefficients) -> tuple[float, float]:
    """Recomputes the path length and the attenuation."""
    rain_rate = float(row["rain_rate_mm_per_h"])
    frequency = float(row["frequency_ghz"])
    elevation = float(row["elevation_deg"])
    latitude = float(row["latitude_deg"])
    station_height = 0.0

    k, alpha = coefficients.rain_specific_attenuation_coefficients(
        frequency, elevation, 0.0
    )
    specific = float(k) * rain_rate ** float(alpha)

    # Equation 2: the slant path length.
    height_difference = RAIN_HEIGHT_KM - station_height
    sine = math.sin(math.radians(elevation))
    slant_length = 2.0 * height_difference / (
        math.sqrt(sine * sine + 2.0 * height_difference / EFFECTIVE_EARTH_RADIUS_KM)
        + sine
    )

    # Equation 3: the horizontal projection.
    horizontal_length = abs(slant_length * math.cos(math.radians(elevation)))

    # Equation 6: the horizontal reduction factor.
    horizontal_reduction = 1.0 / (
        1.0
        + 0.78 * math.sqrt(horizontal_length * specific / frequency)
        - 0.38 * (1.0 - math.exp(-2.0 * horizontal_length))
    )

    # Equation 7: the angle at which the path meets the rain layer.
    eta = math.degrees(
        math.atan2(height_difference, horizontal_length * horizontal_reduction)
    )

    # Equation 8: the longer of the two path estimates.
    if eta > elevation:
        path_length = (
            horizontal_length * horizontal_reduction / math.cos(math.radians(elevation))
        )
    else:
        path_length = height_difference / sine

    # Equation 9: the vertical adjustment factor. The exponential takes the
    # elevation in degrees.
    xi = 36.0 - abs(latitude) if abs(latitude) < 36.0 else 0.0
    vertical_adjustment = 1.0 / (
        1.0
        + math.sqrt(sine)
        * (
            31.0
            * (1.0 - math.exp(-(elevation / (1.0 + xi))))
            * math.sqrt(path_length * specific)
            / frequency**2
            - 0.45
        )
    )

    effective_length = path_length * vertical_adjustment
    return effective_length, specific * effective_length


def main(argv: list[str]) -> int:
    path = Path(argv[1]) if len(argv) > 1 else DEFAULT_INPUT
    if not path.exists():
        print(f"missing input file: {path}", file=sys.stderr)
        print(
            "generate it with: cargo run -p apsis-rs --features rf "
            "--example dump_rain_path > " + str(path),
            file=sys.stderr,
        )
        return 2

    rows = read_rows(path)
    if not rows:
        print("input file has no data rows", file=sys.stderr)
        return 2

    coefficients = _ITU838_3_()
    worst_length = 0.0
    worst_attenuation = 0.0
    worst_at = None

    for row in rows:
        length, attenuation = reference(row, coefficients)
        length_error = abs(float(row["path_length_km"]) - length)
        attenuation_error = abs(float(row["attenuation_db"]) - attenuation)
        if length_error > worst_length or attenuation_error > worst_attenuation:
            worst_at = (
                row["rain_rate_mm_per_h"],
                row["frequency_ghz"],
                row["elevation_deg"],
                row["latitude_deg"],
            )
            worst_length = max(worst_length, length_error)
            worst_attenuation = max(worst_attenuation, attenuation_error)

    print(f"scenarios compared:   {len(rows)}")
    print(f"worst path length:    {worst_length:.3e} km")
    print(f"worst attenuation:    {worst_attenuation:.3e} dB")
    print(f"worst scenario:       {worst_at}")
    print(
        f"tolerance:            {LENGTH_TOLERANCE_KM:.3e} km or "
        f"{ATTENUATION_TOLERANCE_DB:.3e} dB"
    )

    if (
        worst_length <= LENGTH_TOLERANCE_KM
        and worst_attenuation <= ATTENUATION_TOLERANCE_DB
    ):
        print("result:               pass")
        return 0
    print("result:               FAIL")
    return 1


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
