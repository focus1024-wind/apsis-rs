//! Tests for the ITU-R P.839-4 rain height model.
//!
//! The expected values are those of the recommendation's tabulated grid,
//! read at the grid points and interpolated between them. They were produced
//! with the reference implementation of the recommendation.
//!
//! ITU-R P.839-4 雨顶高度模型的测试。
//!
//! 期望值是该建议书所列网格的值，在网格点处读取并在其之间插值。它们由该建议书的
//! 参考实现产生。

use apsis_atmosphere::{
    RAIN_HEIGHT_OFFSET_KM, grid_step_degrees, rain_height, zero_isotherm_height,
};

/// The tolerance on an interpolated height, in kilometers.
///
/// The table is stored in metres, so the interpolation is exact to the
/// resolution the recommendation publishes.
///
/// 插值高度的容差，单位为千米。
///
/// 该表以米存储，故插值在该建议书所发布的分辨率内是精确的。
const TOLERANCE: f64 = 1e-9;

#[test]
fn the_grid_resolution_matches_the_recommendation() {
    assert_eq!(grid_step_degrees(), 1.5);
}

#[test]
fn the_rain_height_offset_matches_the_recommendation() {
    assert_eq!(RAIN_HEIGHT_OFFSET_KM, 0.36);
}

#[test]
fn values_at_grid_points_match_the_recommendation() {
    // These are the tabulated values at exact grid points, so no
    // interpolation is involved.
    // 这些是精确网格点处所列的值，故不涉及插值。
    let cases = [
        (0.0, 0.0, 4.566),
        (45.0, 0.0, 2.356),
        (-45.0, 90.0, 1.158),
        (90.0, 0.0, 2.096),
        (-90.0, 0.0, 2.880),
        (60.0, 30.0, 2.086),
        (-60.0, 300.0, 0.544),
    ];
    for (latitude, longitude, expected) in cases {
        let actual = zero_isotherm_height(latitude, longitude).expect("valid coordinates");
        assert!(
            (actual - expected).abs() < TOLERANCE,
            "({latitude}, {longitude}): got {actual}, expected {expected}"
        );
    }
}

#[test]
fn interpolated_values_match_the_recommendation() {
    let cases = [
        (35.7, 139.7, 3.866_48),
        (-33.9, 151.2, 3.057_68),
        (23.5, 45.0, 3.553_666_666_667),
        (89.9, 0.0, 2.118_666_666_667),
        (-89.9, 180.0, 2.895_933_333_333),
        (10.0, 100.0, 4.676),
    ];
    for (latitude, longitude, expected) in cases {
        let actual = zero_isotherm_height(latitude, longitude).expect("valid coordinates");
        assert!(
            (actual - expected).abs() < 1e-6,
            "({latitude}, {longitude}): got {actual}, expected {expected}"
        );
    }
}

#[test]
fn the_rain_height_is_the_isotherm_height_plus_the_offset() {
    for (latitude, longitude) in [(0.0, 0.0), (45.0, 30.0), (-20.0, 200.0)] {
        let isotherm = zero_isotherm_height(latitude, longitude).expect("valid coordinates");
        let rain = rain_height(latitude, longitude).expect("valid coordinates");
        assert!((rain - isotherm - RAIN_HEIGHT_OFFSET_KM).abs() < TOLERANCE);
    }
}

#[test]
fn longitude_wraps_at_the_seam() {
    // The grid runs to 360 degrees, which is the same meridian as zero, so
    // the height must be continuous across the seam.
    // 网格延伸到 360 度，它与零度是同一子午线，故高度必须在接缝处连续。
    let at_zero = zero_isotherm_height(10.0, 0.0).expect("valid");
    let at_360 = zero_isotherm_height(10.0, 360.0).expect("valid");
    assert!((at_zero - at_360).abs() < TOLERANCE);

    // A negative longitude is the same as its positive equivalent.
    // 负经度与其正等价经度相同。
    let negative = zero_isotherm_height(10.0, -10.0).expect("valid");
    let positive = zero_isotherm_height(10.0, 350.0).expect("valid");
    assert!((negative - positive).abs() < TOLERANCE);
}

#[test]
fn the_height_is_bounded_by_the_table() {
    // Every tabulated value lies between the minimum and maximum of the
    // recommendation's global grid, which are about 0.006 km and 6.281 km.
    // 每个所列值都落在该建议书全球网格的最小与最大之间，即约 0.006 km 与 6.281 km。
    for latitude in [-90.0, -45.0, 0.0, 45.0, 90.0] {
        for longitude in [0.0, 90.0, 180.0, 270.0] {
            let height = zero_isotherm_height(latitude, longitude).expect("valid coordinates");
            assert!(
                (0.0..=7.0).contains(&height),
                "({latitude}, {longitude}) gave {height}"
            );
        }
    }
}

#[test]
fn the_height_is_continuous_across_the_grid() {
    // The interpolation must not jump between neighbouring grid cells.
    // 插值不得在相邻网格单元之间跳变。
    let mut previous = zero_isotherm_height(0.0, 0.0).expect("valid");
    for step in 1..=240 {
        let longitude = step as f64 * 1.5;
        let current = zero_isotherm_height(0.0, longitude).expect("valid coordinates");
        assert!(
            (current - previous).abs() < 1.0,
            "jump at longitude {longitude}: {previous} to {current}"
        );
        previous = current;
    }
}

#[test]
fn the_tropics_are_higher_than_the_poles() {
    // The isotherm is highest near the equator, where the atmosphere is
    // warmest, and lowest at high latitudes.
    // 等温线在赤道附近最高（那里大气最暖），在高纬度最低。
    let equator = zero_isotherm_height(0.0, 100.0).expect("valid");
    let mid_latitude = zero_isotherm_height(50.0, 100.0).expect("valid");
    let pole = zero_isotherm_height(80.0, 100.0).expect("valid");
    assert!(equator > mid_latitude, "{equator} against {mid_latitude}");
    assert!(mid_latitude > pole, "{mid_latitude} against {pole}");
}

#[test]
fn invalid_coordinates_are_rejected() {
    assert_eq!(zero_isotherm_height(f64::NAN, 0.0), None);
    assert_eq!(zero_isotherm_height(0.0, f64::NAN), None);
    assert_eq!(zero_isotherm_height(91.0, 0.0), None);
    assert_eq!(zero_isotherm_height(-91.0, 0.0), None);
    assert_eq!(rain_height(91.0, 0.0), None);
    assert_eq!(rain_height(f64::INFINITY, 0.0), None);
}

#[test]
fn the_poles_are_accepted() {
    assert!(zero_isotherm_height(90.0, 0.0).is_some());
    assert!(zero_isotherm_height(-90.0, 0.0).is_some());
    assert!(zero_isotherm_height(90.0, 359.9).is_some());
}
