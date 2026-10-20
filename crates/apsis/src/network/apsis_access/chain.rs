//! Chain access: the intervals in which every link is available.
//!
//! A chain is available when all of its links are available at the same
//! time. Given one interval list per link, the chain intervals are the
//! intersections of one interval from each list. The lists must be sorted
//! and disjoint, which is the form the access finder produces.
//!
//! 链访问：每条链路同时可用的区间。
//!
//! 当链的所有链路在同一时刻都可用时，链可用。给定每条链路一个区间列表，
//! 链区间就是每个列表各取一个区间的交集。列表须有序且互不相交，这正是访问
//! 查找器所产生的形式。

use crate::apsis_access::pass::Pass;
use crate::apsis_core::{Error, ErrorKind, Result};

/// A half-open access interval in seconds from a common origin.
///
/// 距共同原点的半开访问区间，单位为秒。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AccessInterval {
    /// Start of the interval in seconds.
    ///
    /// 区间起点，单位为秒。
    pub start_seconds: f64,
    /// End of the interval in seconds.
    ///
    /// 区间终点，单位为秒。
    pub end_seconds: f64,
}

impl AccessInterval {
    /// Creates an interval.
    ///
    /// 创建区间。
    #[must_use]
    pub const fn new(start_seconds: f64, end_seconds: f64) -> Self {
        Self {
            start_seconds,
            end_seconds,
        }
    }

    /// Returns the interval duration in seconds.
    ///
    /// 返回区间时长，单位为秒。
    #[must_use]
    pub fn duration(self) -> f64 {
        self.end_seconds - self.start_seconds
    }

    /// Returns true when the interval is finite and positive.
    ///
    /// 区间有限且为正时返回真。
    #[must_use]
    pub fn is_valid(self) -> bool {
        self.start_seconds.is_finite()
            && self.end_seconds.is_finite()
            && self.end_seconds > self.start_seconds
    }
}

impl From<Pass> for AccessInterval {
    fn from(pass: Pass) -> Self {
        Self::new(pass.rise_seconds, pass.set_seconds)
    }
}

/// Returns the intervals in which every link is available.
///
/// Every link must supply at least one interval, and each list must be
/// sorted by start and disjoint. Returns an empty vector when the link
/// availability has no common time.
///
/// 返回所有链路同时可用的区间。
///
/// 每条链路至少提供一个区间，且每个列表须按起点有序、互不相交。链路可用时间
/// 没有交集时返回空向量。
pub fn intersect_intervals(links: &[Vec<AccessInterval>]) -> Result<Vec<AccessInterval>> {
    if links.is_empty() {
        return Err(Error::with_message(
            ErrorKind::InvalidInput,
            "a chain needs at least one link",
        ));
    }
    for link in links {
        if link.is_empty() {
            return Ok(Vec::new());
        }
        for (index, interval) in link.iter().enumerate() {
            if !interval.is_valid() {
                return Err(Error::with_message(
                    ErrorKind::InvalidInput,
                    "access intervals must be finite and positive",
                ));
            }
            if index > 0 && interval.start_seconds < link[index - 1].end_seconds {
                return Err(Error::with_message(
                    ErrorKind::InvalidInput,
                    "access intervals must be sorted and disjoint",
                ));
            }
        }
    }
    let mut current = links[0].clone();
    for link in &links[1..] {
        let mut merged = Vec::new();
        let mut left = 0_usize;
        let mut right = 0_usize;
        while left < current.len() && right < link.len() {
            let start = current[left].start_seconds.max(link[right].start_seconds);
            let end = current[left].end_seconds.min(link[right].end_seconds);
            if end > start {
                merged.push(AccessInterval::new(start, end));
            }
            if current[left].end_seconds < link[right].end_seconds {
                left += 1;
            } else {
                right += 1;
            }
        }
        current = merged;
        if current.is_empty() {
            break;
        }
    }
    Ok(current)
}

/// Returns the chain intervals from one pass list per link.
///
/// 返回每条链路一个过境列表时的链区间。
pub fn chain_passes(links: &[Vec<Pass>]) -> Result<Vec<AccessInterval>> {
    let intervals: Vec<Vec<AccessInterval>> = links
        .iter()
        .map(|passes| {
            passes
                .iter()
                .map(|pass| AccessInterval::from(*pass))
                .collect()
        })
        .collect();
    intersect_intervals(&intervals)
}
