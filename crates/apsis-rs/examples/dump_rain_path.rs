//! Writes rain path attenuation computed by this workspace, for
//! cross-checking the ITU-R P.618-13 slant path method.
//!
//! Usage:
//!
//! ```text
//! cargo run -p apsis-rs --features rf --example dump_rain_path \
//!     > validation/data/expected/rust_rain_path.csv
//! ```

use apsis_atmosphere::{DEFAULT_RAIN_HEIGHT_KM, rain_attenuation};

/// The scenarios sampled. Each is a rain rate in millimeters per hour, a
/// frequency in gigahertz, an elevation in degrees, and a latitude in
/// degrees. The station height is zero and the rain height is the default.
const SCENARIOS: &[(f64, f64, f64, f64)] = &[
    (20.0, 12.0, 30.0, 45.0),
    (20.0, 12.0, 10.0, 45.0),
    (20.0, 20.0, 30.0, 45.0),
    (5.0, 12.0, 30.0, 45.0),
    (50.0, 12.0, 30.0, 45.0),
    (20.0, 12.0, 30.0, 10.0),
    (20.0, 12.0, 60.0, 45.0),
    (20.0, 6.0, 30.0, 45.0),
];

fn main() {
    println!(
        "rain_rate_mm_per_h,frequency_ghz,elevation_deg,latitude_deg,\
         specific_db_per_km,path_length_km,attenuation_db"
    );
    for &(rain_rate, frequency_ghz, elevation_degrees, latitude) in SCENARIOS {
        let elevation = elevation_degrees.to_radians();
        let Some(result) = rain_attenuation(
            rain_rate,
            frequency_ghz * 1.0e9,
            0.0,
            elevation,
            0.0,
            DEFAULT_RAIN_HEIGHT_KM,
            latitude,
        ) else {
            continue;
        };
        println!(
            "{rain_rate},{frequency_ghz},{elevation_degrees},{latitude},{:.12},{:.12},{:.12}",
            result.specific_db_per_km, result.path_length_km, result.attenuation_db
        );
    }
}
