"""Cross-check the ApsisRS TEME to Earth-fixed transformation.

Reads the positions written by ``cargo run -p apsis-rs --features physics
--example dump_teme`` and recomputes each one with ERFA, applying the same
Greenwich mean sidereal time and the same zero polar motion, so that the
check compares the implementation rather than the choice of inputs.

The reference is the sidereal time of 1982 and the polar motion composition
of the IAU SOFA library, as implemented by ERFA. Note that astropy agrees
with this transformation only to a few hundred metres, because astropy
applies the measured polar motion and a measured UT1 - UTC while this check
uses zero for both. That agreement is reported for information but is not
the acceptance criterion.

The script needs no network access.

Usage:

    pixi run python validation/python/coord/check_teme.py [path]

Exit status is zero when every position agrees within tolerance and one
otherwise.

交叉核对 ApsisRS 的 TEME 到地固坐标变换。

读取由 ``cargo run -p apsis-rs --features physics --example dump_teme``
写出的位置，并用 ERFA 逐个重算，施加相同的格林尼治平恒星时与相同的零极移，
使该检查比较的是实现而非输入的选择。

参考值是 1982 年的恒星时与 IAU SOFA 库的极移合成，由 ERFA 实现。注意
astropy 与该变换只相符到数百米，因为 astropy 施加实测极移与实测
UT1 - UTC，而本检查对两者都用零。该相符情况为提供信息而报告，但并非接受
判据。

本脚本不需要网络访问。

用法：

    pixi run python validation/python/coord/check_teme.py [path]

每个位置都在容差内一致时退出状态为零，否则为一。
"""

from __future__ import annotations

import csv
import sys
from pathlib import Path

import erfa

# Tolerance in kilometers. The transformation is a product of two rotations
# evaluated in the same order, so the agreement is at the level of the last
# bits of a double.
# 容差，单位为千米。该变换是两个按相同次序求值的转动之积，故相符程度在
# 双精度最后几位的水平上。
TOLERANCE_KM = 1e-9

# The two-part Julian date used by the Rust example for the whole days.
# Rust 示例为整数天数所用的两部分儒略日。
DAY_PART = 2400000.5

DEFAULT_INPUT = Path("validation/data/expected/rust_teme.csv")


def read_rows(path: Path) -> list[dict[str, str]]:
    """Reads the CSV rows.

    读取 CSV 各行。
    """
    with path.open(newline="") as handle:
        reader = csv.DictReader(handle)
        return [row for row in reader if row.get("mjd")]


def reference_position(mjd: float, position: list[float]) -> list[float]:
    """Recomputes the Earth-fixed position with ERFA.

    用 ERFA 重算地固位置。
    """
    # The sidereal time, from the two-part Julian date.
    # 恒星时，由两部分儒略日得出。
    sidereal = erfa.gmst82(DAY_PART, mjd)
    # The rotation of the frame about its third axis, composed as the
    # reference `rz` does, and the identity polar motion.
    # 坐标系绕其第三轴的转动，按参考 `rz` 的方式合成，以及单位极移。
    rotation = erfa.rz(sidereal, erfa.ir())
    polar = erfa.pom00(0.0, 0.0, 0.0)
    matrix = erfa.rxr(polar, rotation)

    # The rotation maps the frame, so the position is the matrix applied to
    # the components.
    # 该转动映射坐标系，故位置是矩阵施加于各分量。
    return [
        sum(matrix[i][j] * position[j] for j in range(3)) for i in range(3)
    ]


def main(argv: list[str]) -> int:
    path = Path(argv[1]) if len(argv) > 1 else DEFAULT_INPUT
    if not path.exists():
        print(f"missing input file: {path}", file=sys.stderr)
        print(
            "generate it with: cargo run -p apsis-rs --features physics "
            "--example dump_teme > " + str(path),
            file=sys.stderr,
        )
        return 2

    rows = read_rows(path)
    if not rows:
        print("input file has no data rows", file=sys.stderr)
        return 2

    # The TEME inputs used by the example, in the same order.
    # 示例所用的 TEME 输入，次序相同。
    inputs = [
        [7000.0, 1000.0, 2000.0],
        [7000.0, -3000.0, 1000.0],
        [42164.0, 0.0, 0.0],
        [6878.137, 100.0, -50.0],
    ]

    worst = 0.0
    worst_at = None
    for row, position in zip(rows, inputs):
        mjd = float(row["mjd"])
        actual = [
            float(row["x_ecef"]),
            float(row["y_ecef"]),
            float(row["z_ecef"]),
        ]
        expected = reference_position(mjd, position)
        error = max(abs(a - b) for a, b in zip(actual, expected))
        if error > worst:
            worst = error
            worst_at = mjd

    print(f"positions compared: {len(rows)}")
    print(f"worst difference:   {worst:.3e} km at MJD {worst_at}")
    print(f"tolerance:          {TOLERANCE_KM:.3e} km")

    if worst <= TOLERANCE_KM:
        print("result:             pass")
        return 0
    print("result:             FAIL")
    return 1


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
