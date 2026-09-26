# 错误与容差策略

本文件记录跨 crate 的错误表达与数值容差约定。单个 crate 的错误枚举见其
`error.rs` 的文档。

## 错误表达

### 领域错误

每个有领域特定失败的 crate 定义**自己的**错误类型，命名与结构一致：

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum FooErrorKind {
    /// 每个变体描述一个调用方可能想要区分的具体条件。
    ReversedInterval,
    DuplicateResource,
    // ...
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FooError {
    kind: FooErrorKind,
    detail: Option<String>,
}

impl FooError {
    pub fn new(kind: FooErrorKind) -> Self;
    pub fn with_detail(self, detail: impl Into<String>) -> Self;
    pub fn kind(&self) -> FooErrorKind;
    pub fn detail(&self) -> Option<&str>;
}

impl fmt::Display for FooError { /* "kind" 或 "kind: detail" */ }
impl core::error::Error for FooError {}
```

约定：

- **`ErrorKind` 标注 `#[non_exhaustive]`**，这样新增一个失败类别不会破坏
  下游的匹配。当前 15 个 `ErrorKind` 全部标注。
- **`kind` 可精确匹配，`detail` 供人阅读。** 调用方按 `kind` 写恢复逻辑，
  把 `detail` 记入日志。`detail` 不参与匹配，因此其措辞可自由改动。
- **`Display` 输出 `kind`，有 `detail` 时追加 `": detail"`。** 这样日志行
  总是以类别开头。
- **实现 `core::error::Error`**，以便错误可以嵌套为 `source`。当前 15 个
  错误类型全部实现。
- **不返回裸 `String` 或 `Box<dyn Error>` 作为公共错误。** 前者无法匹配，
  后者丢失类别。私有辅助函数内部可以传 `String` 作为原因文本，但公开
  边界必须是枚举。`apsis-scheduler` 的 `precedence_floor` 是这种内部用法
  的实例：它私有的 `String` 最终填入公开的 `TaskState::Infeasible { reason }`。

### 通用错误

`apsis-core::Error` 与 `ErrorKind` 是通用兜底类型，携带粗粒度类别
（`InvalidInput`、`NotFound` 等）。它**不是**各层的统一错误表面：没有任何
层使用它，因为通用类别丢失了调用方需要的区分（`InvalidInput` 无法区分
"区间反向"与"重复标识"）。

它保留在这里有两个理由：给一个失败没有有趣结构的函数一个现成的返回类型；
以及给工作区一个可扩展的兜底形状。新代码若失败有领域含义，应定义领域错误。

### 失败与"无值"的区分

- **`Option`** 表示"没有值"是正常结果，不是错误。例如某个时刻在覆盖区间
  之外（`Ephemeris::state_at` 返回 `None`）、某个角度的正切无定义。
- **`Result`** 表示操作因可诊断的原因失败。例如区间反向、资源重复、
  超出模型定义域。

判断标准：调用方是否需要知道**为什么**。若只需要知道"没有"，用 `Option`；
若需要区分失败原因并据此恢复，用 `Result`。

## 数值容差

### 迭代收敛容差

迭代求解（开普勒方程、章动级数、射线追踪）用 crate 内的私有常量控制收敛：

| crate | 常量 | 值 | 含义 |
| --- | --- | --- | --- |
| `apsis-kepler` | `TOLERANCE` | `1e-14` | 偏近点角牛顿迭代的步长 |
| `apsis-sgp4` | `KEPLER_TOLERANCE` | `1e-12` | 开普勒方程迭代残差 |

这些值在浮点精度与迭代次数之间折中：取到接近 `f64` 的机器精度，同时
保证迭代在有限步内收敛（各有对应的 `MAX_ITERATIONS`）。

### 对拍容差

对拍脚本的容差按**物理量**设定，而不是统一的数值：

| 量 | 容差 |
| --- | --- |
| 位置 | < 1e-3 km |
| 速度 | < 1e-6 km/s |
| 时间 | < 1e-6 s |
| 覆盖几何角度 | 1e-12 rad |
| 覆盖面积、占比 | 相对 1e-12 |

容差设在"远高于观测到的偏差（通常在浮点精度量级），远低于模型本身的
精度"之间。这样既能捕获半径写错、单位漏换算这类错误，又不会因最后一位
浮点差异而误报。详见 `validation/README.md` 的容差表。

### 精确比较

不需要容差时应避免引入：整数索引、计数、布尔标志用精确相等；角度归约
（`wrap_to_pi`）在边界上用半开区间，不引入 epsilon。浮点比较仅在量本身
是连续近似时使用。

## 检查

- 各 crate 测试覆盖其错误的每个 `kind`，包括构造失败与恢复路径。
- 对拍脚本以退出码表达结果：0 通过、1 超差、2 缺输入。
- `cargo clippy -- -D warnings` 保证没有 `unwrap`/`expect` 漏进库代码
  （工作区约定库代码不使用它们）。
