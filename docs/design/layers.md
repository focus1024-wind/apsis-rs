# 分层边界

本文件记录工作区的分层规则，以及为什么某些跨层依赖是允许的。README 的
"分层"一节是规则本身，这里是它的判定标准与边界情况。

## 规则

层自左向右排列：

```text
foundation -> physics -> orbit -> rf -> network -> ops -> sim -> apsis-rs
```

一条依赖边从 crate A 指向 crate B。把 A 所在的层与 B 所在的层分别映射为
下标，规则是：

```text
index(B) <= index(A)     允许
index(B) >  index(A)     违规（反向依赖）
```

即**只能向左依赖，不能向右依赖**。`scripts/check-layers.sh` 对工作区每个
清单执行这条判定，CI 会运行它。

## 允许跨层

规则只看方向，不看距离。从 `network` 直接依赖 `orbit` 是允许的，尽管中间
隔着 `rf`：`orbit` 在 `network` 左边，方向正确。

允许向左跨层是有意的。若强制只能依赖紧邻的左层，会产生两类问题：

- **中转类型不存在。** 把 `orbit` 层的位置传给 `network` 层时，`rf` 层并
  非必然提供一个承载该位置的类型。强行要求经过 `rf` 只会制造只是转发的
  空壳类型。
- **无谓的耦合。** `network` 的星座计算需要 `apsis-kepler` 的开普勒要素，
  它并不需要 `rf`。若禁止跨层，`network` 就得依赖 `rf` 只为拿到一个它不
  使用的类型。

因此规则约束的是**方向**，因为方向才对应真正的分层语义：上层的概念可以
引用下层的概念，反之则说明分层错了。

## 全局基准

`apsis-core`（通用错误、常量、单位标记）与 `apsis-math`（`Vec3`、
`Mat3`、`Quat`）被各层共用。它们本身在 `foundation` 中，所以"只向左依赖"
已经允许任何层使用它们，无需特例。

这与"层内基准"不同：`apsis-core` 也是 `foundation` 内的基准，但它的可用
范围是整个工作区，而不只是同层。

**只有真正使用的依赖才声明。** `apsis-core` 与 `apsis-math` 可以被任何
crate 使用，但一个不引用它们的 crate 不应把它们写进清单：未使用的依赖会
拖慢编译、让 `cargo tree` 与分层检查产生噪声，并让"谁依赖什么"失真。
需要时再加，是一条随时可走的单行改动。`scripts/check-layers.sh` 报告的
边数因此是真实的使用关系，而不是一份预留清单。

## 层内门面

每层以一个门面 crate 作为整合点：

| 层 | 门面 |
| --- | --- |
| orbit | `apsis-orbit` |
| rf | `apsis-rf` |
| sim | `apsis-sim` |
| 对外 | `apsis-rs` |

门面把同层的能力组合成少量类型，并再导出同层的其余 crate。`foundation`、
`physics`、`network`、`ops` 没有单独门面：它们的 crate 各自独立有用，且
`apsis-rs` 会把它们一并再导出。

## 边界情况

- **facade `apsis-rs` 不在层序列中。** 它依赖全部层，因为它就是出口。
  `scripts/check-layers.sh` 通过层目录名判定，`crates/apsis-rs` 不是
  `crates/<layer>/<crate>` 形状，因此被跳过。
- **`dev-dependencies` 也计入检查。** 一条测试里的反向依赖同样说明分层
  错了，应当修正代码而不是放宽测试。
- **门面再导出不构成新依赖。** `apsis-rs` 把 `apsis-orbit` 再导出，是
  门面行为，不是某层对 `orbit` 的依赖。

## 检查

```bash
bash scripts/check-layers.sh
```

脚本输出检查过的依赖边数，并在发现反向依赖时逐条列出后以退出码 1 结束。
对当前工作区，它检查 104 条边，全部方向正确。
