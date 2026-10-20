//! Grid coverage and revisit statistics.
//!
//! A coverage grid is a set of points on the ellipsoid. Each satellite sample
//! covers the points whose central angle from the sub-satellite point is at
//! most the sample's footprint angle. The module reports the covered
//! fraction and the gaps between successive coverages of each point.
//!
//! 网格覆盖与重访统计。
//!
//! 覆盖网格是椭球上的一组点。每个卫星样本覆盖其到星下点中心角不超过该样本
//! 覆盖角的点。模块给出覆盖率以及各点相邻两次覆盖之间的间隔。

use crate::apsis_core::{Error, ErrorKind, Result};
use crate::apsis_geodesy::{Geodetic, WGS84};
use crate::apsis_math::Vec3;

/// A latitude-longitude grid on the WGS 84 ellipsoid.
///
/// 基于 WGS 84 椭球的经纬网格。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CoverageGrid {
    /// Southern latitude limit, in radians.
    ///
    /// 南界纬度，单位为弧度。
    pub lat_min: f64,
    /// Northern latitude limit, in radians.
    ///
    /// 北界纬度，单位为弧度。
    pub lat_max: f64,
    /// Western longitude limit, in radians.
    ///
    /// 西界经度，单位为弧度。
    pub lon_min: f64,
    /// Eastern longitude limit, in radians.
    ///
    /// 东界经度，单位为弧度。
    pub lon_max: f64,
    /// Latitude spacing, in radians.
    ///
    /// 纬向间隔，单位为弧度。
    pub lat_step: f64,
    /// Longitude spacing, in radians.
    ///
    /// 经向间隔，单位为弧度。
    pub lon_step: f64,
}

impl CoverageGrid {
    /// Creates a grid from radian limits and spacings.
    ///
    /// Returns an error unless the limits are ordered, the latitudes lie in
    /// `-pi/2..=pi/2`, and both spacings are positive and finite.
    ///
    /// 由弧度上下限与间隔创建网格。
    ///
    /// 除非上下限有序、纬度位于 `-pi/2..=pi/2`，且两个间隔均为正且有限，
    /// 否则返回错误。
    pub fn new(
        lat_min: f64,
        lat_max: f64,
        lon_min: f64,
        lon_max: f64,
        lat_step: f64,
        lon_step: f64,
    ) -> Result<Self> {
        let half_pi = core::f64::consts::FRAC_PI_2;
        if !lat_min.is_finite()
            || !lat_max.is_finite()
            || !lon_min.is_finite()
            || !lon_max.is_finite()
            || !(-half_pi..=half_pi).contains(&lat_min)
            || !(-half_pi..=half_pi).contains(&lat_max)
            || lat_max < lat_min
            || lon_max < lon_min
        {
            return Err(Error::with_message(
                ErrorKind::InvalidInput,
                "grid limits must be finite and ordered",
            ));
        }
        if !lat_step.is_finite() || lat_step <= 0.0 || !lon_step.is_finite() || lon_step <= 0.0 {
            return Err(Error::with_message(
                ErrorKind::InvalidInput,
                "grid steps must be positive and finite",
            ));
        }
        Ok(Self {
            lat_min,
            lat_max,
            lon_min,
            lon_max,
            lat_step,
            lon_step,
        })
    }

    /// Creates a grid from degrees.
    ///
    /// 由度数创建网格。
    pub fn from_degrees(
        lat_min: f64,
        lat_max: f64,
        lon_min: f64,
        lon_max: f64,
        lat_step: f64,
        lon_step: f64,
    ) -> Result<Self> {
        Self::new(
            lat_min.to_radians(),
            lat_max.to_radians(),
            lon_min.to_radians(),
            lon_max.to_radians(),
            lat_step.to_radians(),
            lon_step.to_radians(),
        )
    }

    /// Returns the number of latitude rows.
    ///
    /// 返回纬向行数。
    #[must_use]
    pub fn rows(&self) -> usize {
        ((self.lat_max - self.lat_min) / self.lat_step).floor() as usize + 1
    }

    /// Returns the number of longitude columns.
    ///
    /// 返回经向列数。
    #[must_use]
    pub fn columns(&self) -> usize {
        ((self.lon_max - self.lon_min) / self.lon_step).floor() as usize + 1
    }

    /// Returns the total number of points.
    ///
    /// 返回总点数。
    #[must_use]
    pub fn len(&self) -> usize {
        self.rows() * self.columns()
    }

    /// Returns true when the grid has no points.
    ///
    /// 网格没有点时返回真。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the latitude and longitude of a point, in radians.
    ///
    /// 返回某点的纬度与经度，单位为弧度。
    #[must_use]
    pub fn point(&self, row: usize, column: usize) -> (f64, f64) {
        (
            self.lat_min + row as f64 * self.lat_step,
            self.lon_min + column as f64 * self.lon_step,
        )
    }

    /// Returns the unit vector of a point on the ellipsoid.
    ///
    /// 返回椭球上某点的单位矢量。
    #[must_use]
    pub fn unit_vector(&self, row: usize, column: usize) -> Vec3 {
        let (latitude, longitude) = self.point(row, column);
        Geodetic::new(latitude, longitude, 0.0)
            .to_cartesian(WGS84)
            .normalize()
            .unwrap_or(Vec3::ZERO)
    }
}

/// One satellite sample for a coverage calculation.
///
/// 覆盖计算的一个卫星样本。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CoverageSample {
    /// Seconds from the common origin.
    ///
    /// 距共同原点的秒数。
    pub seconds: f64,
    /// Earth-fixed position in kilometers.
    ///
    /// 地固位置，单位为千米。
    pub position_ecef_km: Vec3,
    /// Footprint central angle in radians.
    ///
    /// 覆盖区中心角，单位为弧度。
    pub central_angle_rad: f64,
}

impl CoverageSample {
    /// Creates a sample.
    ///
    /// 创建样本。
    #[must_use]
    pub const fn new(seconds: f64, position_ecef_km: Vec3, central_angle_rad: f64) -> Self {
        Self {
            seconds,
            position_ecef_km,
            central_angle_rad,
        }
    }

    /// Returns true when the sample is usable.
    ///
    /// 样本可用时返回真。
    #[must_use]
    pub fn is_valid(self) -> bool {
        self.seconds.is_finite()
            && self.position_ecef_km.is_finite()
            && self.position_ecef_km.norm() > 0.0
            && self.central_angle_rad.is_finite()
            && (0.0..=core::f64::consts::PI).contains(&self.central_angle_rad)
    }
}

/// The coverage statistics over a grid.
///
/// 网格上的覆盖统计。
#[derive(Debug, Clone, PartialEq)]
pub struct CoverageResult {
    /// Number of grid points covered at least once.
    ///
    /// 至少被覆盖一次的网格点数。
    pub covered_points: usize,
    /// Total number of grid points.
    ///
    /// 网格总点数。
    pub total_points: usize,
    /// Covered fraction in `0..=1`.
    ///
    /// 覆盖率，位于 `0..=1`。
    pub fraction: f64,
    /// Longest gap between successive coverages of any point, in seconds.
    ///
    /// 任一点相邻两次覆盖之间最长的间隔，单位为秒。
    pub max_gap_seconds: f64,
    /// Mean of the observed gaps, in seconds.
    ///
    /// 所观测间隔的平均值，单位为秒。
    pub mean_gap_seconds: f64,
    /// Row-major coverage flags for every grid point.
    ///
    /// 每个网格点的行主序覆盖标志。
    pub covered: Vec<bool>,
}

/// Computes grid coverage and revisit gaps from time-ordered samples.
///
/// Returns an error when the samples are empty, not in increasing time
/// order, or contain an unusable value.
///
/// 由按时间排序的样本计算网格覆盖与重访间隔。
///
/// 样本为空、时间非递增或含不可用值时返回错误。
pub fn grid_coverage(grid: &CoverageGrid, samples: &[CoverageSample]) -> Result<CoverageResult> {
    if samples.is_empty() {
        return Err(Error::with_message(
            ErrorKind::InvalidInput,
            "at least one sample is required",
        ));
    }
    for (index, sample) in samples.iter().enumerate() {
        if !sample.is_valid() {
            return Err(Error::with_message(
                ErrorKind::InvalidInput,
                "sample must be finite and usable",
            ));
        }
        if index > 0 && sample.seconds <= samples[index - 1].seconds {
            return Err(Error::with_message(
                ErrorKind::InvalidInput,
                "samples must be in increasing time order",
            ));
        }
    }
    let rows = grid.rows();
    let columns = grid.columns();
    let total = rows * columns;
    let units: Vec<Vec3> = (0..rows)
        .flat_map(|row| (0..columns).map(move |column| (row, column)))
        .map(|(row, column)| grid.unit_vector(row, column))
        .collect();
    let mut covered = vec![false; total];
    let mut previous_covered = vec![false; total];
    let mut last_cover: Vec<Option<f64>> = vec![None; total];
    let mut gaps: Vec<f64> = Vec::new();
    for sample in samples {
        let subpoint = sample
            .position_ecef_km
            .normalize()
            .ok_or_else(|| Error::with_message(ErrorKind::Undefined, "position is zero"))?;
        let limit = sample.central_angle_rad.cos();
        for index in 0..total {
            let is_covered = subpoint.dot(units[index]) >= limit - 1e-12;
            if is_covered {
                covered[index] = true;
                if !previous_covered[index] {
                    if let Some(previous) = last_cover[index] {
                        gaps.push(sample.seconds - previous);
                    }
                }
                last_cover[index] = Some(sample.seconds);
            }
            previous_covered[index] = is_covered;
        }
    }
    let covered_points = covered.iter().filter(|value| **value).count();
    let max_gap = gaps.iter().copied().fold(0.0, f64::max);
    let mean_gap = if gaps.is_empty() {
        0.0
    } else {
        gaps.iter().sum::<f64>() / gaps.len() as f64
    };
    Ok(CoverageResult {
        covered_points,
        total_points: total,
        fraction: if total == 0 {
            0.0
        } else {
            covered_points as f64 / total as f64
        },
        max_gap_seconds: max_gap,
        mean_gap_seconds: mean_gap,
        covered,
    })
}
