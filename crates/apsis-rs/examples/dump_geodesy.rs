//! Writes a grid of geodetic positions and the Earth-fixed Cartesian
//! coordinates this workspace computes for them.
//!
//! The output is CSV on standard output, with a header row. It is consumed
//! by `validation/python/coord/check_geodetic.py`, which recomputes the same
//! positions with a reference implementation and compares the two.
//!
//! 写出一个大地位置网格，以及本工作区为它们计算的地固直角坐标。
//!
//! 输出是标准输出上的 CSV，带一行表头。它由
//! `validation/python/coord/check_geodetic.py` 消费，后者用参考实现重算相同的
//! 位置并比较二者。
//!
//! Usage:
//!
//! ```text
//! cargo run -p apsis-rs --features physics --example dump_geodesy \
//!     > validation/data/expected/rust_geodetic.csv
//! ```
//!
//! 用法：
//!
//! 运行上方的命令即可得到该示例的输出。

use apsis_coord::{Geodetic, WGS84};

fn main() {
    println!("latitude_degrees,longitude_degrees,height_m,x_m,y_m,z_m");
    let latitudes = [-89.0_f64, -60.0, -30.0, 0.0, 30.0, 45.0, 60.0, 89.0];
    let longitudes = [-179.0_f64, -90.0, -45.0, 0.0, 45.0, 90.0, 179.0];
    let heights = [-1_000.0_f64, 0.0, 100.0, 3_000.0, 35_786_000.0];

    for latitude in latitudes {
        for longitude in longitudes {
            for height in heights {
                let point = Geodetic::new(latitude.to_radians(), longitude.to_radians(), height)
                    .to_ecef_on(WGS84)
                    .vector();
                println!(
                    "{latitude},{longitude},{height},{:.9},{:.9},{:.9}",
                    point.x, point.y, point.z
                );
            }
        }
    }
}
