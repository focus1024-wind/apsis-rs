//! Sensor fields of view and target visibility.
//!
//! A sensor is an origin, a boresight direction, and a field of view. The
//! module tests whether a target lies inside a cone or a rectangular field,
//! and computes the ground footprint of a nadir-pointing cone.
//!
//! 传感器的视场与目标可见性。
//!
//! 传感器由一个原点、一个视轴方向与一个视场组成。本模块判定目标是否落在圆锥
//! 或矩形视场内，并计算天底指向圆锥的地面覆盖角。

use crate::apsis_core::{Error, ErrorKind, Result};
use crate::apsis_math::Vec3;

/// The mean Earth radius used by the footprint geometry, in kilometers.
///
/// 覆盖几何所用的地球平均半径，单位为千米。
pub const EARTH_RADIUS_KM: f64 = 6_371.0;

/// The result of observing a target with a sensor.
///
/// 用传感器观测目标的结果。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SensorObservation {
    /// Whether the target is inside the field and range limits.
    ///
    /// 目标是否位于视场与距离限制之内。
    pub visible: bool,
    /// Range to the target in kilometers.
    ///
    /// 到目标的距离，单位为千米。
    pub range_km: f64,
    /// Angle between the boresight and the target, in radians.
    ///
    /// 视轴与目标之间的夹角，单位为弧度。
    pub off_axis_rad: f64,
    /// Whether the target is inside the field of view.
    ///
    /// 目标是否位于视场内。
    pub within_field: bool,
    /// Whether the target is inside the range limits.
    ///
    /// 目标是否位于距离限制内。
    pub within_range: bool,
    /// Horizontal angle from the boresight for a rectangular field.
    ///
    /// 矩形视场中相对视轴的水平角。
    pub horizontal_rad: Option<f64>,
    /// Vertical angle from the boresight for a rectangular field.
    ///
    /// 矩形视场中相对视轴的垂直角。
    pub vertical_rad: Option<f64>,
}

/// Returns true when a range is within the given limits.
///
/// 距离位于给定限制内时返回真。
fn range_within(range_km: f64, minimum_km: f64, maximum_km: f64) -> bool {
    range_km >= minimum_km && range_km <= maximum_km
}

/// Observes a target with a conical field of view.
///
/// The half angle is measured from the boresight. The minimum and maximum
/// ranges are inclusive; the maximum may be infinite. Returns an error when
/// the geometry or a limit is not usable.
///
/// 用圆锥视场观测目标。
///
/// 半角自视轴量起。最小与最大距离为闭区间；最大值可为无穷。几何或限制不可用
/// 时返回错误。
pub fn observe_cone(
    origin_km: Vec3,
    boresight: Vec3,
    target_km: Vec3,
    half_angle_rad: f64,
    minimum_range_km: f64,
    maximum_range_km: f64,
) -> Result<SensorObservation> {
    if !origin_km.is_finite() || !target_km.is_finite() {
        return Err(Error::with_message(
            ErrorKind::InvalidInput,
            "positions must be finite",
        ));
    }
    if !half_angle_rad.is_finite() || !(0.0..=core::f64::consts::PI).contains(&half_angle_rad) {
        return Err(Error::with_message(
            ErrorKind::InvalidInput,
            "half angle must be within 0..=pi",
        ));
    }
    if !minimum_range_km.is_finite()
        || minimum_range_km < 0.0
        || maximum_range_km < minimum_range_km
        || maximum_range_km.is_nan()
    {
        return Err(Error::with_message(
            ErrorKind::InvalidInput,
            "range limits must satisfy 0 <= min <= max",
        ));
    }
    let boresight = boresight.normalize().ok_or_else(|| {
        Error::with_message(ErrorKind::InvalidInput, "boresight must be non-zero")
    })?;
    let offset = target_km - origin_km;
    let range = offset.norm();
    if range == 0.0 {
        return Err(Error::with_message(
            ErrorKind::Undefined,
            "target coincides with the sensor",
        ));
    }
    let direction = offset * (1.0 / range);
    let off_axis = boresight.dot(direction).clamp(-1.0, 1.0).acos();
    let within_field = off_axis <= half_angle_rad + 1e-12;
    let within_range = range_within(range, minimum_range_km, maximum_range_km);
    Ok(SensorObservation {
        visible: within_field && within_range,
        range_km: range,
        off_axis_rad: off_axis,
        within_field,
        within_range,
        horizontal_rad: None,
        vertical_rad: None,
    })
}

/// Observes a target with a rectangular field of view.
///
/// The horizontal and vertical angles are measured from the boresight in
/// the frame defined by the boresight and the up vector. A target behind
/// the sensor plane is never inside the field. Returns an error when the
/// geometry, the up vector, or a limit is not usable.
///
/// 用矩形视场观测目标。
///
/// 水平与垂直角在由视轴与上方向定义的坐标系中自视轴量起。位于传感器平面
/// 之后的目标绝不在视场内。几何、上方向或限制不可用时返回错误。
pub fn observe_rectangular(
    origin_km: Vec3,
    boresight: Vec3,
    up: Vec3,
    target_km: Vec3,
    half_horizontal_rad: f64,
    half_vertical_rad: f64,
    minimum_range_km: f64,
    maximum_range_km: f64,
) -> Result<SensorObservation> {
    if !half_horizontal_rad.is_finite()
        || !(0.0..=core::f64::consts::FRAC_PI_2).contains(&half_horizontal_rad)
        || !half_vertical_rad.is_finite()
        || !(0.0..=core::f64::consts::FRAC_PI_2).contains(&half_vertical_rad)
    {
        return Err(Error::with_message(
            ErrorKind::InvalidInput,
            "rectangular half angles must be within 0..=pi/2",
        ));
    }
    let axis = boresight.normalize().ok_or_else(|| {
        Error::with_message(ErrorKind::InvalidInput, "boresight must be non-zero")
    })?;
    let up = up.normalize().ok_or_else(|| {
        Error::with_message(ErrorKind::InvalidInput, "up vector must be non-zero")
    })?;
    let right = axis.cross(up).normalize().ok_or_else(|| {
        Error::with_message(ErrorKind::Undefined, "up is parallel to the boresight")
    })?;
    let vertical = right.cross(axis);
    let base = observe_cone(
        origin_km,
        axis,
        target_km,
        0.0,
        minimum_range_km,
        maximum_range_km,
    )?;
    let offset = target_km - origin_km;
    let range = offset.norm();
    let direction = offset * (1.0 / range);
    let forward = direction.dot(axis);
    if forward <= 0.0 {
        return Ok(SensorObservation {
            visible: false,
            within_field: false,
            horizontal_rad: Some(0.0),
            vertical_rad: Some(0.0),
            ..base
        });
    }
    let horizontal = direction.dot(right).atan2(forward);
    let vertical_angle = direction.dot(vertical).atan2(forward);
    let within_field = horizontal.abs() <= half_horizontal_rad + 1e-12
        && vertical_angle.abs() <= half_vertical_rad + 1e-12;
    Ok(SensorObservation {
        visible: within_field && base.within_range,
        range_km: base.range_km,
        off_axis_rad: base.off_axis_rad,
        within_field,
        within_range: base.within_range,
        horizontal_rad: Some(horizontal),
        vertical_rad: Some(vertical_angle),
    })
}

/// Returns the ground central angle of a nadir-pointing cone.
///
/// The satellite is at `EARTH_RADIUS_KM + altitude`, the cone axis passes
/// through the Earth center, and the half angle is measured from nadir. The
/// intersection of the cone with the sphere is solved by bisection. A cone
/// at or beyond the horizon produces the horizon central angle.
///
/// Returns an error when the altitude, half angle, or geometry is not
/// usable.
///
/// 返回天底指向圆锥的地面中心角。
///
/// 卫星位于 `EARTH_RADIUS_KM + altitude`，圆锥轴穿过地心，半角自天底量起。
/// 圆锥与球面的交线用二分法求解。达到或超过地平线的圆锥给出地平中心角。
///
/// 高度、半角或几何不可用时返回错误。
pub fn ground_footprint_angle(altitude_km: f64, half_angle_rad: f64) -> Result<f64> {
    if !altitude_km.is_finite() || altitude_km < 0.0 {
        return Err(Error::with_message(
            ErrorKind::InvalidInput,
            "altitude must be finite and non-negative",
        ));
    }
    if !half_angle_rad.is_finite()
        || !(0.0..=core::f64::consts::FRAC_PI_2).contains(&half_angle_rad)
    {
        return Err(Error::with_message(
            ErrorKind::InvalidInput,
            "half angle must be within 0..=pi/2",
        ));
    }
    let radius = EARTH_RADIUS_KM + altitude_km;
    let horizon = (EARTH_RADIUS_KM / radius).acos();
    if half_angle_rad == 0.0 {
        return Ok(0.0);
    }
    if half_angle_rad >= (EARTH_RADIUS_KM / radius).asin() {
        return Ok(horizon);
    }
    let equation = |angle: f64| {
        EARTH_RADIUS_KM * angle.sin()
            / (radius * radius + EARTH_RADIUS_KM * EARTH_RADIUS_KM
                - 2.0 * radius * EARTH_RADIUS_KM * angle.cos())
            .sqrt()
            - half_angle_rad.sin()
    };
    let mut low = 0.0;
    let mut high = horizon;
    for _ in 0..80 {
        let middle = f64::midpoint(low, high);
        if equation(middle) < 0.0 {
            low = middle;
        } else {
            high = middle;
        }
    }
    Ok(f64::midpoint(low, high))
}
