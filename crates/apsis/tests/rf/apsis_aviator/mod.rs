//! Validation tests for the aircraft performance model.
//!
//! 航空器性能模型的验证测试。

use apsis::apsis_aviator::{Aircraft, atmosphere, breguet_cruise, level_flight};

const TOLERANCE: f64 = 1e-6;

#[test]
fn the_sea_level_atmosphere_matches_the_reference() {
    let state = atmosphere(0.0).expect("valid altitude");
    assert!((state.temperature_kelvin - 288.15).abs() < 1e-9);
    assert!((state.pressure_pascals - 101_325.0).abs() < 1e-6);
    assert!((state.density_kg_m3 - 1.225).abs() < 1e-4);
    assert!((state.speed_of_sound_m_s - 340.294).abs() < 1e-3);
}

#[test]
fn the_eleven_kilometre_atmosphere_matches_the_reference() {
    let state = atmosphere(11_000.0).expect("valid altitude");
    assert!((state.temperature_kelvin - 216.774).abs() < 1e-2);
    assert!((state.pressure_pascals - 22_699.96).abs() < 0.1);
    assert!((state.density_kg_m3 - 0.3648).abs() < 1e-4);
}

#[test]
fn the_density_falls_with_altitude() {
    let mut previous = f64::INFINITY;
    for altitude in [0.0, 1_000.0, 5_000.0, 11_000.0, 15_000.0, 20_000.0] {
        let density = atmosphere(altitude).expect("valid").density_kg_m3;
        assert!(density < previous, "altitude {altitude}");
        previous = density;
    }
}

#[test]
fn level_flight_matches_the_hand_computation() {
    let aircraft = Aircraft::default();
    let result = level_flight(&aircraft, 0.0, 50.0, aircraft.max_thrust_n).expect("valid");
    let atmosphere = atmosphere(0.0).expect("valid");
    let weight = aircraft.mass_kg * 9.806_65;
    let dynamic = 0.5 * atmosphere.density_kg_m3 * 50.0 * 50.0;
    let lift = weight / (dynamic * aircraft.wing_area_m2);
    let induced =
        lift * lift / (core::f64::consts::PI * aircraft.oswald_efficiency * aircraft.aspect_ratio);
    let drag_coefficient = aircraft.zero_lift_drag + induced;
    let drag = dynamic * aircraft.wing_area_m2 * drag_coefficient;
    let climb = (aircraft.max_thrust_n - drag) * 50.0 / weight;
    assert!((result.lift_coefficient - lift).abs() < TOLERANCE);
    assert!((result.drag_coefficient - drag_coefficient).abs() < TOLERANCE);
    assert!((result.drag_n - drag).abs() < TOLERANCE);
    assert!((result.climb_rate_m_s - climb).abs() < TOLERANCE);
    assert!(
        (result.fuel_flow_kg_s - aircraft.max_thrust_n * aircraft.tsfc_kg_per_n_s).abs()
            < TOLERANCE
    );
    assert!((result.lift_to_drag - lift / drag_coefficient).abs() < TOLERANCE);
}

#[test]
fn thrust_below_drag_produces_a_descent() {
    let aircraft = Aircraft::default();
    let result = level_flight(&aircraft, 0.0, 50.0, 200.0).expect("valid");
    assert!(result.climb_rate_m_s < 0.0);
}

#[test]
fn the_breguet_range_matches_the_definition() {
    let aircraft = Aircraft::default();
    let cruise = breguet_cruise(&aircraft, 250.0, 15.0, 10_000.0, 8_000.0).expect("valid");
    let expected_endurance =
        15.0 / (9.806_65 * aircraft.tsfc_kg_per_n_s) * (10_000.0f64 / 8_000.0).ln();
    assert!((cruise.endurance_seconds - expected_endurance).abs() < TOLERANCE);
    assert!((cruise.range_meters - 250.0 * expected_endurance).abs() < TOLERANCE);
}

#[test]
fn invalid_inputs_are_rejected() {
    assert!(atmosphere(-1.0).is_err());
    assert!(atmosphere(30_000.0).is_err());
    let aircraft = Aircraft::default();
    assert!(level_flight(&aircraft, 0.0, 0.0, 1_000.0).is_err());
    assert!(breguet_cruise(&aircraft, 250.0, 15.0, 10_000.0, 10_000.0).is_err());
}

#[test]
fn turn_performance_is_consistent() {
    let turn = apsis::apsis_aviator::turn_performance(100.0, 4.0).expect("turn");
    assert!(turn.turn_rate_rad_s > 0.0);
    assert!(turn.turn_radius_m > 0.0);
    assert!((turn.turn_time_s * turn.turn_rate_rad_s - core::f64::consts::TAU).abs() < 1e-12);
    assert!(apsis::apsis_aviator::turn_performance(100.0, 0.5).is_err());
}

#[test]
fn a_climb_segment_integrates_time_distance_and_fuel() {
    let aircraft = Aircraft {
        mass_kg: 10_000.0,
        wing_area_m2: 30.0,
        zero_lift_drag: 0.025,
        oswald_efficiency: 0.8,
        aspect_ratio: 8.0,
        max_thrust_n: 60_000.0,
        tsfc_kg_per_n_s: 2.0e-5,
    };
    let climb = apsis::apsis_aviator::climb_segment(&aircraft, 0.0, 2_000.0, 120.0, 60_000.0, 500.0).expect("climb");
    assert!(climb.time_s > 0.0);
    assert!(climb.horizontal_distance_m > 0.0);
    assert!(climb.fuel_kg > 0.0);
    assert!((climb.average_climb_rate_m_s * climb.time_s - 2_000.0).abs() < 1e-9);
}
