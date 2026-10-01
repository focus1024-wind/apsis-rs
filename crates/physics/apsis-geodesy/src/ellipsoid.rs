//! Reference ellipsoids.

/// A biaxial reference ellipsoid: an ellipse of revolution about its minor
/// axis.
///
/// The shape is fixed by two numbers. This crate stores the semi-major axis
/// and the inverse flattening, which are the two defining parameters of the
/// WGS 84 ellipsoid in the standard. Every other quantity is derived, so
/// there is no risk of the parameters disagreeing with each other.
///
/// Lengths are in meters and angles are in radians.
///
/// 双轴参考椭球：绕短轴旋转所得的椭圆。
///
/// 形状由两个数确定。本 crate 存储半长轴与逆扁率，它们是标准中 WGS 84 椭球的
/// 两个定义参数。其余量都由二者导出，故各参数之间不会失配。
///
/// 长度单位为米，角度单位为弧度。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ellipsoid {
    semi_major_axis: f64,
    inverse_flattening: f64,
}

impl Ellipsoid {
    /// Creates an ellipsoid from its semi-major axis and inverse
    /// flattening.
    ///
    /// Lengths are in meters. The inverse flattening must be positive and
    /// finite; a value of infinity describes a sphere.
    ///
    /// 由半长轴与逆扁率创建椭球。
    ///
    /// 长度单位为米。逆扁率必须为正且有限；取无穷大则描述一个球。
    pub const fn new(semi_major_axis: f64, inverse_flattening: f64) -> Self {
        Self {
            semi_major_axis,
            inverse_flattening,
        }
    }

    /// Returns the semi-major axis, in meters.
    ///
    /// 返回半长轴，单位为米。
    pub const fn semi_major_axis(self) -> f64 {
        self.semi_major_axis
    }

    /// Returns the inverse flattening.
    ///
    /// 返回逆扁率。
    pub const fn inverse_flattening(self) -> f64 {
        self.inverse_flattening
    }

    /// Returns the flattening.
    ///
    /// This is zero for a sphere.
    ///
    /// 返回扁率。
    ///
    /// 对球其值为零。
    pub fn flattening(self) -> f64 {
        1.0 / self.inverse_flattening
    }

    /// Returns the semi-minor axis, in meters.
    ///
    /// 返回半短轴，单位为米。
    pub fn semi_minor_axis(self) -> f64 {
        self.semi_major_axis * (1.0 - self.flattening())
    }

    /// Returns the first eccentricity squared, a dimensionless number in
    /// `[0, 1)`.
    ///
    /// It is `2f - f^2` for flattening `f`.
    ///
    /// 返回第一偏心率平方，为位于 `[0, 1)` 的无量纲数。
    ///
    /// 对扁率 `f`，它等于 `2f - f^2`。
    pub fn eccentricity_squared(self) -> f64 {
        let flattening = self.flattening();
        2.0 * flattening - flattening * flattening
    }

    /// Returns the first eccentricity, a dimensionless number in `[0, 1)`.
    ///
    /// 返回第一偏心率，为位于 `[0, 1)` 的无量纲数。
    pub fn eccentricity(self) -> f64 {
        self.eccentricity_squared().sqrt()
    }

    /// Returns the second eccentricity squared, a dimensionless number.
    ///
    /// It is `e^2 / (1 - e^2)`, which appears in the closed form for
    /// converting a geodetic position to Cartesian coordinates.
    ///
    /// 返回第二偏心率平方，为无量纲数。
    ///
    /// 它等于 `e^2 / (1 - e^2)`，出现在把大地位置转换为笛卡尔坐标的闭式解中。
    pub fn second_eccentricity_squared(self) -> f64 {
        let squared = self.eccentricity_squared();
        squared / (1.0 - squared)
    }

    /// Returns the polar radius of curvature, in meters.
    ///
    /// This is the radius of curvature in the meridian at the pole, equal
    /// to `a / sqrt(1 - e^2)`.
    ///
    /// 返回极半径曲率，单位为米。
    ///
    /// 它是极点处子午线上的曲率半径，等于 `a / sqrt(1 - e^2)`。
    pub fn polar_radius_of_curvature(self) -> f64 {
        self.semi_major_axis / (1.0 - self.eccentricity_squared()).sqrt()
    }

    /// Returns the radius of curvature in the prime vertical at the given
    /// geodetic latitude, in meters.
    ///
    /// This is the distance from the surface to the polar axis measured
    /// along the ellipsoid normal, and it grows from the semi-major axis at
    /// the equator to [`Ellipsoid::polar_radius_of_curvature`] at the pole.
    ///
    /// The latitude is in radians.
    ///
    /// 返回给定大地纬度处卯酉圈上的曲率半径，单位为米。
    ///
    /// 它是沿椭球法线从表面量到极轴的距离，由赤道处的半长轴增长到极点的
    /// [`Ellipsoid::polar_radius_of_curvature`]。
    ///
    /// 纬度单位为弧度。
    pub fn prime_vertical_radius(self, latitude: f64) -> f64 {
        let sin = latitude.sin();
        self.semi_major_axis / (1.0 - self.eccentricity_squared() * sin * sin).sqrt()
    }

    /// Returns the radius of curvature in the meridian at the given
    /// geodetic latitude, in meters.
    ///
    /// The latitude is in radians.
    ///
    /// 返回给定大地纬度处子午线上的曲率半径，单位为米。
    ///
    /// 纬度单位为弧度。
    pub fn meridian_radius(self, latitude: f64) -> f64 {
        let sin = latitude.sin();
        let squared = self.eccentricity_squared();
        let denominator = 1.0 - squared * sin * sin;
        self.semi_major_axis * (1.0 - squared) / denominator.powf(1.5)
    }

    /// Returns the geocentric latitude corresponding to a geodetic
    /// latitude, in radians.
    ///
    /// The geocentric latitude is measured from the equatorial plane to the
    /// radius vector, while the geodetic latitude is measured to the
    /// ellipsoid normal. The geocentric value is the smaller of the two
    /// away from the equator and the poles.
    ///
    /// 返回与给定大地纬度对应的地心纬度，单位为弧度。
    ///
    /// 地心纬度自赤道面量到径向矢量，而大地纬度量到椭球法线。除赤道与极点外，
    /// 地心纬度小于大地纬度。
    pub fn geocentric_latitude(self, geodetic_latitude: f64) -> f64 {
        let squared = self.eccentricity_squared();
        let tangent = (1.0 - squared) * geodetic_latitude.tan();
        tangent.atan()
    }
}

/// The WGS 84 reference ellipsoid.
///
/// The two defining parameters are the semi-major axis `a = 6 378 137 m` and
/// the inverse flattening `1 / f = 298.257 223 563`.
///
/// Source: NIMA TR8350.2, third edition, 2000.
///
/// WGS 84 参考椭球。
///
/// 两个定义参数为半长轴 `a = 6 378 137 m` 与逆扁率 `1 / f = 298.257 223 563`。
///
/// 出处：NIMA TR8350.2 第三版，2000。
pub const WGS84: Ellipsoid = Ellipsoid::new(6_378_137.0, 298.257_223_563);

/// The Earth gravitational parameter for WGS 84, in cubic meters per second
/// squared.
///
/// This is held here rather than in `apsis-core` so that a caller working
/// with the ellipsoid can use the matching gravity field. The value is
/// `3.986 004 418e14 m^3/s^2`.
///
/// WGS 84 的地球引力参数，单位为立方米每二次方秒。
///
/// 该值置于此处而非 `apsis-core`，以便处理椭球的调用方使用与之匹配的重力场。
/// 其值为 `3.986 004 418e14 m^3/s^2`。
pub const WGS84_GRAVITATIONAL_PARAMETER: f64 = 3.986_004_418e14;
