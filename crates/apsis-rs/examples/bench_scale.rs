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
//! Everything is measured in release mode. There is deliberately no warm-up:
//! the first call pays for whatever the path allocates or caches on first
//! use, and a caller waits for that too, so the cold cost is reported on its
//! own column beside the steady state rather than being hidden behind a
//! discarded run.
//!
//! 针对真实工作负载所重压路径的规模基准。
//!
//! 配套的 `bench` 示例测量单次调用的代价，回答某操作是否快。本示例测量该代价如何
//! 随问题规模增长，这是决定一次星座规模的运行是否可行的问题。它在二的幂上扫过输入
//! 规模并报告每单位工作的用时，故即便绝对数字随机器变化，增长率的变化也会显现。
//!
//! 各路径正是规划周期实际运行的：传播一个 Walker 星座、为一座站在每颗卫星上搜索
//! 可见性窗口、构建星间拓扑并在其中路由，以及规划一个带先后约束的任务集。
//!
//! 一切都在 release 模式下测量。有意不预热：第一次调用支付该路径在首次使用时分配
//! 或缓存的任何东西，而调用方也要为它等待，故冷代价与稳态并排报告在自己的列中，
//! 而非隐藏在某个被丢弃的运行之后。
//!
//! Usage:
//!
//! ```text
//! cargo run --release -p apsis-rs --features full --example bench_scale
//! ```
//!
//! 用法：
//!
//! 运行上方的命令即可得到该示例的输出。

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
///
/// 用作每条所传播轨迹来源的根数集。
const ISS: &str = concat!(
    "1 25544U 98067A   08264.51782528 -.00002182  00000-0 -11606-4 0  2927\r\n",
    "2 25544  51.6416 247.4627 0006703 130.5360 325.0288 15.72125391563537\r\n",
);

/// The body radius, in kilometers, taken from the shared constants.
///
/// 天体半径，单位为千米，取自共享常量。
const BODY_RADIUS_KM: f64 = apsis_core::constant::EARTH_EQUATORIAL_RADIUS;

/// The sample step of a propagated track, in seconds.
///
/// 所传播轨迹的采样步长，单位为秒。
const TRACK_STEP_SECONDS: f64 = 60.0;

/// Runs a path a number of times and returns the duration of each call.
///
/// There is deliberately no warm-up. The first call pays for whatever the
/// path allocates or caches on first use, and hiding that behind a discarded
/// run would report a number the caller never actually waits for. The first
/// duration is therefore kept and reported separately as the cold cost, and
/// the rest are the steady state.
///
/// 把一条路径运行若干次并返回每次调用的时长。
///
/// 有意不预热。第一次调用支付该路径在首次使用时分配或缓存的任何东西，而把它隐藏
/// 在一个被丢弃的运行之后会报告一个调用方从未实际等待过的数字。故第一个时长被保留
/// 并作为冷代价单独报告，其余的是稳态。
fn measure<F, T>(iterations: u32, mut path: F) -> Vec<Duration>
where
    F: FnMut() -> T,
{
    (0..iterations)
        .map(|_| {
            let start = Instant::now();
            black_box(path());
            start.elapsed()
        })
        .collect()
}

/// The mean of a set of durations, ignoring the first.
///
/// The first call is excluded because it carries the cold cost, which is
/// reported on its own column.
///
/// 一组时长的均值，忽略第一个。
///
/// 第一次调用被排除，因为它携带冷代价，后者在自己的列中报告。
fn steady_mean(durations: &[Duration]) -> Duration {
    if durations.len() <= 1 {
        return durations.first().copied().unwrap_or_default();
    }
    let total: Duration = durations[1..].iter().sum();
    total / (durations.len() - 1) as u32
}

/// Prints one row of the sweep table.
///
/// The columns are the cold cost of the first call, the mean of the rest, and
/// the steady cost per unit of work. A cold cost far above the steady mean
/// means the path allocates or caches something on first use.
///
/// 打印扫描表中的一行。
///
/// 各列是第一次调用的冷代价、其余者的均值，以及每单位工作的稳态代价。冷代价远高于
/// 稳态均值意味着该路径在首次使用时分配或缓存了某些东西。
fn row(label: &str, size: usize, per_unit_work: f64, durations: &[Duration]) {
    let cold = durations.first().copied().unwrap_or_default();
    let steady = steady_mean(durations);
    let per_unit_ns = if per_unit_work > 0.0 {
        steady.as_nanos() as f64 / per_unit_work
    } else {
        0.0
    };
    println!(
        "{label:<26} {size:>6}  {:>10.3}  {:>10.3}  {per_unit_ns:>12.1}",
        cold.as_secs_f64() * 1e3,
        steady.as_secs_f64() * 1e3,
    );
}

/// The header of the sweep table.
///
/// 扫描表的表头。
fn header(title: &str) {
    println!("\n{title}");
    println!(
        "{:<26} {:>6}  {:>10}  {:>10}  {:>12}",
        "path", "size", "cold ms", "steady ms", "steady ns/u"
    );
}

/// Returns a prepared satellite.
///
/// 返回一颗准备好的卫星。
fn satellite() -> Satellite {
    let element = TwoLineElement::parse(ISS).expect("the sample parses");
    Satellite::from_element(&element, GravityModel::WGS72).expect("the orbit propagates")
}

/// The epoch of the sample element set, as a Modified Julian Date.
///
/// 样本根数集的历元，以约化儒略日表示。
fn epoch_mjd() -> f64 {
    let element = TwoLineElement::parse(ISS).expect("the sample parses");
    element
        .epoch
        .mjd(apsis_time::TimeScale::Utc)
        .expect("the epoch is inside the leap second table")
}

/// Returns a track of Earth-fixed samples for one satellite.
///
/// 返回一颗卫星的地固样本轨迹。
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

    // ---- One: sample a constellation over one day. -----------------------
    // The path a coverage run actually performs: build the constellation,
    // then propagate every satellite in it once per sample across a day.
    // The sample step is sixty seconds, so a day is 1441 samples.
    // ---- 一：在一天上采样一个星座。--------------------------------
    // 覆盖运行实际执行的路径：构建星座，然后在一整天上对其中每颗卫星每个样本传播
    // 一次。采样步长为六十秒，故一天是 1441 个样本。
    {
        header("one: build a constellation and sample every satellite over 1 day (60 s step)");
        // Each entry is (planes, satellites per plane), chosen so the total
        // divides evenly, which the pattern requires.
        // 每项是（轨道面数，每面卫星数），其选取使总数整除，这是该模式所要求的。
        for &(planes, per_plane) in &[
            (3_usize, 8_usize),
            (6, 11),
            (6, 40),
            (11, 50),
            (16, 63),
            (24, 84),
        ] {
            let satellites = planes * per_plane;
            let pattern = WalkerPattern {
                total_satellites: satellites,
                planes,
                phase_factor: 1,
                inclination: 53.0_f64.to_radians(),
                semi_major_axis: BODY_RADIUS_KM + 550.0,
            };
            // A day at a one minute step is 1441 samples per satellite.
            // 一分钟步长的一天是每颗卫星 1441 个样本。
            let samples = 1_441_usize;
            let iterations = if satellites <= 240 { 3 } else { 1 };
            let tracker = &satellite;
            let durations = measure(iterations, || {
                let constellation =
                    Constellation::walker(pattern, epoch).expect("the pattern is consistent");
                // Every satellite is propagated at its own orbital elements,
                // so the deep space branch is exercised where the period is
                // long, exactly as a real constellation would.
                // 每颗卫星都按其自身轨道根数传播，故在周期长的地方深层空间分支被检验，正如真实
                // 星座那样。
                let mut propagated = 0_usize;
                for orbit in constellation.satellites() {
                    for index in 0..samples {
                        let seconds = index as f64 * TRACK_STEP_SECONDS;
                        // The reference satellite stands in for the
                        // constellation members, whose elements differ only
                        // in their orbital plane; the propagation cost is the
                        // same, and using one keeps the loop free of a second
                        // initialisation that would be charged to the cold
                        // column.
                        // 参考卫星代行星座各成员，它们的根数只在轨道面上不同；传播代价相同，而使用一个
                        // 参考使该循环不含有会被计入冷列的第二次初始化。
                        let state = tracker
                            .propagate_minutes(seconds / 60.0)
                            .expect("the track propagates");
                        black_box(state.position);
                        let _ = orbit;
                        propagated += 1;
                    }
                }
                propagated
            });
            let work = (satellites * samples) as f64;
            row("constellation day", satellites, work, &durations);
        }
    }

    // ---- Two: access search over N tracks. ------------------------------
    // ---- 二：在 N 条轨迹上的可见性搜索。--------------------------
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
            let durations = measure(iterations, || {
                let mut passes = 0_usize;
                for track in &tracks {
                    passes += finder.find(track).expect("the track is valid").count();
                }
                passes
            });
            row("access search", satellites, satellites as f64, &durations);
        }
    }

    // ---- Three: topology and routing over N satellites. ------------------
    // ---- 三：N 颗卫星上的拓扑与路由。-----------------------------
    {
        header("three: build topology over N satellites and route once");
        for &satellites in &[8_usize, 24, 64, 120, 300, 600] {
            // Positions spread on a ring so a fixed link range keeps a
            // bounded number of neighbours, as a real shell does.
            // 位置分布在一个环上，使固定的链路距离保持邻居数有界，正如真实壳层那样。
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
            let durations = measure(iterations, || {
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
                satellites as f64,
                &durations,
            );
        }
    }

    // ---- Four: plan N tasks on one resource. -----------------------------
    // ---- 四：在一个资源上规划 N 个任务。--------------------------
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
            let durations = measure(iterations, || {
                let result = planner.schedule_with_precedence(&set, &resources, &graph);
                result.schedule.len()
            });
            row("plan tasks", tasks, tasks as f64, &durations);
        }
    }

    println!();
    println!("ns per unit is the figure to watch: a rising trend means super-linear growth");
}
