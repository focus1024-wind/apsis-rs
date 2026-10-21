//! Tests for area-target coverage.
//!
//! 区域目标覆盖测试。

use apsis::apsis_coverage::{CoverageGrid, CoverageSample, GeoPolygon, WGS84, area_coverage};
use apsis::apsis_geodesy::Geodetic;

fn small_polygon() -> GeoPolygon {
    GeoPolygon::from_degrees(&[(-0.5, -0.5), (-0.5, 0.5), (0.5, 0.5), (0.5, -0.5)])
        .expect("valid polygon")
}

#[test]
fn the_polygon_contains_its_interior_and_rejects_the_exterior() {
    let polygon = small_polygon();
    assert!(polygon.contains(0.0, 0.0));
    assert!(polygon.contains(0.2_f64.to_radians(), (-0.3_f64).to_radians()));
    assert!(!polygon.contains(1.0_f64.to_radians(), 0.0));
    assert!(!polygon.contains(0.0, 1.0_f64.to_radians()));
    assert_eq!(polygon.vertices().len(), 4);
}

#[test]
fn area_coverage_counts_only_points_inside_the_polygon() {
    let polygon = GeoPolygon::from_degrees(&[(-0.5, -0.5), (-0.5, 0.5), (0.5, 0.5), (0.5, -0.5)])
        .expect("valid polygon");
    let grid = CoverageGrid::from_degrees(-2.0, 2.0, -2.0, 2.0, 1.0, 1.0).expect("valid grid");
    let center = Geodetic::new(0.0, 0.0, 0.0).to_cartesian(WGS84);
    let samples = [CoverageSample::new(0.0, center, 0.0)];
    let result = area_coverage(&grid, &polygon, &samples).expect("coverage");
    assert_eq!(result.total_points, 1);
    assert_eq!(result.covered_points, 1);
    assert!((result.fraction - 1.0).abs() < 1e-12);
    assert_eq!(result.inside.iter().filter(|value| **value).count(), 1);
}

#[test]
fn a_polygon_without_grid_points_is_rejected() {
    let polygon =
        GeoPolygon::from_degrees(&[(80.0, 0.0), (80.0, 1.0), (81.0, 1.0)]).expect("valid polygon");
    let grid = CoverageGrid::from_degrees(-2.0, 2.0, -2.0, 2.0, 1.0, 1.0).expect("valid grid");
    let center = Geodetic::new(0.0, 0.0, 0.0).to_cartesian(WGS84);
    let samples = [CoverageSample::new(0.0, center, 0.0)];
    assert!(area_coverage(&grid, &polygon, &samples).is_err());
}
