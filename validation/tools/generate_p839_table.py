#!/usr/bin/env python3
"""Generate the Rust source for the ITU-R P.839-4 rain height table.

The recommendation gives the height of the zero degree isotherm as a
tabulated global grid at 1.5 degree resolution. This script reads the grid
from the ``itur`` reference package and emits a Rust module that embeds it as
integer tenths of a kilometre, which is the resolution the recommendation
publishes.

Usage:

    python validation/tools/generate_p839_table.py > /tmp/p839_table.rs
"""

from __future__ import annotations

import sys

import numpy as np

BASE = (
    "validation/.pixi/envs/default/lib/python3.12/site-packages/itur/data/839/"
)

# The grid resolution, in degrees.
STEP_DEGREES = 1.5


def main() -> int:
    heights = np.load(BASE + "v4_esa0height.npz")["arr_0"]
    latitudes = np.load(BASE + "v4_esalat.npz")["arr_0"]
    longitudes = np.load(BASE + "v4_esalon.npz")["arr_0"]

    rows = heights.shape[0]
    columns = heights.shape[1]

    # The grid is regular: latitude runs from 90 down to -90 and longitude
    # from 0 to 360, both in steps of 1.5 degrees.
    assert np.allclose(latitudes[:, 0], np.linspace(90.0, -90.0, rows)), "latitude grid"
    assert np.allclose(longitudes[0, :], np.linspace(0.0, 360.0, columns)), "longitude grid"

    # The recommendation publishes the height to the nearest metre, so the
    # table is stored in metres to avoid losing precision.
    metres = np.round(heights * 1000.0).astype(np.uint16)

    print("/// The height of the zero degree isotherm, in metres.")
    print("///")
    print("/// The grid runs from 90 degrees north to 90 degrees south in the row")
    print("/// direction and from 0 to 360 degrees east in the column direction,")
    print("/// both in steps of 1.5 degrees, so there are 121 rows and 241 columns.")
    print("/// The value at a fractional latitude or longitude is found by bilinear")
    print("/// interpolation between the four surrounding points.")
    print("///")
    print("/// Source: ITU-R Recommendation P.839-4, *Rain height model for prediction")
    print("/// methods*, which tabulates the isotherm height on this grid.")
    print(f"pub static ZERO_ISOTHERM_HEIGHT_METRES: [[u16; {columns}]; {rows}] = [")
    for row in range(rows):
        values = ", ".join(str(int(value)) for value in metres[row])
        print(f"    [{values}],")
    print("];")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
