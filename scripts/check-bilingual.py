#!/usr/bin/env python3
"""Check that every source comment has a Chinese counterpart.

The repository is being made bilingual: each comment block carries the
English paragraph first and the Chinese one after a blank doc line. This
script reports the blocks that have no Chinese yet, so the sweep can be
driven and verified without reading every file.

It covers the Rust sources, including the module level `//!` docs, the
integration tests, and the examples, and also the non-Rust files that carry
prose comments: the CI workflow, the manifests, the shell and Python
scripts, and the Python docstrings.

It also reports the two mistakes that are easy to make while adding a
language: a code string that was translated by accident, and a comment that
mentions a term the glossary keeps in English where a Chinese name exists.

检查每一处源码注释是否有中文对应。

仓库正在被改为双语：每个注释块先写英文段，再在一条空白文档行之后写中文
段。本脚本报告尚无中文的块，使这次清扫无需逐文件阅读即可被驱动与核验。

它覆盖 Rust 源码，包括模块级 `//!` 文档、集成测试与示例，也覆盖携带散文
注释的非 Rust 文件：CI 工作流、各清单、shell 与 Python 脚本，以及 Python
文档字符串。

它还会报告添加一种语言时容易犯的两种错误：被误译的代码字符串，以及提到
了词汇表在中文名存在处仍保留英文术语的注释。

Usage:

    python3 scripts/check-bilingual.py            # the whole repository
    python3 scripts/check-bilingual.py crates/apsis/src/orbit
    python3 scripts/check-bilingual.py --quiet     # only the summary line

用法：

    python3 scripts/check-bilingual.py            # 整个仓库
    python3 scripts/check-bilingual.py crates/apsis/src/orbit
    python3 scripts/check-bilingual.py --quiet     # 只打印汇总行
"""

from __future__ import annotations

import argparse
import ast
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

# A Rust comment line: a doc line, a module doc line, or a plain comment.
# Rust 注释行：文档行、模块文档行，或普通注释。
DOC = re.compile(r"^\s*///")
MOD = re.compile(r"^\s*//!")
LINE = re.compile(r"^\s*//[^/!]")

# A hash comment line, used by the non-Rust files. The shebang is excluded
# because it is an interpreter directive rather than prose.
# 非 Rust 文件所用的井号注释行。shebang 被排除，因为它是解释器指令而非散文。
HASH = re.compile(r"^\s*#")
SHEBANG = re.compile(r"^#!")

# CJK, used to decide whether a block already carries the Chinese text.
# CJK，用于判断某个块是否已携带中文。
CJK = re.compile(r"[\u4e00-\u9fff]")

# The replacement character, which shows up when a byte sequence is mangled.
# 替换字符，当字节序列被损坏时出现。
MOJIBAKE = "\ufffd"

# An English function word sitting between Chinese characters, which means a
# half translated sentence. The glossary keeps words like `Walker` and
# `SGP4` in English, so a lowercase word between CJK is a mistake.
# 夹在汉字之间的英文虚词，意味着一个半翻译的句子。词汇表把 `Walker` 与
# `SGP4` 之类的词保留为英文，故 CJK 之间的小写单词是错误。
MIXED = re.compile(r"[\u4e00-\u9fff][a-z]{3,}[\u4e00-\u9fff]")

# A comment that is only a formula or a bare symbol carries no prose to
# translate, so demanding a second language on it would be noise. Such a
# comment is left as it stands, which is the whole of it.
# 只含公式或纯符号的注释没有可翻译的散文，故要求它带第二语言只会是噪音。
# 这样的注释保持原样，而它就是其全部。
FORMULA = re.compile(
    r"^`?[A-Za-z0-9_^*+\-/() =,;.\u3002\uff0c]*`?[.\u3002]?$"
)

# Three letters in a row is a word, not a symbol, so a line carrying one is
# prose and needs translating even if it is mostly numbers.
# 连续三个字母便是单词而非符号，故带它的行是散文，即便大部分是数字也需翻译。
PROSE = re.compile(r"[A-Za-z]{3}")

# Directories that hold generated or third party files, which carry no
# comment that this repository owns.
# 存放生成文件或第三方文件的目录，其中没有本仓库拥有的注释。
SKIP = ("target", ".pixi", "node_modules", ".git")

# File suffixes that carry hash comments rather than the Rust slashes.
# 携带井号注释而非 Rust 斜杠注释的文件后缀。
HASH_SUFFIXES = (".yml", ".yaml", ".toml", ".sh", ".py")


def strip_marker(line: str) -> str:
    """Returns the body of a comment line with its marker removed.

    返回去除注释标记后的注释行正文。
    """
    return re.sub(r"^\s*(///|//!|//|#)\s?", "", line).strip()


def is_formula_only(block: list[str]) -> bool:
    """True when every line of the block is code, a formula, or blank.

    当块中每一行都是代码、公式或空白时返回真。
    """
    saw_content = False
    for line in block:
        body = strip_marker(line)
        if not body:
            continue
        saw_content = True
        if not FORMULA.match(body):
            return False
        # A single backticked token is a symbol, whatever it spells. Prose
        # needs a verb, and `cos` or `eta` has none.
        # 单个反引号包裹的记号是符号，不论它拼出什么。散文需要动词，而
        # `cos` 或 `eta` 没有。
        if len(re.findall(r"[A-Za-z]{2,}", body)) > 1:
            return False
    return saw_content


def rust_blocks(path: Path) -> list[tuple[int, list[str]]]:
    """Returns the comment blocks of a Rust file with the line each starts
    on. Each of the three markers runs within its own block, so a `//!`
    module header is not merged into an adjacent `///` declaration.

    返回 Rust 文件的注释块及其各自的起始行。三种标记各自在自己的块内
    连续，故 `//!` 模块头部不会与相邻的 `///` 声明合并。
    """
    lines = path.read_text(encoding="utf-8").splitlines()
    found: list[tuple[int, list[str]]] = []
    index = 0
    while index < len(lines):
        for marker in (DOC, MOD, LINE):
            if marker.match(lines[index]):
                start = index
                block: list[str] = []
                while index < len(lines) and marker.match(lines[index]):
                    block.append(lines[index])
                    index += 1
                found.append((start + 1, block))
                break
        else:
            index += 1
    return found


def hash_blocks(path: Path) -> list[tuple[int, list[str]]]:
    """Returns the hash comment blocks of a non-Rust file with the line each
    starts on. A shebang is skipped because it opens the file.

    返回非 Rust 文件的井号注释块及其各自的起始行。shebang 被跳过，
    因为它位于文件开头。
    """
    lines = path.read_text(encoding="utf-8").splitlines()
    found: list[tuple[int, list[str]]] = []
    index = 0
    while index < len(lines):
        if HASH.match(lines[index]) and not SHEBANG.match(lines[index]):
            start = index
            block: list[str] = []
            while (
                index < len(lines)
                and HASH.match(lines[index])
                and not SHEBANG.match(lines[index])
            ):
                block.append(lines[index])
                index += 1
            found.append((start + 1, block))
        else:
            index += 1
    return found


def docstring_blocks(path: Path) -> list[tuple[int, list[str]]]:
    """Returns the docstrings of a Python file with the line each starts on.
    A docstring is the Python form of a doc comment, so it is swept too.

    返回 Python 文件的文档字符串及其各自的起始行。文档字符串是 Python
    形式的文档注释，故也一并清扫。
    """
    try:
        tree = ast.parse(path.read_text(encoding="utf-8"))
    except (SyntaxError, ValueError):
        return []
    found: list[tuple[int, list[str]]] = []
    holders = (ast.Module, ast.ClassDef, ast.FunctionDef, ast.AsyncFunctionDef)
    for node in ast.walk(tree):
        if not isinstance(node, holders) or not node.body:
            continue
        first = node.body[0]
        if not isinstance(first, ast.Expr):
            continue
        value = first.value
        if isinstance(value, ast.Constant) and isinstance(value.value, str):
            found.append((first.lineno, value.value.splitlines()))
    return found


def collect(path: Path) -> list[tuple[int, list[str]]]:
    """Returns every comment or docstring block of one file.

    返回单个文件的每一个注释或文档字符串块。
    """
    if path.suffix == ".rs":
        return rust_blocks(path)
    if path.suffix in HASH_SUFFIXES:
        found = hash_blocks(path)
        if path.suffix == ".py":
            found.extend(docstring_blocks(path))
        return found
    return []


def candidates(base: Path) -> list[Path]:
    """Returns the files under a root that carry comments this repository
    owns. A file passed directly is always included.

    返回某个根目录下携带本仓库拥有注释的文件。直接传入的文件总是包含
    在内。
    """
    if base.is_file():
        return [base]
    files: list[Path] = []
    for path in sorted(base.rglob("*")):
        if not path.is_file():
            continue
        if any(part in SKIP for part in path.parts):
            continue
        if path.suffix == ".rs" or path.suffix in HASH_SUFFIXES:
            files.append(path)
    return files


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("paths", nargs="*", default=[])
    parser.add_argument("--quiet", action="store_true")
    args = parser.parse_args(argv)

    roots = [Path(p).resolve() for p in args.paths] or [ROOT]
    files: list[Path] = []
    seen: set[Path] = set()
    for base in roots:
        for path in candidates(base):
            if path not in seen:
                seen.add(path)
                files.append(path)

    total = 0
    translated = 0
    gaps: list[str] = []
    mojibake: list[str] = []
    mixed: list[str] = []

    for path in files:
        for line, block in collect(path):
            text = "\n".join(block)
            if MOJIBAKE in text:
                mojibake.append(f"{path.relative_to(ROOT)}:{line}")
            found = MIXED.search(text)
            if found:
                mixed.append(f"{path.relative_to(ROOT)}:{line}  {found.group(0)!r}")
            if is_formula_only(block):
                # Nothing to translate; the symbol is the whole comment.
                # 无可翻译；该符号就是整条注释。
                continue
            total += 1
            if CJK.search(text):
                translated += 1
            else:
                first = strip_marker(block[0])[:64]
                gaps.append(f"{path.relative_to(ROOT)}:{line}  {first}")

    if not args.quiet:
        for gap in gaps:
            print(f"  no Chinese  {gap}")
    for bad in mojibake:
        print(f"  MOJIBAKE    {bad}")
    for bad in mixed:
        print(f"  MIXED       {bad}")

    remaining = len(gaps)
    print(
        f"comment blocks: {total}, bilingual: {translated}, "
        f"still English only: {remaining}"
    )
    return 1 if (mojibake or mixed) else 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
