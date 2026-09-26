//! Micro benchmarks for the hot paths, with no external dependency.
//!
//! Rust's built in benchmarking harness is nightly only and the usual
//! alternative is a third party crate, which this workspace forbids. This
//! example is the replacement: it times each path with the standard
//! library's monotonic clock and prints a table. It is meant to be run in
//! release mode, where the timings are meaningful:
//!
//! ```text
//! cargo run --release -p apsis-rs --features full --example bench
//! ```
//!
//! Each path is run enough times that the measured time is well above the
//! clock resolution, and the reported figure is the total divided by the
//! iteration count. The numbers are a guide to relative cost, not a
//! benchmark to compare across machines by absolute value.
//!
//! Only paths that a caller is likely to run in a loop are included:
//! propagation, ephemeris interpolation, access search, link budget, and
//! task planning. A path that runs once per program is not worth timing.

use apsis_access::{AccessFinder, OrbitSample};
use apsis_coord::{Ecef, Eci};
use apsis_coverage::GroundPoint;
use apsis_ephemeris::Sampler;
use apsis_frames::EarthRotationAngle;
use apsis_geodesy::Geodetic;
use apsis_kepler::{GravityParameter, KeplerianElements};
use apsis_link::{LinkBudget, LinkGeometry};
use apsis_noise::NoiseTemperature;
use apsis_resource::{Resource, ResourceId, ResourceSet, Window};
use apsis_rf::{Decibel, Gain, Power};
use apsis_scheduler::Planner;
use apsis_sgp4::{GravityModel, Satellite};
use apsis_task::{Task, TaskId};
use apsis_tle::TwoLineElement;
use std::hint::black_box;
use std::time::Instant;

/// The number of times each path is run.
///
/// The value is chosen so that every measurement is at least a few
/// milliseconds, which puts it far above the clock resolution while keeping
/// the whole run under a second per path.
const ITERATIONS: u32 = 20_000;

/// The element set used for the propagation and access paths.
const ISS: &str = concat!(
    "1 25544U 98067A   08264.51782528 -.00002182  00000-0 -11606-4 0  2927\r\n",
    "2 25544  51.6416 247.4627 0006703 130.5360 325.0288 15.72125391563537\r\n",
);

/// The number of seconds from the epoch the sample orbit starts at.
const EPOCH_MJD: f64 = 54_723.517_825_28;

/// Runs a path a number of times and returns the mean duration in
/// nanoseconds.
///
/// The closure runs once before the clock starts, so that any lazy
/// initialisation it does is not charged to the first iteration.
fn measure<F, T>(iterations: u32, mut path: F) -> f64
where
    F: FnMut() -> T,
{
    // Warm up.
    black_box(path());
    let start = Instant::now();
    for _ in 0..iterations {
        black_box(path());
    }
    let elapsed = start.elapsed();
    elapsed.as_nanos() as f64 / f64::from(iterations)
}

/// Prints one row of the table.
fn report(name: &str, nanos: f64, iterations: u32) {
    println!("{name:<44} {nanos:>12.1} ns  ({iterations} iterations)");
}

fn main() {
    println!("ApsisRS bench: mean time per call, release mode, no external dependency");
    println!("{:<44} {:>12}     (iterations)", "path", "mean");

    // ---- SGP4 propagation -------------------------------------------------
    let element = TwoLineElement::parse(ISS).expect("the sample parses");
    let satellite =
        Satellite::from_element(&element, GravityModel::WGS72).expect("the orbit propagates");
    let nanos = measure(ITERATIONS, || {
        satellite
            .propagate_minutes(black_box(30.0))
            .expect("propagates")
    });
    report("sgp4 propagate (30 min from epoch)", nanos, ITERATIONS);

    // ---- Two-body Keplerian conversion ------------------------------------
    let elements = KeplerianElements {
        semi_major_axis: 7_000.0,
        eccentricity: 0.01,
        inclination: 0.9,
        right_ascension: 0.5,
        argument_of_perigee: 1.0,
        true_anomaly: 0.3,
    };
    let nanos = measure(ITERATIONS, || {
        elements
            .to_state(black_box(GravityParameter::WGS84))
            .expect("valid")
    });
    report("kepler elements to state", nanos, ITERATIONS);

    // ---- Hermite interpolation --------------------------------------------
    let sampler = Sampler::new(0.0, 600.0, 60.0);
    let ephemeris = sampler
        .sample(|seconds| {
            let state = satellite
                .propagate_minutes(seconds / 60.0)
                .expect("propagates");
            apsis_ephemeris::Sample::new(seconds, state.position, state.velocity)
        })
        .expect("the grid is usable");
    let nanos = measure(ITERATIONS, || {
        ephemeris.state_at(black_box(137.5)).expect("covered")
    });
    report("ephemeris Hermite query", nanos, ITERATIONS);

    let nanos = measure(ITERATIONS, || {
        satellite
            .propagate_minutes(black_box(137.5) / 60.0)
            .expect("propagates")
    });
    report("sgp4 propagate (comparison query)", nanos, ITERATIONS);

    // ---- Earth-fixed rotation --------------------------------------------
    let angle = EarthRotationAngle::from_ut1_days(EPOCH_MJD - 51_544.5);
    let eci = Eci::new(apsis_math::Vec3::new(7_000.0, 0.0, 0.0));
    let nanos = measure(ITERATIONS, || eci.to_ecef(black_box(angle)).vector());
    report("TEME to Earth-fixed rotation", nanos, ITERATIONS);

    // ---- Geodetic conversion ---------------------------------------------
    let ecef = Ecef::new(apsis_math::Vec3::new(6_878_137.0, 0.0, 0.0));
    let nanos = measure(ITERATIONS, || {
        ecef.to_geodetic(black_box(apsis_coord::WGS84))
            .expect("off axis")
    });
    report("Earth-fixed to geodetic", nanos, ITERATIONS);

    // ---- Access search ----------------------------------------------------
    // A minute of samples of a low Earth orbit, which is the input an access
    // search consumes.
    let mut samples = Vec::with_capacity(120);
    for index in 0..120 {
        let seconds = f64::from(index) * 30.0;
        let state = satellite
            .propagate_minutes(seconds / 60.0)
            .expect("propagates");
        let angle = EarthRotationAngle::from_ut1_days(EPOCH_MJD - 51_544.5 + seconds / 86_400.0);
        let fixed = Eci::new(state.position).to_ecef(angle).vector();
        samples.push(OrbitSample::new(seconds, fixed));
    }
    let finder = AccessFinder {
        station: GroundPoint::new(Geodetic::new(0.0, 0.0, 0.0), 10.0_f64.to_radians()),
        sample_step_seconds: 30.0,
        crossing_tolerance_seconds: 0.01,
    };
    let nanos = measure(2_000, || finder.find(black_box(&samples)));
    report("access window search (120 samples)", nanos, 2_000);

    // ---- Link budget ------------------------------------------------------
    let budget = LinkBudget {
        transmit_power: Power::from_dbw(10.0).expect("finite"),
        transmit_gain: Gain::from_dbi(30.0).expect("finite"),
        transmit_loss: Decibel::from_db(1.0),
        receive_gain: Gain::from_dbi(35.0).expect("finite"),
        receive_loss: Decibel::from_db(0.5),
        system_temperature: NoiseTemperature::from_kelvin(150.0),
        bandwidth: 36.0e6,
        geometry: LinkGeometry {
            distance: 38.0e6,
            frequency: 12.0e9,
        },
    };
    let nanos = measure(ITERATIONS, || {
        budget.carrier_to_noise_density().expect("usable")
    });
    report("link budget carrier to noise", nanos, ITERATIONS);

    // ---- Task planning ----------------------------------------------------
    // One station and sixty tasks, which is a small but realistic planning
    // cycle. The whole plan is timed rather than one placement, because a
    // caller runs the planner once per cycle.
    let mut station = Resource::new(ResourceId::new(0), 1.0);
    station
        .set_windows(&[Window::new(0.0, 6_000.0)])
        .expect("usable");
    let resources = ResourceSet::from_resources(vec![station]).expect("distinct");
    let tasks: Vec<Task> = (0..60)
        .map(|index| {
            let start = f64::from(index) * 100.0;
            Task::new(TaskId::new(index as usize), 60.0)
                .with_window(start, start + 90.0)
                .with_priority(1.0 + f64::from(index % 5))
                .try_require(ResourceId::new(0), 1.0)
                .expect("the resource is new")
        })
        .collect();
    let planner = Planner::new();
    let nanos = measure(500, || planner.schedule(black_box(&tasks), &resources));
    report("plan 60 tasks on 1 resource", nanos, 500);

    println!();
    println!("figures are means over the stated iterations; compare paths, not machines");
}
