# 术语与公式约定 / Glossary and formula conventions

[English](./glossary.md) | [简体中文](./glossary.zh.md)

本文件规定代码注释与文档中专业术语的译法，以及公式的写法。翻译注释时以此
为准。

## 原则

**中文段落里，术语一律用标准中文名。** 领域内已有约定俗成译法的词，直接
译，不逐次附英文。英文段落保持原文，两段并列，读者可任选一种阅读。

**专有名称、缩写、人名、模型名、方法名保留英文。** 这些词在原始资料
（ITU-R 建议书、NASA 轨道力学文献、Rust 标准库）中只有英文形式，译成中文
会让读者无法检索到出处。

## 保留英文的词

| 英文 | 类别 | 说明 |
| --- | --- | --- |
| SGP4、SDP4 | 模型名 | 轨道传播模型 |
| Hermite | 人名 | 插值多项式 |
| Walker | 人名 | 星座构型 |
| Newton | 人名 | 迭代方法 |
| Runge-Kutta | 人名 | 积分方法 |
| Vandermonde | 人名 | 插值矩阵 |
| Hohmann | 人名 | 转移轨道 |
| TLE、BSTAR | 格式与字段名 | 两线根数集及其字段 |
| ECI、ECEF、TEME、ITRF、GCRS、ERA、GAST | 缩写 | 坐标系与地球自转量 |
| Isp、TAI、UTC、UT1、MJD、JD | 缩写 | 比冲与时间尺度 |
| TUMIN、XKE、X2O3 | 符号 | 模型内部量 |
| EIRP、C/N0 | 缩写 | 链路指标 |
| delta-v | 惯用写法 | 速度增量，中文亦常写 Δv |

以上在中文段落中同样原样保留。

## 采用标准中文名的词

以下在中文段落中直接使用中文，不附英文：

| 英文 | 中文 |
| --- | --- |
| semi-major axis | 半长轴 |
| semi-minor axis | 半短轴 |
| semi-latus rectum | 半通径 |
| eccentricity | 偏心率 |
| perigee | 近地点 |
| apogee | 远地点 |
| true anomaly | 真近点角 |
| mean anomaly | 平近点角 |
| eccentric anomaly | 偏近点角 |
| right ascension | 赤经 |
| argument of perigee | 近地点幅角 |
| inclination | 倾角 |
| ascending node | 升交点 |
| osculating | 密切 |
| element set | 根数集 |
| two-line element | 两线根数集 |
| epoch | 历元 |
| ephemeris | 星历 |
| drag term | 阻力项 |
| perturbation | 摄动 |
| deep space | 深空 |
| resonance | 共振 |
| nodal | 交点 |
| zenith | 天顶 |
| nadir | 天底 |
| sub-satellite point | 星下点 |
| footprint | 覆盖区 |
| mask | 遮蔽角 |
| elevation | 仰角 |
| azimuth | 方位角 |
| central angle | 地心角 |
| slant range | 斜距 |
| path loss | 路径损耗 |
| attenuation | 衰减 |
| specific attenuation | 比衰减 |
| rain rate | 降雨率 |
| cross-section | 截面 |
| noise temperature | 噪声温度 |
| noise figure | 噪声系数 |
| signal to noise ratio | 信噪比 |
| free space path loss | 自由空间路径损耗 |
| specific impulse | 比冲 |
| exhaust velocity | 喷气速度 |
| propellant | 推进剂 |
| precession | 岁差 |
| nutation | 章动 |
| equinox | 分点 |
| Earth rotation angle | 地球自转角 |
| Modified Julian Date | 修正儒略日 |
| leap second | 闰秒 |
| assumed decimal point | 隐含小数点 |
| two digit year | 两位数年份 |
| checksum | 校验和 |
| scenario | 场景 |
| precedence | 优先约束 |
| placement | 放置 |
| candidate | 候选 |
| boundary | 边界 |
| invariant | 不变量 |
| reachability | 可达性 |
| shortest path | 最短路 |
| mutual visibility | 互见 |
| blocking | 遮挡 |
| cross-check | 对拍 |
| capacity | 容量 |
| demand | 需求 |
| window | 窗口 |
| availability | 可用性 |
| precedence graph | 优先关系图 |

## 公式

- 公式原样保留，不改写、不化简、不补全。
- 注释中出现的公式必须与实现一致。改动代码时同步改注释。
- 变量名、函数名、字段名、模块名一律保持英文原样。
- 行内公式写作 `` `a (1 - e^2)` `` 这样的代码样式；多行公式用 `/// ```text` 代码块。
- 单位写在括号里，例如 `半长轴，单位为千米`。

## 注释的排版

同一段说明的英文与中文之间用一个空的 `///` 行分隔，英文在上：

```rust
/// Returns the phase offset applied between adjacent planes, in radians.
///
/// This is `f * 360 / t` degrees, where `f` is the phase factor and `t`
/// the total satellite count.
///
/// 返回相邻轨道面之间施加的相位偏移，单位为弧度。
///
/// 即 `f * 360 / t` 度，其中 `f` 为相位因子，`t` 为卫星总数。
```
