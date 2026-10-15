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
//! 1. Build a Walker constellation with `apsis_constellation`.
//! 2. Propagate one satellite with `apsis_sgp4` to get a state vector in the
//!    True Equator Mean Equinox frame.
//! 3. Rotate that state into the Earth-fixed frame with `apsis_coord` and
//!    the Earth rotation angle from `apsis_frames`.
//! 4. Search for access windows from a ground station with
//!    `apsis_access`.
//! 5. Build the time series of inter-satellite links with
//!    `apsis_topology` and `apsis_isl`.
//! 6. Route between two terminals with `apsis_routing` and measure the
//!    redundancy of the mesh with `apsis_mesh`.
//!
//! Usage:
//!
//! ```text
//! cargo run --features network --example integrate_network
//! ```
//!
//! 跨层集成检查：星座到可见性窗口。
//!
//! 本示例走完本工作区所提供的整条链，并断言其中每一环都为下一环产生合理的输入。
//! 它不是对照外部参考的数值交叉核对；逐层交叉核对做那件事。它的目的是在某层的
//! 改动破坏另一层的假设时失败。
//!
//! 该链是：
//!
//! 1. 用 `apsis_constellation` 构建 Walker 星座。
//! 2. 用 `apsis_sgp4` 传播一颗卫星，得到真赤道平春分点坐标系中的状态矢量。
//! 3. 用 `apsis_coord` 与来自 `apsis_frames` 的地球自转角，把该状态转入地固系。
//! 4. 用 `apsis_access` 从地面站搜索可见性窗口。
//! 5. 用 `apsis_topology` 与 `apsis_isl` 构建星间链路的时间序列。
//! 6. 用 `apsis_routing` 在两个终端之间路由，并用 `apsis_mesh` 度量网格的
//!    冗余度。

use apsis::apsis_access::{AccessFinder, OrbitSample};
use apsis::apsis_constellation::{Constellation, WalkerPattern};
use apsis::apsis_coord::{Ecef, Eci};
use apsis::apsis_coverage::GroundPoint;
use apsis::apsis_frames::EarthRotationAngle;
use apsis::apsis_isl::mutual_visibility;
use apsis::apsis_math::Vec3;
use apsis::apsis_mesh::Mesh;
use apsis::apsis_routing::shortest_path;
use apsis::apsis_sgp4::{GravityModel, Satellite};
use apsis::apsis_time::{Epoch, TimeScale};
use apsis::apsis_tle::TwoLineElement;
use apsis::apsis_topology::{Topology, TopologySample, TopologySeries};

/// The Earth equatorial radius, in kilometers, used for the link geometry.
///
/// The value comes from the shared constants rather than being repeated.
///
/// 链路几何所用的地球赤道半径，单位为千米。
///
/// 该值来自共享常量而非重复书写。
const BODY_RADIUS_KM: f64 = apsis::apsis_core::constant::EARTH_EQUATORIAL_RADIUS;

/// The element set of a satellite in a 550 km orbit, used as the source of
/// the propagated track.
///
/// 一个 550 km 轨道卫星的根数集，用作所传播轨迹的来源。
const TLE: &str = concat!(
    "1 25544U 98067A   08264.51782528 -.00002182  00000-0 -11606-4 0  2927\r\n",
    "2 25544  51.6416 247.4627 0006703 130.5360 325.0288 15.72125391563537\r\n",
);

/// Returns the time of the element set epoch as a Modified Julian Date.
///
/// 把根数集历元的时间返回为约化儒略日。
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
///
/// 返回自根数集历元起若干秒处地固系中的卫星位置。
///
/// SGP4 状态在真赤道平春分点坐标系中，此处把它当作转动所用的惯性系。这是一个
/// 近似：二者相差分点方程式，不到千分之一度。对集成检查它足够，而把它换成精确
/// 变换是加入岁差与章动的坐标里程碑的事。
fn position_ecef_km(satellite: &Satellite, seconds: f64) -> Vec3 {
    let state = satellite
        .propagate_minutes(seconds / 60.0)
        .expect("the sample orbit propagates");
    let inertial = Eci::new(state.position);
    // The Earth rotation angle needs the UT1 time. The difference between
    // UT1 and TAI is under a second, which is under 0.005 degrees of
    // rotation, so the TAI epoch is used here.
    // 地球自转角需要 UT1 时间。UT1 与 TAI 之差不到一秒，即不到 0.005 度的转动，
    // 故此处使用 TAI 历元。
    let days_since_j2000 = (epoch_mjd() + seconds / 86_400.0) - 51_544.5;
    let angle = EarthRotationAngle::from_ut1_days(days_since_j2000);
    inertial.to_ecef(angle).vector()
}

fn main() {
    // Step 1: the constellation geometry.
    // 第 1 步：星座几何。
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
    // 第 2 与第 3 步：传播轨迹并把它转入地固系。
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
    // 每个样本都必须是合理的地固位置：在地表与其上方数千米之间。
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
    // 第 4 步：来自一个被放在第一个样本星下点的地面站的可见性窗口。把站放在那里
    // 而非固定位置，使本示例只依赖于该链正常工作，而不依赖于该根数集特定的地面轨迹
    // ——它每隔几个轨道才穿过某一给定经度。
    let sub_satellite = apsis::apsis_coverage::sub_satellite_point(
        samples[0].position_ecef_km,
        apsis::apsis_coord::WGS84,
    )
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
    // 一个 53 度倾角的 550 km 轨道在一百分钟里数次经过赤道上空，故必须找到至少
    // 一次过境。
    assert!(summary.count() >= 1, "no pass was found");

    // Step 5: the inter-satellite links at one instant. The constellation
    // elements are used directly to place the satellites on a circle, which
    // is what the Walker construction guarantees, so the link geometry can
    // be tested without propagating every satellite.
    // 第 5 步：某一瞬时的星间链路。星座根数被直接用于把卫星放在一个圆上，这是
    // Walker 构建所保证的，故无需传播每颗卫星即可检验链路几何。
    let mut topology = Topology::new(constellation.len());
    for first in 0..constellation.len() {
        for second in (first + 1)..constellation.len() {
            let first_elements = constellation.satellites()[first].elements;
            let second_elements = constellation.satellites()[second].elements;
            let first_position = elements_position(&first_elements);
            let second_position = elements_position(&second_elements);
            if mutual_visibility(first_position, second_position, BODY_RADIUS_KM) == Some(true) {
                // The cost is the range, so a shorter link is cheaper.
                // 代价即距离，故更短的链路更便宜。
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
    // 第 6 步：路由与冗余。
    let mesh = Mesh::new(topology.clone());
    let reachable = mesh.reachable_from(0).len();
    println!(
        "mesh: {} of {} satellites reachable from the first, {} components",
        reachable,
        mesh.node_count(),
        mesh.summarise().components
    );

    // Route between two satellites that are joined.
    // 在两个相连的卫星之间路由。
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
    // 把同一拓扑作为单样本序列，这正是序列类型的用途。
    let series = TopologySeries::new(vec![TopologySample::new(0.0, topology)])
        .expect("the series is not empty");
    assert_eq!(series.len(), 1);
    assert!(
        series
            .topology_at(0.0)
            .expect("a topology exists")
            .has_edge(0, neighbour)
    );

    // A cross-check of the frame conversion against the geodetic layer. Two
    // things must hold.
    // 把坐标架变换与大地层交叉核对。必须成立两件事。
    //
    // First, the rotation into the Earth-fixed frame preserves the radius,
    // so the geocentric radius of the converted position equals the radius
    // the propagator reports. This is the property that catches a unit
    // mistake: the orbit layer works in kilometers and the coordinate layer
    // in meters, so mixing the two silently places the satellite inside the
    // Earth.
    // 首先，转入地固系的转动保持半径，故转换后位置的地心半径等于传播器所报告的
    // 半径。这正是能捕获单位错误的性质：orbit 层以千米工作，coord 层以米工作，把
    // 二者混用会无声地把卫星放进地球内部。
    let propagated = position_ecef_km(&satellite, 0.0);
    let state = satellite
        .propagate_minutes(0.0)
        .expect("the sample orbit propagates");
    let propagated_radius_km = state.altitude_radius();
    assert!(
        (propagated.norm() - propagated_radius_km).abs() < 1e-6,
        "the converted radius {} km does not match the propagated {} km",
        propagated.norm(),
        propagated_radius_km
    );

    // Second, the geodetic height agrees with the radius to within the
    // flattening. A point at geocentric radius `r` has a geodetic height
    // between `r - a` and `r - b`, where `a` and `b` are the equatorial and
    // polar radii: the difference is at most the flattening times the
    // equatorial radius, about twenty one kilometers. Bracketing the height
    // tests the conversion without assuming an altitude the element set does
    // not have.
    // 其次，大地高度与半径之差在扁率以内。地心半径为 `r` 的点，其大地高度介于
    // `r - a` 与 `r - b` 之间，其中 `a` 与 `b` 是赤道与极半径：差值至多为扁率乘以
    // 赤道半径，约二十一千米。用区间夹住高度即可检验换算，而不假定根数集所没有的
    // 高度。
    let ecef = Ecef::new(propagated * 1_000.0);
    let geodetic = ecef
        .to_geodetic(apsis::apsis_coord::WGS84)
        .expect("the satellite is off the polar axis");
    let radius_m = propagated_radius_km * 1_000.0;
    let lower = radius_m - apsis::apsis_coord::WGS84.semi_major_axis();
    let upper = radius_m - apsis::apsis_coord::WGS84.semi_minor_axis();
    assert!(
        (lower - 1.0..=upper + 1.0).contains(&geodetic.height()),
        "the geodetic height {} m is outside [{lower:.1}, {upper:.1}]",
        geodetic.height()
    );
    assert!(geodetic.ellipsoidal().is_valid());
    assert_eq!(sub_satellite.position.height, 0.0);

    // The sub-satellite point must lie at the same latitude and longitude
    // as the satellite itself.
    // 星下点必须位于与卫星本身相同的纬度与经度。
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
    // 根数集的历元必须经时间层往返一致。
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
///
/// 由卫星的开普勒根数、在其所携带的真近点角处返回惯性位置。
///
/// Walker 构建是圆形的，故半径即半长轴，位置由三个取向角与该近点角得出。
fn elements_position(elements: &apsis::apsis_kepler::KeplerianElements) -> Vec3 {
    let radius = elements.semi_major_axis;
    let angle = elements.true_anomaly;
    // The position in the perifocal frame.
    // 近焦点系中的位置。
    let perifocal = Vec3::new(radius * angle.cos(), radius * angle.sin(), 0.0);
    let rotation = apsis::apsis_kepler::perifocal_to_inertial(
        elements.argument_of_perigee,
        elements.inclination,
        elements.right_ascension,
    );
    rotation * perifocal
}
