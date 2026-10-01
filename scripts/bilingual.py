#!/usr/bin/env python3
"""Apply a table of English-to-bilingual comment replacements to a file.

The sweep over the repository is a long series of edits that pair an English
comment block with the English paragraph followed by a Chinese one. Writing
each pair twice, as two separate replacements, is error prone, so this
helper takes a list of pairs and rewrites the file with them applied, then
reports how many did not match. A replacement that does not match is almost
always a reflow of the English text, and is worth seeing rather than
skipping.

Usage, from the repository root:

    python3 scripts/bilingual.py crates/orbit/apsis-kepler/src/elements.rs

对文件施加一张「英文到双语注释」的替换表。

对仓库的清扫是一长串编辑，每一次都把一段英文注释块改写为英文段后接中文
段。要把每一对写两遍（作为两次独立替换）容易出错，故此辅助工具接收一张
替换对列表，施加后重写文件，并报告有多少未匹配。未匹配的替换几乎总是
英文文本的重新折行，值得一看而非跳过。

用法（在仓库根下）：

    python3 scripts/bilingual.py crates/orbit/apsis-kepler/src/elements.rs
"""

from __future__ import annotations

import sys
from pathlib import Path


def apply(path: Path, pairs: list[tuple[str, str]]) -> tuple[int, list[str]]:
    """Replaces each pair and returns how many applied with the misses.

    替换每一对，并返回已施加的数量与未匹配者。
    """
    text = path.read_text(encoding="utf-8")
    missed: list[str] = []
    applied = 0
    for old, new in pairs:
        if old in text:
            text = text.replace(old, new, 1)
            applied += 1
        else:
            head = old.strip().splitlines()[0][:56] if old.strip() else "<empty>"
            missed.append(head)
    path.write_text(text, encoding="utf-8")
    return applied, missed
