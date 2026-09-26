//! Writes the coverage geometry this workspace computes for a set of cases.
//!
//! The output is CSV on standard output, with a header row. It is consumed
//! by `validation/python/constellation/check_coverage.py`, which recomputes
//! the same quantities from the spherical triangle relations and compares.
//!
//! The cases are chosen to span the interesting geometry: a low Earth orbit
//! at a low mask, where the footprint is small relative to the Earth; a
//! higher orbit, where it is large; the equatorial and polar cases; and a
//! zero mask, where the footprint reaches the horizon.
//!
//! Usage:
//!
//! ```text
//! cargo run -p apsis-rs --features network --example dump_coverage \
//!     > validation/data/expected/rust_coverage.csv
//! ```

use apsis_coverage::{
    earth_central_angle, elevation_from_central_angle, footprint_area, footprint_central_angle,
    footprint_fraction, nadir_angle, slant_range,
};
use core::f64::consts::PI;

/// The WGS 84 equatorial radius, in kilometers.
///
/// The coverage geometry is spherical, so a single radius is used. The
/// value comes from the shared constants rather than being repeated, so a
/// change to the ellipsoid reaches here.
const BODY_RADIUS_KM: f64 = apsis_core::constant::EARTH_EQUATORIAL_RADIUS;

/// One coverage case: an orbital radius and an elevation mask.
struct Case {
    /// A name for the case, so a failure in the report is readable.
    name: &'static str,
    /// The geocentric radius of the satellite, in kilometers.
    satellite_radius_km: f64,
    /// The elevation mask at the ground point, in degrees.
    elevation_mask_degrees: f64,
}

/// The cases checked.
///
/// The satellite radii span a low Earth orbit, two higher altitudes, and a
/// geostationary one. The masks span the horizon, a typical station mask, and
/// a strict one.
const CASES: &[Case] = &[
    Case {
        name: "leo_low_mask",
        satellite_radius_km: 6_928.137,
        elevation_mask_degrees: 5.0,
    },
    Case {
        name: "leo_ten_degrees",
        satellite_radius_km: 6_928.137,
        elevation_mask_degrees: 10.0,
    },
    Case {
        name: "leo_strict",
        satellite_radius_km: 6_928.137,
        elevation_mask_degrees: 30.0,
    },
    Case {
        name: "meo",
        satellite_radius_km: 26_560.0,
        elevation_mask_degrees: 10.0,
    },
    Case {
        name: "geo",
        satellite_radius_km: 42_164.0,
        elevation_mask_degrees: 10.0,
    },
    Case {
        name: "horizon",
        satellite_radius_km: 6_928.137,
        elevation_mask_degrees: 0.0,
    },
];

/// A set of central angles, in degrees, for the inverse relation check.
///
/// Each is fed to `elevation_from_central_angle` and the elevation it returns
/// is fed back through `earth_central_angle`, so the pair must compose to the
/// identity. The angles span zero to a little past the largest footprint the
/// cases produce.
const CENTRAL_ANGLES_DEGREES: &[f64] = &[0.0, 1.0, 5.0, 15.0, 30.0, 45.0, 60.0, 75.0];

fn main() {
    // The header names every column so that the Python side can read by name
    // rather than by position.
    println!(
        "case,satellite_radius_km,body_radius_km,elevation_mask_rad,\
         nadir_rad,central_rad,slant_range_km,footprint_central_rad,\
         footprint_area_km2,footprint_fraction"
    );

    for case in CASES {
        let elevation = case.elevation_mask_degrees * PI / 180.0;
        let nadir = nadir_angle(case.satellite_radius_km, BODY_RADIUS_KM, elevation)
            .expect("the case geometry is usable");
        let central = earth_central_angle(case.satellite_radius_km, BODY_RADIUS_KM, elevation)
            .expect("the case geometry is usable");
        let range = slant_range(case.satellite_radius_km, BODY_RADIUS_KM, elevation)
            .expect("the case geometry is usable");
        let footprint_central =
            footprint_central_angle(case.satellite_radius_km, BODY_RADIUS_KM, elevation)
                .expect("the case geometry is usable");
        let area = footprint_area(case.satellite_radius_km, BODY_RADIUS_KM, elevation)
            .expect("the case geometry is usable");
        let fraction = footprint_fraction(case.satellite_radius_km, BODY_RADIUS_KM, elevation)
            .expect("the case geometry is usable");

        println!(
            "{},{},{},{},{},{},{},{},{},{}",
            case.name,
            case.satellite_radius_km,
            BODY_RADIUS_KM,
            elevation,
            nadir,
            central,
            range,
            footprint_central,
            area,
            fraction
        );
    }

    // The inverse relation, written with its own header so the Python side
    // can tell the two blocks apart by the first column.
    println!();
    println!("inverse_central_degrees,elevation_rad,round_trip_central_degrees");
    for degrees in CENTRAL_ANGLES_DEGREES {
        let central = degrees * PI / 180.0;
        // Use the equatorial radius as the satellite radius for the inverse
        // check, so the geometry is the degenerate one where the elevation is
        // always zero. A more useful check uses a satellite altitude.
        let satellite_radius = BODY_RADIUS_KM + 550.0;
        let Ok(elevation) =
            elevation_from_central_angle(satellite_radius, BODY_RADIUS_KM, central).ok_or(())
        else {
            // Beyond the horizon the inverse has no solution, which is
            // reported as an empty row rather than skipped, so the count of
            // rows stays fixed.
            println!("{degrees},,");
            continue;
        };
        let Ok(back) = earth_central_angle(satellite_radius, BODY_RADIUS_KM, elevation).ok_or(())
        else {
            println!("{degrees},,");
            continue;
        };
        println!("{degrees},{elevation},{}", back * 180.0 / PI);
    }
}
