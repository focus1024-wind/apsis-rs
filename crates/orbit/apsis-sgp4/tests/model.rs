//! Tests for the gravity models.
//!
//! 引力模型的测试。

use apsis_sgp4::GravityModel;

#[test]
fn wgs72_defining_values_match_the_standard() {
    assert_eq!(GravityModel::WGS72.mu, 398_600.8);
    assert_eq!(GravityModel::WGS72.radius, 6_378.135);
    assert_eq!(GravityModel::WGS72.j2, 0.001_082_616);
    assert_eq!(GravityModel::WGS72.j3, -0.000_002_538_81);
    assert_eq!(GravityModel::WGS72.j4, -0.000_001_655_97);
}

#[test]
fn wgs84_defining_values_match_the_standard() {
    assert_eq!(GravityModel::WGS84.mu, 398_600.5);
    assert_eq!(GravityModel::WGS84.radius, 6_378.137);
    assert_eq!(GravityModel::WGS84.j2, 0.001_082_629_89);
}

#[test]
fn kepler_constant_matches_the_reference_value() {
    // The reference implementation uses xke = 0.07436691613317342 for WGS
    // 72, derived as 60 / sqrt(radius^3 / mu).
    // 参考实现对 WGS 72 使用 xke = 0.07436691613317342，由 60 / sqrt(radius^3 / mu)
    // 导出。
    let xke = GravityModel::WGS72.xke();
    assert!((xke - 0.074_366_916_133_173_42).abs() < 1e-15, "got {xke}");
}

#[test]
fn time_unit_is_the_reciprocal_of_the_kepler_constant() {
    let model = GravityModel::WGS72;
    assert!((model.tumin() * model.xke() - 1.0).abs() < 1e-15);
    // The value is close to 13.44684 minutes for WGS 72.
    // 对 WGS 72，该值接近 13.44684 分钟。
    assert!((model.tumin() - 13.446_839_696_959_309).abs() < 1e-12);
}

#[test]
fn j3_over_j2_matches_the_quotient() {
    let model = GravityModel::WGS72;
    let expected = model.j3 / model.j2;
    assert!((model.j3_over_j2() - expected).abs() < 1e-18);
    // The value is about -0.002345069720011528.
    // 该值约为 -0.002345069720011528。
    assert!((model.j3_over_j2() + 0.002_345_069_720_011_528).abs() < 1e-15);
}

#[test]
fn sqrt_and_reciprocal_sqrt_are_consistent() {
    let model = GravityModel::WGS84;
    assert!((model.sqrt_mu() * model.reciprocal_sqrt_mu() - 1.0).abs() < 1e-15);
    assert!((model.sqrt_mu() - 398_600.5_f64.sqrt()).abs() < 1e-9);
}
