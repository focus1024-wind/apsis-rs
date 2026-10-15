"""Cross-check the ApsisRS exact slant path against ITU-R P.676-12.

Reads the values written by ``cargo run --features rf --example
dump_gas_exact`` and recomputes each one with the ``itur`` package, which
implements the exact method of the recommendation directly. The exact method
integrates the specific attenuation through a layered standard atmosphere and
traces the refracted ray, so it is the form to use at low elevation.

The script needs no network access.

Usage:

    pixi run python validation/python/atmosphere/check_gas_exact.py [path]

Exit status is zero when every value agrees within tolerance and one
otherwise.

用 ITU-R P.676-12 交叉核对 ApsisRS 的精确斜路径。

读取由 ``cargo run --features rf --example dump_gas_exact``
写出的值，并用 ``itur`` 包逐个重算，后者直接实现该建议书的精确方法。精确
方法在分层标准大气中积分比衰减并追踪折射后的射线，故它是低仰角下宜用的
形式。

本脚本不需要网络访问。

用法：

    pixi run python validation/python/atmosphere/check_gas_exact.py [path]

每个值都在容差内一致时退出状态为零，否则为一。
"""

from __future__ import annotations

import csv
import sys
from pathlib import Path

from itur.models.itu676 import _ITU676_12_

# The reference atmosphere, matching the Rust side.
# 参考大气，与 Rust 侧一致。
PRESSURE_HPA = 1013.25
WATER_VAPOUR_DENSITY = 7.5
TEMPERATURE_K = 288.15

# The implementation reproduces the reference to about nine significant
# digits, so this allows for the rounding of the recorded values.
# 该实现把参考值复现到约九位有效数字，故此值容许所记录值的舍入。
RELATIVE_TOLERANCE = 1e-7

DEFAULT_INPUT = Path("validation/data/expected/rust_gas_exact.csv")


def read_rows(path: Path) -> list[dict[str, str]]:
    """Reads the CSV rows.

    读取 CSV 各行。
    """
    with path.open(newline="") as handle:
        reader = csv.DictReader(handle)
        return [row for row in reader if row.get("frequency_ghz")]


def main(argv: list[str]) -> int:
    path = Path(argv[1]) if len(argv) > 1 else DEFAULT_INPUT
    if not path.exists():
        print(f"missing input file: {path}", file=sys.stderr)
        print(
            "generate it with: cargo run --features rf "
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
