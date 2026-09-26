"""Cross-check the ApsisRS link budget against the link equation.

Reads the scenarios written by ``cargo run -p apsis-rs --features rf
--example dump_link`` and recomputes each figure from the link equation in
plain Python, then reports the largest difference in each column.

The reference here is the link equation itself rather than a library, because
the equation is closed form and short enough to transcribe without
ambiguity. The constants are taken from the 2019 SI definitions, so they
match the values the Rust side uses.

The script needs no network access.

Usage:

    pixi run python validation/python/link/check_link.py [path]

Exit status is zero when every figure agrees within tolerance and one
otherwise.
"""

from __future__ import annotations

import csv
import math
import sys
from pathlib import Path

# Defining constants, exact in the SI.
BOLTZMANN = 1.380649e-23
SPEED_OF_LIGHT = 299792458.0

# Tolerance in decibels. The two sides evaluate the same expressions with
# double precision, so the only difference should be rounding.
TOLERANCE_DB = 1e-9

DEFAULT_INPUT = Path("validation/data/expected/rust_link.csv")


def read_rows(path: Path) -> list[dict[str, str]]:
    """Reads the CSV rows."""
    with path.open(newline="") as handle:
        reader = csv.DictReader(handle)
        return [row for row in reader if row.get("transmit_dbw")]


def reference(row: dict[str, str]) -> dict[str, float]:
    """Recomputes a budget from the link equation."""
    transmit_dbw = float(row["transmit_dbw"])
    transmit_dbi = float(row["transmit_dbi"])
    transmit_loss = float(row["transmit_loss_db"])
    receive_dbi = float(row["receive_dbi"])
    receive_loss = float(row["receive_loss_db"])
    temperature = float(row["temperature_k"])
    bandwidth = float(row["bandwidth_hz"])
    distance = float(row["distance_m"])
    frequency = float(row["frequency_hz"])

    eirp_dbw = transmit_dbw + transmit_dbi - transmit_loss
    wavelength = SPEED_OF_LIGHT / frequency
    path_loss_db = 20.0 * math.log10(4.0 * math.pi * distance / wavelength)
    carrier_dbw = eirp_dbw - path_loss_db + receive_dbi - receive_loss
    noise_density_dbw = 10.0 * math.log10(BOLTZMANN * temperature)
    cn0_db = carrier_dbw - noise_density_dbw
    cn_db = cn0_db - 10.0 * math.log10(bandwidth)
    # The example reports the energy per bit ratio at a data rate equal to
    # the bandwidth.
    ebn0_db = cn0_db - 10.0 * math.log10(bandwidth)

    return {
        "eirp_dbw": eirp_dbw,
        "carrier_dbw": carrier_dbw,
        "cn0_db": cn0_db,
        "cn_db": cn_db,
        "ebn0_db": ebn0_db,
    }


def main(argv: list[str]) -> int:
    path = Path(argv[1]) if len(argv) > 1 else DEFAULT_INPUT
    if not path.exists():
        print(f"missing input file: {path}", file=sys.stderr)
        print(
            "generate it with: cargo run -p apsis-rs --features rf "
            "--example dump_link > " + str(path),
            file=sys.stderr,
        )
        return 2

    rows = read_rows(path)
    if not rows:
        print("input file has no data rows", file=sys.stderr)
        return 2

    columns = ["eirp_dbw", "carrier_dbw", "cn0_db", "cn_db", "ebn0_db"]
    worst = {column: 0.0 for column in columns}
    worst_at = {column: None for column in columns}

    for row in rows:
        expected = reference(row)
        for column in columns:
            difference = abs(float(row[column]) - expected[column])
            if difference > worst[column]:
                worst[column] = difference
                worst_at[column] = row["distance_m"]

    print(f"scenarios compared: {len(rows)}")
    for column in columns:
        print(f"{column:>12}: worst {worst[column]:.3e} dB at distance {worst_at[column]} m")
    print(f"{'tolerance':>12}:       {TOLERANCE_DB:.3e} dB")

    if all(value <= TOLERANCE_DB for value in worst.values()):
        print("result:       pass")
        return 0
    print("result:       FAIL")
    return 1


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
