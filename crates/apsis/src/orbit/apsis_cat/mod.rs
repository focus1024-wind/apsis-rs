//! Conjunction assessment.
//!
//! The module compares two state histories sampled on the same time grid and
//! reports the closest approach. The geometry is expressed in the radial,
//! along-track, and cross-track frame of the primary object, which is the
//! frame conjunction reports use. The closest approach is refined from the
//! three samples around the minimum by fitting the squared distance with a
//! parabola, so the answer is not limited to the sample grid.
//!
//! 交会分析。
//!
//! 本模块比较在同一时间网格上采样的两条状态历史并给出最接近时刻。几何量在
//! 主目标的径向、沿迹与横向坐标系中表达，这是交会报告所用的坐标系。最接近
//! 时刻由最小值附近三个样本对平方距离作抛物线拟合来细化，故答案不限于采样
//! 网格。

use crate::apsis_core::{Error, ErrorKind, Result};
use crate::apsis_math::Vec3;

/// One state sample on a common time grid.
///
/// 共同时间网格上的一个状态样本。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StateSample {
    /// Seconds from the common origin.
    ///
    /// 距共同原点的秒数。
    pub seconds: f64,
    /// Position in kilometers.
    ///
    /// 位置，单位为千米。
    pub position_km: Vec3,
    /// Velocity in kilometers per second.
    ///
    /// 速度，单位为千米每秒。
    pub velocity_km_s: Vec3,
}

impl StateSample {
    /// Creates a sample.
    ///
    /// 创建样本。
    #[must_use]
    pub const fn new(seconds: f64, position_km: Vec3, velocity_km_s: Vec3) -> Self {
        Self {
            seconds,
            position_km,
            velocity_km_s,
        }
    }

    /// Returns true when the sample is finite.
    ///
    /// 样本为有限值时返回真。
    #[must_use]
    pub fn is_valid(self) -> bool {
        self.seconds.is_finite() && self.position_km.is_finite() && self.velocity_km_s.is_finite()
    }
}

/// The relative geometry at one instant, in the primary's RIC frame.
///
/// 某瞬时在主目标 RIC 坐标系中的相对几何。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RelativeState {
    /// Distance between the objects in kilometers.
    ///
    /// 两目标之间的距离，单位为千米。
    pub distance_km: f64,
    /// Magnitude of the relative velocity in kilometers per second.
    ///
    /// 相对速度的大小，单位为千米每秒。
    pub relative_speed_km_s: f64,
    /// Radial separation in kilometers, positive outward.
    ///
    /// 径向间距，单位为千米，向外为正。
    pub radial_km: f64,
    /// Along-track separation in kilometers, positive in the direction of
    /// motion.
    ///
    /// 沿迹间距，单位为千米，沿运动方向为正。
    pub along_track_km: f64,
    /// Cross-track separation in kilometers, positive along the orbit normal.
    ///
    /// 横向间距，单位为千米，沿轨道法线为正。
    pub cross_track_km: f64,
}

/// The closest approach of two sampled histories.
///
/// 两条采样历史的最接近结果。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClosestApproach {
    /// Time of closest approach, in seconds from the common origin.
    ///
    /// 最接近时刻，单位为距共同原点的秒数。
    pub tca_seconds: f64,
    /// Minimum distance in kilometers.
    ///
    /// 最小距离，单位为千米。
    pub distance_km: f64,
    /// Relative speed at closest approach in kilometers per second.
    ///
    /// 最接近时刻的相对速度，单位为千米每秒。
    pub relative_speed_km_s: f64,
    /// Radial separation at closest approach, in kilometers.
    ///
    /// 最接近时刻的径向间距，单位为千米。
    pub radial_km: f64,
    /// Along-track separation at closest approach, in kilometers.
    ///
    /// 最接近时刻的沿迹间距，单位为千米。
    pub along_track_km: f64,
    /// Cross-track separation at closest approach, in kilometers.
    ///
    /// 最接近时刻的横向间距，单位为千米。
    pub cross_track_km: f64,
}

/// Returns the relative geometry of two samples in the primary's RIC frame.
///
/// Returns an error when a sample is not usable, or when the primary has no
/// position or no orbit normal.
///
/// 返回两个样本在主目标 RIC 坐标系中的相对几何。
///
/// 样本不可用，或主目标没有位置或轨道法线时返回错误。
pub fn relative_state(primary: StateSample, secondary: StateSample) -> Result<RelativeState> {
    if !primary.is_valid() || !secondary.is_valid() {
        return Err(Error::with_message(
            ErrorKind::InvalidInput,
            "state samples must be finite",
        ));
    }
    let radial = primary
        .position_km
        .normalize()
        .ok_or_else(|| Error::with_message(ErrorKind::Undefined, "primary position is zero"))?;
    let normal = primary
        .position_km
        .cross(primary.velocity_km_s)
        .normalize()
        .ok_or_else(|| Error::with_message(ErrorKind::Undefined, "primary orbit normal is zero"))?;
    let along = normal.cross(radial);
    let offset = secondary.position_km - primary.position_km;
    let relative_velocity = secondary.velocity_km_s - primary.velocity_km_s;
    Ok(RelativeState {
        distance_km: offset.norm(),
        relative_speed_km_s: relative_velocity.norm(),
        radial_km: offset.dot(radial),
        along_track_km: offset.dot(along),
        cross_track_km: offset.dot(normal),
    })
}

/// Estimates collision probability with an isotropic two-dimensional Gaussian.
///
/// The miss vector is assumed to lie in the encounter plane with a circular
/// covariance of standard deviation `sigma_km`. The hard-body radius supplies
/// the combined object size. This is the small-target approximation commonly
/// used when no full covariance or relative-velocity direction is available;
/// it is not a substitute for a covariance-based probability calculation.
///
/// Returns an error when an input is negative or non-finite, or when the
/// standard deviation is not positive.
///
/// 用各向同性二维高斯估计碰撞概率。
///
/// 假定交会平面内的脱靶向量具有标准差为 `sigma_km` 的圆对称协方差，硬体半径
/// 给出组合目标尺寸。没有完整协方差或相对速度方向时，这是常用的小目标近似；
/// 不能替代基于协方差的概率计算。
///
/// 输入为负或非有限，或标准差不为正时返回错误。
pub fn isotropic_collision_probability(
    miss_distance_km: f64,
    hard_body_radius_km: f64,
    sigma_km: f64,
) -> Result<f64> {
    if !miss_distance_km.is_finite()
        || !hard_body_radius_km.is_finite()
        || !sigma_km.is_finite()
        || miss_distance_km < 0.0
        || hard_body_radius_km < 0.0
        || sigma_km <= 0.0
    {
        return Err(Error::with_message(
            ErrorKind::InvalidInput,
            "miss distance and radius must be non-negative, sigma must be positive",
        ));
    }
    if hard_body_radius_km == 0.0 {
        return Ok(0.0);
    }
    let variance = sigma_km * sigma_km;
    let radial = 1.0 - (-(hard_body_radius_km * hard_body_radius_km) / (2.0 * variance)).exp();
    let miss = (-(miss_distance_km * miss_distance_km) / (2.0 * variance)).exp();
    Ok((radial * miss).clamp(0.0, 1.0))
}

/// Finds the closest approach of two histories sampled on the same grid.
///
/// Both slices must have the same length, at least two samples, identical
/// and uniformly spaced times, and finite states. A minimum at the first or
/// last sample is reported at that sample; an interior minimum is refined by
/// a parabola through the squared distance at the three surrounding samples,
/// and the states are linearly interpolated to the refined time.
///
/// Returns an error when the grid is not usable.
///
/// 在同一网格上采样的两条历史中寻找最接近时刻。
///
/// 两个切片须长度相同、至少两个样本、时间完全相同且均匀间隔，状态有限。最小
/// 值位于首末样本时按该样本报告；内部最小值由相邻三个样本的平方距离作抛物线
/// 细化，并在细化时刻对状态作线性插值。
///
/// 网格不可用时返回错误。
pub fn closest_approach(
    primary: &[StateSample],
    secondary: &[StateSample],
) -> Result<ClosestApproach> {
    if primary.len() != secondary.len() {
        return Err(Error::with_message(
            ErrorKind::Inconsistent,
            "the two histories must have the same number of samples",
        ));
    }
    if primary.len() < 2 {
        return Err(Error::with_message(
            ErrorKind::InvalidInput,
            "at least two samples are required",
        ));
    }
    for index in 0..primary.len() {
        if !primary[index].is_valid() || !secondary[index].is_valid() {
            return Err(Error::with_message(
                ErrorKind::InvalidInput,
                "state samples must be finite",
            ));
        }
        if (primary[index].seconds - secondary[index].seconds).abs() > 1e-9 {
            return Err(Error::with_message(
                ErrorKind::Inconsistent,
                "the two histories must share the same sample times",
            ));
        }
        if index > 0 {
            let step = primary[index].seconds - primary[index - 1].seconds;
            if step <= 0.0 || !step.is_finite() {
                return Err(Error::with_message(
                    ErrorKind::InvalidInput,
                    "sample times must increase",
                ));
            }
        }
    }
    let step = primary[1].seconds - primary[0].seconds;
    for index in 1..primary.len() - 1 {
        let local = primary[index + 1].seconds - primary[index].seconds;
        if (local - step).abs() > 1e-6 * step.abs().max(1.0) {
            return Err(Error::with_message(
                ErrorKind::InvalidInput,
                "the sample grid must be uniform",
            ));
        }
    }

    let squared: Vec<f64> = primary
        .iter()
        .zip(secondary.iter())
        .map(|(a, b)| (b.position_km - a.position_km).norm_squared())
        .collect();
    let mut minimum = 0;
    for index in 1..squared.len() {
        if squared[index] < squared[minimum] {
            minimum = index;
        }
    }

    let (tca_seconds, primary_state, secondary_state) =
        if minimum == 0 || minimum == primary.len() - 1 {
            (
                primary[minimum].seconds,
                primary[minimum],
                secondary[minimum],
            )
        } else {
            let before = squared[minimum - 1];
            let at = squared[minimum];
            let after = squared[minimum + 1];
            let denominator = before - 2.0 * at + after;
            if denominator.abs() < 1e-15 {
                (
                    primary[minimum].seconds,
                    primary[minimum],
                    secondary[minimum],
                )
            } else {
                let offset = (0.5 * (before - after) / denominator).clamp(-1.0, 1.0);
                // Quadratic interpolation through the three samples. A linear
                // interpolation of a rotating relative offset would shorten it;
                // the parabola preserves the curvature to second order.
                // 通过三个样本作二次插值。旋转的相对偏移若作线性插值会被缩短；
                // 抛物线把曲率保留到二阶。
                let before_weight = 0.5 * offset * (offset - 1.0);
                let centre_weight = 1.0 - offset * offset;
                let after_weight = 0.5 * offset * (offset + 1.0);
                let interpolate = |a: StateSample, b: StateSample, c: StateSample| StateSample {
                    seconds: b.seconds + offset * step,
                    position_km: a.position_km * before_weight
                        + b.position_km * centre_weight
                        + c.position_km * after_weight,
                    velocity_km_s: a.velocity_km_s * before_weight
                        + b.velocity_km_s * centre_weight
                        + c.velocity_km_s * after_weight,
                };
                (
                    primary[minimum].seconds + offset * step,
                    interpolate(primary[minimum - 1], primary[minimum], primary[minimum + 1]),
                    interpolate(
                        secondary[minimum - 1],
                        secondary[minimum],
                        secondary[minimum + 1],
                    ),
                )
            }
        };
    let relative = relative_state(primary_state, secondary_state)?;
    Ok(ClosestApproach {
        tca_seconds,
        distance_km: relative.distance_km,
        relative_speed_km_s: relative.relative_speed_km_s,
        radial_km: relative.radial_km,
        along_track_km: relative.along_track_km,
        cross_track_km: relative.cross_track_km,
    })
}
