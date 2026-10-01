//! Writes the ITU-R S.465 reference pattern computed by this workspace, for
//! cross-checking against the recommendation.
//!
//! 写出本工作区计算的 ITU-R S.465 参考方向图，用于对照该建议书交叉核对。
//!
//! Usage:
//!
//! ```text
//! cargo run -p apsis-rs --features rf --example dump_s465 \
//!     > validation/data/expected/rust_s465.csv
//! ```

use apsis_antenna::{main_lobe_break_angle, reference_pattern_gain, reference_peak_gain};

/// The antenna sizes sampled, as diameter to wavelength ratios.
///
/// 所采样的天线尺寸，以口径与波长之比表示。
const RATIOS: &[f64] = &[50.0, 100.0, 200.0];

/// The off-axis angles sampled, in degrees.
///
/// 所采样的偏轴角，单位为度。
const ANGLES_DEGREES: &[f64] = &[0.1, 0.5, 1.0, 5.0, 20.0, 36.0, 48.0, 90.0];

fn main() {
    println!("ratio,angle_deg,peak_dbi,break_deg,gain_dbi");
    for &ratio in RATIOS {
        let Some(peak) = reference_peak_gain(ratio) else {
            continue;
        };
        let Some(break_angle) = main_lobe_break_angle(ratio) else {
            continue;
        };
        for &angle_degrees in ANGLES_DEGREES {
            let angle = angle_degrees.to_radians();
            let Some(gain) = reference_pattern_gain(ratio, angle) else {
                continue;
            };
            println!("{ratio},{angle_degrees},{peak:.9},{break_angle:.9},{gain:.9}");
        }
    }
}
