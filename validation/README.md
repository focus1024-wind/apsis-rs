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
│   └── constellation/   # 星座覆盖对拍
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
```

Python 侧读取同一批输入，用参考实现重算并输出误差表：

```bash
pixi run python validation/python/coord/check_geodetic.py
pixi run python validation/python/time/check_time.py
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
