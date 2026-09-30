# ApsisRS / 拱点

商业航天与卫星互联网领域的 Rust 计算内核。

[English](./README.md) | 简体中文

## 定位

- 纯 Rust、`std`、零第三方 crate 依赖
- 领域边界清晰，按层单向依赖
- 对外门面 crate 名为 `apsis-rs`，库名为 `apsis`
- 当前阶段所有 crate 均 `publish = false`，不执行发布

## 分层

```text
foundation -> physics -> orbit -> rf -> network -> ops -> sim -> apsis-rs
```

约束：

- 依赖只能从左向右。下层可依赖其上任意层，上层不得依赖其下任意层。
  例如 `network` 可以依赖 `orbit`（向左），但 `orbit` 不得依赖 `network`
  （向右，即反向）。左向依赖不限层数，允许跨层，因为把 `orbit` 的值
  传到 `network` 时中间层未必提供中转类型。
- `foundation` 的 `apsis-core` 是全局基准，任何层任何 crate 均可依赖它。
  `apsis-math` 同理，因为线性代数被各层共用。
- 层内以该层的门面 crate（`apsis-orbit`、`apsis-rf`、`apsis-sim`）作为整合点。

这套规则由 `scripts/check-layers.sh` 强制，CI 会运行它。

各层内容：

| 层 | 职责 | crates |
| --- | --- | --- |
| foundation | 基础类型、单位、数学、宏 | `apsis-core`、`apsis-units`、`apsis-math`、`apsis-macros` |
| physics | 时间、闰秒、坐标、大地基准 | `apsis-time`、`apsis-leap`、`apsis-coord`、`apsis-geodesy`、`apsis-frames` |
| orbit | TLE、SGP4、二体、星历、机动 | `apsis-tle`、`apsis-sgp4`、`apsis-kepler`、`apsis-ephemeris`、`apsis-maneuver`、`apsis-orbit` |
| rf | 射频、天线、链路预算 | `apsis-rf`、`apsis-antenna`、`apsis-doppler`、`apsis-atmosphere`、`apsis-noise`、`apsis-interference`、`apsis-link` |
| network | 星座、覆盖、接入、拓扑、路由 | `apsis-constellation`、`apsis-coverage`、`apsis-access`、`apsis-topology`、`apsis-isl`、`apsis-routing`、`apsis-handover`、`apsis-mesh` |
| ops | 资源、任务、调度、冲突、计划 | `apsis-resource`、`apsis-task`、`apsis-scheduler`、`apsis-conflict`、`apsis-schedule` |
| sim | 仿真引擎、场景、事件、输出 | `apsis-sim-engine`、`apsis-scenario`、`apsis-event`、`apsis-output`、`apsis-sim` |
| facade | 对外门面 | `apsis-rs` |

## 构建

所有 Rust 命令在仓库根目录执行：

```bash
cargo check --workspace
cargo test -p apsis-core -p apsis-units -p apsis-math
```

`apsis-rs` 门面通过 feature 选择需要的层，例如：

```toml
[dependencies]
apsis-rs = { path = "...", features = ["orbit"] }
```

可选 feature：`foundation`、`physics`、`orbit`、`rf`、`network`、`ops`、`sim`、`full`。
每个层 feature 自动启用其下层。`full` 启用全部层（含 `sim`）；`ops` 等价于除
`sim` 以外的所有层，适合不运行仿真的项目。

## 约定

- toolchain：`stable`，`edition 2024`，`rust-version 1.85`
- 默认禁止任何第三方 crate，包括开发依赖与 `build.rs`
- 库代码不使用 `unwrap()` / `expect()` / `panic!` / `unsafe`
  - 外部输入的失败一律走 `Result` / `Option`，不用 `unwrap` / `expect` 转换
  - 允许对**已守卫的内部不变量**用 `expect("…")` 断言不可能的分支（如刚 `push`
    后的 `last()`、前置 `is_none()` 已检查后的取值）。这些断言不处理不可信
    输入，而是把"若此处为空则是编码错误"写成可读的失败信息；全库共 12 处，
    逐处均有前置守卫。
- 公共 API 必须有文档注释，注释自包含

## 目录

```text
apsis-rs/
├── Cargo.toml
├── crates/          # Rust 工作区成员
│   └── apsis-rs/
│       ├── examples/   # 对拍导出与跨层集成检查
│       └── ...
├── scripts/         # 仓库检查脚本
├── validation/      # Python 对拍验证
└── docs/design/     # 设计文档
```

测试在各 crate 的 `tests/` 目录，为集成测试，只通过公共 API 访问被测 crate。
跨层集成检查与零依赖基准测试是 `apsis-rs` 的 example：

```bash
# 跨层集成检查（CI 会执行）
cargo run -p apsis-rs --features network --example integrate_network
cargo run -p apsis-rs --features ops --example integrate_ops
cargo run -p apsis-rs --features sim --example integrate_sim

# 热点路径基准（release 下运行）
cargo run --release -p apsis-rs --features full --example bench

# 规模扫描：输入规模递增，观察每单位工作量的增长趋势
cargo run --release -p apsis-rs --features full --example bench_scale

# 并行扫描：同一批工作量在 1/2/4/8/16 线程下的加速比与效率
cargo run --release -p apsis-rs --features full --example bench_parallel
```

## 许可

GPL-3.0-or-later，见 `LICENSE`。
