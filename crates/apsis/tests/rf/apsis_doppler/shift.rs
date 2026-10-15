//! Tests for the Doppler shift.
//!
//! 多普勒频移的测试。

use apsis::apsis_doppler::{
    DopplerShift, SPEED_OF_LIGHT, acquisition_span, angle_from_range_rate, classical_shift,
    maximum_shift, range_rate_from_velocity, relativistic_doppler_factor,
};
use apsis::apsis_math::Vec3;
use core::f64::consts::PI;

const TOLERANCE: f64 = 1e-9;

#[test]
fn range_rate_is_the_projection_of_velocity_on_the_line_of_sight() {
    // Moving away along the line of sight.
    // 沿视线远离。
    assert_eq!(
        range_rate_from_velocity(Vec3::new(1_000.0, 0.0, 0.0), Vec3::new(10.0, 0.0, 0.0)),
        Some(10.0)
    );
    // Moving across the line of sight gives no range rate.
    // 横穿视线不给出距离变化率。
    assert_eq!(
        range_rate_from_velocity(Vec3::new(1_000.0, 0.0, 0.0), Vec3::new(0.0, 10.0, 0.0)),
        Some(0.0)
    );
    // Approaching along the line of sight.
    // 沿视线接近。
    assert_eq!(
        range_rate_from_velocity(Vec3::new(1_000.0, 0.0, 0.0), Vec3::new(-10.0, 0.0, 0.0)),
        Some(-10.0)
    );
}

#[test]
fn range_rate_uses_the_magnitude_of_the_separation() {
    // The velocity is doubled but the separation is also doubled, so the
    // projection along the diagonal works out to the same fraction.
    // 速度加倍但间距也加倍，故沿对角线的投影算得相同比例。
    let rate = range_rate_from_velocity(Vec3::new(3.0, 4.0, 0.0), Vec3::new(6.0, 8.0, 0.0))
        .expect("non-zero separation");
    // The unit line of sight is (0.6, 0.8, 0), so the rate is
    // 0.6 * 6 + 0.8 * 8 = 10.
    // 单位视线为 (0.6, 0.8, 0)，故变化率为 0.6 * 6 + 0.8 * 8 = 10。
    assert!((rate - 10.0).abs() < TOLERANCE);
}

#[test]
fn a_zero_separation_has_no_range_rate() {
    assert_eq!(
        range_rate_from_velocity(Vec3::ZERO, Vec3::new(1.0, 0.0, 0.0)),
        None
    );
}

#[test]
fn non_finite_arguments_are_rejected() {
    assert_eq!(
        range_rate_from_velocity(Vec3::splat(f64::NAN), Vec3::ZERO),
        None
    );
    assert_eq!(
        range_rate_from_velocity(Vec3::X, Vec3::splat(f64::INFINITY)),
        None
    );
}

#[test]
fn the_classical_shift_is_negative_for_a_receding_terminal() {
    let shift = classical_shift(12.0e9, 1_000.0).expect("valid arguments");
    assert!(shift < 0.0);
    assert!((shift + 12.0e9 * 1_000.0 / SPEED_OF_LIGHT).abs() < 1e-3);
}

#[test]
fn a_known_range_rate_gives_a_known_shift() {
    // A range rate of 7 km/s at 12 GHz gives about 280.2 kHz.
    // 12 GHz 下 7 km/s 的距离变化率给出约 280.2 kHz。
    let shift = classical_shift(12.0e9, 7_000.0).expect("valid");
    assert!((shift + 280_194.0).abs() < 1.0, "shift {shift}");
    assert!((shift.abs() / 12.0e9 * 1.0e6 - 23.35).abs() < 0.01);
}

#[test]
fn approaching_and_receding_shifts_are_mirror_images() {
    let receding = classical_shift(1.0e9, 5_000.0).expect("valid");
    let approaching = classical_shift(1.0e9, -5_000.0).expect("valid");
    assert!((receding + approaching).abs() < 1e-9);
}

#[test]
fn a_zero_range_rate_gives_no_shift() {
    assert_eq!(classical_shift(12.0e9, 0.0), Some(0.0));
}

#[test]
fn invalid_arguments_are_rejected() {
    assert_eq!(classical_shift(0.0, 1_000.0), None);
    assert_eq!(classical_shift(-1.0, 1_000.0), None);
    assert_eq!(classical_shift(12.0e9, f64::NAN), None);
    assert_eq!(classical_shift(f64::NAN, 1_000.0), None);
    assert_eq!(classical_shift(12.0e9, f64::INFINITY), None);
}

#[test]
fn the_relativistic_factor_is_one_at_rest() {
    let factor = relativistic_doppler_factor(0.0).expect("valid");
    assert!((factor - 1.0).abs() < TOLERANCE);
}

#[test]
fn the_relativistic_factor_opposes_the_classical_shift() {
    // For a receding terminal the factor is below one, which reinforces the
    // classical redshift.
    // 对远离的终端该因子小于一，这加强了经典红移。
    let receding = relativistic_doppler_factor(7_000.0).expect("valid");
    assert!(receding < 1.0);
    // For an approaching terminal it is above one.
    // 对接近的终端它大于一。
    let approaching = relativistic_doppler_factor(-7_000.0).expect("valid");
    assert!(approaching > 1.0);
    // The two are reciprocals, because swapping the roles of the terminals
    // inverts the ratio.
    // 二者互为倒数，因为交换两终端的角色会反转该比值。
    assert!((receding * approaching - 1.0).abs() < 1e-15);
}

#[test]
fn the_relativistic_factor_contains_the_classical_shift() {
    // The factor's expansion begins with 1 - beta, which is the classical
    // result, so the two treatments must agree to first order. They differ
    // only by terms of order beta squared.
    // 该因子的展开以 1 - beta 开头，即经典结果，故两种处理必须在一阶上一致。它们
    // 只相差 `beta` 平方量级的项。
    let beta = 7_000.0 / SPEED_OF_LIGHT;
    let factor = relativistic_doppler_factor(7_000.0).expect("valid");
    let difference = (factor - 1.0 + beta).abs();
    // The leading difference is beta^2 / 2, about 2.7e-10.
    // 主导差值为 beta^2 / 2，约 2.7e-10。
    assert!(difference < beta * beta, "difference {difference}");
    assert!((difference - beta * beta / 2.0).abs() < 1e-12);
}

#[test]
fn the_two_treatments_agree_to_second_order() {
    // The classical shift and the relativistic one differ by a fraction of
    // the carrier that is of order beta squared.
    // 经典频移与相对论频移相差一个 `beta` 平方量级的载波比例。
    let carrier = 12.0e9;
    let shift = DopplerShift::from_range_rate(carrier, 7_000.0).expect("valid");
    let classical = shift.shift();
    let relativistic = shift.shift_including_relativity().expect("valid");
    let relative_difference = (relativistic - classical).abs() / carrier;
    let beta = 7_000.0 / SPEED_OF_LIGHT;
    assert!(
        relative_difference < beta * beta,
        "difference {relative_difference}"
    );
    // The absolute difference is the second order term, about 3.3 Hz at
    // 12 GHz: tiny compared with the 280 kHz classical shift, but real.
    // 绝对差是二阶项，在 12 GHz 处约 3.3 Hz：与 280 kHz 的经典频移相比很小，但真实
    // 存在。
    assert!(
        (relativistic - classical).abs() > 1.0,
        "difference {}",
        (relativistic - classical).abs()
    );
    assert!((relativistic - classical).abs() < 10.0);
}

#[test]
fn a_speed_of_light_range_rate_is_rejected() {
    assert_eq!(relativistic_doppler_factor(SPEED_OF_LIGHT), None);
    assert_eq!(relativistic_doppler_factor(-SPEED_OF_LIGHT), None);
    assert_eq!(relativistic_doppler_factor(SPEED_OF_LIGHT * 2.0), None);
    assert_eq!(relativistic_doppler_factor(f64::NAN), None);
}

#[test]
fn the_shift_type_reports_the_carrier_and_rate() {
    let shift = DopplerShift::from_range_rate(12.0e9, 1_000.0).expect("valid");
    assert_eq!(shift.carrier_frequency(), 12.0e9);
    assert_eq!(shift.range_rate(), 1_000.0);
    assert!(shift.is_receding());
}

#[test]
fn the_shift_type_converts_between_frequencies() {
    let shift = DopplerShift::from_range_rate(12.0e9, 1_000.0).expect("valid");
    let received = shift.shifted_frequency();
    assert!(received < 12.0e9);
    assert!((received - (12.0e9 + shift.shift())).abs() < 1e-6);
}

#[test]
fn an_approaching_terminal_has_a_higher_received_frequency() {
    let shift = DopplerShift::from_range_rate(12.0e9, -1_000.0).expect("valid");
    assert!(!shift.is_receding());
    assert!(shift.shifted_frequency() > 12.0e9);
}

#[test]
fn the_shift_in_parts_per_million_is_consistent() {
    let shift = DopplerShift::from_range_rate(12.0e9, 7_000.0).expect("valid");
    let ppm = shift.shift_in_ppm();
    assert!((ppm + 7_000.0 / SPEED_OF_LIGHT * 1.0e6).abs() < 1e-9);
    assert!((ppm + 23.349).abs() < 0.01, "ppm {ppm}");
}

#[test]
fn the_relativistic_shift_is_close_to_the_classical_one() {
    let shift = DopplerShift::from_range_rate(12.0e9, 7_000.0).expect("valid");
    let classical = shift.shift();
    let total = shift.shift_including_relativity().expect("valid");
    // Both are negative for a receding terminal and agree in sign.
    // 对远离的终端二者都为负，符号一致。
    assert!(classical < 0.0);
    assert!(total < 0.0);
    // The second order difference is a few hertz on a shift of a few
    // hundred kilohertz.
    // 二阶差在几百千赫兹的频移上是几赫兹。
    assert!((total - classical).abs() > 1.0);
    assert!((total - classical).abs() < 10.0);
}

#[test]
fn invalid_construction_is_rejected() {
    assert_eq!(DopplerShift::from_range_rate(0.0, 1_000.0), None);
    assert_eq!(DopplerShift::from_range_rate(12.0e9, f64::NAN), None);
    assert_eq!(DopplerShift::from_range_rate(-12.0e9, 1_000.0), None);
}

#[test]
fn the_shift_rate_follows_the_range_acceleration() {
    let shift = DopplerShift::from_range_rate(12.0e9, 1_000.0).expect("valid");
    // A range acceleration of 10 m/s^2 changes the shift by
    // 12 GHz * 10 / c, about 0.4 Hz per second.
    // 10 m/s^2 的距离加速度使频移改变 12 GHz * 10 / c，约每秒 0.4 Hz。
    let rate = shift.shift_rate(10.0).expect("valid");
    assert!((rate + 400.3).abs() < 0.1, "rate {rate}");
    assert_eq!(shift.shift_rate(f64::NAN), None);
}

#[test]
fn the_angle_from_a_range_rate_is_the_angle_to_the_velocity() {
    let velocity = Vec3::new(0.0, 1_000.0, 0.0);
    // The full speed along the line of sight is zero degrees.
    // 沿视线的全速对应零度。
    let along = angle_from_range_rate(velocity, 1_000.0).expect("valid");
    assert!(along.abs() < 1e-9);
    // The full speed across the line of sight is a right angle.
    // 横穿视线的全速对应直角。
    let across = angle_from_range_rate(velocity, 0.0).expect("valid");
    assert!((across - PI / 2.0).abs() < TOLERANCE);
    // The opposite direction is a straight angle.
    // 相反方向是平角。
    let opposite = angle_from_range_rate(velocity, -1_000.0).expect("valid");
    assert!((opposite - PI).abs() < 1e-9);
}

#[test]
fn the_angle_from_a_range_rate_rejects_invalid_input() {
    assert_eq!(angle_from_range_rate(Vec3::ZERO, 1.0), None);
    assert_eq!(angle_from_range_rate(Vec3::X, f64::NAN), None);
}

#[test]
fn the_maximum_shift_covers_the_line_of_sight_case() {
    let shift = maximum_shift(12.0e9, 7_000.0).expect("valid");
    assert!((shift - 12.0e9 * 7_000.0 / SPEED_OF_LIGHT).abs() < 1e-3);
    // A range rate beyond the speed is not physical, so the speed must be
    // non-negative and finite.
    // 超过该速度的距离变化率不物理，故速度必须非负且有限。
    assert_eq!(maximum_shift(12.0e9, -1.0), None);
    assert_eq!(maximum_shift(-1.0, 7_000.0), None);
    assert_eq!(maximum_shift(12.0e9, f64::INFINITY), None);
}

#[test]
fn the_acquisition_span_is_twice_the_maximum_shift() {
    let span = acquisition_span(12.0e9, 7_000.0).expect("valid");
    let maximum = maximum_shift(12.0e9, 7_000.0).expect("valid");
    assert!((span - 2.0 * maximum).abs() < 1e-3);
    // For a low Earth orbit at 12 GHz the span is about 560 kHz.
    // 对 12 GHz 下的低轨，该跨度为约 560 kHz。
    assert!((span - 560_388.0).abs() < 2.0, "span {span}");
}

#[test]
fn display_reports_the_shift_and_the_carrier() {
    let shift = DopplerShift::from_range_rate(12.0e9, 1_000.0).expect("valid");
    let text = shift.to_string();
    assert!(text.contains("Hz"));
    assert!(text.contains("ppm"));
}
