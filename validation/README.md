# 对拍验证

本目录用于 Rust 实现与公开参考实现的数值对拍。Python 侧仅作验证用途，
不属于 Rust 依赖。

## Python 环境

使用 [pixi](https://pixi.sh/) 管理。允许的库：

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
    └── expected/        # Python 生成的参考值
```

## 流程

1. Rust 侧导出输入与输出，格式为简单 CSV 或 TXT，不使用 serde：

   ```bash
   cargo run -p apsis-rs --example dump_orbit > validation/data/rust_output.csv
   ```

2. Python 侧读取同一输入并计算参考值：

   ```bash
   pixi run python validation/python/orbit/check_sgp4.py
   ```

3. 比较误差：

   | 量 | 容差 |
   | --- | --- |
   | 位置 | < 1e-3 km |
   | 速度 | < 1e-6 km/s |
   | 时间 | < 1e-6 s |

超差时修正 Rust 侧算法，不放宽容差，除非经确认。

每个脚本必须可独立离线运行，不访问网络。
