//! Scale benchmark for the paths a real workload hammers.
//!
//! The companion `bench` example measures the cost of a single call, which
//! answers whether an operation is fast. This one measures how that cost
//! grows with the size of the problem, which is the question that decides
//! whether a constellation sized run is feasible. It sweeps the input size
//! over powers of two and reports the time per unit of work, so a change in
//! the growth rate shows up even when the absolute numbers move with the
//! machine.
//!
//! The paths are the ones a planning cycle actually runs: propagating a
//! Walker constellation, searching access windows for one station over every
//! satellite, building the inter-satellite topology and routing through it,
//! and planning a task set with precedence.
//!
//! Everything is measured in release mode with a fixed iteration count, and
//! the harness warms up before timing so that lazy initialisation is not
//! charged to the first call.
//!
//! Usage:
//!
//! ```text
//! cargo run --release -p apsis-rs --features full --example bench_scale
//! ```

use apsis_access::{AccessFinder, OrbitSample};
use apsis_constellation::{Constellation, WalkerPattern};
use apsis_coord::Eci;
use apsis_coverage::GroundPoint;
use apsis_frames::EarthRotationAngle;
use apsis_geodesy::Geodetic;
use apsis_isl::mutual_visibility;
use apsis_math::Vec3;
use apsis_resource::{Resource, ResourceId, ResourceSet, Window};
use apsis_scheduler::Planner;
use apsis_sgp4::{GravityModel, Satellite};
use apsis_task::{PrecedenceGraph, Task, TaskId};
use apsis_tle::TwoLineElement;
use apsis_topology::Topology;
use std::hint::black_box;
use std::time::{Duration, Instant};

/// The element set used as the source of every propagated track.
const ISS: &str = concat!(
    "1 25544U 98067A   08264.51782528 -.00002182  00000-0 -11606-4 0  2927\r\n",
    "2 25544  51.6416 247.4627 0006703 130.5360 325.0288 15.72125391563537\r\n",
);

/// The body radius, in kilometers, taken from the shared constants.
const BODY_RADIUS_KM: f64 = apsis_core::constant::EARTH_EQUATORIAL_RADIUS;

/// The sample step of a propagated track, in seconds.
const TRACK_STEP_SECONDS: f64 = 60.0;

/// Runs a path a number of times and returns the total duration.
fn measure<F, T>(iterations: u32, mut path: F) -> Duration
where
    F: FnMut() -> T,
{
    // Two warm-up calls, so that any allocation or caching the first call
    // performs is not charged to the measurement.
    black_box(path());
    black_box(path());
    let start = Instant::now();
    for _ in 0..iterations {
        black_box(path());
    }
    start.elapsed()
}

/// Prints one row of the sweep table.
fn row(label: &str, size: usize, iterations: u32, total: Duration, per_unit_ns: f64) {
    println!(
        "{label:<28} {size:>6}  {iterations:>5}  {:>10.3}  {per_unit_ns:>14.1}",
        total.as_secs_f64() * 1e3,
    );
}

/// The header of the sweep table.
fn header(title: &str) {
    println!("\n{title}");
    println!(
        "{:<28} {:>6}  {:>5}  {:>10}  {:>14}",
        "path", "size", "runs", "total ms", "ns per unit"
    );
}

/// Returns a prepared satellite.
fn satellite() -> Satellite {
    let element = TwoLineElement::parse(ISS).expect("the sample parses");
    Satellite::from_element(&element, GravityModel::WGS72).expect("the orbit propagates")
}

/// The epoch of the sample element set, as a Modified Julian Date.
fn epoch_mjd() -> f64 {
    let element = TwoLineElement::parse(ISS).expect("the sample parses");
    element
        .epoch
        .mjd(apsis_time::TimeScale::Utc)
        .expect("the epoch is inside the leap second table")
}

/// Returns a track of Earth-fixed samples for one satellite.
fn track(satellite: &Satellite, samples: usize, epoch_mjd: f64) -> Vec<OrbitSample> {
    (0..samples)
        .map(|index| {
            let seconds = index as f64 * TRACK_STEP_SECONDS;
            let state = satellite
                .propagate_minutes(seconds / 60.0)
                .expect("the track propagates");
            let angle =
                EarthRotationAngle::from_ut1_days(epoch_mjd - 51_544.5 + seconds / 86_400.0);
            let fixed = Eci::new(state.position).to_ecef(angle).vector();
            OrbitSample::new(seconds, fixed)
        })
        .collect()
}

fn main() {
    println!("ApsisRS scale benchmark: release build, no external dependency");
    println!("growth is what matters; absolute times depend on the machine");

    let satellite = satellite();
    let epoch = epoch_mjd();

    // ---- One: sample a constellation onto a grid. ------------------------
    {
        header("one: sample N satellites onto a 60 s grid (180 min track)");
        for &(planes, per_plane) in &[(3_usize, 8_usize), (6, 11), (6, 40), (10, 40)] {
            let total = planes * per_plane;
            let samples = 181_usize; // 180 minutes at one minute.
            let pattern = WalkerPattern {
                total_satellites: total,
                planes,
                phase_factor: 1,
                inclination: 53.0_f64.to_radians(),
                semi_major_axis: BODY_RADIUS_KM + 550.0,
            };
            let iterations = if total <= 48 { 20 } else { 5 };
            let tracker = &satellite;
            let total_time = measure(iterations, || {
                let constellation =
                    Constellation::walker(pattern, epoch).expect("the pattern is consistent");
                // Propagate every satellite once per second of the track, at
                // the sample step, which is the work a coverage run does.
                let mut propagated = 0_usize;
                for _orbit in constellation.satellites() {
                    for index in 0..samples {
                        let seconds = index as f64 * TRACK_STEP_SECONDS;
                        let state = tracker
                            .propagate_minutes(seconds / 60.0)
                            .expect("the track propagates");
                        black_box(state.position);
                        propagated += 1;
                    }
                }
                propagated
            });
            row(
                "constellation propagate",
                total,
                iterations,
                total_time,
                total_time.as_nanos() as f64 / (iterations as f64 * (total * samples) as f64),
            );
        }
    }

    // ---- Two: access search over N tracks. ------------------------------
    {
        header("two: access search for one station over N tracks (2 h each)");
        for &satellites in &[8_usize, 32, 64, 128] {
            let tracks: Vec<Vec<OrbitSample>> = (0..satellites)
                .map(|_| track(&satellite, 120, epoch))
                .collect();
            let finder = AccessFinder {
                station: GroundPoint::new(Geodetic::new(0.0, 0.0, 0.0), 10.0_f64.to_radians()),
                sample_step_seconds: TRACK_STEP_SECONDS,
                crossing_tolerance_seconds: 0.01,
            };
            let iterations = if satellites <= 32 { 50 } else { 10 };
            let total_time = measure(iterations, || {
                let mut passes = 0_usize;
                for track in &tracks {
                    passes += finder.find(track).expect("the track is valid").count();
                }
                passes
            });
            row(
                "access search",
                satellites,
                iterations,
                total_time,
                total_time.as_nanos() as f64 / (iterations as f64 * satellites as f64),
            );
        }
    }

    // ---- Three: topology and routing over N satellites. ------------------
    {
        header("three: build topology over N satellites and route once");
        for &satellites in &[8_usize, 24, 64, 120, 300, 600] {
            // Positions spread on a ring so a fixed link range keeps a
            // bounded number of neighbours, as a real shell does.
            let positions: Vec<Vec3> = (0..satellites)
                .map(|index| {
                    let angle = index as f64 * 0.4;
                    Vec3::new(
                        (BODY_RADIUS_KM + 550.0) * angle.cos(),
                        (BODY_RADIUS_KM + 550.0) * angle.sin(),
                        index as f64 * 20.0,
                    )
                })
                .collect();
            let iterations = if satellites <= 24 {
                50
            } else if satellites <= 120 {
                10
            } else {
                2
            };
            let total_time = measure(iterations, || {
                let mut topology = Topology::new(satellites);
                for first in 0..satellites {
                    for second in 0..satellites {
                        if first == second {
                            continue;
                        }
                        if mutual_visibility(positions[first], positions[second], 4_500.0)
                            .is_some_and(|visible| visible)
                        {
                            let _ = topology.add_edge(first, second, 1.0);
                        }
                    }
                }
                let _ = apsis_routing::shortest_path(&topology, 0, satellites - 1);
                topology.edge_count()
            });
            row(
                "topology + route",
                satellites,
                iterations,
                total_time,
                total_time.as_nanos() as f64 / (iterations as f64 * satellites as f64),
            );
        }
    }

    // ---- Four: plan N tasks on one resource. -----------------------------
    {
        header("four: plan N tasks on one resource, with a 50 long chain");
        for &tasks in &[60_usize, 200, 500, 1_000, 2_000] {
            let mut resource = Resource::new(ResourceId::new(0), 1.0);
            resource
                .set_windows(&[Window::new(0.0, tasks as f64 * 200.0 + 10_000.0)])
                .expect("the window is usable");
            let resources = ResourceSet::from_resources(vec![resource]).expect("distinct");
            let set: Vec<Task> = (0..tasks)
                .map(|index| {
                    let start = (index % 50) as f64 * 200.0;
                    Task::new(TaskId::new(index), 60.0)
                        .with_window(start, start + 180.0)
                        .with_priority(1.0 + (index % 7) as f64)
                        .try_require(ResourceId::new(0), 1.0)
                        .expect("the resource is new")
                })
                .collect();
            let mut graph = PrecedenceGraph::new();
            for index in 1..tasks.min(50) {
                let _ = graph.add(TaskId::new(index - 1), TaskId::new(index));
            }
            let iterations = if tasks <= 200 { 20 } else { 5 };
            let planner = Planner::new();
            let total_time = measure(iterations, || {
                let result = planner.schedule_with_precedence(&set, &resources, &graph);
                result.schedule.len()
            });
            row(
                "plan tasks",
                tasks,
                iterations,
                total_time,
                total_time.as_nanos() as f64 / (iterations as f64 * tasks as f64),
            );
        }
    }

    println!();
    println!("ns per unit is the figure to watch: a rising trend means super-linear growth");
}
