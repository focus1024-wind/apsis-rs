//! The geometry of a slant path through the atmosphere.

use core::f64::consts::PI;

/// Returns the elevation angle for a zenith angle.
///
/// The elevation is measured from the horizon and the zenith angle from the
/// vertical, so the two add to a right angle.
pub fn elevation_from_zenith(zenith_angle: f64) -> f64 {
    PI / 2.0 - zenith_angle
}

/// Returns the zenith angle for an elevation angle.
pub fn zenith_to_elevation(elevation: f64) -> f64 {
    PI / 2.0 - elevation
}

/// Returns the air mass along a slant path at the given elevation angle.
///
/// The air mass is the ratio of the path length through a flat atmosphere to
/// the vertical path length. For a plane parallel atmosphere it is the
/// cosecant of the elevation,
///
/// ```text
/// m = 1 / sin(elevation)
/// ```
///
/// so the value is one at the zenith and grows without bound at the horizon.
/// The plane parallel assumption fails below about ten degrees elevation,
/// where the curvature of the Earth shortens the path compared with this
/// expression and the true air mass is smaller.
///
/// The value is not clamped at low elevation. A caller that needs a finite
/// bound near the horizon should impose one; this function reports the plane
/// parallel geometry it models.
///
/// Returns infinity at or below the horizon, where no direct path exists.
pub fn air_mass(elevation: f64) -> f64 {
    let sine = elevation.sin();
    if sine <= 0.0 {
        f64::INFINITY
    } else {
        1.0 / sine
    }
}
