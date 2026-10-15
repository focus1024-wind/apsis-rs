#!/usr/bin/env bash
#
# Builds the API documentation with the vendored MathJax in place.
#
# `cargo doc` injects the MathJax loader through `.cargo/config.toml` when
# `RUSTDOCFLAGS` is unset, and `crates/apsis/build.rs` copies the vendored
# file into the documentation root. That build script only reruns when its
# inputs change, so removing `target/doc` on its own leaves the file missing.
# This script runs the copy itself, which makes `bash scripts/build-docs.sh`
# the reliable way to produce documentation that renders math offline.
#
# Any argument is passed to `cargo doc`, for example:
#
#     scripts/build-docs.sh --workspace --all-features --no-deps
#
# 构建 API 文档，并使内置的 MathJax 就位。
#
# 未设置 `RUSTDOCFLAGS` 时，`cargo doc` 会经 `.cargo/config.toml` 注入 MathJax
# 加载器，而 `crates/apsis/build.rs` 会把内置文件复制到文档根。该构建脚本
# 只在其输入变化时重跑，故仅删除 `target/doc` 而不改输入会使文件缺失。本脚本
# 自行执行该复制，使 `bash scripts/build-docs.sh` 成为生成可离线渲染数学的
# 文档的可靠方式。
#
# 任何参数都会传给 `cargo doc`，例如：
#
#     scripts/build-docs.sh --workspace --all-features --no-deps

set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

cargo doc "$@"

# The target directory can be moved, so ask cargo rather than assuming it.
# 目标目录可能被移动，故问 cargo 而非假定。
target="$(cargo metadata --format-version 1 --no-deps \
    | python3 -c 'import json, sys; print(json.load(sys.stdin)["target_directory"])')"
destination="$target/doc/vendor/mathjax"

mkdir -p "$destination"
cp docs/vendor/mathjax/* "$destination/"

echo "documentation built at $target/doc with the vendored MathJax in place"
