"""Cross-check the ApsisRS gaseous attenuation against ITU-R P.676-12.

Reads the values written by ``cargo run -p apsis-rs --features rf --example
dump_gas`` and recomputes each one with the ``itur`` package, which
implements the ITU-R recommendations directly, then reports the largest
difference in each column.

The ``itur`` package is a reference implementation of the recommendations
maintained as part of the ITU-R study programme, so it is an independent
check on the line tables and the formulae rather than a restatement of them.

The script needs no network access.

Usage:

    pixi run python validation/python/atmosphere/check_gas.py [path]

Exit status is zero when every value agrees within tolerance and one
otherwise.

用 ITU-R P.676-12 交叉核对 ApsisRS 的气体衰减。

读取由 ``cargo run -p apsis-rs --features rf --example dump_gas`` 写出的
值，并用 ``itur`` 包逐个重算，后者直接实现 ITU-R 各建议书，然后报告每列
的最大差异。

``itur`` 包是作为 ITU-R 研究计划一部分维护的建议书参考实现，故它是对线表
与各公式的独立核对，而非对它们的复述。

本脚本不需要网络访问。

用法：

    pixi run python validation/python/atmosphere/check_gas.py [path]

每个值都在容差内一致时退出状态为零，否则为一。
"""

from __future__ import annotations

import csv
import sys
from pathlib import Path

import numpy as np
import astropy.units as u

from itur.models.itu676 import _ITU676_12_

# The reference atmosphere of the recommendation, matching the Rust side.
# 该建议书的参考大气，与 Rust 侧一致。
PRESSURE_HPA = 1013.25
WATER_VAPOUR_DENSITY = 7.5
TEMPERATURE_K = 288.15

# Tolerances.
#
# The specific attenuation spans several orders of magnitude, and the water
# vapour term at low frequencies is a small difference between larger
# contributions, so a pure relative test is too strict there. The check
# passes when either the relative difference is small or the absolute
# difference is negligible.
#
# 容差。
#
# 比衰减跨越数个数量级，且低频处的水汽项是较大贡献之间的小差值，故纯粹
# 的相对判据在那里过严。当相对差异很小或绝对差异可忽略时，检查即通过。
RELATIVE_TOLERANCE = 1e-8
ABSOLUTE_TOLERANCE = 1e-10

DEFAULT_INPUT = Path("validation/data/expected/rust_gas.csv")


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
            "generate it with: cargo run -p apsis-rs --features rf "
            "--example dump_gas > " + str(path),
            file=sys.stderr,
        )
        return 2

    rows = read_rows(path)
    if not rows:
        print("input file has no data rows", file=sys.stderr)
        return 2

    model = _ITU676_12_()

    columns = ["dry_db_per_km", "wet_db_per_km", "dry_height_km", "wet_height_km", "zenith_db"]
    worst = {column: 0.0 for column in columns}
    worst_absolute = {column: 0.0 for column in columns}
    worst_at = {column: None for column in columns}

    for row in rows:
        frequency = float(row["frequency_ghz"])
        dry = float(model.gamma0_exact(frequency, PRESSURE_HPA, WATER_VAPOUR_DENSITY, TEMPERATURE_K))
        wet = float(model.gammaw_exact(frequency, PRESSURE_HPA, WATER_VAPOUR_DENSITY, TEMPERATURE_K))
        heights = model.slant_inclined_path_equivalent_height(
            frequency, PRESSURE_HPA, WATER_VAPOUR_DENSITY, TEMPERATURE_K
        )
        dry_height = float(np.asarray(heights[0]).ravel()[0])
        wet_height = float(np.asarray(heights[1]).ravel()[0])
        zenith = dry * dry_height + wet * wet_height

        expected = {
            "dry_db_per_km": dry,
            "wet_db_per_km": wet,
            "dry_height_km": dry_height,
            "wet_height_km": wet_height,
            "zenith_db": zenith,
        }
        for column, reference in expected.items():
            actual = float(row[column])
            absolute = abs(actual - reference)
            relative = absolute / max(abs(reference), ABSOLUTE_TOLERANCE)
            # The column passes on either measure, so the reported figure is
            # the relative one and the acceptance test below also allows the
            # absolute difference to be negligible.
            # 该列以两种度量之一通过即可，故所报告的数字是相对值，而下面的
            # 接受判据也允许绝对差异可忽略。
            if relative > worst[column]:
                worst[column] = relative
                worst_at[column] = frequency
            if absolute > worst_absolute[column]:
                worst_absolute[column] = absolute

    print(f"frequencies compared: {len(rows)}")
    for column in columns:
        print(
            f"{column:>15}: worst relative {worst[column]:.3e} "
            f"absolute {worst_absolute[column]:.3e} at {worst_at[column]} GHz"
        )
    print(
        f"{'tolerance':>15}:       {RELATIVE_TOLERANCE:.3e} relative or "
        f"{ABSOLUTE_TOLERANCE:.3e} absolute"
    )

    passes = all(
        worst[column] <= RELATIVE_TOLERANCE
        or worst_absolute[column] <= ABSOLUTE_TOLERANCE
        for column in columns
    )
    if passes:
        print("result:          pass")
        return 0
    print("result:          FAIL")
    return 1


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
