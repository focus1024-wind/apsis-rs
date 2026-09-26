# ApsisRS / 拱点

商业航天与卫星互联网领域的 Rust 计算内核。

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

- 依赖只能从左向右
- 禁止跨层跳跃，例如 `orbit` 不得直接依赖 `network`
- 禁止同层平级依赖

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
- 公共 API 必须有文档注释，注释自包含

## 目录

```text
apsis-rs/
├── Cargo.toml
├── crates/          # Rust 工作区成员
├── tests/           # 集成测试
├── benches/         # 基准测试
├── validation/      # Python 对拍验证
└── docs/design/     # 设计文档
```

## 许可

MIT OR Apache-2.0，见 `LICENSE-MIT` 与 `LICENSE-APACHE`。
