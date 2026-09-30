# 单位与坐标约定

[English](./units.md) | 简体中文

本文件记录跨层的单位与坐标约定。单个 crate 的内部约定见其模块文档；这里只
记录**跨层边界**必须一致的部分，因为边界处的单位不一致是本仓库最容易引入
也最难发现的错误。

## 长度的两种单位

仓库同时使用千米与米，边界在 orbit 层与 physics 层之间：

| 层 / crate | 长度单位 |
| --- | --- |
| `apsis-kepler`、`apsis-sgp4`、`apsis-tle` | 千米 |
| `apsis-ephemeris` | 千米 |
| `apsis-maneuver` | 千米 |
| `apsis-constellation`、`apsis-coverage`、`apsis-access`、`apsis-isl` | 千米 |
| `apsis-topology`、`apsis-routing`、`apsis-mesh` | 长度仅在 `apsis-isl` 计算，边的 `cost` 是调用方定义的无量纲值 |
| `apsis-handover` | 不涉及长度；`quality` 为调用方定义，时间用秒 |
| `apsis-coord`（`Eci`、`Ecef`） | 米 |
| `apsis-geodesy`（`Geodetic` 高度、椭球半轴、笛卡尔位置） | 米 |
| `apsis-frames` | 无量纲（角度与旋转） |
| `apsis-rf`、`apsis-antenna`、`apsis-atmosphere`、`apsis-noise`、`apsis-doppler`、`apsis-interference`、`apsis-link` | 米、赫兹 |

选择千米还是米并非任意：卫星轨道用千米表达更自然（`apsis-sgp4` 输出的量级
是几千），射频链路用米表达更自然（波长与距离用米，与 ITU-R 建议书一致）。
代价是边界处必须显式换算。

### 边界处的换算

从 orbit 层进入 physics 层时乘以 `1000.0`，返回时除以 `1000.0`：

```rust
// orbit 层的千米位置转 physics 层的米位置。
let position_km = satellite.propagate_minutes(0.0)?.position;  // 千米
let ecef = Ecef::new(position_km * 1_000.0);                    // 米
```

漏掉这一步会把卫星放到地球内部，而且数值仍然"看起来合理"，不会立刻报错。
`integrate_network` 示例对这一边界做了断言：换算后的半径必须与传播器报告
的半径一致，这是唯一能自动发现漏乘或漏除的检查。

## 角度

**所有计算用角度一律为弧度**。纬度、经度、倾角、升交点赤经、近地点幅角、
真近点角、仰角、方位角、天线方向图角都是弧度。

唯一的例外是两线元素集：`apsis-tle::TwoLineElement` 的原始字段按 TLE 格式
用度与每日圈数，因为那是文件的编码。`apsis-sgp4` 在初始化时换算为弧度与
弧度每分钟。

## 时间

时间统一用秒表示，原点由调用方选择：

- ops 与 sim 层：从场景原点（scenario epoch）起算的秒数。
- orbit 层：`apsis-sgp4` 用"距元素历元的分钟数"，`apsis-ephemeris` 与
  其余 crate 用秒。
- 日历时间：`apsis-time` 的 `Epoch` 存 TAI 儒略日；UTC 换算经 `apsis-leap`
  的闰秒表。

`Orbit::sample_ephemeris` 是两种时间表示的边界：入参是距元素历元的分钟，
返回的星历内部用秒，换算在方法内完成。

## 坐标框架

| 框架 | 类型 | 说明 |
| --- | --- | --- |
| TEME | `apsis_sgp4::State` | SGP4 定义的真赤道平春分点框架 |
| ECI | `apsis_coord::Eci` | 惯性笛卡尔坐标 |
| ECEF | `apsis_coord::Ecef` | 地固笛卡尔坐标 |
| 大地坐标 | `apsis_geodesy::Geodetic` | 纬度、经度、大地高 |
| 地面点 | `apsis_coverage::GroundPoint` | 大地坐标加仰角遮蔽 |

TEME 到 ECEF 需要地球自转角（`apsis-frames::EarthRotationAngle`）加上岁差
章动；`apsis-coord` 提供变换，`apsis-frames` 提供角。TEME 与 ECI 的差别是
分点方程，量级低于千分之一度；`integrate_network` 示例中直接把它当作惯性
系处理，并注明这是该示例的近似。

## 其它量

- 引力参数：`apsis-kepler::GravityParameter`，单位 km³/s²。WGS 72 对应
  SGP4，WGS 84 对应其余用途；选错会使密切半长轴偏约百万分之一。门面
  `apsis-orbit::Orbit` 把引力参数与传播器绑定，避免这一错误。
- 速度：与长度同层，km/s 或 m/s。
- 功率用瓦、频率用赫兹、温度用开尔文；分贝为功率比，`dBi` 相对各向同性、
  `dBd` 相对半波偶极子（相差 2.15 dB）。
- 相对量一律无量纲：偏心率、填充率、利用率、信噪比的分贝值。

## 检查

单位边界没有类型系统保护，靠测试与文档：

- 各层测试覆盖层内换算。
- `integrate_network` 断言千米与米的边界换算保持半径。
- `integrate_ops`、`integrate_sim` 覆盖 ops 与 sim 层的时间轴约定。
- `cargo run --release -p apsis-rs --features full --example bench` 覆盖
  各层热点路径，任一层的接口变更导致编译失败都会在此暴露。
