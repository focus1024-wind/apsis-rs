//! Independent validation of the impulsive maneuvers against the vis-viva
//! equations.
//!
//! No expected value in this file calls apsis. Every burn and time is formed
//! from `sqrt(mu (2/r - 1/a))`, the circular speed, and the half period of the
//! transfer ellipse with raw arithmetic.
//!
//! 脉冲机动对活力公式的独立验证。
//!
//! 本文件中的期望值都不调用 apsis。每次点火与时间都由 `sqrt(mu (2/r - 1/a))`、
//! 圆轨道速度与转移椭圆的半周期以原始算术构成。

use apsis::apsis_kepler::GravityParameter;
use apsis::apsis_maneuver::{Impulse, hohmann_transfer};
use apsis::apsis_math::Vec3;

/// The WGS 84 gravitational parameter in km^3/s^2.
///
/// WGS 84 引力常数，单位为 km^3/s^2。
const MU: f64 = 398_600.5;

/// Returns the two burns and the transfer time from the defining equations.
///
/// 由定义方程返回两次点火与转移时间。
fn closed_form(from: f64, to: f64) -> [f64; 4] {
    let semi_major = f64::midpoint(from, to);
    let circular_from = (MU / from).sqrt();
    let circular_to = (MU / to).sqrt();
    let transfer_from = (MU * (2.0 / from - 1.0 / semi_major)).sqrt();
    let transfer_to = (MU * (2.0 / to - 1.0 / semi_major)).sqrt();
    let first = transfer_from - circular_from;
    let second = circular_to - transfer_to;
    let time = core::f64::consts::PI * (semi_major.powi(3) / MU).sqrt();
    [first, second, time, semi_major]
}

fn check(from: f64, to: f64) {
    let expected = closed_form(from, to);
    let transfer = hohmann_transfer(from, to, GravityParameter::WGS84).expect("usable radii");
    assert!(
        (transfer.first_burn - expected[0]).abs() < 1e-12,
        "first burn {} expected {}",
        transfer.first_burn,
        expected[0]
    );
    assert!(
        (transfer.second_burn - expected[1]).abs() < 1e-12,
        "second burn {} expected {}",
        transfer.second_burn,
        expected[1]
    );
    assert!(
        (transfer.transfer_time - expected[2]).abs() < 1e-6,
        "time {} expected {}",
        transfer.transfer_time,
        expected[2]
    );
    assert!((transfer.transfer_semi_major_axis - expected[3]).abs() < 1e-12);
    assert!((transfer.total_delta_v() - expected[0].abs() - expected[1].abs()).abs() < 1e-12);
}

#[test]
fn a_leo_to_geo_transfer_matches_the_vis_viva_result() {
    check(6_678.0, 42_164.0);
}

#[test]
fn a_leo_to_leo_raise_matches_the_vis_viva_result() {
    check(6_678.0, 7_078.0);
}

#[test]
fn a_geo_to_leo_descent_matches_the_vis_viva_result() {
    check(42_164.0, 6_678.0);
}

#[test]
fn a_descent_has_negative_burns_and_a_positive_cost() {
    let transfer = hohmann_transfer(42_164.0, 6_678.0, GravityParameter::WGS84).expect("usable");
    assert!(transfer.first_burn < 0.0);
    assert!(transfer.second_burn < 0.0);
    assert!(transfer.total_delta_v() > 0.0);
    assert!(!transfer.is_raising());
    assert!(
        (transfer.first_burn_magnitude() + transfer.second_burn_magnitude()
            - transfer.total_delta_v())
        .abs()
            < 1e-12
    );
}

#[test]
fn a_raise_has_positive_burns() {
    let transfer = hohmann_transfer(6_678.0, 42_164.0, GravityParameter::WGS84).expect("usable");
    assert!(transfer.first_burn > 0.0);
    assert!(transfer.second_burn > 0.0);
    assert!(transfer.is_raising());
}

#[test]
fn the_transfer_time_is_half_the_ellipse_period() {
    let transfer = hohmann_transfer(6_678.0, 42_164.0, GravityParameter::WGS84).expect("usable");
    let semi_major = transfer.transfer_semi_major_axis;
    let period = core::f64::consts::TAU * (semi_major.powi(3) / MU).sqrt();
    assert!((transfer.transfer_time - period / 2.0).abs() < 1e-6);
}

#[test]
fn an_impulse_changes_only_the_velocity() {
    let velocity = Vec3::new(0.0, 7.5, 0.0);
    let impulse = Impulse::along(velocity, 0.1).expect("valid");
    assert!((impulse.magnitude() - 0.1).abs() < 1e-12);
    let changed = impulse.apply_to_velocity(velocity);
    assert!((changed - velocity).norm() - 0.1 < 1e-12);
    assert!(changed.norm() > velocity.norm());
}

#[test]
fn degenerate_and_invalid_radii_are_rejected() {
    assert!(hohmann_transfer(7_000.0, 7_000.0, GravityParameter::WGS84).is_err());
    assert!(hohmann_transfer(0.0, 7_000.0, GravityParameter::WGS84).is_err());
    assert!(hohmann_transfer(-1.0, 7_000.0, GravityParameter::WGS84).is_err());
    assert!(hohmann_transfer(f64::NAN, 7_000.0, GravityParameter::WGS84).is_err());
}
