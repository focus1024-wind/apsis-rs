//! Validation tests for the grid coverage.
//!
//! 网格覆盖的验证测试。

use apsis::apsis_coverage::{CoverageGrid, CoverageSample, grid_coverage};
use apsis::apsis_math::Vec3;

fn sample(seconds: f64, position: Vec3, angle_deg: f64) -> CoverageSample {
    CoverageSample::new(seconds, position, angle_deg.to_radians())
}

#[test]
fn a_single_sample_covers_the_points_within_its_footprint() {
    let grid = CoverageGrid::from_degrees(-10.0, 10.0, -10.0, 10.0, 5.0, 5.0).expect("grid");
    let samples = [sample(0.0, Vec3::new(7_000.0, 0.0, 0.0), 6.0)];
    let result = grid_coverage(&grid, &samples).expect("coverage");
    // The grid has 25 points. The center and the four points at five
    // degrees along one axis lie within six degrees; the diagonal points at
    // about 7.07 degrees do not.
    // 网格有 25 个点。中心与沿单轴偏五度的四点位于六度以内；对角点约 7.07 度，
    // 不在其中。
    assert_eq!(result.total_points, 25);
    assert_eq!(result.covered_points, 5);
    assert!((result.fraction - 5.0 / 25.0).abs() < 1e-12);
}

#[test]
fn a_full_footprint_covers_every_point() {
    let grid = CoverageGrid::from_degrees(-60.0, 60.0, -180.0, 170.0, 10.0, 10.0).expect("grid");
    let samples = [sample(0.0, Vec3::new(7_000.0, 0.0, 0.0), 180.0)];
    let result = grid_coverage(&grid, &samples).expect("coverage");
    assert_eq!(result.covered_points, result.total_points);
    assert!((result.fraction - 1.0).abs() < 1e-12);
}

#[test]
fn a_zero_footprint_outside_the_subpoint_covers_nothing() {
    let grid = CoverageGrid::from_degrees(-10.0, 10.0, 5.0, 15.0, 5.0, 5.0).expect("grid");
    let samples = [sample(0.0, Vec3::new(7_000.0, 0.0, 0.0), 0.0)];
    let result = grid_coverage(&grid, &samples).expect("coverage");
    assert_eq!(result.covered_points, 0);
}

#[test]
fn the_gap_between_two_coverages_is_measured_in_time() {
    let grid = CoverageGrid::from_degrees(0.0, 0.0, 0.0, 0.0, 5.0, 5.0).expect("grid");
    let samples = [
        sample(0.0, Vec3::new(7_000.0, 0.0, 0.0), 5.0),
        sample(40.0, Vec3::new(7_000.0, 7_000.0, 0.0), 0.0),
        sample(100.0, Vec3::new(7_000.0, 0.0, 0.0), 5.0),
    ];
    let result = grid_coverage(&grid, &samples).expect("coverage");
    assert!((result.max_gap_seconds - 100.0).abs() < 1e-12);
    assert!((result.mean_gap_seconds - 100.0).abs() < 1e-12);
}

#[test]
fn the_mean_gap_averages_the_observed_gaps() {
    let grid = CoverageGrid::from_degrees(0.0, 0.0, 0.0, 0.0, 5.0, 5.0).expect("grid");
    let samples = [
        sample(0.0, Vec3::new(7_000.0, 0.0, 0.0), 5.0),
        sample(40.0, Vec3::new(7_000.0, 7_000.0, 0.0), 0.0),
        sample(100.0, Vec3::new(7_000.0, 0.0, 0.0), 5.0),
        sample(150.0, Vec3::new(7_000.0, 7_000.0, 0.0), 0.0),
        sample(210.0, Vec3::new(7_000.0, 0.0, 0.0), 5.0),
    ];
    let result = grid_coverage(&grid, &samples).expect("coverage");
    assert!((result.max_gap_seconds - 110.0).abs() < 1e-12);
    assert!((result.mean_gap_seconds - 105.0).abs() < 1e-12);
}

#[test]
fn invalid_grids_and_samples_are_rejected() {
    assert!(CoverageGrid::from_degrees(10.0, -10.0, 0.0, 10.0, 5.0, 5.0).is_err());
    assert!(CoverageGrid::from_degrees(-10.0, 10.0, 0.0, 10.0, 0.0, 5.0).is_err());
    let grid = CoverageGrid::from_degrees(-10.0, 10.0, -10.0, 10.0, 5.0, 5.0).expect("grid");
    assert!(grid_coverage(&grid, &[]).is_err());
    assert!(grid_coverage(&grid, &[sample(0.0, Vec3::ZERO, 5.0)]).is_err());
}
