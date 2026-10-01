#!/usr/bin/env python3
"""Generate the Rust source for the IAU 2000 complementary terms.

The complementary terms are the small periodic additions to the equation of
the equinoxes that the nutation in longitude does not contain. They come
from the same theory as the nutation and are needed for the equation of the
equinoxes to be consistent with the nutation series.

The reference is the ERFA library, a derivative of the IAU SOFA library.

Fetch the reference with:

    curl -O https://raw.githubusercontent.com/liberfa/erfa/master/src/eect00.c

Usage:

    python validation/tools/generate_eect_table.py path/to/eect00.c

生成 IAU 2000 分点方程式补充项的 Rust 源码。

补充项是黄经章动不包含的、分点方程式的小周期附加项。它们来自与章动相同的
理论，为使分点方程式与章动级数一致所必需。

参考实现是 ERFA 库，它是 IAU SOFA 库的衍生物。

获取参考源码：

    curl -O https://raw.githubusercontent.com/liberfa/erfa/master/src/eect00.c

用法：

    python validation/tools/generate_eect_table.py path/to/eect00.c
"""

from __future__ import annotations

import re
import sys
from pathlib import Path


def extract_array(source: str, name: str) -> str:
    """Returns the body of a C array initializer.

    返回一个 C 数组初始化器的正文。
    """
    start = source.index(name + "[] = {")
    brace = source.index("{", start)
    depth = 0
    index = brace
    while index < len(source):
        if source[index] == "{":
            depth += 1
        elif source[index] == "}":
            depth -= 1
            if depth == 0:
                break
        index += 1
    return source[brace : index + 1]


def parse_terms(body: str) -> list[tuple[str, str]]:
    """Returns the multiplier and coefficient groups of each term.

    返回每一项的乘数组与系数组。
    """
    body = re.sub(r"/\*.*?\*/", "", body, flags=re.S)
    return re.findall(r"\{\{([^{}]*)\},\s*([^}]*)\}", body)


def emit(name: str, doc: str, terms: list[tuple[str, str]]) -> None:
    """Prints one Rust constant.

    打印一个 Rust 常量。
    """
    print(doc)
    print(f"pub static {name}: [ComplementaryTerm; {len(terms)}] = [")
    for multipliers, coefficients in terms:
        arguments = ", ".join(value.strip() for value in multipliers.split(","))
        values = ", ".join(value.strip() for value in coefficients.split(","))
        print(
            f"    ComplementaryTerm {{ argument: [{arguments}], "
            f"coefficients: [{values}] }},"
        )
    print("];")
    print()


def main(argv: list[str]) -> int:
    if len(argv) < 2:
        print(__doc__, file=sys.stderr)
        return 2
    source = Path(argv[1]).read_text()
    constant = parse_terms(extract_array(source, "e0"))
    linear = parse_terms(extract_array(source, "e1"))

    print("//! The IAU 2000 complementary terms of the equation of the equinoxes.")
    print("//!")
    print("//! The table is generated from the reference implementation by")
    print("//! `validation/tools/generate_eect_table.py` and is reproduced without")
    print("//! change.")
    print("//!")
    print("//! Source: the MHB2000 model, as distributed with the IAU SOFA library")
    print("//! and implemented by ERFA.")
    print("//!")
    print("//! 分点方程式的 IAU 2000 补充项。")
    print("//!")
    print("//! 该表由 `validation/tools/generate_eect_table.py` 从参考实现生成，")
    print("//! 原样再现，未经改动。")
    print("//!")
    print("//! 出处：MHB2000 模型，随 IAU SOFA 库分发并由 ERFA 实现。")
    print()
    print("use super::ComplementaryTerm;")
    print()
    emit(
        "CONSTANT_TERMS",
        "/// The terms that do not depend on time.\n///\n/// The eight multipliers apply to the arguments `l`, `l'`, `F`, `D`, `Om`,\n/// the mean longitude of Venus, the mean longitude of the Earth, and the\n/// general accumulated precession, in that order.\n///\n/// 不依赖时间的各项。\n///\n/// 八个乘数按 `l`、`l'`、`F`、`D`、`Om`、金星的平黄经、地球的平黄经，以及\n/// 一般累积岁差的次序施加于这些参数。",
        constant,
    )
    emit(
        "LINEAR_TERMS",
        "/// The terms whose amplitude grows linearly with time.\n///\n/// The arguments are the same as for the constant terms.\n///\n/// 幅度随时间线性增长的各项。\n///\n/// 参数与常数项相同。",
        linear,
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
