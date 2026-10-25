//! Coverage of a latitude-longitude polygon.
//!
//! An area target is a closed polygon on the ellipsoid. This module tests
//! whether a grid point lies inside the polygon and computes the same
//! coverage and revisit statistics as [`grid_coverage`](crate::apsis_coverage::grid_coverage),
//! restricted to the points inside the polygon.
//!
//! The polygon is interpreted in the longitude-latitude plane, so edges are
//! straight in that plane rather than great-circle arcs. That is the usual
//! convention for an area target drawn on a map, and it keeps the inside
//! test exact and inexpensive. A polygon must not cross the antimeridian;
//! split such a target into two polygons.
//!
//! 经纬度多边形区域的覆盖。
//!
//! 区域目标是椭球上的闭合多边形。本模块判定网格点是否位于多边形内，并在
//! 多边形内的点上计算与 [`grid_coverage`](crate::apsis_coverage::grid_coverage)
//! 相同的覆盖率与重访统计。
//!
//! 多边形在经纬度平面中解释，边在该平面内为直线，而不是大圆弧。这是在地图上
//! 绘制区域目标的常用约定，也使内外判定精确且廉价。多边形不得跨越 180 度
//! 经线；跨越时应拆成两个多边形。

use crate::apsis_core::{Error, ErrorKind, Result};
use crate::apsis_math::Vec3;

use super::grid::{CoverageGrid, CoverageSample};

/// A closed polygon in the longitude-latitude plane.
///
/// Vertices are stored in radians. The closing edge from the last vertex to
/// the first is implicit. The polygon must not cross the antimeridian.
///
/// 经纬度平面中的闭合多边形。
///
/// 顶点以弧度存储。最后一个顶点到第一个顶点的闭合边是隐含的。多边形不得跨越
/// 180 度经线。
#[derive(Debug, Clone, PartialEq)]
pub struct GeoPolygon {
    vertices: Vec<(f64, f64)>,
}

impl GeoPolygon {
    /// Creates a polygon from radian vertices.
    ///
    /// Returns an error when there are fewer than three vertices, a value is
    /// not finite, or a latitude is outside `-pi/2..=pi/2`.
    ///
    /// 由弧度顶点创建多边形。
    ///
    /// 顶点少于三个、值非有限，或纬度超出 `-pi/2..=pi/2` 时返回错误。
    pub fn new(vertices: &[(f64, f64)]) -> Result<Self> {
        if vertices.len() < 3 {
            return Err(Error::with_message(
                ErrorKind::InvalidInput,
                "a polygon needs at least three vertices",
            ));
        }
        let half_pi = core::f64::consts::FRAC_PI_2;
        for (latitude, longitude) in vertices {
            if !latitude.is_finite()
                || !longitude.is_finite()
                || !(-half_pi..=half_pi).contains(latitude)
            {
                return Err(Error::with_message(
                    ErrorKind::InvalidInput,
                    "polygon vertices must be finite and have a valid latitude",
                ));
            }
        }
        Ok(Self {
            vertices: vertices.to_vec(),
        })
    }

    /// Creates a polygon from degree vertices.
    ///
    /// 由度数顶点创建多边形。
    pub fn from_degrees(vertices: &[(f64, f64)]) -> Result<Self> {
        let radians: Vec<(f64, f64)> = vertices
            .iter()
            .map(|(latitude, longitude)| (latitude.to_radians(), longitude.to_radians()))
            .collect();
        Self::new(&radians)
    }

    /// Returns the vertices in radians.
    ///
    /// 返回以弧度表示的顶点。
    #[must_use]
    pub fn vertices(&self) -> &[(f64, f64)] {
        &self.vertices
    }

    /// Returns the latitude-longitude bounds as `(south, north, west, east)`.
    ///
    /// 返回 `(南, 北, 西, 东)` 的经纬度边界。
    #[must_use]
    pub fn bounds(&self) -> (f64, f64, f64, f64) {
        let mut south = f64::INFINITY;
        let mut north = f64::NEG_INFINITY;
        let mut west = f64::INFINITY;
        let mut east = f64::NEG_INFINITY;
        for (latitude, longitude) in &self.vertices {
            south = south.min(*latitude);
            north = north.max(*latitude);
            west = west.min(*longitude);
            east = east.max(*longitude);
        }
        (south, north, west, east)
    }

    /// Returns true when a point lies inside the polygon.
    ///
    /// Uses the even-odd ray casting rule in the longitude-latitude plane.
    /// Points exactly on an edge are not guaranteed to be classified either
    /// way.
    ///
    /// 当点位于多边形内时返回真。
    ///
    /// 在经纬度平面中使用奇偶射线法。恰位于边上的点不保证归入哪一侧。
    #[must_use]
    pub fn contains(&self, latitude: f64, longitude: f64) -> bool {
        let mut inside = false;
        let count = self.vertices.len();
        let mut previous = self.vertices[count - 1];
        for &current in &self.vertices {
            let (previous_latitude, previous_longitude) = previous;
            let (current_latitude, current_longitude) = current;
            let crosses = (current_latitude > latitude) != (previous_latitude > latitude);
            if crosses {
                let fraction =
                    (latitude - current_latitude) / (previous_latitude - current_latitude);
                let crossing_longitude =
                    current_longitude + fraction * (previous_longitude - current_longitude);
                if longitude < crossing_longitude {
                    inside = !inside;
                }
            }
            previous = current;
        }
        inside
    }
}

/// Coverage statistics restricted to the points inside a polygon.
///
/// 限定在多边形内各点的覆盖统计。
#[derive(Debug, Clone, PartialEq)]
pub struct AreaCoverageResult {
    /// Number of inside points covered at least once.
    ///
    /// 至少被覆盖一次的内部点数。
    pub covered_points: usize,
    /// Total number of grid points inside the polygon.
    ///
    /// 多边形内的网格点总数。
    pub total_points: usize,
    /// Covered fraction among the inside points, in `0..=1`.
    ///
    /// 内部点的覆盖率，位于 `0..=1`。
    pub fraction: f64,
    /// Longest gap between successive coverages of any inside point, in seconds.
    ///
    /// 任一内部点相邻两次覆盖之间最长的间隔，单位为秒。
    pub max_gap_seconds: f64,
    /// Mean of the observed gaps among the inside points, in seconds.
    ///
    /// 内部点所观测间隔的平均值，单位为秒。
    pub mean_gap_seconds: f64,
    /// Number of observed gaps between successive coverages.
    ///
    /// 相邻两次覆盖之间观测到的间隙数量。
    pub gap_count: usize,
    /// Shortest observed gap, in seconds; zero when no gap was observed.
    ///
    /// 最短观测间隙，单位为秒；没有间隙时为零。
    pub min_gap_seconds: f64,
    /// Row-major flags for the full grid; true only for covered inside points.
    ///
    /// 全网格的行主序标志；仅被覆盖的内部点为真。
    pub covered: Vec<bool>,
    /// Row-major flags marking the grid points inside the polygon.
    ///
    /// 标记多边形内网格点的行主序标志。
    pub inside: Vec<bool>,
}

/// Computes coverage statistics for the part of a grid inside a polygon.
///
/// Returns an error when the samples are empty, not in increasing time order,
/// contain an unusable value, or when no grid point lies inside the polygon.
///
/// 计算网格中位于多边形内部分的覆盖统计。
///
/// 样本为空、时间递减、含不可用值，或多边形内没有网格点时返回错误。
pub fn area_coverage(
    grid: &CoverageGrid,
    polygon: &GeoPolygon,
    samples: &[CoverageSample],
) -> Result<AreaCoverageResult> {
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
        if index > 0 && sample.seconds < samples[index - 1].seconds {
            return Err(Error::with_message(
                ErrorKind::InvalidInput,
                "samples must be in non-decreasing time order",
            ));
        }
    }
    let rows = grid.rows();
    let columns = grid.columns();
    let total = rows * columns;
    let mut inside = vec![false; total];
    let mut units: Vec<Vec3> = Vec::with_capacity(total);
    for row in 0..rows {
        for column in 0..columns {
            let (latitude, longitude) = grid.point(row, column);
            let index = row * columns + column;
            inside[index] = polygon.contains(latitude, longitude);
            units.push(grid.unit_vector(row, column));
        }
    }
    let inside_points = inside.iter().filter(|value| **value).count();
    if inside_points == 0 {
        return Err(Error::with_message(
            ErrorKind::InvalidInput,
            "the polygon contains no grid points",
        ));
    }
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
            if !inside[index] {
                continue;
            }
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
    let covered_points = (0..total).filter(|index| covered[*index]).count();
    let max_gap = gaps.iter().copied().fold(0.0, f64::max);
    let mean_gap = if gaps.is_empty() {
        0.0
    } else {
        gaps.iter().sum::<f64>() / gaps.len() as f64
    };
    let min_gap = gaps.iter().copied().fold(f64::INFINITY, f64::min);
    let min_gap = if min_gap.is_finite() { min_gap } else { 0.0 };
    Ok(AreaCoverageResult {
        covered_points,
        total_points: inside_points,
        fraction: covered_points as f64 / inside_points as f64,
        max_gap_seconds: max_gap,
        mean_gap_seconds: mean_gap,
        gap_count: gaps.len(),
        min_gap_seconds: min_gap,
        covered,
        inside,
    })
}

/// The intervals during which any part of an area target is visible.
///
/// 区域目标任一部分可见的区间。
#[derive(Debug, Clone, PartialEq)]
pub struct AreaAccessResult {
    /// Visible intervals as `(start, end)` seconds.
    ///
    /// 可见区间，单位为秒，形式为 `(开始, 结束)`。
    pub intervals: Vec<(f64, f64)>,
    /// Total visible duration, in seconds.
    ///
    /// 总可见时长，单位为秒。
    pub total_seconds: f64,
    /// Fraction of the sampled span that is visible.
    ///
    /// 采样跨度中的可见比例。
    pub fraction: f64,
}

/// Determines when at least one point inside a polygon is visible.
///
/// A sample makes the area visible when its footprint covers at least one
/// grid point inside the polygon. This is an area-level access test, not an
/// access test for a specific point.
///
/// 判定多边形内至少一点可见的时刻。
///
/// 当某样本的覆盖区覆盖多边形内至少一个网格点时，区域即视为可见。这是区域级
/// 的可见性判定，而不是针对某个具体点的可见性。
pub fn area_access(
    grid: &CoverageGrid,
    polygon: &GeoPolygon,
    samples: &[CoverageSample],
) -> Result<AreaAccessResult> {
    if samples.is_empty() {
        return Err(Error::with_message(
            ErrorKind::InvalidInput,
            "at least one sample is required",
        ));
    }
    let rows = grid.rows();
    let columns = grid.columns();
    let mut inside = Vec::new();
    for row in 0..rows {
        for column in 0..columns {
            let (latitude, longitude) = grid.point(row, column);
            if polygon.contains(latitude, longitude) {
                inside.push(grid.unit_vector(row, column));
            }
        }
    }
    if inside.is_empty() {
        return Err(Error::with_message(
            ErrorKind::InvalidInput,
            "the polygon contains no grid points",
        ));
    }
    let mut intervals = Vec::new();
    let mut start: Option<f64> = None;
    let mut last = 0.0;
    for (index, sample) in samples.iter().enumerate() {
        if !sample.is_valid() {
            return Err(Error::with_message(
                ErrorKind::InvalidInput,
                "sample must be finite and usable",
            ));
        }
        if index > 0 && sample.seconds < samples[index - 1].seconds {
            return Err(Error::with_message(
                ErrorKind::InvalidInput,
                "samples must be in non-decreasing time order",
            ));
        }
        let subpoint = sample
            .position_ecef_km
            .normalize()
            .ok_or_else(|| Error::with_message(ErrorKind::Undefined, "position is zero"))?;
        let limit = sample.central_angle_rad.cos();
        let visible = inside
            .iter()
            .any(|unit| subpoint.dot(*unit) >= limit - 1e-12);
        if visible {
            if start.is_none() {
                start = Some(sample.seconds);
            }
            last = sample.seconds;
        } else if let Some(begin) = start.take() {
            intervals.push((begin, sample.seconds));
        }
    }
    if let Some(begin) = start {
        intervals.push((begin, last));
    }
    let total: f64 = intervals
        .iter()
        .map(|(begin, end)| (end - begin).max(0.0))
        .sum();
    let span = samples
        .last()
        .map(|sample| sample.seconds - samples[0].seconds)
        .unwrap_or(0.0);
    Ok(AreaAccessResult {
        intervals,
        total_seconds: total,
        fraction: if span > 0.0 { total / span } else { 0.0 },
    })
}
