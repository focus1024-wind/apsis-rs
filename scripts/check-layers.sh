#!/usr/bin/env bash
#
# Checks the layer dependency rule stated in README.md.
#
# The crate is organized into dependency ordered layers:
#
#     foundation -> physics -> orbit -> rf -> network -> ops -> sim
#
# Every module lives at `crates/apsis/src/<layer>/<module>`, and the
# integration tests mirror that at `crates/apsis/tests/<layer>/<module>`. A
# module or a test may reference any module in a layer to its left through a
# `crate::apsis_<name>` or `apsis::apsis_<name>` path, but never one to its
# right. This script reads every module directory of both trees, works out
# which layer each referenced module belongs to, and fails if any reference
# points to the right.
#
# `crate::apsis_core` and `crate::apsis_math` are exempt as global
# prerequisites: they sit in `foundation` and every layer may use them, which
# the left-only rule already permits, so no special case is needed for them.
#
# Usage:
#
#     scripts/check-layers.sh
#
# Exit status is zero when the rule holds and one otherwise.
#
# 检查 README.md 所述的分层依赖规则。
#
# 本 crate 组织为按依赖排序的各层：
#
#     foundation -> physics -> orbit -> rf -> network -> ops -> sim
#
# 每个模块位于 `crates/apsis/src/<layer>/<module>`，集成测试在
# `crates/apsis/tests/<layer>/<module>` 下镜像该结构。模块或测试可以经
# `crate::apsis_<name>` 或 `apsis::apsis_<name>` 路径引用其左侧层中的任何
# 模块，但绝不引用其右侧的。本脚本读取两棵树的每个模块目录，判定每个被
# 引用模块属于哪一层，并在任何引用指向右侧时失败。
#
# `crate::apsis_core` 与 `crate::apsis_math` 作为全局前提被豁免：它们位于
# `foundation`，每一层都可使用它们，而仅向左的规则本已允许，故它们无需
# 特殊处理。
#
# 用法：
#
#     scripts/check-layers.sh
#
# 规则成立时退出状态为零，否则为一。

set -euo pipefail

# The layer order, left to right. A reference to a layer with a lower index
# is allowed; to a higher index is a reverse dependency and fails.
# 层的次序，从左到右。引用到下标更低的层是允许的；到更高的下标即反向
# 依赖，会失败。
LAYERS=(foundation physics orbit rf network ops sim)

# The root of the workspace, taken from this script's location so that the
# check works from any working directory.
# 工作区的根，取自本脚本的位置，使检查可在任何工作目录下工作。
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

# The roots that hold layer directories: the crate sources and the
# integration tests, which mirror the layers.
# 包含层目录的两个根：crate 源码与镜像各层的集成测试。
source_root="crates/apsis/src"
tests_root="crates/apsis/tests"

# Returns the index of a layer name, or an empty string when the name is not
# a layer.
# 返回某层名的下标；该名不是层时返回空串。
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

# Returns the layer that holds a module, or an empty string when the name is
# not a module of this crate.
# 返回包含某模块的层；该名不是本 crate 的模块时返回空串。
module_layer() {
    local module="$1"
    local directory
    directory="$(find "$source_root" -mindepth 2 -maxdepth 2 -type d -name "$module" 2>/dev/null | head -1)"
    [ -z "$directory" ] && return 0
    echo "$directory" | cut -d/ -f4
}

failures=0
references=0

for directory in $(find "$source_root" "$tests_root" -mindepth 2 -maxdepth 2 -type d | sort); do
    # The layer directory is the fourth path component, as in
    # `crates/apsis/<root>/<layer>/<module>`.
    # 层目录是路径的第四段，如 `crates/apsis/<root>/<layer>/<module>`。
    layer="$(echo "$directory" | cut -d/ -f4)"
    source_index="$(layer_index "$layer")"
    [ -z "$source_index" ] && continue
    module="$(basename "$directory")"

    # Every `crate::apsis_<name>` or `apsis::apsis_<name>` path is a reference
    # to another module. The module itself resolves to the same layer and is
    # skipped by the index comparison below.
    # 每个 `crate::apsis_<name>` 或 `apsis::apsis_<name>` 路径都是对另一模块
    # 的引用。模块自身解析到同一层，会被下面的下标比较跳过。
    for reference in $(grep -rhoE --include='*.rs' '(crate|apsis)::apsis_[a-z0-9_]+' "$directory" 2>/dev/null | sed -E 's/^(crate|apsis):://' | sort -u || true); do
        dependency_layer="$(module_layer "$reference")"
        [ -z "$dependency_layer" ] && continue
        dependency_index="$(layer_index "$dependency_layer")"
        [ -z "$dependency_index" ] && continue

        if [ "$dependency_index" -gt "$source_index" ]; then
            echo "reverse dependency: $layer -> $dependency_layer ($module -> $reference)" >&2
            failures=$((failures + 1))
        fi
        references=$((references + 1))
    done
done

if [ "$failures" -gt 0 ]; then
    echo "layer check failed: $failures reverse reference(s)" >&2
    exit 1
fi

echo "layer check passed: $references reference edge(s), none reversed"
