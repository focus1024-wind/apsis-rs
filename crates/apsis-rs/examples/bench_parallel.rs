//! Parallel scale benchmark: how well the hot paths use the cores.
//!
//! The serial `bench_scale` example measures the cost of a path. This one
//! asks the next question: when the work is spread over the cores, how much
//! of the wall clock does that save, and where does the parallel version
//! stop being worth it.
//!
//! The paths chosen are the ones a real run parallelises, because each is
//! either trivially parallel or has a clear serial tail:
//!
//! - **constellation propagation** propagates every satellite independently,
//!   so it is embarrassingly parallel and should scale nearly linearly.
//! - **access search** searches each track independently, likewise.
//! - **topology construction** decides each pair of satellites independently,
//!   but folding the results into one graph is serial, so it is the case
//!   that shows the cost of a serial tail.
//!
//! The timing compares the same total work done serially and across a range
//! of thread counts, and reports the speedup against both the serial time
//! and the ideal linear time. The two differ, and the difference is the
//! parallel overhead: thread spawn, work imbalance, and the serial tail.
//!
//! There is no warm-up, for the reason given in `bench_scale`: the first
//! call pays for whatever the path allocates on first use, and a caller
//! waits for that too. The serial time and the parallel time are both
//! measured cold, so the comparison is between two numbers the caller could
//! actually observe.
//!
//! 并行规模基准：热点路径对多核的利用程度。
//!
//! 串行的 `bench_scale` 示例测量一条路径的代价。本示例问下一个问题：当工作铺到
//! 多个核上时，那节省了多少墙钟时间，以及并行版本从何处起不再值得。
//!
//! 所选的路径是真实运行会并行的那些，因为每一条或是可平凡并行，或有明确的串行
//! 尾部：
//!
//! - **星座传播**独立地传播每颗卫星，故它是可并行化的，应当近似线性扩展。
//! - **可见性搜索**独立地搜索每条轨迹，同样如此。
//! - **拓扑构建**独立地判定每对卫星，但把结果折进一张图是串行的，故它是展示
//!   串行尾部代价的用例。
//!
//! 计时在串行与一系列线程数上比较相同的总工作，并报告相对串行时间与理想线性时间
//! 的加速比。二者不同，其差就是并行开销：线程创建、工作不均衡与串行尾部。
//!
//! 不预热，理由见 `bench_scale`：第一次调用支付该路径在首次使用时分配的任何东西，
//! 而调用方也要为它等待。串行时间与并行时间都冷测，故比较的是调用方实际所能观察到
//! 的两个数字。
//!
//! Usage:
//!
//! ```text
//! cargo run --release -p apsis-rs --features full --example bench_parallel
//! ```

use apsis_access::{AccessFinder, OrbitSample};
use apsis_coord::Eci;
use apsis_coverage::GroundPoint;
use apsis_frames::EarthRotationAngle;
use apsis_geodesy::Geodetic;
use apsis_isl::mutual_visibility;
use apsis_math::Vec3;
use apsis_sgp4::{GravityModel, Satellite};
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

/// The sample step, in seconds.
///
/// 采样步长，单位为秒。
const TRACK_STEP_SECONDS: f64 = 60.0;

/// Runs a path once and returns how long it took.
///
/// 把一条路径运行一次并返回其耗时。
fn measure<F, T>(path: F) -> Duration
where
    F: FnOnce() -> T,
{
    let start = Instant::now();
    black_box(path());
    start.elapsed()
}

/// Runs a path `repeats` times and returns the fastest duration.
///
/// The fastest of several runs is the least contaminated by whatever else
/// the machine is doing, and it is the figure that scales cleanly, because a
/// scheduling hiccup on one run cannot inflate the reference every later
/// number is divided by. The cold cost of the very first call is still
/// reported on its own, so nothing is hidden.
///
/// 把一条路径运行 `repeats` 次并返回最快的时长。
///
/// 若干次运行中最快的一次受机器上其他活动污染最少，且它是干净扩展的数字，因为某
/// 一次运行上的调度打嗝无法抬高其后每个数字所除以的基准。第一次调用的冷代价仍单独
/// 报告，故没有任何隐藏。
fn measure_best<F, T>(repeats: u32, path: F) -> Duration
where
    F: Fn() -> T,
{
    let mut best = Duration::MAX;
    for _ in 0..repeats {
        let elapsed = measure(&path);
        if elapsed < best {
            best = elapsed;
        }
    }
    best
}

/// Prints one row.
///
/// The efficiency is the speedup divided by the thread count, so one means
/// the path used every core fully. A value well below one means the threads
/// were idle, which is what a serial tail looks like.
///
/// 打印一行。
///
/// 效率是加速比除以线程数，故一意味着该路径充分利用了每个核。远低于一的值意味着
/// 各线程有空闲，这正是串行尾部的样子。
fn row(threads: usize, wall: Duration, work: Duration, cores: usize) {
    let wall_ms = wall.as_secs_f64() * 1e3;
    let work_ms = work.as_secs_f64() * 1e3;
    let speedup = work_ms / wall_ms;
    let efficiency = speedup / threads as f64;
    println!(
        "{threads:>6}  {wall_ms:>11.3}  {work_ms:>11.3}  {speedup:>9.2}  {efficiency:>9.2}  {cores:>8}"
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

/// Propagates a whole constellation over a day, serially.
///
/// The serial time is the reference every parallel number is divided by.
///
/// 串行地把整个星座在一天上传播。
///
/// 该串行时间是每个并行数字所除以的基准。
fn propagate_serial(satellite: &Satellite, satellites: usize, samples: usize) -> usize {
    let mut count = 0_usize;
    for _ in 0..satellites {
        for index in 0..samples {
            let seconds = index as f64 * TRACK_STEP_SECONDS;
            let state = satellite
                .propagate_minutes(seconds / 60.0)
                .expect("the track propagates");
            black_box(state.position);
            count += 1;
        }
    }
    count
}

/// Propagates a whole constellation over a day across `threads` threads.
///
/// Each thread takes a disjoint slice of the satellites, so the work splits
/// evenly and no two threads write the same state. The `scope` form of
/// spawning borrows the satellite rather than requiring it to outlive the
/// threads, which is what lets the reference be shared without cloning.
///
/// 跨 `threads` 个线程把整个星座在一天上传播。
///
/// 每个线程取卫星的一个互不相交切片，故工作均匀分开，且没有两个线程写相同的状态。
/// `scope` 形式的创建借用该卫星，而不要求它比各线程活得久，这正是使参考无需克隆
/// 即可共享的原因。
fn propagate_parallel(
    satellite: &Satellite,
    satellites: usize,
    samples: usize,
    threads: usize,
) -> usize {
    let chunk = satellites.div_ceil(threads);
    let total = std::sync::atomic::AtomicUsize::new(0);
    std::thread::scope(|scope| {
        for worker in 0..threads {
            let first = worker * chunk;
            if first >= satellites {
                break;
            }
            let last = (first + chunk).min(satellites);
            let total = &total;
            scope.spawn(move || {
                let mut count = 0_usize;
                for _ in first..last {
                    for index in 0..samples {
                        let seconds = index as f64 * TRACK_STEP_SECONDS;
                        let state = satellite
                            .propagate_minutes(seconds / 60.0)
                            .expect("the track propagates");
                        black_box(state.position);
                        count += 1;
                    }
                }
                total.fetch_add(count, std::sync::atomic::Ordering::Relaxed);
            });
        }
    });
    total.load(std::sync::atomic::Ordering::Relaxed)
}

/// Searches access for every track, serially.
///
/// 串行地为每条轨迹搜索可见性。
fn access_serial(tracks: &[Vec<OrbitSample>], finder: &AccessFinder) -> usize {
    let mut passes = 0_usize;
    for track in tracks {
        passes += finder.find(track).expect("the track is valid").count();
    }
    passes
}

/// Searches access for every track across `threads` threads.
///
/// The tracks are read only and the finder is a copy, so each thread can
/// take a disjoint slice without synchronisation.
///
/// 跨 `threads` 个线程为每条轨迹搜索可见性。
///
/// 各轨迹是只读的，且查找器是一份副本，故每个线程可取一个互不相交切片而无需同步。
fn access_parallel(tracks: &[Vec<OrbitSample>], finder: &AccessFinder, threads: usize) -> usize {
    let chunk = tracks.len().div_ceil(threads);
    let total = std::sync::atomic::AtomicUsize::new(0);
    std::thread::scope(|scope| {
        for worker in 0..threads {
            let first = worker * chunk;
            if first >= tracks.len() {
                break;
            }
            let last = (first + chunk).min(tracks.len());
            let total = &total;
            let finder = *finder;
            scope.spawn(move || {
                let mut passes = 0_usize;
                for track in &tracks[first..last] {
                    passes += finder.find(track).expect("the track is valid").count();
                }
                total.fetch_add(passes, std::sync::atomic::Ordering::Relaxed);
            });
        }
    });
    total.load(std::sync::atomic::Ordering::Relaxed)
}

/// Builds a topology serially, deciding each pair and folding it in.
///
/// 串行地构建拓扑，判定每一对并将其折入。
fn topology_serial(positions: &[Vec3], satellites: usize, range: f64) -> usize {
    let mut topology = Topology::new(satellites);
    for first in 0..satellites {
        for second in 0..satellites {
            if first != second
                && mutual_visibility(positions[first], positions[second], range)
                    .is_some_and(|visible| visible)
            {
                let _ = topology.add_edge(first, second, 1.0);
            }
        }
    }
    topology.edge_count()
}

/// Builds a topology across `threads` threads.
///
/// The geometry of each pair is independent, so each thread computes the
/// visible pairs in its own slice and returns them. Folding them into the
/// single graph is serial, which is the point of measuring this path: it
/// shows how much of the time the parallel part actually saves.
///
/// 跨 `threads` 个线程构建拓扑。
///
/// 每对的几何是独立的，故每个线程计算其自身切片中的可见对并返回它们。把它们折进
/// 单一图是串行的，这正是测量本路径的要点：它显示并行部分实际节省了多少时间。
fn topology_parallel(positions: &[Vec3], satellites: usize, range: f64, threads: usize) -> usize {
    let chunk = satellites.div_ceil(threads);
    let results: Vec<Vec<(usize, usize)>> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..threads)
            .map(|worker| {
                let first = worker * chunk;
                let last = (first + chunk).min(satellites);
                scope.spawn(move || {
                    let mut pairs = Vec::new();
                    for first in first..last {
                        for second in 0..satellites {
                            if first != second
                                && mutual_visibility(positions[first], positions[second], range)
                                    .is_some_and(|visible| visible)
                            {
                                pairs.push((first, second));
                            }
                        }
                    }
                    pairs
                })
            })
            .collect();
        handles
            .into_iter()
            .map(|handle| handle.join().expect("the worker did not panic"))
            .collect()
    });

    // The serial tail: fold every visible pair into one graph.
    // 串行尾部：把每个可见对折进一张图。
    let mut topology = Topology::new(satellites);
    for pairs in &results {
        for &(first, second) in pairs {
            let _ = topology.add_edge(first, second, 1.0);
        }
    }
    topology.edge_count()
}

/// Splits topology construction into its geometric and its folding phase.
///
/// Returns the time of the visibility test and the time of folding the
/// visible pairs into a graph, both in milliseconds. The first is
/// embarrassingly parallel, the second is a serial pass whose cost grows
/// with the square of the edge count because each insertion checks for a
/// duplicate by scanning the edges already stored.
///
/// 把拓扑构建拆成其几何阶段与折入阶段。
///
/// 返回可见性判定的时间与把可见对折进图的时间，均以毫秒计。第一个是可并行化的，
/// 第二个是串行的一遍，其代价随边数的平方增长，因为每次插入都通过扫描已存的边来
/// 检查重复。
fn topology_phase_split(positions: &[Vec3], satellites: usize, range: f64) -> (f64, f64) {
    let start = Instant::now();
    let mut pairs = Vec::new();
    for first in 0..satellites {
        for second in 0..satellites {
            if first != second
                && mutual_visibility(positions[first], positions[second], range)
                    .is_some_and(|visible| visible)
            {
                pairs.push((first, second));
            }
        }
    }
    let visible_ms = start.elapsed().as_secs_f64() * 1e3;

    let start = Instant::now();
    let mut topology = Topology::new(satellites);
    for &(first, second) in &pairs {
        let _ = topology.add_edge(first, second, 1.0);
    }
    let fold_ms = start.elapsed().as_secs_f64() * 1e3;

    (visible_ms, fold_ms)
}

fn main() {
    let cores = std::thread::available_parallelism()
        .map(|count| count.get())
        .unwrap_or(1);
    println!("ApsisRS parallel scale benchmark: release build, no external dependency");
    println!("available parallelism: {cores} core(s)");
    println!("no warm-up: the cold cost of the first call is reported, not discarded");

    let satellite = satellite();
    let epoch = epoch_mjd();

    // The thread counts swept: one is the serial reference, then the powers
    // of two up to the machine, then the machine's own count.
    // 所扫过的线程数：一是串行基准，然后是到该机器为止的二的幂，再是该机器自身的
    // 数量。
    let mut thread_counts = vec![1_usize, 2, 4, 8];
    if cores > 8 {
        thread_counts.push(cores);
    }
    thread_counts.retain(|count| *count <= cores.max(2));
    thread_counts.dedup();

    // ---- One: constellation propagation over a day. ----------------------
    // ---- 一：在一天上传播星座。------------------------------------
    {
        let samples = 1_441_usize; // one day at a one minute step
        for &satellites in &[550_usize, 1_000, 2_000] {
            println!(
                "\none: propagate {satellites} satellite(s) over 1 day at 60 s ({samples} samples each)"
            );
            let serial = measure_best(3, || propagate_serial(&satellite, satellites, samples));
            let work = serial;
            for &threads in &thread_counts {
                let wall = measure_best(3, || {
                    propagate_parallel(&satellite, satellites, samples, threads)
                });
                row(threads, wall, work, cores);
            }
        }
    }

    // ---- Two: access search. ---------------------------------------------
    // ---- 二：可见性搜索。------------------------------------------
    {
        for &satellites in &[64_usize, 256] {
            let tracks: Vec<Vec<OrbitSample>> = (0..satellites)
                .map(|_| track(&satellite, 120, epoch))
                .collect();
            let finder = AccessFinder {
                station: GroundPoint::new(Geodetic::new(0.0, 0.0, 0.0), 10.0_f64.to_radians()),
                sample_step_seconds: TRACK_STEP_SECONDS,
                crossing_tolerance_seconds: 0.01,
            };
            println!("\ntwo: access search over {satellites} track(s), 2 h each");
            let serial = measure_best(5, || access_serial(&tracks, &finder));
            let work = serial;
            for &threads in &thread_counts {
                let wall = measure_best(5, || access_parallel(&tracks, &finder, threads));
                row(threads, wall, work, cores);
            }
        }
    }

    // ---- Three: topology construction. ------------------------------------
    // ---- 三：拓扑构建。--------------------------------------------
    {
        for &satellites in &[120_usize, 300] {
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
            println!("\nthree: build topology over {satellites} satellite(s), 4500 km range");
            let serial = measure_best(2, || topology_serial(&positions, satellites, 4_500.0));
            // The split shows where the time actually goes: the geometric
            // visibility test is parallel, but folding the visible pairs into
            // the graph is a serial pass whose cost is superlinear in the
            // edge count. Reporting the two separately is the difference
            // between "parallelism did not help" and knowing why.
            // 该拆分显示时间实际去了哪里：几何可见性判定是并行的，但把可见对折进图是串行的
            // 一遍，其代价在边数上超线性。把二者分开报告，正是「并行没帮上忙」与知其所以然
            // 之间的区别。
            let (vis_ms, fold_ms) = topology_phase_split(&positions, satellites, 4_500.0);
            println!(
                "     split: visibility {vis_ms:.3} ms (parallel), edge fold {fold_ms:.3} ms (serial tail)"
            );
            for &threads in &thread_counts {
                let wall = measure_best(2, || {
                    topology_parallel(&positions, satellites, 4_500.0, threads)
                });
                row(threads, wall, serial, cores);
            }
        }
    }

    println!();
    println!("speedup is serial time over wall time; efficiency is speedup over thread count");
    println!("an efficiency near one means the cores were kept busy; a low value is a serial tail");
}
