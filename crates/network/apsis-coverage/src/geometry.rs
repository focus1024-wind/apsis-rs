//! The spherical geometry of a satellite, a ground point, and the Earth's
//! centre.
//!
//! The three points form a triangle. The angles of that triangle are the
//! elevation at the ground point, the nadir angle at the satellite, and the
//! central angle at the centre of the Earth. Because the ground point is
//! directly below the local horizon, the elevation and the adjacent angle at
//! the ground point are complementary, so the three angles satisfy
//!
//! ```text
//! elevation + nadir + central = 90 degrees
//! ```
//!
//! The quantities of interest follow from the law of sines applied to that
//! triangle:
//!
//! ```text
//! sin(nadir) / body_radius = cos(elevation) / satellite_radius
//! ```
//!
//! where the cosine appears because the angle at the ground point, inside
//! the triangle, is the complement of the elevation.
//!
//! All angles are in radians and all lengths in kilometers. The body radius
//! is the distance from the centre of the Earth to the ground point, so a
//! station above sea level should pass its own radius rather than the
//! equatorial radius.
//!
//! 卫星、地面点与地心的球面几何。
//!
//! 三个点构成一个三角形。该三角形的角是地面点处的仰角、卫星处的天底角，以及
//! 地心处的中心角。因为地面点正位于当地地平之下，仰角与地面点处的邻角互补，故
//! 三个角满足
//!
//! ```text
//! elevation + nadir + central = 90 degrees
//! ```
//!
//! 所关注的各量由正弦定理施加于该三角形得出：
//!
//! ```text
//! sin(nadir) / body_radius = cos(elevation) / satellite_radius
//! ```
//!
//! 其中出现余弦，是因为三角形内地面点处的角是仰角的余角。
//!
//! 所有角以弧度计，所有长度以千米计。天体半径是自地心到地面点的距离，故海拔高于
//! 海平面的站应传其自身的半径，而非赤道半径。

use core::f64::consts::FRAC_PI_2;

/// Returns the nadir angle at the satellite, in radians.
///
/// The nadir angle is measured at the satellite between the direction to the
/// centre of the Earth and the direction to the ground point. It is zero when
/// the ground point is directly below the satellite and grows as the point
/// moves toward the horizon.
///
/// Returns `None` when the radii are not usable, when the body radius
/// exceeds the satellite radius, or when the elevation is not finite.
///
/// 返回卫星处的天底角，单位为弧度。
///
/// 天底角在卫星处量取，介于到地心的方向与到地面点的方向之间。当地面点
/// 恰在卫星正下方时它为零，并随该点移向地平而增大。
///
/// 半径不可用、天体半径超过卫星半径，或仰角非有限时返回 `None`。
pub fn nadir_angle(satellite_radius: f64, body_radius: f64, elevation: f64) -> Option<f64> {
    if !satellite_radius.is_finite() || !body_radius.is_finite() || !elevation.is_finite() {
        return None;
    }
    if body_radius <= 0.0 || satellite_radius < body_radius {
        return None;
    }
    let sine = body_radius * elevation.cos() / satellite_radius;
    // Rounding can push the sine a little above one for a grazing geometry.
    // 对掠射几何，舍入可能把正弦略微推到一以上。
    Some(sine.clamp(-1.0, 1.0).asin())
}

/// Returns the earth central angle between the sub-satellite point and the
/// ground point, in radians.
///
/// The central angle is the angle at the centre of the Earth between the two
/// radial directions. It is the distance along the surface, divided by the
/// body radius.
///
/// Returns `None` when the geometry is not usable.
///
/// 返回星下点与地面点之间的地心中心角，单位为弧度。
///
/// 中心角是地心处两个径向方向之间的角。它是沿地表的距离除以天体半径。
///
/// 几何不可用时返回 `None`。
pub fn earth_central_angle(satellite_radius: f64, body_radius: f64, elevation: f64) -> Option<f64> {
    let nadir = nadir_angle(satellite_radius, body_radius, elevation)?;
    // Rounding can leave a small negative value at the zenith, where the
    // true angle is zero. The negative part is not physical, so it is
    // removed before the result is returned.
    // 在天顶处（真实角为零）舍入可能留下小的负值。该负值没有物理意义，
    // 故在返回结果前将其去掉。
    Some((FRAC_PI_2 - elevation - nadir).max(0.0))
}

/// Returns the slant range between the satellite and the ground point, in
/// kilometers.
///
/// The range is the straight line distance, not the distance along the
/// surface. It is the quantity a link budget needs, because it sets the free
/// space path loss.
///
/// The relation is the law of cosines applied to the triangle formed by the
/// satellite, the ground point, and the centre of the Earth,
///
/// ```text
/// range^2 = satellite_radius^2 + body_radius^2
///           - 2 satellite_radius body_radius cos(central)
/// ```
///
/// The law of cosines is used rather than the equivalent form
/// `Rs sin(central) / cos(elevation)` because the latter is singular at the
/// zenith, where the elevation is a right angle and its cosine is zero,
/// even though the range there is simply the altitude.
///
/// Returns `None` when the geometry is not usable.
///
/// 返回卫星与地面点之间的斜距，单位为千米。
///
/// 该距离是直线距离，而非沿地表的距离。它是链路预算所需的量，因为它决定自由
/// 空间路径损耗。
///
/// 该关系是余弦定理施加于卫星、地面点与地心构成的三角形：
///
/// ```text
/// range^2 = satellite_radius^2 + body_radius^2
///           - 2 satellite_radius body_radius cos(central)
/// ```
///
/// 此处用余弦定理而非等价形式 `Rs sin(central) / cos(elevation)`，因为后者在
/// 天顶处奇异，那里仰角是直角、其余弦为零，尽管那里的距离恰是高度。
///
/// 几何不可用时返回 `None`。
pub fn slant_range(satellite_radius: f64, body_radius: f64, elevation: f64) -> Option<f64> {
    let central = earth_central_angle(satellite_radius, body_radius, elevation)?;
    if !central.is_finite() || central < 0.0 {
        return None;
    }
    let squared = satellite_radius * satellite_radius + body_radius * body_radius
        - 2.0 * satellite_radius * body_radius * central.cos();
    if squared < 0.0 {
        return None;
    }
    Some(squared.sqrt())
}

/// Returns the elevation angle for a central angle, in radians.
///
/// This inverts [`earth_central_angle`]. It is used when the position of a
/// ground point relative to the sub-satellite point is known and the
/// elevation at that point is wanted.
///
/// The relation is
///
/// ```text
/// tan(elevation) = (cos(central) - body_radius / satellite_radius)
///                  / sin(central)
/// ```
///
/// Returns `None` when the radii are not usable or when the central angle is
/// not finite.
///
/// 返回某中心角对应的仰角，单位为弧度。
///
/// 它反解 [`earth_central_angle`]。当已知地面点相对星下点的位置而要求
/// 该点处仰角时使用它。
///
/// 关系为
///
/// ```text
/// tan(elevation) = (cos(central) - body_radius / satellite_radius)
///                  / sin(central)
/// ```
///
/// 半径不可用或中心角非有限时返回 `None`。
pub fn elevation_from_central_angle(
    satellite_radius: f64,
    body_radius: f64,
    central_angle: f64,
) -> Option<f64> {
    if !satellite_radius.is_finite() || !body_radius.is_finite() || !central_angle.is_finite() {
        return None;
    }
    if body_radius <= 0.0 || satellite_radius < body_radius {
        return None;
    }
    let ratio = body_radius / satellite_radius;
    let sine = central_angle.sin();
    if sine == 0.0 {
        // Directly below the satellite the elevation is a right angle.
        // 在卫星正下方仰角为直角。
        return Some(FRAC_PI_2);
    }
    Some(((central_angle.cos() - ratio) / sine).atan())
}
