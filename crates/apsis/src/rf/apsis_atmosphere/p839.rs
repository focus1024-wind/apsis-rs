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
//!
//! ITU-R 建议书 P.839 的雨顶高度模型。
//!
//! 雨层的高度取为零度等温线在海平面以上的高度。该建议书把该高度列在一个全球网格
//! 上，而给定位置处的值由周围四个网格点之间的双线性插值求得。
//!
//! 雨顶高度是下落雪花融化的高度，其下降水为液态。其上的衰减小得多，故它是衰减层的
//! 有效厚度，也是建议书 P.618 路径计算的输入。
//!
//! 出处：ITU-R 建议书 P.839-4，*Rain height model for prediction methods*。

use crate::apsis_atmosphere::p839_table::ZERO_ISOTHERM_HEIGHT_METRES;

/// The grid resolution of the tabulated isotherm height, in degrees.
///
/// 列表等温线高度的网格分辨率，单位为度。
const STEP_DEGREES: f64 = 1.5;

/// The number of rows in the table, running from north to south.
///
/// 表中的行数，自北向南排列。
const ROWS: usize = 121;

/// The number of columns in the table, running from east to east.
///
/// 表中的列数，自东向东排列。
const COLUMNS: usize = 241;

/// The offset added to the isotherm height to obtain the rain height, in
/// kilometers.
///
/// The recommendation states that the rain height is the isotherm height
/// plus 0.36 km, which represents the depth of the melting layer below the
/// isotherm.
///
/// 加到等温线高度上以获得雨顶高度的偏移，单位为千米。
///
/// 建议书指出雨顶高度是等温线高度加 0.36 km，它代表等温线以下融化层的
/// 深度。
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
///
/// 返回某位置零度等温线的高度，以平均海平面以上千米计。
///
/// 纬度以度计、北正；经度以度计、东正。该值由列表网格双线性插值得到，
/// 经度先归约到 `[0, 360)`，故任何约定都被接受。
///
/// 任一坐标非有限，或纬度超出网格范围时返回 `None`。
///
/// 出处：ITU-R 建议书 P.839-4，式 1 及所列等温线高度。
#[must_use]
pub fn zero_isotherm_height(latitude: f64, longitude: f64) -> Option<f64> {
    if !latitude.is_finite() || !longitude.is_finite() {
        return None;
    }
    if !(-90.0..=90.0).contains(&latitude) {
        return None;
    }

    // The rows run from north to south, so the row index grows as the
    // latitude falls.
    // 各行自北向南排列，故行下标随纬度下降而增大。
    let row_position = (90.0 - latitude) / STEP_DEGREES;
    let row_lower = (row_position.floor() as usize).min(ROWS - 1);
    let row_upper = (row_lower + 1).min(ROWS - 1);
    let row_fraction = row_position - row_lower as f64;

    // The columns run from 0 to 360 degrees east, wrapping at the seam.
    // 各列自东经 0 度到 360 度排列，在接缝处环绕。
    let longitude = longitude.rem_euclid(360.0);
    let column_position = longitude / STEP_DEGREES;
    let column_lower = (column_position.floor() as usize).min(COLUMNS - 1);
    let column_upper = (column_lower + 1) % COLUMNS;
    let column_fraction = column_position - column_lower as f64;

    let sample =
        |row: usize, column: usize| f64::from(ZERO_ISOTHERM_HEIGHT_METRES[row][column]) / 1000.0;

    // Bilinear interpolation.
    // 双线性插值。
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
///
/// 返回某位置的雨顶高度，以平均海平面以上千米计。
///
/// 它是零度等温线高度加 [`RAIN_HEIGHT_OFFSET_KM`]，如建议书所定义。
///
/// 坐标不可用时返回 `None`。
#[must_use]
pub fn rain_height(latitude: f64, longitude: f64) -> Option<f64> {
    Some(zero_isotherm_height(latitude, longitude)? + RAIN_HEIGHT_OFFSET_KM)
}

/// Returns the grid resolution of the tabulated isotherm height, in degrees.
///
/// 返回列表等温线高度的网格分辨率，单位为度。
#[must_use]
pub const fn grid_step_degrees() -> f64 {
    STEP_DEGREES
}
