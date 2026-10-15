# ApsisRS / 拱点

商业航天与卫星互联网领域的 Rust 计算内核。

[English](./README.md) | 简体中文

## 定位

- 纯 Rust、`std`、零第三方 crate 依赖
- 领域边界清晰，按层单向依赖
- 单一 crate `apsis`，每一层是模块，由 crate 根重导出
- 已具备 crates.io 首发条件

## 分层

```text
foundation -> physics -> orbit -> rf -> network -> ops -> sim
```

约束：

- 引用只能向左。下层可引用其上任意层，上层不得引用其下任意层。
  例如 `network` 可以引用 `orbit`（向左），但 `orbit` 不得引用 `network`
  （向右，即反向）。左向引用不限层数，允许跨层，因为把 `orbit` 的值
  传到 `network` 时中间层未必提供中转类型。
- `foundation` 的 `apsis_core` 是全局基准，任何层任何模块均可引用它。
  `apsis_math` 同理，因为线性代数被各层共用。
- 层内以该层的门面模块（`apsis_orbit`、`apsis_rf`、`apsis_sim`）作为整合点。

这套规则由 `scripts/check-layers.sh` 强制，CI 会运行它。

各层内容：

| 层 | 职责 | 模块 |
| --- | --- | --- |
| foundation | 基础类型、单位、数学、宏 | `apsis_core`、`apsis_units`、`apsis_math`、`apsis_macros` |
| physics | 时间、闰秒、坐标、大地基准 | `apsis_time`、`apsis_leap`、`apsis_coord`、`apsis_geodesy`、`apsis_frames` |
| orbit | TLE、SGP4、二体、星历、机动 | `apsis_tle`、`apsis_sgp4`、`apsis_kepler`、`apsis_ephemeris`、`apsis_maneuver`、`apsis_orbit` |
| rf | 射频、天线、链路预算 | `apsis_rf`、`apsis_antenna`、`apsis_doppler`、`apsis_atmosphere`、`apsis_noise`、`apsis_interference`、`apsis_link` |
| network | 星座、覆盖、接入、拓扑、路由 | `apsis_constellation`、`apsis_coverage`、`apsis_access`、`apsis_topology`、`apsis_isl`、`apsis_routing`、`apsis_handover`、`apsis_mesh` |
| ops | 资源、任务、调度、冲突、计划 | `apsis_resource`、`apsis_task`、`apsis_scheduler`、`apsis_conflict`、`apsis_schedule` |
| sim | 仿真引擎、场景、事件、输出 | `apsis_sim_engine`、`apsis_scenario`、`apsis_event`、`apsis_output`、`apsis_sim` |
| facade | crate 根再导出 | crate 根，即 `apsis` |

## 构建

所有 Rust 命令在仓库根目录执行：

```bash
cargo check --workspace
cargo test --workspace --all-features
```

本 crate 通过 feature 选择需要的层，例如：

```toml
[dependencies]
apsis = { path = "crates/apsis", features = ["orbit"] }
```

可选 feature：`foundation`、`physics`、`orbit`、`rf`、`network`、`ops`、`sim`、`full`。
每个层 feature 自动启用其下层。`full` 启用全部层（含 `sim`）；`ops` 等价于除
`sim` 以外的所有层，适合不运行仿真的项目。

## 性能

三个基准都是本 crate 的 example。它们在 release 下运行，且除本工作区外不依赖任何
东西，故一次构建只需要编译器本身。下表是 AMD Ryzen 7 8845HS（8 核 16 线程）
上的一次运行；绝对时间随机器变化，应比较同一张表内的比例。方法、每次规模扫描
的全部数据以及每条结论的依据见
[压测方法与报告](docs/design/bench.zh.md)。

**热点路径的单次调用**

| 路径 | 每次调用 |
| --- | --- |
| Hermite 星历插值查询 | 27 ns |
| TEME 到地固系旋转 | 24 ns |
| 开普勒根数转状态 | 47 ns |
| 地固系转大地坐标 | 50 ns |
| 链路预算 C/N0 | 60 ns |
| SGP4 传播（历元后 30 分） | 265 ns |
| 可见性搜索（120 样本） | 6.4 μs |
| 规划 60 个任务（1 资源） | 240 μs |

星历查询比它所代替的那次传播便宜一个数量级，这正是被密集查询的轨道应先建表
再插值的原因。

**随规模的增长。** 每单位成本持平才说明复杂度正确：

- 星座一日传播对星数线性：24 星 8.1 ms，2016 星 0.68 s，每样本约 235 ns，
  增长指数 0.00。
- 可见性搜索在 8 到 128 轨之间恒为每轨约 7 μs。
- 拓扑构建约为 O(n 平方)，贪心规划由 O(n 平方) 缓向 O(n)；两者都受算法而非
  实现限制，设计说明给出了原因。

**并行。** 数值内核按卫星切分。2000 星一日传播单线程 669 ms、16 线程 68 ms，
加速 9.9 倍；256 轨的可见性搜索加速 4.9 倍。8 线程每核一线程时效率约九成二；
16 线程每核两线程，效率降至约六成。并行结果与串行逐位相同，故加速只换来吞吐，
从不改变数值。

## 约定

- toolchain：`stable`，`edition 2024`，`rust-version 1.85`
- 默认禁止任何第三方 crate，包括开发依赖与 `build.rs`
- 库代码不使用 `unwrap()` / `expect()` / `panic!` / `unsafe`
  - 外部输入的失败一律走 `Result` / `Option`，不用 `unwrap` / `expect` 转换
  - 允许对**已守卫的内部不变量**用 `expect("…")` 断言不可能的分支（如刚 `push`
    后的 `last()`、前置 `is_none()` 已检查后的取值）。这些断言不处理不可信
    输入，而是把"若此处为空则是编码错误"写成可读的失败信息；全库共 13 处，
    逐处均有前置守卫。
- 公共 API 必须有文档注释，注释自包含

## 目录

```text
apsis-rs/
├── Cargo.toml
├── crates/
│   └── apsis/              # 唯一的 crate
│       ├── build.rs
│       ├── examples/       # 对拍导出与跨层集成检查
│       ├── src/
│       │   ├── lib.rs      # 按 feature 重导出
│       │   ├── foundation/ # 每层一个目录
│       │   ├── physics/
│       │   ├── orbit/
│       │   ├── rf/
│       │   ├── network/
│       │   ├── ops/
│       │   └── sim/
│       └── tests/          # 镜像各层的集成测试
├── scripts/                # 仓库检查脚本
├── validation/             # Python 对拍验证
└── docs/design/            # 设计文档
```

测试在 `crates/apsis/tests/` 下，是名为 `integration` 的单个集成测试 target，
镜像 `src/` 的层与模块结构；只经公开 API 访问本 crate，故 crate 根缺少某个
重导出时会在此失败。
跨层集成检查与零依赖基准测试是本 crate 的 example：

```bash
# 跨层集成检查（CI 会执行）
cargo run --features network --example integrate_network
cargo run --features ops --example integrate_ops
cargo run --features sim --example integrate_sim

# 热点路径基准（release 下运行）
cargo run --release --features full --example bench

# 规模扫描：输入规模递增，观察每单位工作量的增长趋势
cargo run --release --features full --example bench_scale

# 并行扫描：同一批工作量在 1/2/4/8/16 线程下的加速比与效率
cargo run --release --features full --example bench_parallel
```

生成的 API 文档会离线渲染数学公式。`docs/vendor/mathjax` 下内置了一份
MathJax，`docs/mathjax.html` 负责加载，`crates/apsis/build.rs` 会把它复制到
文档根，故生成的页面无需任何网络访问。用
`bash scripts/build-docs.sh --workspace --all-features --no-deps` 构建；直接
`cargo doc` 也可以，脚本只是使复制可靠。行间公式写作 `<div>` 包裹的
`$$ ... $$`；外层 div 使 markdown 层不改写其中的 LaTeX。

## 许可

GPL-3.0-or-later，见 `LICENSE`。

`docs/vendor/mathjax` 下内置的 MathJax 为 Apache-2.0，见该目录内的 `LICENSE`。

## 文档

设计说明记录了单个模块的读者无法推断的跨层决策：

- [单位与坐标](docs/design/units.zh.md)
- [错误与容差](docs/design/errors.zh.md)
- [层边界](docs/design/layers.zh.md)
- [并行](docs/design/parallel.zh.md)
- [压测方法与报告](docs/design/bench.zh.md)
- [术语与公式约定](docs/design/glossary.zh.md)
- [仿真层](docs/design/sim.zh.md)
