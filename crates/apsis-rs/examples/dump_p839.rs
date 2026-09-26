//! Writes the rain height computed by this workspace, for cross-checking
//! against ITU-R P.839-4.
//!
//! Usage:
//!
//! ```text
//! cargo run -p apsis-rs --features rf --example dump_p839 \
//!     > validation/data/expected/rust_p839.csv
//! ```

use apsis_atmosphere::{rain_height, zero_isotherm_height};

/// The locations sampled, as latitude and longitude in degrees.
const POINTS: &[(f64, f64)] = &[
    (0.0, 0.0),
    (45.0, 0.0),
    (-45.0, 90.0),
    (51.5, -0.1),
    (35.7, 139.7),
    (-33.9, 151.2),
    (28.6, -80.6),
    (0.0, 359.9),
    (0.0, -0.1),
    (60.0, 30.0),
    (-60.0, 300.0),
    (23.5, 45.0),
    (89.9, 0.0),
    (-89.9, 180.0),
    (10.0, 100.0),
];

fn main() {
    println!("latitude_deg,longitude_deg,isotherm_km,rain_height_km");
    for &(latitude, longitude) in POINTS {
        let Some(isotherm) = zero_isotherm_height(latitude, longitude) else {
            continue;
        };
        let Some(height) = rain_height(latitude, longitude) else {
            continue;
        };
        println!("{latitude},{longitude},{isotherm:.12},{height:.12}");
    }
}
