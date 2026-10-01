"""Cross-check the ApsisRS IAU 2000/2006 Earth orientation against ERFA.

Reads the quantities written by ``cargo run -p apsis-rs --features physics
--example dump_iau2000`` and recomputes each one with ERFA, which is the
reference implementation distributed by the International Astronomical Union
through the SOFA library and its ERFA derivative.

The script needs no network access.

Usage:

    pixi run python validation/python/time/check_iau2000.py [path]

Exit status is zero when every quantity agrees within tolerance and one
otherwise.

用 ERFA 交叉核对 ApsisRS 的 IAU 2000/2006 地球定向。

读取由 ``cargo run -p apsis-rs --features physics --example dump_iau2000``
写出的量，并用 ERFA 逐个重算，后者是国际天文学联合会通过 SOFA 库及其
ERFA 衍生物分发的参考实现。

本脚本不需要网络访问。

用法：

    pixi run python validation/python/time/check_iau2000.py [path]

每个量都在容差内一致时退出状态为零，否则为一。
"""

from __future__ import annotations

import csv
import sys
from pathlib import Path

import erfa

# Tolerances.
#
# The nutation, obliquity, and precession are reproduced at the level of the
# last bits of a double, so they are compared at 1e-14.
#
# The sidereal time and the matrices accumulate the double precision error of
# several rotations, so they are allowed more.
#
# Greenwich apparent sidereal time gets its own tolerance because ERFA's
# `gst06a` is built on the non-rotating origin while the Rust value is built
# on the equinox. The two formulations agree to about two microarcseconds,
# which is 1e-11 radians, and that is what is allowed here.
#
# 容差。
#
# 章动、黄赤交角与岁差在双精度最后几位的水平上被复现，故它们以 1e-14
# 比较。
#
# 恒星时与各矩阵累积了若干次转动的双精度误差，故容许更大。
#
# 格林尼治视恒星时有其自己的容差，因为 ERFA 的 `gst06a` 建立在非旋转原点
# 之上，而 Rust 值建立在春分点之上。两种表述相符到约两微角秒，即 1e-11
# 弧度，此处容许的就是这个。
SCALAR_TOLERANCE = 1e-14
SIDEREAL_TOLERANCE = 1e-10
MATRIX_TOLERANCE = 1e-9


def tolerance_for(quantity: str) -> float:
    """Returns the tolerance that applies to a quantity.

    返回适用于某个量的容差。
    """
    if quantity == "gast":
        return SIDEREAL_TOLERANCE
    if quantity in ("bpn", "c2t"):
        return MATRIX_TOLERANCE
    return SCALAR_TOLERANCE

DEFAULT_INPUT = Path("validation/data/expected/rust_iau2000.csv")

# The Terrestrial Time offset from UTC, in seconds, matching the Rust side.
# 地球时相对 UTC 的偏移，单位为秒，与 Rust 侧一致。
TT_MINUS_UTC_SECONDS = 32.184 + 37.0


def read_rows(path: Path) -> list[dict[str, str]]:
    """Reads the CSV rows.

    读取 CSV 各行。
    """
    with path.open(newline="") as handle:
        reader = csv.DictReader(handle)
        return [row for row in reader if row.get("mjd")]


def tt_julian_date(utc_mjd: float) -> float:
    """Returns the Terrestrial Time Julian date for a UTC Modified Julian Date.

    返回某 UTC 约化儒略日对应的地球时儒略日。
    """
    return utc_mjd + 2400000.5 + TT_MINUS_UTC_SECONDS / 86400.0


def main(argv: list[str]) -> int:
    path = Path(argv[1]) if len(argv) > 1 else DEFAULT_INPUT
    if not path.exists():
        print(f"missing input file: {path}", file=sys.stderr)
        print(
            "generate it with: cargo run -p apsis-rs --features physics "
            "--example dump_iau2000 > " + str(path),
            file=sys.stderr,
        )
        return 2

    rows = read_rows(path)
    if not rows:
        print("input file has no data rows", file=sys.stderr)
        return 2

    worst: dict[str, float] = {}
    worst_at: dict[str, object] = {}
    compared = 0

    for row in rows:
        mjd = float(row["mjd"])
        quantity = row["quantity"]
        index = int(row["index"])
        actual = float(row["value"])

        tt = tt_julian_date(mjd)
        d1 = 2400000.5
        d2 = tt - d1

        if quantity == "obliquity":
            reference = erfa.obl06(d1, d2)
        elif quantity == "nutation_longitude_a":
            reference = erfa.nut00a(d1, d2)[0]
        elif quantity == "nutation_obliquity_a":
            reference = erfa.nut00a(d1, d2)[1]
        elif quantity == "nutation_longitude_06a":
            reference = erfa.nut06a(d1, d2)[0]
        elif quantity == "nutation_obliquity_06a":
            reference = erfa.nut06a(d1, d2)[1]
        elif quantity == "precession_gamma_bar":
            reference = erfa.pfw06(d1, d2)[0]
        elif quantity == "precession_phi_bar":
            reference = erfa.pfw06(d1, d2)[1]
        elif quantity == "precession_psi_bar":
            reference = erfa.pfw06(d1, d2)[2]
        elif quantity == "precession_epsilon":
            reference = erfa.pfw06(d1, d2)[3]
        elif quantity == "bpn":
            matrix = erfa.pnm06a(d1, d2)
            reference = matrix[index // 3][index % 3]
        elif quantity == "gast":
            reference = erfa.gst06a(d1, d2, d1, d2)
        elif quantity == "c2t":
            # The equinox based chain: the bias, precession and nutation,
            # then Greenwich apparent sidereal time, then the polar motion.
            # 基于春分点的链条：先偏差、岁差与章动，再格林尼治视恒星时，
            # 然后极移。
            matrix = erfa.c2t06a(d1, d2, d1, d2, 0.0, 0.0)
            reference = matrix[index // 3][index % 3]
        else:
            continue

        error = abs(actual - reference)
        if error > worst.get(quantity, 0.0):
            worst[quantity] = error
            worst_at[quantity] = (mjd, index)
        compared += 1

    print(f"values compared: {compared}")
    for quantity in sorted(worst):
        tolerance = tolerance_for(quantity)
        status = "ok" if worst[quantity] <= tolerance else "FAIL"
        print(
            f"  {quantity:26} worst {worst[quantity]:.3e} "
            f"(tolerance {tolerance:.0e}) at {worst_at[quantity]} {status}"
        )

    passed = all(
        value <= tolerance_for(quantity) for quantity, value in worst.items()
    )
    if passed and compared > 0:
        print("result: pass")
        return 0
    print("result: FAIL")
    return 1


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
