# 对拍验证

本目录用于 Rust 实现与公开参考实现的数值对拍。Python 侧仅作验证用途，
不属于 Rust 依赖，也不参与 Rust 编译。

## Python 环境

使用 [pixi](https://pixi.sh/) 管理：

```bash
cd validation
pixi install
```

已允许的库：

- `sgp4`：SGP4 传播对拍
- `astropy`：时间、坐标对拍
- `numpy`：数值计算
- `skyfield`：轨道、可见性对拍（可选）
- `itur`：ITU-R 建议书参考实现（P.676、P.838、P.618），用于大气与雨衰对拍

引入新的 Python 库前需要评估并确认。

## 目录

```text
validation/
├── pixi.toml
├── python/
│   ├── orbit/           # SGP4、开普勒对拍
│   ├── time/            # 时间尺度、儒略日对拍
│   ├── coord/           # 坐标变换对拍
│   ├── link/            # 链路预算对拍
│   └── constellation/   # 覆盖几何对拍
└── data/
    ├── tle/             # 公开 TLE 样本
    └── expected/        # Rust 侧导出的待比对数据
```

## 流程

Rust 侧通过 `apsis-rs` 的 example 导出数据，输出为 CSV（手写格式化，不引 serde）：

```bash
# 大地坐标 -> ECEF
cargo run -p apsis-rs --features physics --example dump_geodesy \
    > validation/data/expected/rust_geodetic.csv

# 时间尺度换算
cargo run -p apsis-rs --features physics --example dump_time \
    > validation/data/expected/rust_time.csv

# SGP4/SDP4 传播
cargo run -p apsis-rs --features orbit --example dump_orbit \
    > validation/data/expected/rust_sgp4.csv

# 链路预算
cargo run -p apsis-rs --features rf --example dump_link \
    > validation/data/expected/rust_link.csv

# ITU-R P.676-12 气体吸收
cargo run -p apsis-rs --features rf --example dump_gas \
    > validation/data/expected/rust_gas.csv

# ITU-R P.838-3 雨衰系数
cargo run -p apsis-rs --features rf --example dump_rain \
    > validation/data/expected/rust_rain.csv

# ITU-R P.618-13 雨衰斜路径
cargo run -p apsis-rs --features rf --example dump_rain_path \
    > validation/data/expected/rust_rain_path.csv

# ITU-R S.465 参考天线方向图
cargo run -p apsis-rs --features rf --example dump_s465 \
    > validation/data/expected/rust_s465.csv

# ITU-R P.839-4 零度等温线高度
cargo run -p apsis-rs --features rf --example dump_p839 \
    > validation/data/expected/rust_p839.csv

# ITU-R P.676-12 精确斜路径（分层积分 + 折射率射线追踪）
cargo run -p apsis-rs --features rf --example dump_gas_exact \
    > validation/data/expected/rust_gas_exact.csv

# IAU 2000/2006 地球定向（岁差、章动、框架偏移、GCRS→ITRF）
cargo run -p apsis-rs --features physics --example dump_iau2000 \
    > validation/data/expected/rust_iau2000.csv

# TEME → 地固系（SGP4 输出链路）
cargo run -p apsis-rs --features physics --example dump_teme \
    > validation/data/expected/rust_teme.csv

# 覆盖几何（仰角、星下点角、地心角、斜距、覆盖面积）
cargo run -p apsis-rs --features network --example dump_coverage \
    > validation/data/expected/rust_coverage.csv
```

Python 侧读取同一批输入，用参考实现重算并输出误差表：

```bash
pixi run python validation/python/coord/check_geodetic.py
pixi run python validation/python/time/check_time.py
pixi run python validation/python/orbit/check_sgp4.py
pixi run python validation/python/link/check_link.py
pixi run python validation/python/atmosphere/check_gas.py
pixi run python validation/python/atmosphere/check_rain.py
pixi run python validation/python/atmosphere/check_rain_path.py
pixi run python validation/python/link/check_s465.py
pixi run python validation/python/atmosphere/check_p839.py
pixi run python validation/python/atmosphere/check_gas_exact.py
pixi run python validation/python/time/check_iau2000.py
pixi run python validation/python/coord/check_teme.py
pixi run python validation/python/constellation/check_coverage.py
```

两个脚本都以退出码 0 表示通过、1 表示超差、2 表示缺少输入文件。

## 容差

| 量 | 容差 |
| --- | --- |
| 位置 | < 1e-3 km |
| 速度 | < 1e-6 km/s |
| 时间 | < 1e-6 s |

超差时修正 Rust 侧算法，不放宽容差，除非经确认。

脚本必须可独立离线运行，不访问网络。`check_time.py` 会关闭 astropy 的
Earth orientation 数据自动下载；本 crate 涉及的换算不需要该数据。

## 已实现的检查

| 脚本 | 对比内容 | 参考实现 | 当前最差误差 |
| --- | --- | --- | --- |
| `coord/check_geodetic.py` | `Geodetic` → ECEF，280 个点，覆盖全球纬度、经度与高度 | `astropy.coordinates.EarthLocation` | 1.1e-8 m |
| `time/check_time.py` | UTC → TAI，117 个时刻，覆盖全部闰秒边界 | `astropy.time.Time` | 5.3e-7 s |
| `orbit/check_sgp4.py` | SGP4/SDP4 传播，4 组 TLE（近地、高偏心、深空、GTO）× 9 个时刻 | `sgp4` 参考库 | 位置 1.2e-8 km，速度 5.3e-12 km/s |
| `link/check_link.py` | 链路预算，4 个场景（Ku 下行、S 波段、Ka 宽带、L 波段） | 链路方程闭式解（Python 手写） | 5.0e-13 dB |
| `atmosphere/check_gas.py` | 气体吸收，18 个频率，涵盖 60 GHz 氧气复体、22/183 GHz 水汽线 | `itur` 的 ITU-R P.676-12 实现 | 干空气 6.1e-11，水汽 6.4e-9（相对） |
| `atmosphere/check_rain.py` | 雨衰系数，17 个频率 × 4 组仰角/极化 | `itur` 的 ITU-R P.838-3 实现 | k 7.5e-9，alpha 6.5e-13（相对） |
| `atmosphere/check_rain_path.py` | 雨衰斜路径，8 个场景 | ITU-R P.618-13 方程（Python 转写，系数来自 `itur`） | 3.4e-13 km / 4.3e-13 dB |
| `link/check_s465.py` | 参考天线方向图，3 种口径 × 8 个离轴角 | ITU-R S.465 方程（Python 转写） | 4.0e-10 dB |
| `atmosphere/check_p839.py` | 零度等温线高度，15 个全球点 | `itur` 的 ITU-R P.839-4 实现 | 3.3e-13 km |
| `atmosphere/check_gas_exact.py` | 精确斜路径，11 个场景（1–100 GHz，1°–90° 仰角） | `itur` 的 ITU-R P.676-12 exact 方法 | 9.6e-9（相对） |
| `time/check_iau2000.py` | 岁差、章动、框架偏移、GAST、GCRS→ITRF，8 个历元 × 28 个量 | ERFA 2.0.1（IAU SOFA 派生） | 章动 6.6e-15 rad；BPN 矩阵 6.0e-15；GAST 1.1e-11 rad |
| `coord/check_teme.py` | TEME → 地固系，4 个历元 | ERFA 的 `gmst82` 与 `pom00`（同为 0 极移） | 4.9e-10 km |

### 关于射频层对拍的说明

射频层分两类：有权威参考实现的模型与纯闭式算式。

- 有参考实现的，直接对拍：地球定向用 `erfa`（IAU SOFA 的派生实现），大气与雨衰用 `itur`；
  链路预算的每一步都是初等运算，`check_link.py` 用 Python 独立重写同一方程。
- 其余以解析性质断言为主：各 crate 的单元测试验证模型必须满足的性质，例如 3 dB 点、
  平方律下降、Friis 级联行为、P.838-3 系数在表列频率处的取值。

所有建议书均采用**现行版本**：P.676-12、P.838-3、P.618-13、P.839-4、S.465。
P.838-3 的系数是闭式曲线拟合，不是已被取代的 P.838-1 数值表。

### 关于 SGP4 对拍的约定

TLE 的历元是 UTC 日历时刻，但 SGP4 把它当作均匀时间原点使用，这是理论拟合时
采用的约定。若先把历元换算到 TAI 再送入传播器，会引入约 33 秒的闰秒偏移，该偏移
会传播到深空分支的日月摄动级数中。对拍时两侧都直接使用 TLE 写明的 UTC MJD。

### 关于时间对拍的约定差异

UTC 的浮点日表示存在两种约定：

- 本仓库使用线性 MJD，`TAI - UTC` 是阶跃函数，闰秒日当天其余时刻偏移不变。
- astropy / ERFA 使用准儒略日，闰秒日有 86401 秒，其 `tai - utc` 差值在
  该日内线性变化。

因此直接比较两者的儒略日差会测量到“约定差异”而非数值误差。`check_time.py`
改比较 **TAI 瞬时**：两侧给定同一 UTC 日历时刻，再比较得到的 TAI 修正儒略日。
比较采用“整数日 + 日内小数”两段式，避免单个 `f64` 在 MJD ≈ 60000 处约
40 µs 的分辨率损失。

`dump_time` 也因此以两段式输出 TAI 时刻，而不是单个浮点日。
