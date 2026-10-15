//! The geometry of a slant path through the atmosphere.
//!
//! 穿过大气的斜路径的几何。

use core::f64::consts::PI;

/// Returns the elevation angle for a zenith angle.
///
/// The elevation is measured from the horizon and the zenith angle from the
/// vertical, so the two add to a right angle.
///
/// 返回某天顶角对应的仰角。
///
/// 仰角自地平量起，天顶角自垂线量起，故两者相加为直角。
#[must_use]
pub fn elevation_from_zenith(zenith_angle: f64) -> f64 {
    PI / 2.0 - zenith_angle
}

/// Returns the zenith angle for an elevation angle.
///
/// 返回某仰角对应的天顶角。
#[must_use]
pub fn zenith_to_elevation(elevation: f64) -> f64 {
    PI / 2.0 - elevation
}

/// Returns the air mass along a slant path at the given elevation angle.
///
/// The air mass is the ratio of the path length through a flat atmosphere to
/// the vertical path length. For a plane parallel atmosphere it is the
/// cosecant of the elevation,
///
/// <div>
/// $$
/// m = \frac{1}{\sin(\text{elevation})}
/// $$
/// </div>
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
///
/// 返回给定仰角下沿斜路径的大气质量。
///
/// 大气质量是穿过平坦大气的路径长度与垂直路径长度之比。对平面平行
/// 大气它是仰角的余割，
///
/// <div>
/// $$
/// m = \frac{1}{\sin(\text{elevation})}
/// $$
/// </div>
///
/// 故在天顶处为一，在地平处无界增长。平面平行假定在约十度仰角以下
/// 失效，那里地球曲率使路径比该式更短，真实大气质量更小。
///
/// 该值在低仰角处不被截断。需要在地平附近取有限上界的调用方应自行
/// 施加；本函数报告它所建模的平面平行几何。
///
/// 在地平及其以下不存在直接路径，返回无穷。
#[must_use]
pub fn air_mass(elevation: f64) -> f64 {
    let sine = elevation.sin();
    if sine <= 0.0 {
        f64::INFINITY
    } else {
        1.0 / sine
    }
}
