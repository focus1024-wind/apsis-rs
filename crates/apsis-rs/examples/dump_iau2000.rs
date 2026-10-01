//! Writes the IAU 2000/2006 Earth orientation quantities computed by this
//! workspace, for cross-checking against ERFA.
//!
//! 写出本工作区计算的 IAU 2000/2006 地球定向量，用于对照 ERFA 交叉核对。
//!
//! Usage:
//!
//! ```text
//! cargo run -p apsis-rs --features physics --example dump_iau2000 \
//!     > validation/data/expected/rust_iau2000.csv
//! ```
//!
//! 用法：
//!
//! 运行上方的命令即可得到该示例的输出。

use apsis_frames::iau2000::{
    PolarMotion, bias_precession_nutation_matrix, celestial_to_terrestrial_matrix,
    greenwich_apparent_sidereal_time, mean_obliquity, nutation_2000a,
    nutation_2000a_with_2006_adjustment, precession_angles,
};
use apsis_math::Mat3;

/// The Modified Julian Dates sampled. They span from 1970 to 2050 so that
/// the precession is exercised over a range of a century.
///
/// 所采样的约化儒略日。它们从 1970 到 2050，使岁差在一个世纪的跨度上被
/// 检验。
const MJD: &[f64] = &[
    40_587.0, 51_544.5, 53_000.0, 55_000.0, 57_000.0, 58_000.0, 60_000.0, 62_000.0,
];

/// The Terrestrial Time offset from TAI, in seconds.
///
/// 地球时相对 TAI 的偏移，单位为秒。
const TT_MINUS_TAI: f64 = 32.184;

/// The TAI minus UTC offset used for this check, in seconds. It is the value
/// in force from 2017 onward.
///
/// 本检查所用的 TAI 减 UTC 偏移，单位为秒。它是自 2017 年起生效的值。
const TAI_MINUS_UTC: f64 = 37.0;

/// Returns the TT Julian date for a Modified Julian Date on the UTC scale.
///
/// 返回 UTC 时标上某约化儒略日对应的 TT 儒略日。
fn tt_julian_date(utc_mjd: f64) -> f64 {
    // TAI = UTC + 37, TT = TAI + 32.184, so TT = UTC + 69.184 seconds.
    // TAI = UTC + 37，TT = TAI + 32.184，故 TT = UTC + 69.184 秒。
    utc_mjd + 2_400_000.5 + (TT_MINUS_TAI + TAI_MINUS_UTC) / 86_400.0
}

fn main() {
    println!("mjd,quantity,index,value");
    for &mjd in MJD {
        let tt = tt_julian_date(mjd);
        let t = (tt - 2_451_545.0) / 36_525.0;

        // The obliquity, in radians.
        // 黄赤交角，单位为弧度。
        println!("{mjd},obliquity,0,{:.17e}", mean_obliquity(t));

        // The nutation, from the IAU 2000A model.
        // 章动，取自 IAU 2000A 模型。
        let nutation = nutation_2000a(t);
        println!("{mjd},nutation_longitude_a,0,{:.17e}", nutation.longitude);
        println!("{mjd},nutation_obliquity_a,0,{:.17e}", nutation.obliquity);

        // The nutation with the IAU 2006 adjustment.
        // 施加 IAU 2006 调整后的章动。
        let adjusted = nutation_2000a_with_2006_adjustment(t);
        println!("{mjd},nutation_longitude_06a,0,{:.17e}", adjusted.longitude);
        println!("{mjd},nutation_obliquity_06a,0,{:.17e}", adjusted.obliquity);

        // The four precession angles.
        // 四个岁差角。
        let angles = precession_angles(t);
        println!("{mjd},precession_gamma_bar,0,{:.17e}", angles.gamma_bar);
        println!("{mjd},precession_phi_bar,0,{:.17e}", angles.phi_bar);
        println!("{mjd},precession_psi_bar,0,{:.17e}", angles.psi_bar);
        println!("{mjd},precession_epsilon,0,{:.17e}", angles.epsilon);

        // The bias-precession-nutation matrix, which carries the celestial
        // reference frame to the true equator and equinox of date.
        // 偏差-岁差-章动矩阵，它把天球参考系带到该日的真赤道与真春分点。
        let bpn = bias_precession_nutation_matrix(t);
        for row in 0..3 {
            for column in 0..3 {
                let value = bpn.get(row, column).expect("the indices are in range");
                println!("{mjd},bpn,{},{:.17e}", row * 3 + column, value);
            }
        }

        // Greenwich apparent sidereal time, in radians.
        // 格林尼治视恒星时，单位为弧度。
        println!(
            "{mjd},gast,0,{:.17e}",
            greenwich_apparent_sidereal_time(tt, tt)
        );

        // The celestial-to-terrestrial matrix, with zero polar motion.
        // 天球到地球坐标系矩阵，极移为零。
        let matrix = celestial_to_terrestrial_matrix(tt, tt, PolarMotion::NONE);
        for row in 0..3 {
            for column in 0..3 {
                let value = matrix.get(row, column).expect("the indices are in range");
                println!("{mjd},c2t,{},{:.17e}", row * 3 + column, value);
            }
        }
    }
}

/// Keeps the matrix type in scope for the documentation build.
///
/// 为使文档构建保持在作用域内而保留矩阵类型。
#[allow(dead_code)]
fn _uses_matrix(_matrix: Mat3) {}
