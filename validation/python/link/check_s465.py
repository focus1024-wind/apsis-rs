"""Cross-check the ApsisRS ITU-R S.465 reference pattern.

Reads the values written by ``cargo run -p apsis-rs --features rf --example
dump_s465`` and recomputes each one from the equations of the recommendation.

The script needs no network access.

Usage:

    pixi run python validation/python/link/check_s465.py [path]

Exit status is zero when every value agrees within tolerance and one
otherwise.

交叉核对 ApsisRS 的 ITU-R S.465 参考方向图。

读取由 ``cargo run -p apsis-rs --features rf --example dump_s465`` 写出的
值，并从该建议书的各式逐个重算。

本脚本不需要网络访问。

用法：

    pixi run python validation/python/link/check_s465.py [path]

每个值都在容差内一致时退出状态为零，否则为一。
"""

from __future__ import annotations

import csv
import math
import sys
from pathlib import Path

# The peak gain offset and the branch constants of the recommendation.
# 该建议书的峰值增益偏移与各分支常量。
PEAK_GAIN_OFFSET_DB = 7.7
LARGE_APERTURE_RATIO = 100.0

# Tolerance in decibels.
# 容差，单位为分贝。
TOLERANCE_DB = 1e-6

DEFAULT_INPUT = Path("validation/data/expected/rust_s465.csv")


def read_rows(path: Path) -> list[dict[str, str]]:
    """Reads the CSV rows.

    读取 CSV 各行。
    """
    with path.open(newline="") as handle:
        reader = csv.DictReader(handle)
        return [row for row in reader if row.get("ratio")]


def break_angle(ratio: float) -> tuple[float, float]:
    """Returns the branch constant and the break angle, in degrees.

    返回分支常量与转折角，单位为度。
    """
    head, floor_angle = (29.0, 36.0) if ratio >= LARGE_APERTURE_RATIO else (32.0, 48.0)
    peak = 20.0 * math.log10(ratio) + PEAK_GAIN_OFFSET_DB

    def difference(angle_degrees: float) -> float:
        main_lobe = peak - 2.5e-3 * (ratio * angle_degrees) ** 2
        sidelobe = head - 25.0 * math.log10(angle_degrees)
        return main_lobe - sidelobe

    low, high = 1.0e-9, floor_angle
    if difference(high) > 0.0:
        return head, high
    for _ in range(200):
        middle = 0.5 * (low + high)
        if difference(middle) > 0.0:
            low = middle
        else:
            high = middle
    return head, 0.5 * (low + high)


def reference(ratio: float, angle_degrees: float) -> float:
    """Recomputes the pattern gain in decibels.

    The API takes the angle in radians, so the degrees recorded in the file
    are converted to radians and back before the branch decision. That
    reproduces the rounding the implementation performs, which matters at
    the branch boundaries where the recommendation has a small step.

    以分贝重算方向图增益。

    该 API 以弧度接收角度，故在分支判定之前，文件中以度记录的角度先转成
    弧度再转回。这复现了实现所执行的舍入，在各分支边界处要紧，因为该建议书
    在那里有小的台阶。
    """
    peak = 20.0 * math.log10(ratio) + PEAK_GAIN_OFFSET_DB
    head, break_at = break_angle(ratio)
    head, floor_angle = (
        (29.0, 36.0) if ratio >= LARGE_APERTURE_RATIO else (32.0, 48.0)
    )
    angle_degrees = math.degrees(math.radians(angle_degrees))
    if angle_degrees < break_at:
        return peak - 2.5e-3 * (ratio * angle_degrees) ** 2
    if angle_degrees <= floor_angle:
        return head - 25.0 * math.log10(angle_degrees)
    return -10.0


def main(argv: list[str]) -> int:
    path = Path(argv[1]) if len(argv) > 1 else DEFAULT_INPUT
    if not path.exists():
        print(f"missing input file: {path}", file=sys.stderr)
        print(
            "generate it with: cargo run -p apsis-rs --features rf "
            "--example dump_s465 > " + str(path),
            file=sys.stderr,
        )
        return 2

    rows = read_rows(path)
    if not rows:
        print("input file has no data rows", file=sys.stderr)
        return 2

    worst_gain = 0.0
    worst_break = 0.0
    worst_at = None

    for row in rows:
        ratio = float(row["ratio"])
        angle_degrees = float(row["angle_deg"])
        _, expected_break = break_angle(ratio)
        expected_gain = reference(ratio, angle_degrees)

        break_error = abs(float(row["break_deg"]) - expected_break)
        gain_error = abs(float(row["gain_dbi"]) - expected_gain)
        if break_error > worst_break or gain_error > worst_gain:
            worst_at = (ratio, angle_degrees)
            worst_break = max(worst_break, break_error)
            worst_gain = max(worst_gain, gain_error)

    print(f"points compared:  {len(rows)}")
    print(f"worst break:      {worst_break:.3e} deg")
    print(f"worst gain:       {worst_gain:.3e} dB")
    print(f"worst point:      {worst_at}")
    print(f"tolerance:        {TOLERANCE_DB:.3e} dB")

    if worst_break <= TOLERANCE_DB and worst_gain <= TOLERANCE_DB:
        print("result:           pass")
        return 0
    print("result:           FAIL")
    return 1


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
