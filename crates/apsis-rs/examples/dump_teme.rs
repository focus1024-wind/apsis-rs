//! Writes the TEME to Earth-fixed transformation computed by this workspace,
//! for cross-checking against astropy.
//!
//! Usage:
//!
//! ```text
//! cargo run -p apsis-rs --features physics --example dump_teme \
//!     > validation/data/expected/rust_teme.csv
//! ```

use apsis_frames::iau2000::{PolarMotion, teme_to_terrestrial_matrix};

/// The samples, as a UTC Modified Julian Date and a TEME position in
/// kilometers.
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
        let matrix = teme_to_terrestrial_matrix(2_400_000.5, utc_mjd, PolarMotion::NONE);
        let vector = apsis_math::Vec3::new(position[0], position[1], position[2]);
        let result = matrix * vector;
        println!("{utc_mjd},{:.9},{:.9},{:.9}", result.x, result.y, result.z);
    }
}
