#!/usr/bin/env bash
#
# Checks the layer dependency rule stated in README.md.
#
# The workspace is organized into dependency ordered layers:
#
#     foundation -> physics -> orbit -> rf -> network -> ops -> sim
#
# A crate may depend on any crate in a layer to its left, but never on one to
# its right. This script reads every workspace manifest, works out which
# layer each dependency belongs to, and fails if any dependency points to the
# right.
#
# `apsis-core` and `apsis-math` are exempt as global prerequisites: they sit
# in `foundation` and every layer may use them, which the left-only rule
# already permits, so no special case is needed for them.
#
# Usage:
#
#     scripts/check-layers.sh
#
# Exit status is zero when the rule holds and one otherwise.
#
# 检查 README.md 所述的分层依赖规则。
#
# 工作区组织为按依赖排序的各层：
#
#     foundation -> physics -> orbit -> rf -> network -> ops -> sim
#
# 一个 crate 可以依赖其左侧层中的任何 crate，但绝不依赖其右侧的。本脚本
# 读取每个工作区清单，判定每个依赖属于哪一层，并在任何依赖指向右侧时
# 失败。
#
# `apsis-core` 与 `apsis-math` 作为全局前提被豁免：它们位于
# `foundation`，每一层都可使用它们，而仅向左的规则本已允许，故它们无需
# 特殊处理。
#
# 用法：
#
#     scripts/check-layers.sh
#
# 规则成立时退出状态为零，否则为一。

set -euo pipefail

# The layer order, left to right. A dependency to a layer with a lower index
# is allowed; to a higher index is a reverse dependency and fails.
# 层的次序，从左到右。依赖到下标更低的层是允许的；到更高的下标即反向
# 依赖，会失败。
LAYERS=(foundation physics orbit rf network ops sim)

# The root of the workspace, taken from this script's location so that the
# check works from any working directory.
# 工作区的根，取自本脚本的位置，使检查可在任何工作目录下工作。
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

# Returns the index of a layer name, or an empty string when the name is not
# a layer. A crate outside these directories is not part of the layer scheme
# and is skipped rather than reported.
# 返回某层名的下标；该名不是层时返回空串。位于这些目录之外的 crate 不属于
# 分层方案，会被跳过而非报告。
layer_index() {
    local name="$1"
    local index=0
    for layer in "${LAYERS[@]}"; do
        if [ "$layer" = "$name" ]; then
            echo "$index"
            return 0
        fi
        index=$((index + 1))
    done
    echo ""
}

failures=0
checked=0

for manifest in $(find crates -name Cargo.toml | sort); do
    directory="$(dirname "$manifest")"
    # The layer directory is the second path component, as in
    # `crates/<layer>/<crate>/Cargo.toml`.
    # 层目录是路径的第二段，如 `crates/<layer>/<crate>/Cargo.toml`。
    layer="$(echo "$directory" | cut -d/ -f2)"
    source_index="$(layer_index "$layer")"
    [ -z "$source_index" ] && continue

    # Every `apsis-...` name in the manifest is a dependency or a package
    # name. The package name is the crate itself, which resolves to the same
    # layer and is skipped by the index comparison below.
    # 清单中每个 `apsis-...` 名称要么是依赖，要么是包名。包名即该 crate
    # 本身，它解析到同一层，会被下面的下标比较跳过。
    for dependency in $(grep -oE 'apsis-[a-z-]+' "$manifest" | sort -u); do
        dependency_directory="$(find crates -maxdepth 2 -type d -name "$dependency" 2>/dev/null | head -1)"
        [ -z "$dependency_directory" ] && continue
        dependency_layer="$(echo "$dependency_directory" | cut -d/ -f2)"
        dependency_index="$(layer_index "$dependency_layer")"
        [ -z "$dependency_index" ] && continue

        if [ "$dependency_index" -gt "$source_index" ]; then
            echo "reverse dependency: $layer -> $dependency_layer ($(basename "$directory") -> $dependency)" >&2
            failures=$((failures + 1))
        fi
        checked=$((checked + 1))
    done
done

if [ "$failures" -gt 0 ]; then
    echo "layer check failed: $failures reverse dependency(ies)" >&2
    exit 1
fi

echo "layer check passed: $checked dependency edge(s), none reversed"
