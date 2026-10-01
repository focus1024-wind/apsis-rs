#!/usr/bin/env python3
"""Generate the Rust source for the ITU-R P.839-4 rain height table.

The recommendation gives the height of the zero degree isotherm as a
tabulated global grid at 1.5 degree resolution. This script reads the grid
from the ``itur`` reference package and emits a Rust module that embeds it as
integer tenths of a kilometre, which is the resolution the recommendation
publishes.

Usage:

    python validation/tools/generate_p839_table.py > /tmp/p839_table.rs

生成 ITU-R P.839-4 雨顶高度表的 Rust 源码。

该建议书把零度等温线高度给出为 1.5 度分辨率的全球列表网格。本脚本从
``itur`` 参考包读取该网格，并输出一个 Rust 模块，把它以十分之一千米的整数
嵌入，这正是该建议书发布的分辨率。

用法：

    python validation/tools/generate_p839_table.py > /tmp/p839_table.rs
"""

from __future__ import annotations

import sys

import numpy as np

BASE = (
    "validation/.pixi/envs/default/lib/python3.12/site-packages/itur/data/839/"
)

# The grid resolution, in degrees.
# 网格分辨率，单位为度。
STEP_DEGREES = 1.5


def main() -> int:
    heights = np.load(BASE + "v4_esa0height.npz")["arr_0"]
    latitudes = np.load(BASE + "v4_esalat.npz")["arr_0"]
    longitudes = np.load(BASE + "v4_esalon.npz")["arr_0"]

    rows = heights.shape[0]
    columns = heights.shape[1]

    # The grid is regular: latitude runs from 90 down to -90 and longitude
    # from 0 to 360, both in steps of 1.5 degrees.
    # 该网格是规则的：纬度自 90 降到 -90，经度自 0 到 360，步长均为 1.5 度。
    assert np.allclose(latitudes[:, 0], np.linspace(90.0, -90.0, rows)), "latitude grid"
    assert np.allclose(longitudes[0, :], np.linspace(0.0, 360.0, columns)), "longitude grid"

    # The recommendation publishes the height to the nearest metre, so the
    # table is stored in metres to avoid losing precision.
    # 该建议书把高度发布到最近一米，故该表以米存储以避免损失精度。
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
    print("///")
    print("/// 零度等温线的高度，单位为米。")
    print("///")
    print("/// 网格在行方向上自北纬 90 度到南纬 90 度，在列方向上自东经 0 度到")
    print("/// 360 度，步长均为 1.5 度，故有 121 行 241 列。分数纬度或经度处的值")
    print("/// 由周围四点之间的双线性插值求得。")
    print("///")
    print("/// 出处：ITU-R 建议书 P.839-4，*Rain height model for prediction methods*，")
    print("/// 它在该网格上列出等温线高度。")
    print(f"pub static ZERO_ISOTHERM_HEIGHT_METRES: [[u16; {columns}]; {rows}] = [")
    for row in range(rows):
        values = ", ".join(str(int(value)) for value in metres[row])
        print(f"    [{values}],")
    print("];")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
