//! Writes the TEME to Earth-fixed transformation computed by this workspace,
//! for cross-checking against astropy.
//!
//! 写出本工作区计算的 TEME 到地固坐标变换，用于对照 astropy 交叉核对。
//!
//! Usage:
//!
//! ```text
//! cargo run --features physics --example dump_teme \
//!     > validation/data/expected/rust_teme.csv
//! ```
//!
//! 用法：
//!
//! 运行上方的命令即可得到该示例的输出。

use apsis::apsis_frames::iau2000::{PolarMotion, teme_to_terrestrial_matrix};

/// The samples, as a UTC Modified Julian Date and a TEME position in
/// kilometers.
///
/// 各样本，为一个 UTC 约化儒略日与一个以千米计的 TEME 位置。
const SAMPLES: &[(f64, [f64; 3])] = &[
    (60_481.5, [7_000.0, 1_000.0, 2_000.0]),
    (51_544.5, [7_000.0, -3_000.0, 1_000.0]),
    (40_587.0, [42_164.0, 0.0, 0.0]),
    (60_000.25, [6_878.137, 100.0, -50.0]),
];

fn main() {
    println!("mjd,x_ecef,y_ecef,z_ecef");
    for &(utc_mjd, position) in SAMPLES {
        // The transformation needs the UT1 time, because it is the Earth's
        // rotation that carries the frame. UT1 differs from UTC by under
        // 0.9 seconds, which is 0.0037 degrees of rotation, so the UTC value
        // is used directly here and the check tolerates that difference.
        // The Julian date is split into a whole number of days and a
        // fraction, which preserves more of the day than a single value.
        // 该变换需要 UT1 时间，因为是地球自转在带动坐标系。UT1 与 UTC 相差不到
        // 0.9 秒，即 0.0037 度的转动，故此处直接使用 UTC 值，而检查容忍该差异。儒略日
        // 被拆成整数天数与零头，这比单个值保留更多当日信息。
        let matrix = teme_to_terrestrial_matrix(2_400_000.5, utc_mjd, PolarMotion::NONE);
        let vector = apsis::apsis_math::Vec3::new(position[0], position[1], position[2]);
        let result = matrix * vector;
        println!("{utc_mjd},{:.9},{:.9},{:.9}", result.x, result.y, result.z);
    }
}
