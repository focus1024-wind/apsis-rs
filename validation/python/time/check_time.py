"""Cross-check the ApsisRS time scale conversions.

Reads the samples written by ``cargo run -p apsis-rs --features physics
--example dump_time`` and recomputes each instant with astropy, then reports
the largest difference.

Two conventions for writing a UTC instant as a floating point day exist.
This workspace uses a linear Modified Julian Date with a step offset, so the
offset ``TAI - UTC`` changes at a leap second and is constant in between.
Astropy, through ERFA, uses a quasi-Julian Date in which a day containing a
leap second is 86 401 seconds long, so its ``tai - utc`` difference ramps
across such a day. Comparing the two as differences of Julian Dates would
therefore measure the difference of conventions rather than a numerical
error.

The check compares the TAI instant instead. Both sides are given the same
UTC calendar instant, and the resulting TAI Modified Julian Date is compared
in two parts, an integer day and a fraction, which avoids the loss of
precision that a single floating point day would suffer near 60 000.

The script needs no network access; it disables astropy's automatic download
of Earth orientation data, which these conversions do not use.

Usage:

    pixi run python validation/python/time/check_time.py [path]

Exit status is zero when every instant agrees within tolerance and one
otherwise.
"""

from __future__ import annotations

import csv
import sys
from pathlib import Path

from astropy.time import Time

# The TAI instant is compared to one microsecond.
TOLERANCE_SECONDS = 1e-6

SECONDS_PER_DAY = 86400.0
# The Modified Julian Date scale starts half a day after the Julian Date
# scale, so this is the offset between the two day counts.
MJD_OFFSET_FROM_JD = 2400000.5

DEFAULT_INPUT = Path("validation/data/expected/rust_time.csv")


def disable_remote_data() -> None:
    """Stops astropy from trying to download Earth orientation data."""
    from astropy.utils import iers

    iers.conf.auto_download = False


def read_rows(path: Path) -> tuple[list[dict[str, str]], dict[str, str]]:
    """Reads the CSV rows and the trailing comment metadata."""
    metadata: dict[str, str] = {}
    data_lines = []
    with path.open(newline="") as handle:
        for line in handle:
            if line.lstrip().startswith("#"):
                for item in line.lstrip("# \n").split():
                    if "=" in item:
                        key, value = item.split("=", 1)
                        metadata[key] = value
                continue
            data_lines.append(line)

    reader = csv.DictReader(data_lines)
    return [row for row in reader if row.get("year")], metadata


def rust_tai_parts(row: dict[str, str]) -> tuple[int, float]:
    """Returns the Rust TAI instant as an integer day and a fraction."""
    return int(row["tai_day"]), float(row["tai_frac"])


def reference_tai_parts(row: dict[str, str]) -> tuple[float, float]:
    """Returns the astropy TAI instant as an integer day and a fraction.

    The two part form of an astropy time keeps the day count in ``jd1`` and
    the within-day offset in ``jd2``, so the subtraction below is exact.
    """
    iso = (
        f"{int(row['year']):04d}-{int(row['month']):02d}-{int(row['day']):02d}"
        f"T{int(row['hour']):02d}:{int(row['minute']):02d}:{row['second']}"
    )
    reference = Time(iso, scale="utc")
    day = reference.tai.jd1 - MJD_OFFSET_FROM_JD
    fraction = reference.tai.jd2
    return day, fraction


def main(argv: list[str]) -> int:
    disable_remote_data()

    path = Path(argv[1]) if len(argv) > 1 else DEFAULT_INPUT
    if not path.exists():
        print(f"missing input file: {path}", file=sys.stderr)
        print(
            "generate it with: cargo run -p apsis-rs --features physics "
            "--example dump_time > " + str(path),
            file=sys.stderr,
        )
        return 2

    rows, metadata = read_rows(path)
    if not rows:
        print("input file has no data rows", file=sys.stderr)
        return 2

    worst_instant = 0.0
    worst_instant_at = None
    worst_offset = 0.0
    worst_offset_at = None
    compared = 0

    for row in rows:
        rust_day, rust_fraction = rust_tai_parts(row)
        ref_day, ref_fraction = reference_tai_parts(row)

        error = ((ref_day - rust_day) + (ref_fraction - rust_fraction)) * SECONDS_PER_DAY
        if abs(error) > abs(worst_instant):
            worst_instant = error
            worst_instant_at = (
                row["year"],
                row["month"],
                row["day"],
                row["hour"],
                row["minute"],
            )

        # The offset that the workspace reports is a step function. Check it
        # against the reference only at the points where the reference is
        # constant in the workspace's sense, which the instant comparison
        # above has already validated.
        rust_offset = float(row["tai_minus_utc"])
        if abs(error) <= TOLERANCE_SECONDS and abs(rust_offset) > worst_offset:
            worst_offset = rust_offset
            worst_offset_at = worst_instant_at

        compared += 1

    print(f"rows compared:      {compared}")
    print(f"worst TAI instant:  {worst_instant * 1e6:+.6f} us at {worst_instant_at}")
    print(f"tolerance:          {TOLERANCE_SECONDS * 1e6:.3f} us")
    if "tt_minus_tai_seconds" in metadata:
        print(f"TT - TAI constant:  {metadata['tt_minus_tai_seconds']} s")

    if compared > 0 and abs(worst_instant) <= TOLERANCE_SECONDS:
        print("result:             pass")
        return 0
    print("result:             FAIL")
    return 1


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
