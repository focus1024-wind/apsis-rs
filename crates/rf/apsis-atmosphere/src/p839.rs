//! The rain height model of ITU-R Recommendation P.839.
//!
//! The height of the rain layer is taken as the height of the zero degree
//! isotherm above sea level. The recommendation tabulates that height on a
//! global grid, and the value at a given location is found by bilinear
//! interpolation between the four surrounding grid points.
//!
//! The rain height is the height at which the falling snow melts, below
//! which the precipitation is liquid. Above it, the attenuation is much
//! smaller, so it is the effective thickness of the attenuating layer and an
//! input to the path calculation of Recommendation P.618.
//!
//! Source: ITU-R Recommendation P.839-4, *Rain height model for prediction
//! methods*.

use crate::p839_table::ZERO_ISOTHERM_HEIGHT_METRES;

/// The grid resolution of the tabulated isotherm height, in degrees.
const STEP_DEGREES: f64 = 1.5;

/// The number of rows in the table, running from north to south.
const ROWS: usize = 121;

/// The number of columns in the table, running from east to east.
const COLUMNS: usize = 241;

/// The offset added to the isotherm height to obtain the rain height, in
/// kilometers.
///
/// The recommendation states that the rain height is the isotherm height
/// plus 0.36 km, which represents the depth of the melting layer below the
/// isotherm.
pub const RAIN_HEIGHT_OFFSET_KM: f64 = 0.36;

/// Returns the height of the zero degree isotherm at a location, in
/// kilometers above mean sea level.
///
/// The latitude is in degrees, positive north, and the longitude in degrees,
/// positive east. The value is interpolated bilinearly from the tabulated
/// grid, and longitudes are reduced into `[0, 360)` first so that any
/// convention is accepted.
///
/// Returns `None` when either coordinate is not finite, or when the latitude
/// is outside the range of the grid.
///
/// Source: ITU-R Recommendation P.839-4, equation 1 and the tabulated
/// isotherm height.
pub fn zero_isotherm_height(latitude: f64, longitude: f64) -> Option<f64> {
    if !latitude.is_finite() || !longitude.is_finite() {
        return None;
    }
    if !(-90.0..=90.0).contains(&latitude) {
        return None;
    }

    // The rows run from north to south, so the row index grows as the
    // latitude falls.
    let row_position = (90.0 - latitude) / STEP_DEGREES;
    let row_lower = (row_position.floor() as usize).min(ROWS - 1);
    let row_upper = (row_lower + 1).min(ROWS - 1);
    let row_fraction = row_position - row_lower as f64;

    // The columns run from 0 to 360 degrees east, wrapping at the seam.
    let longitude = longitude.rem_euclid(360.0);
    let column_position = longitude / STEP_DEGREES;
    let column_lower = (column_position.floor() as usize).min(COLUMNS - 1);
    let column_upper = (column_lower + 1) % COLUMNS;
    let column_fraction = column_position - column_lower as f64;

    let sample =
        |row: usize, column: usize| f64::from(ZERO_ISOTHERM_HEIGHT_METRES[row][column]) / 1000.0;

    // Bilinear interpolation.
    let top = sample(row_lower, column_lower)
        + (sample(row_lower, column_upper) - sample(row_lower, column_lower)) * column_fraction;
    let bottom = sample(row_upper, column_lower)
        + (sample(row_upper, column_upper) - sample(row_upper, column_lower)) * column_fraction;
    Some(top + (bottom - top) * row_fraction)
}

/// Returns the rain height at a location, in kilometers above mean sea level.
///
/// This is the zero degree isotherm height plus
/// [`RAIN_HEIGHT_OFFSET_KM`], as the recommendation defines it.
///
/// Returns `None` when the coordinates are not usable.
pub fn rain_height(latitude: f64, longitude: f64) -> Option<f64> {
    Some(zero_isotherm_height(latitude, longitude)? + RAIN_HEIGHT_OFFSET_KM)
}

/// Returns the grid resolution of the tabulated isotherm height, in degrees.
pub const fn grid_step_degrees() -> f64 {
    STEP_DEGREES
}
