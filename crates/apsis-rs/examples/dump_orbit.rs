//! Writes propagated states for a set of element sets and times.
//!
//! The output is CSV on standard output, with a header row that names the
//! element set index, the minutes from epoch, and the position and velocity
//! components. It is consumed by `validation/python/orbit/check_sgp4.py`,
//! which propagates the same element sets with the reference implementation
//! and compares.
//!
//! Usage:
//!
//! ```text
//! cargo run -p apsis-rs --features orbit --example dump_orbit \
//!     > validation/data/expected/rust_sgp4.csv
//! ```

use apsis_sgp4::{GravityModel, Satellite};
use apsis_tle::TwoLineElement;

/// The element sets used by the cross-check. They are the historical test
/// cases distributed with the SGP4 reference implementation, which cover
/// low, medium, and high eccentricity and the deep space branch.
const ELEMENT_SETS: &[&str] = &[
    // International Space Station, near circular low Earth orbit.
    concat!(
        "1 25544U 98067A   08264.51782528 -.00002182  00000-0 -11606-4 0  2927\r\n",
        "2 25544  51.6416 247.4627 0006703 130.5360 325.0288 15.72125391563537\r\n",
    ),
    // A highly eccentric orbit, which exercises the eccentric anomaly
    // iteration and the drag terms away from circular motion.
    concat!(
        "1 00005U 58002B   00179.78495062  .00000023  00000-0  28098-4 0  4753\r\n",
        "2 00005  34.2682 348.7242 1859667 331.7664  19.3264 10.82419157413667\r\n",
    ),
    // A geostationary transfer orbit with a low perigee.
    concat!(
        "1 27651U 03004A   06185.99374200  .00000109  00000-0  19909-4 0   445\r\n",
        "2 27651   7.1743 182.9443 7867251 269.0386  18.6601  1.03461646  9884\r\n",
    ),
    // A deep space orbit, whose period exceeds 225 minutes.
    concat!(
        "1 09880U 77021A   06174.85710247  .00000090  00000-0  10000-3 0  7322\r\n",
        "2 09880  64.1526 313.1296 6546925 103.4683 305.3980  2.00754276 13047\r\n",
    ),
];

/// The times to sample, in minutes from the element epoch.
const TIMES_MINUTES: &[f64] = &[
    0.0, 1.0, 60.0, 360.0, 720.0, 1_440.0, 2_880.0, 10_080.0, -1_440.0,
];

fn main() {
    println!("set,minutes,error,x_km,y_km,z_km,vx_kms,vy_kms,vz_kms");

    for (index, text) in ELEMENT_SETS.iter().enumerate() {
        let Ok(element) = TwoLineElement::parse(text) else {
            eprintln!("element set {index} did not parse");
            continue;
        };
        let Some(satellite) = Satellite::from_element(&element, GravityModel::WGS72) else {
            eprintln!("element set {index} was rejected");
            continue;
        };
        for minutes in TIMES_MINUTES {
            match satellite.propagate_minutes(*minutes) {
                Ok(state) => {
                    println!(
                        "{index},{minutes:.1},0,{:.9},{:.9},{:.9},{:.12},{:.12},{:.12}",
                        state.position.x,
                        state.position.y,
                        state.position.z,
                        state.velocity.x,
                        state.velocity.y,
                        state.velocity.z
                    );
                }
                Err(error) => {
                    println!("{index},{minutes:.1},{error},0,0,0,0,0,0");
                }
            }
        }
    }
}
