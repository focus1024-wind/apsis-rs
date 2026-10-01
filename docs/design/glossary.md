# Glossary and formula conventions

English | [简体中文](./glossary.zh.md)

This file fixes how a technical term is rendered in a comment and how a
formula is written. Follow it when translating a comment.

## The principle

**A term with an established Chinese name uses that name in the Chinese
paragraph.** The vocabulary of the field already has one, so the Chinese
paragraph uses it directly and does not append the English each time. The
English paragraph is unchanged, and the two sit side by side, so a reader may
read either.

**A proper name, an abbreviation, a person's name, a model or a method keeps
its English form.** Those appear in the original sources only, the ITU-R
recommendations, the NASA orbital mechanics literature and the Rust standard
library, and translating them would leave a reader unable to find the source.

## Terms that keep their English form

| English | Kind | Notes |
| --- | --- | --- |
| SGP4, SDP4 | model | the orbit propagators |
| Hermite | name | the interpolation polynomial |
| Walker | name | the constellation pattern |
| Newton | name | the iteration |
| Runge-Kutta | name | the integrator |
| Vandermonde | name | the interpolation matrix |
| Hohmann | name | the transfer orbit |
| TLE, BSTAR | format and field | the element set and one of its fields |
| ECI, ECEF, TEME, ITRF, GCRS, ERA, GAST | abbreviation | frames and Earth rotation quantities |
| Isp, TAI, UTC, UT1, MJD, JD | abbreviation | specific impulse and time scales |
| TUMIN, XKE, X2O3 | symbol | internal model quantities |
| EIRP, C/N0 | abbreviation | link figures |
| delta-v | idiomatic | velocity increment, also written Δv in Chinese |

These stay as they are in the Chinese paragraph too.

## Terms that use their standard Chinese name

The Chinese paragraph uses the Chinese directly, with no English appended:

| English | Chinese |
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

## Formulas

- A formula is kept as it stands: not rewritten, not simplified, not completed.
- A formula in a comment must agree with the code. Changing the code means
  changing the comment in the same edit.
- A variable, function, field or module name keeps its English form.
- An inline formula is written as code, `` `a (1 - e^2)` ``. A formula that
  needs several lines goes in a `/// ```text` block.
- A unit goes in parentheses after the quantity, as in `半长轴，单位为千米`.

## The layout of a comment

The English and the Chinese of one passage are separated by an empty `///`
line, English first:

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
