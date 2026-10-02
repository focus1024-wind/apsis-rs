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

## 性能

三个基准都是门面的 example。它们在 release 下运行，且除本工作区外不依赖任何
东西，故一次构建只需要编译器本身。下表是 AMD Ryzen 7 8845HS（8 核 16 线程）
上的一次运行；绝对时间随机器变化，应比较同一张表内的比例。方法、每次规模扫描
的全部数据以及每条结论的依据见
[压测方法与报告](docs/design/bench.zh.md)。

**热点路径的单次调用**

| 路径 | 每次调用 |
| --- | --- |
| Hermite 星历插值查询 | 24 ns |
| TEME 到地固系旋转 | 27 ns |
| 开普勒根数转状态 | 38 ns |
| 地固系转大地坐标 | 50 ns |
| 链路预算 C/N0 | 61 ns |
| SGP4 传播（历元后 30 分） | 249 ns |
| 可见性搜索（120 样本） | 7.5 μs |
| 规划 60 个任务（1 资源） | 239 μs |

星历查询比它所代替的那次传播便宜一个数量级，这正是被密集查询的轨道应先建表
再插值的原因。

**随规模的增长。** 每单位成本持平才说明复杂度正确：

- 星座一日传播对星数线性：24 星 8.7 ms，2016 星 0.68 s，每样本约 234 ns，
  增长指数 0.00。
- 可见性搜索在 8 到 128 轨之间恒为每轨约 7.3 μs。
- 拓扑构建约为 O(n 平方)，贪心规划由 O(n 平方) 缓向 O(n)；两者都受算法而非
  实现限制，设计说明给出了原因。

**并行。** 数值内核按卫星切分。2000 星一日传播单线程 671 ms、16 线程 85 ms，
加速 7.9 倍；256 轨的可见性搜索加速 5.3 倍。8 线程每核一线程时效率约八成五；
16 线程每核两线程，收益约为一半。并行结果与串行逐位相同，故加速只换来吞吐，
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

生成的 API 文档会离线渲染数学公式。`docs/vendor/mathjax` 下内置了一份
MathJax，`docs/mathjax.html` 负责加载，`crates/apsis-rs/build.rs` 会把它复制到
文档根，故生成的页面无需任何网络访问。用
`bash scripts/build-docs.sh --workspace --all-features --no-deps` 构建；直接
`cargo doc` 也可以，脚本只是使复制可靠。行间公式写作 `<div>` 包裹的
`$$ ... $$`；外层 div 使 markdown 层不改写其中的 LaTeX。

## 许可

GPL-3.0-or-later，见 `LICENSE`。

`docs/vendor/mathjax` 下内置的 MathJax 为 Apache-2.0，见该目录内的 `LICENSE`。

## 文档

设计说明记录了单个 crate 的读者无法推断的跨 crate 决策：

- [单位与坐标](docs/design/units.zh.md)
- [错误与容差](docs/design/errors.zh.md)
- [层边界](docs/design/layers.zh.md)
- [并行](docs/design/parallel.zh.md)
- [压测方法与报告](docs/design/bench.zh.md)
- [术语与公式约定](docs/design/glossary.zh.md)
- [仿真层](docs/design/sim.zh.md)
