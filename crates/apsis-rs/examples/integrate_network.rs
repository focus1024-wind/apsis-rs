//! Cross-layer integration check: constellation to access windows.
//!
//! This example walks the whole chain that the workspace provides and
//! asserts that each link in it produces sensible input for the next. It is
//! not a numerical cross-check against an external reference; the per-layer
//! cross-checks do that. Its purpose is to fail when a change in one layer
//! breaks the assumptions of another.
//!
//! The chain is:
//!
//! 1. Build a Walker constellation with `apsis-constellation`.
//! 2. Propagate one satellite with `apsis-sgp4` to get a state vector in the
//!    True Equator Mean Equinox frame.
//! 3. Rotate that state into the Earth-fixed frame with `apsis-coord` and
//!    the Earth rotation angle from `apsis-frames`.
//! 4. Search for access windows from a ground station with
//!    `apsis-access`.
//! 5. Build the time series of inter-satellite links with
//!    `apsis-topology` and `apsis-isl`.
//! 6. Route between two terminals with `apsis-routing` and measure the
//!    redundancy of the mesh with `apsis-mesh`.
//!
//! Usage:
//!
//! ```text
//! cargo run -p apsis-rs --features network --example integrate_network
//! ```

use apsis_access::{AccessFinder, OrbitSample};
use apsis_constellation::{Constellation, WalkerPattern};
use apsis_coord::{Ecef, Eci};
use apsis_coverage::GroundPoint;
use apsis_frames::EarthRotationAngle;
use apsis_isl::mutual_visibility;
use apsis_math::Vec3;
use apsis_mesh::Mesh;
use apsis_routing::shortest_path;
use apsis_sgp4::{GravityModel, Satellite};
use apsis_time::{Epoch, TimeScale};
use apsis_tle::TwoLineElement;
use apsis_topology::{Topology, TopologySample, TopologySeries};

/// The Earth equatorial radius, in kilometers, used for the link geometry.
const BODY_RADIUS_KM: f64 = 6_378.137;

/// The element set of a satellite in a 550 km orbit, used as the source of
/// the propagated track.
const TLE: &str = concat!(
    "1 25544U 98067A   08264.51782528 -.00002182  00000-0 -11606-4 0  2927\r\n",
    "2 25544  51.6416 247.4627 0006703 130.5360 325.0288 15.72125391563537\r\n",
);

/// Returns the time of the element set epoch as a Modified Julian Date.
fn epoch_mjd() -> f64 {
    let element = TwoLineElement::parse(TLE).expect("the sample element set parses");
    element
        .epoch
        .mjd(TimeScale::Utc)
        .expect("the epoch is inside the leap second table")
}

/// Returns the satellite position in the Earth-fixed frame at a number of
/// seconds from the element set epoch.
///
/// The SGP4 state is in the True Equator Mean Equinox frame, which this
/// treats as the inertial frame for the rotation. That is an approximation:
/// the two differ by the equation of the equinoxes, which is below a
/// thousandth of a degree. It is adequate for an integration check, and
/// replacing it with the exact transformation is the job of the coordinate
/// milestone that adds precession and nutation.
fn position_ecef_km(satellite: &Satellite, seconds: f64) -> Vec3 {
    let state = satellite
        .propagate_minutes(seconds / 60.0)
        .expect("the sample orbit propagates");
    let inertial = Eci::new(state.position);
    // The Earth rotation angle needs the UT1 time. The difference between
    // UT1 and TAI is under a second, which is under 0.005 degrees of
    // rotation, so the TAI epoch is used here.
    let days_since_j2000 = (epoch_mjd() + seconds / 86_400.0) - 51_544.5;
    let angle = EarthRotationAngle::from_ut1_days(days_since_j2000);
    inertial.to_ecef(angle).vector()
}

fn main() {
    // Step 1: the constellation geometry.
    let pattern = WalkerPattern {
        total_satellites: 24,
        planes: 3,
        phase_factor: 1,
        inclination: 53.0_f64.to_radians(),
        semi_major_axis: 6_928.137,
    };
    let constellation =
        Constellation::walker(pattern, epoch_mjd()).expect("the pattern is consistent");
    println!(
        "constellation: {} satellites in {} planes, altitude {:.1} km",
        constellation.len(),
        constellation.plane_count(),
        constellation
            .altitude_above(BODY_RADIUS_KM)
            .expect("the constellation is not empty")
    );
    assert_eq!(constellation.len(), 24);
    assert_eq!(constellation.plane_count(), 3);

    // Step 2 and 3: propagate the track and rotate it into the Earth-fixed
    // frame.
    let element = TwoLineElement::parse(TLE).expect("the sample element set parses");
    let satellite =
        Satellite::from_element(&element, GravityModel::WGS72).expect("the orbit propagates");
    let mut samples = Vec::new();
    for second in (0..=6_000).step_by(10) {
        let seconds = f64::from(second);
        samples.push(OrbitSample::new(
            seconds,
            position_ecef_km(&satellite, seconds),
        ));
    }
    // Every sample must be a plausible Earth-fixed position: between the
    // surface and a few thousand kilometers above it.
    for sample in &samples {
        let radius = sample.position_ecef_km.norm();
        assert!(
            (BODY_RADIUS_KM..BODY_RADIUS_KM + 2_000.0).contains(&radius),
            "the radius {radius} km is outside the expected range"
        );
    }
    println!(
        "propagated {} samples over {} s",
        samples.len(),
        samples.last().expect("non-empty").seconds
    );

    // Step 4: the access windows from a station placed at the sub-satellite
    // point of the first sample. Placing the station there rather than at a
    // fixed location makes the example depend only on the chain working,
    // not on the particular ground track of this element set, which crosses
    // a given longitude only every few orbits.
    let sub_satellite =
        apsis_coverage::sub_satellite_point(samples[0].position_ecef_km, apsis_coord::WGS84)
            .expect("the satellite is off the polar axis");
    let station = GroundPoint::new(sub_satellite.position, 10.0_f64.to_radians());
    let finder = AccessFinder {
        station,
        sample_step_seconds: 10.0,
        crossing_tolerance_seconds: 0.001,
    };
    let summary = finder.find(&samples).expect("the samples are usable");
    println!(
        "access: {} passes, {:.1} s total, {:.1}% of the window",
        summary.count(),
        summary.total_access_seconds(),
        summary.access_fraction().expect("a positive interval") * 100.0
    );
    // A 550 km orbit in a 53 degree inclination passes over the equator a
    // few times in a hundred minutes, so at least one pass must be found.
    assert!(summary.count() >= 1, "no pass was found");

    // Step 5: the inter-satellite links at one instant. The constellation
    // elements are used directly to place the satellites on a circle, which
    // is what the Walker construction guarantees, so the link geometry can
    // be tested without propagating every satellite.
    let mut topology = Topology::new(constellation.len());
    for first in 0..constellation.len() {
        for second in (first + 1)..constellation.len() {
            let first_elements = constellation.satellites()[first].elements;
            let second_elements = constellation.satellites()[second].elements;
            let first_position = elements_position(&first_elements);
            let second_position = elements_position(&second_elements);
            if mutual_visibility(first_position, second_position, BODY_RADIUS_KM) == Some(true) {
                // The cost is the range, so a shorter link is cheaper.
                let cost = first_position.distance(second_position);
                topology
                    .add_bidirectional_edge(first, second, cost)
                    .expect("the edge is new");
            }
        }
    }
    println!(
        "links: {} directed edges over {} satellites",
        topology.edge_count(),
        topology.node_count()
    );
    assert!(
        topology.edge_count() > 0,
        "no inter-satellite link was found"
    );

    // Step 6: routing and redundancy.
    let mesh = Mesh::new(topology.clone());
    let reachable = mesh.reachable_from(0).len();
    println!(
        "mesh: {} of {} satellites reachable from the first, {} components",
        reachable,
        mesh.node_count(),
        mesh.summarise().components
    );

    // Route between two satellites that are joined.
    let neighbour = topology
        .neighbours(0)
        .first()
        .copied()
        .expect("the first satellite has a neighbour");
    let route = shortest_path(&topology, 0, neighbour).expect("a route exists");
    println!(
        "route: 0 to {neighbour} in {} hop(s), cost {:.1} km",
        route.hop_count(),
        route.cost
    );
    assert_eq!(route.source(), Some(0));
    assert_eq!(route.destination(), Some(neighbour));

    // The same topology as a one sample series, which is what the series
    // type is for.
    let series = TopologySeries::new(vec![TopologySample::new(0.0, topology)])
        .expect("the series is not empty");
    assert_eq!(series.len(), 1);
    assert!(
        series
            .topology_at(0.0)
            .expect("a topology exists")
            .has_edge(0, neighbour)
    );

    // A cross-check of the frame conversion against the geodetic layer: the
    // propagated position must convert to a geodetic position at the
    // altitude the orbit layer implies, and the sub-satellite point derived
    // from it must sit at zero height.
    // The orbit layer works in kilometers while the coordinate layer works
    // in meters, so the conversion needs an explicit scale. Mixing the two
    // silently places the satellite inside the Earth.
    let propagated = position_ecef_km(&satellite, 0.0);
    let ecef = Ecef::new(propagated * 1_000.0);
    let geodetic = ecef
        .to_geodetic(apsis_coord::WGS84)
        .expect("the satellite is off the polar axis");
    assert!(
        (geodetic.height() - 550_000.0).abs() < 50_000.0,
        "the altitude {} m is not near the expected 550 km",
        geodetic.height()
    );
    assert!(geodetic.ellipsoidal().is_valid());
    assert_eq!(sub_satellite.position.height, 0.0);

    // The sub-satellite point must lie at the same latitude and longitude
    // as the satellite itself.
    assert!(
        (sub_satellite.position.latitude - geodetic.latitude()).abs() < 1e-9,
        "latitude {} against {}",
        sub_satellite.position.latitude,
        geodetic.latitude()
    );
    assert!(
        (sub_satellite.position.longitude - geodetic.longitude()).abs() < 1e-9,
        "longitude {} against {}",
        sub_satellite.position.longitude,
        geodetic.longitude()
    );
    // The epoch of the element set must round trip through the time layer.
    let epoch: Epoch = element.epoch;
    let calendar = epoch
        .calendar(TimeScale::Utc)
        .expect("the epoch is inside the leap second table");
    println!(
        "epoch: {}-{:02}-{:02} at {:.1} s",
        calendar.year, calendar.month, calendar.day, calendar.second
    );

    println!("integration check passed");
}

/// Returns the inertial position of a satellite from its Keplerian elements
/// at the true anomaly carried by the elements.
///
/// The Walker construction is circular, so the radius is the semi-major axis
/// and the position follows from the three orientation angles and the
/// anomaly.
fn elements_position(elements: &apsis_kepler::KeplerianElements) -> Vec3 {
    let radius = elements.semi_major_axis;
    let angle = elements.true_anomaly;
    // The position in the perifocal frame.
    let perifocal = Vec3::new(radius * angle.cos(), radius * angle.sin(), 0.0);
    let rotation = apsis_kepler::perifocal_to_inertial(
        elements.argument_of_perigee,
        elements.inclination,
        elements.right_ascension,
    );
    rotation * perifocal
}
