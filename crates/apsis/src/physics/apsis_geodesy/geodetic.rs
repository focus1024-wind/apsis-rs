//! Geodetic positions and their conversion to Earth-fixed Cartesian
//! coordinates.
//!
//! A Cartesian position here is a plain [`Vec3`] in meters. This module
//! deliberately does not name the frame: the same conversion applies to any
//! frame whose third axis is the ellipsoid's minor axis, and the choice of
//! frame is the responsibility of the caller. `apsis_coord` supplies
//! position types that carry the frame.
//!
//! 大地位置及其到地固直角坐标的换算。
//!
//! 此处的直角坐标位置是以米计的普通 [`Vec3`]。本模块有意不为该坐标系
//! 命名：同一换算适用于第三轴为椭球短轴的任何坐标系，坐标系的选择是调用方
//! 的责任。`apsis_coord` 提供携带坐标系的各位置类型。

use crate::apsis_geodesy::ellipsoid::Ellipsoid;
use crate::apsis_math::Vec3;

/// A geodetic position: `latitude_rad`, `longitude_rad`, and `height_m` above an ellipsoid.
///
/// Latitude and `longitude_rad` are in radians and `height_m` is in meters. Latitude
/// is geodetic, measured between the ellipsoid normal and the equatorial
/// plane, and lies in `[-pi/2, pi/2]`. Longitude is measured east from the
/// prime meridian. Height is the distance along the ellipsoid normal, taken
/// as positive outside the ellipsoid.
///
/// 大地位置：纬度、经度与椭球高。
///
/// 纬度与经度用弧度，高用米。纬度为大地纬度，在椭球法线与赤道面之间量取，位于
/// `[-pi/2, pi/2]`。经度自本初子午线向东量取。高是沿椭球法线的距离，椭球
/// 之外取正。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Geodetic {
    /// Geodetic `latitude_rad`, in radians.
    ///
    /// 大地纬度，单位为弧度。
    pub latitude_rad: f64,
    /// Longitude, in radians, positive east.
    ///
    /// 经度，单位为弧度，向东为正。
    pub longitude_rad: f64,
    /// Height above the ellipsoid, in meters.
    ///
    /// 椭球高，单位为米。
    pub height_m: f64,
}

impl Geodetic {
    /// Creates a geodetic position from `latitude_rad`, `longitude_rad`, and `height_m`.
    ///
    /// 由纬度、经度与高创建大地位置。
    #[must_use]
    pub const fn new(latitude_rad: f64, longitude_rad: f64, height_m: f64) -> Self {
        Self {
            latitude_rad,
            longitude_rad,
            height_m,
        }
    }

    /// Converts this geodetic position to Earth-fixed Cartesian coordinates
    /// on the given ellipsoid.
    ///
    /// The frame has its third axis along the ellipsoid's minor axis through
    /// the north pole, its first axis through the intersection of the
    /// equator and the prime meridian, and its second axis completing a
    /// right-handed set. The result is in meters.
    ///
    /// The closed form is
    ///
    /// <div>
    /// $$
    /// \begin{aligned}
    /// N &= \frac{a}{\sqrt{1 - e^2 \sin^2\varphi}} \\
    /// x &= (N + h) \cos\varphi \cos\lambda \\
    /// y &= (N + h) \cos\varphi \sin\lambda \\
    /// z &= \left(N(1 - e^2) + h\right) \sin\varphi
    /// \end{aligned}
    /// $$
    /// </div>
    ///
    /// where `N` is the prime vertical radius of curvature. This form is
    /// exact everywhere, including at the poles, where the cosine factors
    /// vanish.
    ///
    /// 把该大地位置转换为给定椭球上的地固笛卡尔坐标。
    ///
    /// 该系的第三轴沿椭球短轴指向北极，第一轴过赤道与本初子午线之交，第二轴构成
    /// 右手系。结果单位为米。
    ///
    /// 闭式解为
    ///
    /// <div>
    /// $$
    /// \begin{aligned}
    /// N &= \frac{a}{\sqrt{1 - e^2 \sin^2\varphi}} \\
    /// x &= (N + h) \cos\varphi \cos\lambda \\
    /// y &= (N + h) \cos\varphi \sin\lambda \\
    /// z &= \left(N(1 - e^2) + h\right) \sin\varphi
    /// \end{aligned}
    /// $$
    /// </div>
    ///
    /// 其中 `N` 为卯酉圈曲率半径。该形式处处精确，包括余弦因子为零的极点。
    #[must_use]
    pub fn to_cartesian(self, ellipsoid: Ellipsoid) -> Vec3 {
        cartesian_from_geodetic(self, ellipsoid)
    }

    /// Returns `true` when every field is finite and the `latitude_rad` and
    /// `longitude_rad` are inside their ranges.
    ///
    /// 当各字段都有限，且纬度与经度位于其范围内时返回 `true`。
    #[must_use]
    pub fn is_valid(self) -> bool {
        let half_pi = core::f64::consts::FRAC_PI_2;
        self.latitude_rad.is_finite()
            && self.longitude_rad.is_finite()
            && self.height_m.is_finite()
            && self.latitude_rad >= -half_pi
            && self.latitude_rad <= half_pi
            && self.longitude_rad >= -core::f64::consts::PI
            && self.longitude_rad <= core::f64::consts::PI
    }
}

/// Converts a geodetic position to Earth-fixed Cartesian coordinates.
///
/// See [`Geodetic::to_cartesian`] for the frame definition and the closed
/// form.
///
/// 把大地位置转换为地固笛卡尔坐标。
///
/// 坐标系定义与闭式解见 [`Geodetic::to_cartesian`]。
#[must_use]
pub fn cartesian_from_geodetic(geodetic: Geodetic, ellipsoid: Ellipsoid) -> Vec3 {
    let Geodetic {
        latitude_rad,
        longitude_rad,
        height_m,
    } = geodetic;
    let (sin_latitude, cos_latitude) = latitude_rad.sin_cos();
    let (sin_longitude, cos_longitude) = longitude_rad.sin_cos();
    let prime_vertical = ellipsoid.prime_vertical_radius_m(latitude_rad);
    let eccentricity_squared = ellipsoid.eccentricity_squared();

    let horizontal = prime_vertical + height_m;
    let x = horizontal * cos_latitude * cos_longitude;
    let y = horizontal * cos_latitude * sin_longitude;
    let z = (prime_vertical * (1.0 - eccentricity_squared) + height_m) * sin_latitude;
    Vec3::new(x, y, z)
}

/// Converts Earth-fixed Cartesian coordinates to a geodetic position.
///
/// The `latitude_rad` and `height_m` are recovered by Bowring's method, which solves
/// the meridian equation in closed form and then refines the result with a
/// few fixed point steps. The iteration converges to well below a
/// micrometer for positions from below the ellipsoid out to geostationary
/// radius.
///
/// Returns `None` when the position is not finite, or when it lies on the
/// polar axis so that no `longitude_rad` is defined.
///
/// 把地固笛卡尔坐标转换为大地位置。
///
/// 纬度与高由 Bowring 方法求得：先以闭式解解出子午线方程，再用若干不动点迭代
/// 精化。对从椭球下方到地球同步半径范围内的位置，该迭代收敛到远优于一微米。
///
/// 当位置非有限，或落在极轴上以致经度未定义时返回 `None`。
#[must_use]
pub fn geodetic_from_cartesian(cartesian: Vec3, ellipsoid: Ellipsoid) -> Option<Geodetic> {
    let Vec3 { x, y, z } = cartesian;
    if !cartesian.is_finite() {
        return None;
    }

    let squared = ellipsoid.eccentricity_squared();
    let second_squared = ellipsoid.second_eccentricity_squared();
    let semi_major = ellipsoid.semi_major_axis_m();
    let semi_minor = ellipsoid.semi_minor_axis_m();

    let horizontal_squared = x * x + y * y;
    if horizontal_squared == 0.0 {
        // On the polar axis the longitude_rad is undefined.
        // 在极轴上经度未定义。
        return None;
    }
    let horizontal = horizontal_squared.sqrt();
    let longitude_rad = y.atan2(x);

    // Bowring's initial latitude_rad estimate, obtained by treating the point
    // as if it lay on a locally spherical surface.
    // Bowring 的纬度初值估计，做法是把该点视为位于局部球面上。
    let theta = (z * semi_major).atan2(horizontal * semi_minor);
    let (sin_theta, cos_theta) = theta.sin_cos();
    let mut latitude_rad = (z + second_squared * semi_minor * sin_theta * sin_theta * sin_theta)
        .atan2(horizontal - squared * semi_major * cos_theta * cos_theta * cos_theta);

    // Refine with fixed point steps on the ellipsoid equation. Three steps
    // reduce the error to the floating point noise floor for every
    // terrestrial altitude.
    // 用椭球方程的不动点迭代精化。三步即可把误差降到浮点噪声水平，适用于
    // 任何地面高度。
    for _ in 0..3 {
        let (sin_latitude, _) = latitude_rad.sin_cos();
        latitude_rad =
            (z + squared * ellipsoid.prime_vertical_radius_m(latitude_rad) * sin_latitude)
                .atan2(horizontal);
    }

    let (sin_latitude, cos_latitude) = latitude_rad.sin_cos();
    // Guard against a latitude_rad that has converged exactly to a pole, where
    // the division below would be singular.
    // 防止纬度恰好收敛到极点，否则下文的除法为奇异。
    if cos_latitude == 0.0 {
        return None;
    }
    let height_m = if cos_latitude.abs() > 0.5 {
        horizontal / cos_latitude - ellipsoid.prime_vertical_radius_m(latitude_rad)
    } else {
        z / sin_latitude - ellipsoid.prime_vertical_radius_m(latitude_rad) * (1.0 - squared)
    };

    Some(Geodetic::new(latitude_rad, longitude_rad, height_m))
}
