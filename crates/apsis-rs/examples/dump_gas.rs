//! Writes the specific and zenith gaseous attenuation computed by this
//! workspace, for cross-checking against ITU-R P.676-12.
//!
//! 写出本工作区计算的比与天顶气体衰减，用于对照 ITU-R P.676-12 交叉核对。
//!
//! Usage:
//!
//! ```text
//! cargo run -p apsis-rs --features rf --example dump_gas \
//!     > validation/data/expected/rust_gas.csv
//! ```

use apsis_atmosphere::{
    Atmosphere, equivalent_heights, specific_attenuation_dry_air, specific_attenuation_water_vapour,
};

/// The frequencies sampled, in gigahertz. They span the oxygen complex near
/// 60 GHz, the water vapour lines at 22.235 and 183.31 GHz, and the windows
/// between them.
///
/// 所采样的频率，单位为吉赫兹。它们跨越 60 GHz 附近的氧复合体、22.235 与
/// 183.31 GHz 的水汽线，以及其间的窗口。
const FREQUENCIES_GHZ: &[f64] = &[
    1.0, 2.0, 4.0, 6.0, 10.0, 12.0, 15.0, 20.0, 22.235, 30.0, 40.0, 50.0, 60.0, 70.0, 100.0, 118.0,
    183.0, 300.0,
];

fn main() {
    println!("frequency_ghz,dry_db_per_km,wet_db_per_km,dry_height_km,wet_height_km,zenith_db");
    let atmosphere = Atmosphere::REFERENCE;
    for &frequency in FREQUENCIES_GHZ {
        let Some(dry) = specific_attenuation_dry_air(frequency, atmosphere) else {
            continue;
        };
        let Some(wet) = specific_attenuation_water_vapour(frequency, atmosphere) else {
            continue;
        };
        let Some(heights) = equivalent_heights(frequency, atmosphere) else {
            continue;
        };
        let zenith = dry * heights.dry_km + wet * heights.wet_km;
        println!(
            "{frequency},{dry:.12},{wet:.12},{:.9},{:.9},{zenith:.12}",
            heights.dry_km, heights.wet_km
        );
    }
}
