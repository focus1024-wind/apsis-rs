//! Writes the exact slant path gaseous attenuation computed by this
//! workspace, for cross-checking against ITU-R P.676-12.
//!
//! Usage:
//!
//! ```text
//! cargo run -p apsis-rs --features rf --example dump_gas_exact \
//!     > validation/data/expected/rust_gas_exact.csv
//! ```

use apsis_atmosphere::{Atmosphere, slant_path_attenuation_exact};

/// The scenarios sampled, as frequency in gigahertz and elevation in
/// degrees.
const SCENARIOS: &[(f64, f64)] = &[
    (12.0, 90.0),
    (12.0, 30.0),
    (12.0, 10.0),
    (12.0, 5.0),
    (12.0, 1.0),
    (20.0, 30.0),
    (20.0, 5.0),
    (30.0, 10.0),
    (60.0, 30.0),
    (1.0, 30.0),
    (100.0, 5.0),
];

fn main() {
    println!("frequency_ghz,elevation_deg,attenuation_db");
    let atmosphere = Atmosphere::REFERENCE;
    for &(frequency, elevation_degrees) in SCENARIOS {
        let Some(attenuation) =
            slant_path_attenuation_exact(frequency, elevation_degrees.to_radians(), atmosphere)
        else {
            continue;
        };
        println!("{frequency},{elevation_degrees},{attenuation:.12}");
    }
}
