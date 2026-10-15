//! Writes the coverage geometry this workspace computes for a set of cases.
//!
//! The output is CSV on standard output, with a header row. It is consumed
//! by `validation/python/constellation/check_coverage.py`, which recomputes
//! the same quantities from the spherical triangle relations and compares.
//!
//! The cases are chosen to span the interesting geometry: a low Earth orbit
//! at a low mask, where the footprint is small relative to the Earth; a
//! higher orbit, where it is large; the equatorial and polar cases; and a
//! zero mask, where the footprint reaches the horizon.
//!
//! 写出本工作区为一组用例计算的覆盖几何。
//!
//! 输出是标准输出上的 CSV，带一行表头。它由
//! `validation/python/constellation/check_coverage.py` 消费，后者从球面三角形
//! 关系重算相同的量并比较。
//!
//! 各用例的选取跨越了有意思的几何：低仰角遮蔽的低轨，其覆盖区相对地球很小；
//! 更高的轨道，其覆盖区很大；赤道与极地情形；以及零遮蔽，其覆盖区直到地平。
//!
//! Usage:
//!
//! ```text
//! cargo run --features network --example dump_coverage \
//!     > validation/data/expected/rust_coverage.csv
//! ```
//!
//! 用法：
//!
//! 运行上方的命令即可得到该示例的输出。

use apsis::apsis_coverage::{
    earth_central_angle, elevation_from_central_angle, footprint_area, footprint_central_angle,
    footprint_fraction, nadir_angle, slant_range,
};
use core::f64::consts::PI;

/// The WGS 84 equatorial radius, in kilometers.
///
/// The coverage geometry is spherical, so a single radius is used. The
/// value comes from the shared constants rather than being repeated, so a
/// change to the ellipsoid reaches here.
///
/// WGS 84 赤道半径，单位为千米。
///
/// 覆盖几何是球面的，故只用一个半径。该值来自共享常量而非重复书写，故椭球的
/// 改动会传到这里。
const BODY_RADIUS_KM: f64 = apsis::apsis_core::constant::EARTH_EQUATORIAL_RADIUS;

/// One coverage case: an orbital radius and an elevation mask.
///
/// 一个覆盖用例：一个轨道半径与一个仰角遮蔽。
struct Case {
    /// A name for the case, so a failure in the report is readable.
    ///
    /// 用例的名称，使报告中的失败可读。
    name: &'static str,
    /// The geocentric radius of the satellite, in kilometers.
    ///
    /// 卫星的地心半径，单位为千米。
    satellite_radius_km: f64,
    /// The elevation mask at the ground point, in degrees.
    ///
    /// 地面点处的仰角遮蔽，单位为度。
    elevation_mask_degrees: f64,
}

/// The cases checked.
///
/// The satellite radii span a low Earth orbit, two higher altitudes, and a
/// geostationary one. The masks span the horizon, a typical station mask, and
/// a strict one.
///
/// 被检查的各用例。
///
/// 卫星半径跨越低轨、两个更高高度以及一个地球静止轨道。遮蔽跨越地平、典型
/// 地面站遮蔽，以及一个严格遮蔽。
const CASES: &[Case] = &[
    Case {
        name: "leo_low_mask",
        satellite_radius_km: 6_928.137,
        elevation_mask_degrees: 5.0,
    },
    Case {
        name: "leo_ten_degrees",
        satellite_radius_km: 6_928.137,
        elevation_mask_degrees: 10.0,
    },
    Case {
        name: "leo_strict",
        satellite_radius_km: 6_928.137,
        elevation_mask_degrees: 30.0,
    },
    Case {
        name: "meo",
        satellite_radius_km: 26_560.0,
        elevation_mask_degrees: 10.0,
    },
    Case {
        name: "geo",
        satellite_radius_km: 42_164.0,
        elevation_mask_degrees: 10.0,
    },
    Case {
        name: "horizon",
        satellite_radius_km: 6_928.137,
        elevation_mask_degrees: 0.0,
    },
];

/// A set of central angles, in degrees, for the inverse relation check.
///
/// Each is fed to `elevation_from_central_angle` and the elevation it returns
/// is fed back through `earth_central_angle`, so the pair must compose to the
/// identity. The angles span zero to a little past the largest footprint the
/// cases produce.
///
/// 一组用于反解关系检查的中心角，单位为度。
///
/// 每个都被送入 `elevation_from_central_angle`，其返回的仰角经
/// `earth_central_angle` 送回，故这一对必须合成为恒等。各角从零到略超过各用例
/// 所产生的最大覆盖区。
const CENTRAL_ANGLES_DEGREES: &[f64] = &[0.0, 1.0, 5.0, 15.0, 30.0, 45.0, 60.0, 75.0];

fn main() {
    // The header names every column so that the Python side can read by name
    // rather than by position.
    // 表头为每一列命名，使 Python 侧可以按名而非按位置读取。
    println!(
        "case,satellite_radius_km,body_radius_km,elevation_mask_rad,\
         nadir_rad,central_rad,slant_range_km,footprint_central_rad,\
         footprint_area_km2,footprint_fraction"
    );

    for case in CASES {
        let elevation = case.elevation_mask_degrees * PI / 180.0;
        let nadir = nadir_angle(case.satellite_radius_km, BODY_RADIUS_KM, elevation)
            .expect("the case geometry is usable");
        let central = earth_central_angle(case.satellite_radius_km, BODY_RADIUS_KM, elevation)
            .expect("the case geometry is usable");
        let range = slant_range(case.satellite_radius_km, BODY_RADIUS_KM, elevation)
            .expect("the case geometry is usable");
        let footprint_central =
            footprint_central_angle(case.satellite_radius_km, BODY_RADIUS_KM, elevation)
                .expect("the case geometry is usable");
        let area = footprint_area(case.satellite_radius_km, BODY_RADIUS_KM, elevation)
            .expect("the case geometry is usable");
        let fraction = footprint_fraction(case.satellite_radius_km, BODY_RADIUS_KM, elevation)
            .expect("the case geometry is usable");

        println!(
            "{},{},{},{},{},{},{},{},{},{}",
            case.name,
            case.satellite_radius_km,
            BODY_RADIUS_KM,
            elevation,
            nadir,
            central,
            range,
            footprint_central,
            area,
            fraction
        );
    }

    // The inverse relation, written with its own header so the Python side
    // can tell the two blocks apart by the first column.
    // 反解关系，带上它自己的表头，使 Python 侧可以按第一列区分两个块。
    println!();
    println!("inverse_central_degrees,elevation_rad,round_trip_central_degrees");
    for degrees in CENTRAL_ANGLES_DEGREES {
        let central = degrees * PI / 180.0;
        // Use the equatorial radius as the satellite radius for the inverse
        // check, so the geometry is the degenerate one where the elevation is
        // always zero. A more useful check uses a satellite altitude.
        // 反解检查把赤道半径用作卫星半径，故几何是退化的那种，其中仰角恒为零。更
        // 有用的检查要用卫星高度。
        let satellite_radius = BODY_RADIUS_KM + 550.0;
        let Ok(elevation) =
            elevation_from_central_angle(satellite_radius, BODY_RADIUS_KM, central).ok_or(())
        else {
            // Beyond the horizon the inverse has no solution, which is
            // reported as an empty row rather than skipped, so the count of
            // rows stays fixed.
            // 在地平之外反解无解，故报告为一个空行而非跳过，使行数保持固定。
            println!("{degrees},,");
            continue;
        };
        let Ok(back) = earth_central_angle(satellite_radius, BODY_RADIUS_KM, elevation).ok_or(())
        else {
            println!("{degrees},,");
            continue;
        };
        println!("{degrees},{elevation},{}", back * 180.0 / PI);
    }
}
