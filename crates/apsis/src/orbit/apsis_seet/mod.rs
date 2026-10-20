//! Space environment models for mission effects.
//!
//! The module supplies the environment quantities a mission analysis needs:
//! the geomagnetic dipole field and the L shell of a position, the fraction
//! of a track that lies in the Earth's shadow, the direction of the Sun for
//! a date, the equilibrium temperature of a plate, and the particle fluence
//! accumulated over a duration. Each is an explicit, documented model rather
//! than a black box, so a caller can state the assumptions behind a result.
//!
//! 任务效应所需的空间环境模型。
//!
//! 本模块提供任务分析所需的环境量：某位置的地磁偶极场与 L 壳层、轨迹处于
//! 地球阴影中的比例、某日期的太阳方向、平板平衡温度，以及一段时间内累积的
//! 粒子注量。每一项都是显式且有文档的模型而非黑箱，故调用方可以说明结果背后
//! 的假设。

use crate::apsis_core::{Error, ErrorKind, Result};
use crate::apsis_math::Vec3;

/// The mean Earth radius used by the environment models, in kilometers.
///
/// 环境模型所用的地球平均半径，单位为千米。
pub const EARTH_RADIUS_KM: f64 = 6_371.0;

/// The equatorial surface field of the reference dipole, in nanotesla.
///
/// 参考偶极子的赤道表面磁场，单位为纳特斯拉。
pub const EQUATORIAL_FIELD_NT: f64 = 30_000.0;

/// The solar constant at one astronomical unit, in watts per square metre.
///
/// 一个天文单位处的太阳常数，单位为瓦特每平方米。
pub const SOLAR_CONSTANT_W_M2: f64 = 1_361.0;

/// The Stefan-Boltzmann constant, in watts per square metre kelvin to the
/// fourth.
///
/// 斯特藩-玻尔兹曼常数，单位为瓦特每平方米四次方开尔文。
pub const STEFAN_BOLTZMANN: f64 = 5.670_374_419e-8;

/// The geomagnetic field at one position, from an aligned dipole.
///
/// 某位置由对齐偶极子给出的地磁场。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DipoleField {
    /// Field vector in nanotesla.
    ///
    /// 磁场矢量，单位为纳特斯拉。
    pub field_nt: Vec3,
    /// Field magnitude in nanotesla.
    ///
    /// 磁场大小，单位为纳特斯拉。
    pub magnitude_nt: f64,
    /// Magnetic latitude in radians, positive north.
    ///
    /// 磁纬度，单位为弧度，向北为正。
    pub magnetic_latitude_rad: f64,
    /// L shell in Earth radii; infinite at the magnetic pole.
    ///
    /// L 壳层，单位为地球半径；在磁极处为无穷大。
    pub l_shell: f64,
    /// Geocentric radius in kilometers.
    ///
    /// 地心半径，单位为千米。
    pub radius_km: f64,
}

/// Returns the aligned dipole field at a geocentric position.
///
/// The dipole axis is the third axis, with the moment pointing south, which
/// gives a northward field at the equator and a southward field at the north
/// pole, matching the Earth. The surface equatorial magnitude is
/// [`EQUATORIAL_FIELD_NT`] and the field falls as the inverse cube of the
/// radius.
///
/// Returns an error when the position is not finite or is at the center.
///
/// 返回某地心位置处对齐偶极子的磁场。
///
/// 偶极轴为第三轴，磁矩指向南方，故赤道处磁场向北、北极处向南，与地球一致。
/// 表面赤道大小为 [`EQUATORIAL_FIELD_NT`]，磁场按半径的立方反比衰减。
///
/// 位置非有限或位于地心时返回错误。
pub fn dipole_field(position_km: Vec3) -> Result<DipoleField> {
    if !position_km.is_finite() {
        return Err(Error::with_message(
            ErrorKind::InvalidInput,
            "position must be finite",
        ));
    }
    let radius = position_km.norm();
    if radius == 0.0 {
        return Err(Error::with_message(
            ErrorKind::Undefined,
            "the dipole field is undefined at the center",
        ));
    }
    let x = position_km.x;
    let y = position_km.y;
    let z = position_km.z;
    let sin_latitude = z / radius;
    let cos_latitude = (x * x + y * y).sqrt() / radius;
    let ratio = (EARTH_RADIUS_KM / radius).powi(3);
    let radial = -2.0 * EQUATORIAL_FIELD_NT * ratio * sin_latitude;
    let latitudinal = EQUATORIAL_FIELD_NT * ratio * cos_latitude;
    let radial_unit = position_km * (1.0 / radius);
    let longitude = y.atan2(x);
    let (sin_longitude, cos_longitude) = longitude.sin_cos();
    let latitude_unit = Vec3::new(
        -sin_latitude * cos_longitude,
        -sin_latitude * sin_longitude,
        cos_latitude,
    );
    let field_nt = radial_unit * radial + latitude_unit * latitudinal;
    let l_shell = if cos_latitude.abs() < 1e-12 {
        f64::INFINITY
    } else {
        (radius / EARTH_RADIUS_KM) / (cos_latitude * cos_latitude)
    };
    Ok(DipoleField {
        field_nt,
        magnitude_nt: field_nt.norm(),
        magnetic_latitude_rad: sin_latitude.asin(),
        l_shell,
        radius_km: radius,
    })
}

/// Returns the unit vector from the Earth toward the Sun at a TT date.
///
/// The position uses the low precision solar coordinates of Meeus,
/// *Astronomical Algorithms*, chapter 25: the mean longitude and anomaly
/// give the ecliptic longitude, which the obliquity rotates into the
/// equatorial frame. The accuracy is a fraction of a degree, enough for
/// eclipse geometry but not for pointing.
///
/// Returns an error when the date is not finite.
///
/// 返回某 TT 日期从地球指向太阳的单位矢量。
///
/// 位置采用 Meeus《Astronomical Algorithms》第 25 章的低精度太阳坐标：由平黄经
/// 与平近点角得出黄经，再由黄赤交角转入赤道系。精度为几分之一度，足以用于
/// 阴影几何，但不适用于指向。
///
/// 日期非有限时返回错误。
pub fn sun_direction_unit(jd_tt: f64) -> Result<Vec3> {
    if !jd_tt.is_finite() {
        return Err(Error::with_message(
            ErrorKind::InvalidInput,
            "date must be finite",
        ));
    }
    let n = jd_tt - 2_451_545.0;
    let mean_longitude = (280.460 + 0.985_647_4 * n).to_radians();
    let mean_anomaly = (357.528 + 0.985_600_3 * n).to_radians();
    let ecliptic_longitude = mean_longitude
        + 1.915_f64.to_radians() * mean_anomaly.sin()
        + 0.020_f64.to_radians() * (2.0 * mean_anomaly).sin();
    let obliquity = (23.439 - 4.0e-7 * n).to_radians();
    let direction = Vec3::new(
        ecliptic_longitude.cos(),
        obliquity.cos() * ecliptic_longitude.sin(),
        obliquity.sin() * ecliptic_longitude.sin(),
    );
    direction
        .normalize()
        .ok_or_else(|| Error::with_message(ErrorKind::Undefined, "solar direction is undefined"))
}

/// Returns true when a position lies in the Earth's cylindrical shadow.
///
/// The shadow is the cylinder behind the Earth along the anti-solar
/// direction with the Earth radius. The model ignores the penumbra and the
/// atmosphere, which is the usual screening approximation.
///
/// Returns an error when the position or direction is not usable.
///
/// 当某位置处于地球柱形阴影中时返回真。
///
/// 阴影是地球背后沿背日方向、半径等于地球半径的圆柱。模型忽略半影与大气，
/// 这是常用的筛选近似。
///
/// 位置或方向不可用时返回错误。
pub fn is_in_eclipse(position_km: Vec3, sun_direction: Vec3) -> Result<bool> {
    if !position_km.is_finite() {
        return Err(Error::with_message(
            ErrorKind::InvalidInput,
            "position must be finite",
        ));
    }
    let sun = sun_direction.normalize().ok_or_else(|| {
        Error::with_message(ErrorKind::InvalidInput, "sun direction must be non-zero")
    })?;
    let projection = position_km.dot(sun);
    if projection >= 0.0 {
        return Ok(false);
    }
    let perpendicular = (position_km - sun * projection).norm();
    Ok(perpendicular <= EARTH_RADIUS_KM)
}

/// Returns the fraction of sampled positions that lie in eclipse.
///
/// Returns an error when the sample set is empty or a position is not
/// usable.
///
/// 返回采样位置中处于阴影内的比例。
///
/// 样本为空或某位置不可用时返回错误。
pub fn eclipse_fraction(positions_km: &[Vec3], sun_direction: Vec3) -> Result<f64> {
    if positions_km.is_empty() {
        return Err(Error::with_message(
            ErrorKind::InvalidInput,
            "at least one position is required",
        ));
    }
    let mut shadowed = 0_usize;
    for position in positions_km {
        if is_in_eclipse(*position, sun_direction)? {
            shadowed += 1;
        }
    }
    Ok(shadowed as f64 / positions_km.len() as f64)
}

/// Returns the equilibrium temperature of a flat plate, in kelvin.
///
/// The absorbed flux is the absorptivity times the sum of the direct solar
/// flux, the albedo-reflected solar flux, and the Earth infrared flux. The
/// plate radiates from both sides with the given emissivity, so the balance
/// is `alpha * flux = epsilon * sigma * T^4`. The model assumes a plate
/// normal to the Sun, no conduction, and no view-factor geometry.
///
/// Returns an error when a value is not finite or outside its range.
///
/// 返回平板平衡温度，单位为开尔文。
///
/// 吸收通量为吸收率乘以直射太阳通量、反照太阳通量与地球红外通量之和。平板
/// 两面以给定发射率辐射，故平衡为 `alpha * flux = epsilon * sigma * T^4`。
/// 模型假设平板正对太阳、无传导、无视角因子几何。
///
/// 某值非有限或超出范围时返回错误。
pub fn equilibrium_temperature(
    solar_flux_w_m2: f64,
    albedo: f64,
    earth_infrared_w_m2: f64,
    absorptivity: f64,
    emissivity: f64,
) -> Result<f64> {
    for value in [solar_flux_w_m2, earth_infrared_w_m2] {
        if !value.is_finite() || value < 0.0 {
            return Err(Error::with_message(
                ErrorKind::InvalidInput,
                "fluxes must be finite and non-negative",
            ));
        }
    }
    if !albedo.is_finite() || !(0.0..=1.0).contains(&albedo) {
        return Err(Error::with_message(
            ErrorKind::InvalidInput,
            "albedo must be within 0..=1",
        ));
    }
    if !absorptivity.is_finite() || !(0.0..=1.0).contains(&absorptivity) || absorptivity == 0.0 {
        return Err(Error::with_message(
            ErrorKind::InvalidInput,
            "absorptivity must be within 0..=1 and non-zero",
        ));
    }
    if !emissivity.is_finite() || !(0.0..=1.0).contains(&emissivity) || emissivity == 0.0 {
        return Err(Error::with_message(
            ErrorKind::InvalidInput,
            "emissivity must be within 0..=1 and non-zero",
        ));
    }
    let absorbed = absorptivity * (solar_flux_w_m2 * (1.0 + albedo) + earth_infrared_w_m2);
    Ok((absorbed / (emissivity * STEFAN_BOLTZMANN)).powf(0.25))
}

/// Returns the particle fluence over a duration, in particles per square
/// metre.
///
/// 返回一段时间内的粒子注量，单位为粒子每平方米。
pub fn particle_fluence(flux_per_m2_s: f64, seconds: f64) -> Result<f64> {
    if !flux_per_m2_s.is_finite() || flux_per_m2_s < 0.0 {
        return Err(Error::with_message(
            ErrorKind::InvalidInput,
            "flux must be finite and non-negative",
        ));
    }
    if !seconds.is_finite() || seconds < 0.0 {
        return Err(Error::with_message(
            ErrorKind::InvalidInput,
            "duration must be finite and non-negative",
        ));
    }
    Ok(flux_per_m2_s * seconds)
}
