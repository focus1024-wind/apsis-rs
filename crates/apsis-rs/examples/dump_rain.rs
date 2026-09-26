//! Writes the P.838-3 rain coefficients computed by this workspace, for
//! cross-checking against ITU-R P.838-3.
//!
//! Usage:
//!
//! ```text
//! cargo run -p apsis-rs --features rf --example dump_rain \
//!     > validation/data/expected/rust_rain.csv
//! ```

use apsis_atmosphere::rain_coefficients;

/// The frequencies sampled, in gigahertz.
const FREQUENCIES_GHZ: &[f64] = &[
    1.0, 2.0, 4.0, 6.0, 8.0, 10.0, 12.0, 15.0, 20.0, 22.0, 25.0, 30.0, 40.0, 50.0, 60.0, 70.0,
    100.0,
];

fn main() {
    println!("frequency_ghz,elevation_deg,polarisation_deg,k,alpha");
    for &frequency in FREQUENCIES_GHZ {
        // A horizontal path, which removes the elevation dependence and
        // isolates the coefficient curves.
        for &(elevation_degrees, polarisation_degrees) in
            &[(90.0_f64, 0.0_f64), (90.0, 90.0), (30.0, 0.0), (30.0, 45.0)]
        {
            let elevation = elevation_degrees.to_radians();
            let polarisation = polarisation_degrees.to_radians();
            let Some((k, alpha)) = rain_coefficients(frequency * 1.0e9, polarisation, elevation)
            else {
                continue;
            };
            println!("{frequency},{elevation_degrees},{polarisation_degrees},{k:.12},{alpha:.12}");
        }
    }
}
