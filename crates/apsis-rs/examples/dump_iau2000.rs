//! Writes the IAU 2000/2006 Earth orientation quantities computed by this
//! workspace, for cross-checking against ERFA.
//!
//! Usage:
//!
//! ```text
//! cargo run -p apsis-rs --features physics --example dump_iau2000 \
//!     > validation/data/expected/rust_iau2000.csv
//! ```

use apsis_frames::iau2000::{
    PolarMotion, bias_precession_nutation_matrix, celestial_to_terrestrial_matrix,
    greenwich_apparent_sidereal_time, mean_obliquity, nutation_2000a,
    nutation_2000a_with_2006_adjustment, precession_angles,
};
use apsis_math::Mat3;

/// The Modified Julian Dates sampled. They span from 1970 to 2050 so that
/// the precession is exercised over a range of a century.
const MJD: &[f64] = &[
    40_587.0, 51_544.5, 53_000.0, 55_000.0, 57_000.0, 58_000.0, 60_000.0, 62_000.0,
];

/// The Terrestrial Time offset from TAI, in seconds.
const TT_MINUS_TAI: f64 = 32.184;

/// The TAI minus UTC offset used for this check, in seconds. It is the value
/// in force from 2017 onward.
const TAI_MINUS_UTC: f64 = 37.0;

/// Returns the TT Julian date for a Modified Julian Date on the UTC scale.
fn tt_julian_date(utc_mjd: f64) -> f64 {
    // TAI = UTC + 37, TT = TAI + 32.184, so TT = UTC + 69.184 seconds.
    utc_mjd + 2_400_000.5 + (TT_MINUS_TAI + TAI_MINUS_UTC) / 86_400.0
}

fn main() {
    println!("mjd,quantity,index,value");
    for &mjd in MJD {
        let tt = tt_julian_date(mjd);
        let t = (tt - 2_451_545.0) / 36_525.0;

        // The obliquity, in radians.
        println!("{mjd},obliquity,0,{:.17e}", mean_obliquity(t));

        // The nutation, from the IAU 2000A model.
        let nutation = nutation_2000a(t);
        println!("{mjd},nutation_longitude_a,0,{:.17e}", nutation.longitude);
        println!("{mjd},nutation_obliquity_a,0,{:.17e}", nutation.obliquity);

        // The nutation with the IAU 2006 adjustment.
        let adjusted = nutation_2000a_with_2006_adjustment(t);
        println!("{mjd},nutation_longitude_06a,0,{:.17e}", adjusted.longitude);
        println!("{mjd},nutation_obliquity_06a,0,{:.17e}", adjusted.obliquity);

        // The four precession angles.
        let angles = precession_angles(t);
        println!("{mjd},precession_gamma_bar,0,{:.17e}", angles.gamma_bar);
        println!("{mjd},precession_phi_bar,0,{:.17e}", angles.phi_bar);
        println!("{mjd},precession_psi_bar,0,{:.17e}", angles.psi_bar);
        println!("{mjd},precession_epsilon,0,{:.17e}", angles.epsilon);

        // The bias-precession-nutation matrix, which carries the celestial
        // reference frame to the true equator and equinox of date.
        let bpn = bias_precession_nutation_matrix(t);
        for row in 0..3 {
            for column in 0..3 {
                let value = bpn.get(row, column).expect("the indices are in range");
                println!("{mjd},bpn,{},{:.17e}", row * 3 + column, value);
            }
        }

        // Greenwich apparent sidereal time, in radians.
        println!(
            "{mjd},gast,0,{:.17e}",
            greenwich_apparent_sidereal_time(tt, tt)
        );

        // The celestial-to-terrestrial matrix, with zero polar motion.
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
#[allow(dead_code)]
fn _uses_matrix(_matrix: Mat3) {}
