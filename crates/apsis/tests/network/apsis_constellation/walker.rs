//! Tests for the Walker constellation pattern.
//!
//! Walker 星座模式的测试。

use apsis::apsis_constellation::{Constellation, ConstellationErrorKind, Satellite, WalkerPattern};
use apsis::apsis_kepler::true_from_mean;
use core::f64::consts::PI;

const TOLERANCE: f64 = 1e-12;

/// A Walker 53 degree pattern with 24 satellites in 3 planes, as used by a
/// mid latitude broadband constellation.
///
/// 一个 53 度的 Walker 模式，3 个轨道面、24 颗卫星，如中纬度宽带星座所用。
fn broadband_pattern() -> WalkerPattern {
    WalkerPattern {
        total_satellites: 24,
        planes: 3,
        phase_factor: 1,
        inclination: 53.0_f64.to_radians(),
        semi_major_axis: 6_878.137,
    }
}

#[test]
fn the_derived_counts_are_correct() {
    let pattern = broadband_pattern();
    assert_eq!(pattern.satellites_per_plane(), Some(8));
    assert!((pattern.plane_spacing().expect("valid") - 2.0 * PI / 3.0).abs() < TOLERANCE);
    // The phase offset is 1 * 360 / 24 degrees.
    // 相位偏移为 1 * 360 / 24 度。
    let expected = 2.0 * PI / 24.0;
    assert!((pattern.phase_offset().expect("valid") - expected).abs() < TOLERANCE);
}

#[test]
fn instantiation_produces_the_right_number_of_satellites() {
    let constellation = Constellation::walker(broadband_pattern(), 60_000.0).expect("valid");
    assert_eq!(constellation.len(), 24);
    assert_eq!(constellation.plane_count(), 3);
    assert_eq!(constellation.plane_populations(), vec![8, 8, 8]);
    assert!(!constellation.is_empty());
}

#[test]
fn every_satellite_is_circular_and_at_the_same_radius() {
    let constellation = Constellation::walker(broadband_pattern(), 60_000.0).expect("valid");
    for satellite in constellation.satellites() {
        assert_eq!(satellite.elements.eccentricity, 0.0);
        assert!((satellite.elements.semi_major_axis - 6_878.137).abs() < TOLERANCE);
        assert!((satellite.elements.inclination - 53.0_f64.to_radians()).abs() < TOLERANCE);
    }
}

#[test]
fn the_planes_are_evenly_spaced_in_right_ascension() {
    let constellation = Constellation::walker(broadband_pattern(), 60_000.0).expect("valid");
    let expected = [0.0, 2.0 * PI / 3.0, 4.0 * PI / 3.0];
    for (plane, expected_right_ascension) in expected.iter().enumerate() {
        for satellite in constellation.plane(plane) {
            assert!(
                (satellite.elements.right_ascension - expected_right_ascension).abs() < 1e-9,
                "plane {plane}: {} against {expected_right_ascension}",
                satellite.elements.right_ascension
            );
        }
    }
}

#[test]
fn satellites_within_a_plane_are_evenly_spaced_in_anomaly() {
    let constellation = Constellation::walker(broadband_pattern(), 60_000.0).expect("valid");
    for plane in 0..3 {
        let satellites = constellation.plane(plane);
        let mut anomalies: Vec<f64> = satellites
            .iter()
            .map(|satellite| satellite.elements.mean_anomaly())
            .collect();
        anomalies.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
        for pair in anomalies.windows(2) {
            let spacing = pair[1] - pair[0];
            assert!((spacing - 2.0 * PI / 8.0).abs() < 1e-9, "spacing {spacing}");
        }
    }
}

#[test]
fn the_phase_offset_appears_between_adjacent_planes() {
    let pattern = broadband_pattern();
    let constellation = Constellation::walker(pattern, 60_000.0).expect("valid");
    let offset = pattern.phase_offset().expect("valid");

    // The first satellite of each plane is advanced by the phase offset
    // relative to the first satellite of the previous plane.
    // 每个轨道面的第一颗卫星相对前一面第一颗卫星超前一个相位偏移。
    let first_of_plane = |plane: usize| -> f64 {
        constellation
            .plane(plane)
            .iter()
            .min_by(|a, b| {
                a.index_in_plane
                    .partial_cmp(&b.index_in_plane)
                    .expect("indices are integers")
            })
            .expect("the plane is not empty")
            .elements
            .mean_anomaly()
    };
    let first = first_of_plane(0);
    let second = first_of_plane(1);
    let third = first_of_plane(2);
    assert!(
        (second - first - offset).abs() < 1e-9,
        "{second} against {first}"
    );
    assert!(
        (third - second - offset).abs() < 1e-9,
        "{third} against {second}"
    );
}

#[test]
fn a_zero_phase_factor_gives_no_offset_between_planes() {
    let pattern = WalkerPattern {
        phase_factor: 0,
        ..broadband_pattern()
    };
    let constellation = Constellation::walker(pattern, 0.0).expect("valid");
    assert_eq!(constellation.len(), 24);
    let first_of_plane = |plane: usize| -> f64 {
        constellation
            .plane(plane)
            .iter()
            .find(|satellite| satellite.index_in_plane == 0)
            .expect("the plane is not empty")
            .elements
            .mean_anomaly()
    };
    // Every plane starts at the same anomaly.
    // 每个轨道面从相同的近点角开始。
    assert!((first_of_plane(0) - first_of_plane(1)).abs() < 1e-9);
    assert!((first_of_plane(1) - first_of_plane(2)).abs() < 1e-9);
}

#[test]
fn the_true_anomaly_matches_the_mean_anomaly_for_a_circular_orbit() {
    let constellation = Constellation::walker(broadband_pattern(), 0.0).expect("valid");
    for satellite in constellation.satellites() {
        let mean = satellite.elements.mean_anomaly();
        let expected = true_from_mean(mean, 0.0);
        assert!((satellite.elements.true_anomaly - expected).abs() < 1e-12);
    }
}

#[test]
fn a_single_plane_pattern_is_accepted() {
    let pattern = WalkerPattern {
        total_satellites: 6,
        planes: 1,
        phase_factor: 0,
        inclination: 0.0,
        semi_major_axis: 7_000.0,
    };
    let constellation = Constellation::walker(pattern, 0.0).expect("valid");
    assert_eq!(constellation.len(), 6);
    assert_eq!(constellation.plane_count(), 1);
    // All satellites share a plane, so they share a right ascension.
    // 所有卫星共享一个轨道面，故共享一个赤经。
    for satellite in constellation.satellites() {
        assert_eq!(satellite.elements.right_ascension, 0.0);
    }
}

#[test]
fn a_pattern_with_one_satellite_per_plane_is_accepted() {
    let pattern = WalkerPattern {
        total_satellites: 4,
        planes: 4,
        phase_factor: 3,
        inclination: 1.0,
        semi_major_axis: 7_000.0,
    };
    let constellation = Constellation::walker(pattern, 0.0).expect("valid");
    assert_eq!(constellation.len(), 4);
    assert_eq!(constellation.plane_count(), 4);
    assert_eq!(constellation.plane_populations(), vec![1, 1, 1, 1]);
}

#[test]
fn an_uneven_split_is_rejected() {
    let pattern = WalkerPattern {
        total_satellites: 25,
        planes: 3,
        ..broadband_pattern()
    };
    assert_eq!(pattern.satellites_per_plane(), None);
    assert!(!pattern.is_valid());
    let error = pattern
        .instantiate()
        .expect_err("25 is not a multiple of 3");
    assert_eq!(error.kind(), ConstellationErrorKind::UnevenPlanes);
}

#[test]
fn a_zero_satellite_count_is_rejected() {
    let pattern = WalkerPattern {
        total_satellites: 0,
        ..broadband_pattern()
    };
    let error = pattern.instantiate().expect_err("no satellites");
    assert_eq!(error.kind(), ConstellationErrorKind::NoSatellites);
}

#[test]
fn a_zero_plane_count_is_rejected() {
    let pattern = WalkerPattern {
        planes: 0,
        ..broadband_pattern()
    };
    let error = pattern.instantiate().expect_err("no planes");
    assert_eq!(error.kind(), ConstellationErrorKind::NoPlanes);
}

#[test]
fn an_out_of_range_phase_factor_is_rejected() {
    let pattern = WalkerPattern {
        phase_factor: 5,
        ..broadband_pattern()
    };
    assert!(!pattern.is_valid());
    let error = pattern
        .instantiate()
        .expect_err("phase factor exceeds the plane count");
    assert_eq!(error.kind(), ConstellationErrorKind::InvalidOrbit);
}

#[test]
fn an_invalid_inclination_is_rejected() {
    let pattern = WalkerPattern {
        inclination: PI + 0.1,
        ..broadband_pattern()
    };
    assert!(!pattern.is_valid());
    let error = pattern
        .instantiate()
        .expect_err("inclination is out of range");
    assert_eq!(error.kind(), ConstellationErrorKind::InvalidOrbit);
}

#[test]
fn a_non_positive_semi_major_axis_is_rejected() {
    let pattern = WalkerPattern {
        semi_major_axis: 0.0,
        ..broadband_pattern()
    };
    assert!(!pattern.is_valid());
    assert_eq!(
        pattern.instantiate().expect_err("zero radius").kind(),
        ConstellationErrorKind::InvalidOrbit
    );
}

#[test]
fn a_non_finite_reference_time_is_rejected() {
    let error =
        Constellation::walker(broadband_pattern(), f64::NAN).expect_err("the time is not finite");
    assert_eq!(error.kind(), ConstellationErrorKind::InvalidOrbit);
}

#[test]
fn the_reference_time_is_retained() {
    let constellation = Constellation::walker(broadband_pattern(), 60_123.5).expect("valid");
    assert_eq!(constellation.reference_mjd(), 60_123.5);
}

#[test]
fn the_pattern_is_retained() {
    let pattern = broadband_pattern();
    let constellation = Constellation::walker(pattern, 0.0).expect("valid");
    assert_eq!(constellation.pattern(), Some(pattern));
}

#[test]
fn the_satellite_indices_are_consistent() {
    let constellation = Constellation::walker(broadband_pattern(), 0.0).expect("valid");
    for (position, satellite) in constellation.satellites().iter().enumerate() {
        assert_eq!(satellite.index, position);
        // The satellites are emitted plane by plane, so the plane index is
        // the quotient of the position and the plane population.
        // 卫星逐面发射，故轨道面下标是位置除以每面卫星数之商。
        assert_eq!(satellite.plane, position / 8);
        assert_eq!(satellite.index_in_plane, position % 8);
    }
}

#[test]
fn the_altitude_is_measured_above_the_given_radius() {
    let constellation = Constellation::walker(broadband_pattern(), 0.0).expect("valid");
    let altitude = constellation
        .altitude_above(6_378.137)
        .expect("the constellation is not empty");
    assert!((altitude - 500.0).abs() < 1e-9, "altitude {altitude}");
}

#[test]
fn the_notation_reports_the_walker_fields() {
    let pattern = WalkerPattern {
        total_satellites: 24,
        planes: 3,
        phase_factor: 1,
        inclination: 53.0_f64.to_radians(),
        semi_major_axis: 6_878.137,
    };
    let notation = pattern.notation();
    assert!(notation.starts_with("53"), "notation {notation}");
    assert!(notation.contains("24/3/1"), "notation {notation}");
}

#[test]
fn a_custom_constellation_groups_satellites_by_right_ascension() {
    use apsis::apsis_kepler::KeplerianElements;
    let elements = [
        KeplerianElements {
            semi_major_axis: 7_000.0,
            eccentricity: 0.001,
            inclination: 0.9,
            right_ascension: 0.0,
            argument_of_perigee: 0.0,
            true_anomaly: 0.0,
        },
        KeplerianElements {
            semi_major_axis: 7_000.0,
            eccentricity: 0.001,
            inclination: 0.9,
            right_ascension: 0.0,
            argument_of_perigee: 0.0,
            true_anomaly: PI,
        },
        KeplerianElements {
            semi_major_axis: 7_000.0,
            eccentricity: 0.001,
            inclination: 0.9,
            right_ascension: 2.0,
            argument_of_perigee: 0.0,
            true_anomaly: 0.5,
        },
    ];
    let constellation = Constellation::from_satellites(&elements, 0.0).expect("valid");
    assert_eq!(constellation.len(), 3);
    assert_eq!(constellation.plane_count(), 2);
    assert_eq!(constellation.plane_populations(), vec![2, 1]);
    assert_eq!(constellation.pattern(), None);

    // The first two share a plane.
    // 前两颗共享一个轨道面。
    assert_eq!(constellation.satellites()[0].plane, 0);
    assert_eq!(constellation.satellites()[1].plane, 0);
    assert_eq!(constellation.satellites()[2].plane, 1);
    assert_eq!(constellation.satellites()[0].index_in_plane, 0);
    assert_eq!(constellation.satellites()[1].index_in_plane, 1);
}

#[test]
fn an_empty_custom_constellation_is_rejected() {
    let error = Constellation::from_satellites(&[], 0.0).expect_err("no satellites");
    assert_eq!(error.kind(), ConstellationErrorKind::NoSatellites);
}

#[test]
fn a_custom_constellation_rejects_invalid_elements() {
    use apsis::apsis_kepler::KeplerianElements;
    let elements = [KeplerianElements {
        semi_major_axis: -1.0,
        eccentricity: 0.0,
        inclination: 0.0,
        right_ascension: 0.0,
        argument_of_perigee: 0.0,
        true_anomaly: 0.0,
    }];
    let error = Constellation::from_satellites(&elements, 0.0).expect_err("negative radius");
    assert_eq!(error.kind(), ConstellationErrorKind::InvalidOrbit);
}

#[test]
fn an_out_of_range_plane_query_is_empty() {
    let constellation = Constellation::walker(broadband_pattern(), 0.0).expect("valid");
    assert_eq!(constellation.plane(99), &[]);
}

#[test]
fn the_satellite_type_is_copyable() {
    // The type must remain cheap to pass around, since coverage and access
    // analysis iterate over it repeatedly.
    // 该类型必须保持传递开销小，因为覆盖与可见性分析会反复遍历它。
    fn assert_copy<T: Copy>() {}
    assert_copy::<Satellite>();
}

#[test]
fn the_walker_phasing_matches_the_definition() {
    // The definition places the j-th satellite in the i-th plane at
    // RAAN = i * 360 / p and mean anomaly = j * 360 / (t / p) + i * f * 360 / t.
    // 定义把第 i 个轨道面的第 j 颗卫星放在
    // RAAN = i * 360 / p，平近点角 = j * 360 / (t / p) + i * f * 360 / t。
    let pattern = WalkerPattern {
        total_satellites: 24,
        planes: 3,
        phase_factor: 1,
        inclination: 53.0_f64.to_radians(),
        semi_major_axis: 6_878.137,
    };
    let constellation = Constellation::walker(pattern, 0.0).expect("valid");
    let per_plane = pattern.satellites_per_plane().expect("valid");
    for plane in 0..pattern.planes {
        for (index, satellite) in constellation.plane(plane).iter().enumerate() {
            let expected_raan = plane as f64 * 2.0 * PI / pattern.planes as f64;
            let expected_mean = index as f64 * 2.0 * PI / per_plane as f64
                + plane as f64 * pattern.phase_offset().expect("valid");
            let actual_mean = satellite.elements.mean_anomaly().rem_euclid(2.0 * PI);
            let expected_mean = expected_mean.rem_euclid(2.0 * PI);
            assert!(
                (satellite.elements.right_ascension - expected_raan).abs() < 1e-9,
                "plane {plane} RAAN"
            );
            assert!(
                (actual_mean - expected_mean).abs() < 1e-9,
                "plane {plane} satellite {index}: {actual_mean} against {expected_mean}"
            );
        }
    }
}

#[test]
fn plane_outlines_sample_each_plane_on_its_circular_orbit() {
    let constellation = Constellation::walker(broadband_pattern(), 60_000.0).expect("valid");
    let outlines = constellation.plane_outlines(72);
    assert_eq!(outlines.len(), 3);
    let radius = broadband_pattern().semi_major_axis;
    let vertical = radius * broadband_pattern().inclination.sin();
    for (plane, outline) in outlines.iter().enumerate() {
        assert_eq!(outline.plane, plane);
        assert_eq!(outline.points.len(), 73);
        for point in &outline.points {
            assert!(
                (point.norm() - radius).abs() < 1e-8,
                "plane {plane} radius {}",
                point.norm()
            );
            assert!(
                point.z.abs() <= vertical + 1e-8,
                "plane {plane} z {} exceeds the inclination bound {vertical}",
                point.z
            );
        }
    }
    assert!(constellation.plane_outlines(0).is_empty());
}
