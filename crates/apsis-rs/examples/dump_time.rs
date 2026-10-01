//! Writes the leap second offset and the time scale conversions this
//! workspace computes over a range of instants.
//!
//! The output is CSV on standard output, with a header row. It is consumed
//! by `validation/python/time/check_time.py`, which recomputes each instant
//! with a reference implementation.
//!
//! The TAI instant is written in two parts, an integer Modified Julian Date
//! and a fraction of a day, because a single `f64` cannot hold a Modified
//! Julian Date near 60 000 to better than about 40 microseconds. The two
//! parts let the checker compare instants to well below a microsecond.
//!
//! 写出闰秒偏移以及本工作区在一系列瞬时上计算的时标换算。
//!
//! 输出是标准输出上的 CSV，带一行表头。它由
//! `validation/python/time/check_time.py` 消费，后者用参考实现重算每个瞬时。
//!
//! TAI 瞬时按两部分写出，一个整数约化儒略日与一天的零头，因为单个 `f64` 无法
//! 把接近 60 000 的约化儒略日保持到优于约 40 微秒。两部分使检查器能把瞬时比较
//! 到远低于一微秒。
//!
//! Usage:
//!
//! ```text
//! cargo run -p apsis-rs --features physics --example dump_time \
//!     > validation/data/expected/rust_time.csv
//! ```

use apsis_leap::{LEAP_SECONDS, tai_minus_utc_at_mjd};
use apsis_time::{Calendar, Epoch, TT_MINUS_TAI, TimeScale};

/// Splits a Modified Julian Date into its integer day and its fraction.
///
/// 把约化儒略日拆为其整数日与零头。
fn split(mjd: f64) -> (i64, f64) {
    let day = mjd.floor();
    (day as i64, mjd - day)
}

fn emit(calendar: Calendar, epoch: Epoch) {
    let offset = tai_minus_utc_at_mjd(
        epoch
            .mjd(TimeScale::Utc)
            .expect("every emitted instant is in UTC"),
    )
    .expect("every emitted instant is covered by the table");
    let (tai_day, tai_frac) = split(epoch.tai_mjd());
    println!(
        "{:04},{:02},{:02},{:02},{:02},{:.3},{:.1},{tai_day},{tai_frac:.17}",
        calendar.year,
        calendar.month,
        calendar.day,
        calendar.hour,
        calendar.minute,
        calendar.second,
        offset
    );
}

fn main() {
    println!("year,month,day,hour,minute,second,tai_minus_utc,tai_day,tai_frac");

    // Sample every leap second boundary at several times of day, so that
    // the step behaviour and the interior of each interval are covered.
    // 在一天中的若干时刻对每个闰秒边界采样，使阶跃行为与每个区间的内部都被覆盖。
    for entry in LEAP_SECONDS {
        for (hour, minute, second) in [(0, 0, 0.0), (6, 0, 0.0), (12, 0, 0.0), (18, 0, 0.0)] {
            let calendar = Calendar::new(entry.year, entry.month, entry.day, hour, minute, second);
            if let Some(epoch) = Epoch::from_calendar(calendar, TimeScale::Utc) {
                emit(calendar, epoch);
            }
        }
    }

    // The last second before a leap second and the first second after it.
    // 闰秒前的最后一秒与其后的第一秒。
    for (year, month, day, hour, minute, second) in [
        (2016, 12, 31, 23, 59, 59.0),
        (2017, 1, 1, 0, 0, 0.0),
        (2016, 12, 31, 12, 0, 0.0),
        (1998, 12, 31, 23, 59, 59.0),
        (1999, 1, 1, 0, 0, 0.0),
    ] {
        let calendar = Calendar::new(year, month, day, hour, minute, second);
        if let Some(epoch) = Epoch::from_calendar(calendar, TimeScale::Utc) {
            emit(calendar, epoch);
        }
    }

    // The Terrestrial Time offset is a defining constant, written here so
    // that the checker can verify it without recomputing the calendar.
    // 地球时偏移是一个定义常量，写在这里使检查器无需重算日历即可核验它。
    println!("# tt_minus_tai_seconds={TT_MINUS_TAI:.3}");
}
