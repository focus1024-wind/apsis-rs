//! Tests for the combined orbit type.
//!
//! 合并后的轨道类型的测试。

use apsis_orbit::{GravityModel, Orbit};
use apsis_tle::TwoLineElement;

const ISS: &str = concat!(
    "1 25544U 98067A   08264.51782528 -.00002182  00000-0 -11606-4 0  2927\r\n",
    "2 25544  51.6416 247.4627 0006703 130.5360 325.0288 15.72125391563537\r\n",
);

const DEEP_SPACE: &str = concat!(
    "1 09880U 77021A   06174.85710247  .00000090  00000-0  10000-3 0  7322\r\n",
    "2 09880  64.1526 313.1296 6546925 103.4683 305.3980  2.00754276 13047\r\n",
);

fn orbit(text: &str) -> Orbit {
    let element = TwoLineElement::parse(text).expect("test element set parses");
    Orbit::from_element(&element, GravityModel::WGS72).expect("test element set is propagatable")
}

#[test]
fn construction_succeeds_for_a_valid_element_set() {
    let orbit = orbit(ISS);
    assert!(!orbit.is_deep_space());
    assert_eq!(
        orbit.gravity_parameter(),
        apsis_orbit::GravityParameter::WGS72
    );
}

#[test]
fn deep_space_orbits_are_reported() {
    assert!(orbit(DEEP_SPACE).is_deep_space());
}

#[test]
fn propagation_matches_the_underlying_propagator() {
    let orbit = orbit(ISS);
    let via_orbit = orbit.propagate_minutes(360.0).expect("state exists");
    let via_satellite = orbit
        .satellite()
        .propagate_minutes(360.0)
        .expect("state exists");
    assert_eq!(via_orbit, via_satellite);
}

#[test]
fn position_and_radius_accessors_agree_with_the_state() {
    let orbit = orbit(ISS);
    let state = orbit.propagate_minutes(120.0).expect("state exists");
    assert_eq!(
        orbit.position_at(120.0).expect("state exists"),
        state.position
    );
    assert!((orbit.radius_at(120.0).expect("state exists") - state.position.norm()).abs() < 1e-12);
}

#[test]
fn propagate_at_uses_the_element_epoch() {
    let orbit = orbit(ISS);
    let epoch = orbit.epoch_mjd();
    let at_epoch = orbit.propagate_at(epoch).expect("state exists");
    let at_zero = orbit.propagate_minutes(0.0).expect("state exists");
    assert!((at_epoch.position - at_zero.position).norm() < 1e-6);

    // One day later is 1440 minutes.
    // 一天后是 1440 分钟。
    let next_day = orbit.propagate_at(epoch + 1.0).expect("state exists");
    let one_day = orbit.propagate_minutes(1440.0).expect("state exists");
    assert!((next_day.position - one_day.position).norm() < 1e-3);
}

#[test]
fn osculating_elements_describe_a_low_earth_orbit() {
    let orbit = orbit(ISS);
    let elements = orbit.osculating_elements(0.0).expect("state exists");
    // These values were checked against an independent computation from the
    // same state vector. The inclination agrees with the catalogued value
    // closely, because the oblateness has little effect on it, while the
    // eccentricity differs from the catalogued 0.0006703. That difference
    // is expected: the element set stores mean elements and the osculating
    // eccentricity includes the short period variation the two-body model
    // does not reproduce.
    // 这些值是针对同一天量矢量的一份独立计算核对的。倾角与编目值接近，因为扁率对它
    // 影响很小，而偏心率不同于编目的 0.0006703。该差异是预期的：根数集保存的是平
    // 根数，而密切偏心率包含二体模型不复现的短周期变化。
    assert!(
        (elements.semi_major_axis - 6_725.542).abs() < 0.01,
        "semi-major axis {}",
        elements.semi_major_axis
    );
    assert!(
        (elements.eccentricity - 0.000_832_1).abs() < 1e-6,
        "eccentricity {}",
        elements.eccentricity
    );
    assert!(
        (elements.inclination - 51.6217_f64.to_radians()).abs() < 1e-4,
        "inclination {}",
        elements.inclination.to_degrees()
    );
}

#[test]
fn osculating_elements_reproduce_the_state() {
    let orbit = orbit(ISS);
    let state = orbit.propagate_minutes(45.0).expect("state exists");
    let elements = orbit.osculating_elements(45.0).expect("state exists");
    let rebuilt = elements
        .to_state(orbit.gravity_parameter())
        .expect("elements are valid");
    assert!((rebuilt.position - state.position).norm() < 1e-6);
    assert!((rebuilt.velocity - state.velocity).norm() < 1e-9);
}

#[test]
fn osculating_elements_change_along_the_orbit() {
    // The osculating elements are not the mean elements, so they vary
    // because the perturbations SGP4 models are absent from the two-body
    // reconstruction.
    // 密切根数不是平根数，故它们变化，因为 SGP4 所建模的摄动在二体重构中缺席。
    let orbit = orbit(ISS);
    let start = orbit.osculating_elements(0.0).expect("state exists");
    let later = orbit.osculating_elements(720.0).expect("state exists");
    assert!((start.eccentricity - later.eccentricity).abs() > 1e-7);
}

#[test]
fn osculating_elements_work_for_a_deep_space_orbit() {
    let orbit = orbit(DEEP_SPACE);
    let elements = orbit.osculating_elements(0.0).expect("state exists");
    // The catalogued mean values are an eccentricity of 0.6546925 and an
    // inclination of 64.1526 degrees. The osculating values are close but
    // not identical for the reason given in the low Earth orbit test.
    // 编目的平值为偏心率 0.6546925、倾角 64.1526 度。密切值接近但不相同，理由同低轨
    // 测试中所述。
    assert!(
        (elements.eccentricity - 0.655_871_3).abs() < 1e-6,
        "eccentricity {}",
        elements.eccentricity
    );
    assert!(
        (elements.inclination - 64.1696_f64.to_radians()).abs() < 1e-4,
        "inclination {}",
        elements.inclination.to_degrees()
    );
    assert!(
        (elements.semi_major_axis - 26_543.982).abs() < 0.01,
        "semi-major axis {}",
        elements.semi_major_axis
    );
}

#[test]
fn the_gravity_parameter_matches_the_model() {
    let element = TwoLineElement::parse(ISS).expect("parses");
    let wgs72 = Orbit::from_element(&element, GravityModel::WGS72).expect("propagatable");
    assert_eq!(
        wgs72.gravity_parameter(),
        apsis_orbit::GravityParameter::WGS72
    );

    let wgs84 = Orbit::from_element(&element, GravityModel::WGS84).expect("propagatable");
    assert_eq!(
        wgs84.gravity_parameter(),
        apsis_orbit::GravityParameter::WGS84
    );
}

#[test]
fn the_facade_reexports_the_layer() {
    // The facade must make the element parser and the propagator types
    // reachable without a separate dependency.
    // 门面必须使根数解析器与传播器类型无需单独依赖即可达到。
    let element = apsis_orbit::TwoLineElement::parse(ISS).expect("parses");
    assert_eq!(element.satellite_number, 25544);
    let satellite =
        apsis_orbit::Satellite::from_element(&element, GravityModel::WGS72).expect("propagatable");
    assert!(satellite.propagate_minutes(0.0).is_ok());
}
