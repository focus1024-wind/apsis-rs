//! The time series of topologies.
//!
//! 拓扑的时间序列。

use crate::error::{TopologyError, TopologyErrorKind};
use crate::graph::Topology;

/// One topology and the time from which it applies.
///
/// 一个拓扑及其开始适用的时间。
#[derive(Debug, Clone, PartialEq)]
pub struct TopologySample {
    /// The time from which this topology applies, in seconds from the
    /// caller's origin.
    ///
    /// 该拓扑开始适用的时间，以距调用方原点的秒数表示。
    pub seconds: f64,
    /// The topology in force from that time.
    ///
    /// 自该时间起生效的拓扑。
    pub topology: Topology,
}

impl TopologySample {
    /// Creates a sample.
    ///
    /// 创建一个样本。
    pub const fn new(seconds: f64, topology: Topology) -> Self {
        Self { seconds, topology }
    }
}

/// A sequence of topologies over time.
///
/// The sequence is a step function. The entry at index `i` applies from its
/// own time until the time of the entry at index `i + 1`, and the last entry
/// applies for all later times. Querying a time before the first entry
/// returns the first topology, so the series behaves as if it extends
/// backwards as well as forwards.
///
/// The times must be strictly increasing. Two entries at the same time would
/// make the topology at that instant ambiguous.
///
/// 拓扑随时间变化的序列。
///
/// 该序列是阶跃函数。下标 `i` 处的项自其自身时间起适用，直到下标
/// `i + 1` 处项的时间；最后一项适用于其后所有时间。查询首项之前的时间
/// 返回第一个拓扑，故该序列的行为像是向前向后都延伸。
///
/// 各时间必须严格递增。同一时间的两项会使该瞬时的拓扑含糊。
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TopologySeries {
    samples: Vec<TopologySample>,
}

impl TopologySeries {
    /// Creates a series from a list of samples.
    ///
    /// Returns an error when the list is empty, when a time is not finite,
    /// or when the times are not strictly increasing.
    ///
    /// 由样本列表创建序列。
    ///
    /// 列表为空、某时间非有限，或各时间非严格递增时返回错误。
    pub fn new(samples: Vec<TopologySample>) -> Result<Self, TopologyError> {
        if samples.is_empty() {
            return Err(TopologyError::new(TopologyErrorKind::EmptySeries));
        }
        let mut previous = f64::NEG_INFINITY;
        for (index, sample) in samples.iter().enumerate() {
            if !sample.seconds.is_finite() {
                return Err(TopologyError::new(TopologyErrorKind::InvalidTime)
                    .with_detail(format!("entry {index} has time {}", sample.seconds)));
            }
            if sample.seconds <= previous {
                return Err(
                    TopologyError::new(TopologyErrorKind::UnorderedTimes).with_detail(format!(
                        "entry {index} at {} does not follow {previous}",
                        sample.seconds
                    )),
                );
            }
            previous = sample.seconds;
        }
        Ok(Self { samples })
    }

    /// Creates a series from times and topologies as separate lists.
    ///
    /// The two lists must have the same length.
    ///
    /// Returns an error when the lists differ in length or when the series
    /// itself is not valid.
    ///
    /// 由分别给出的时间列表与拓扑列表创建序列。
    ///
    /// 两个列表必须长度相同。
    ///
    /// 列表长度不同或序列自身无效时返回错误。
    pub fn from_parts(times: Vec<f64>, topologies: Vec<Topology>) -> Result<Self, TopologyError> {
        if times.len() != topologies.len() {
            return Err(
                TopologyError::new(TopologyErrorKind::EmptySeries).with_detail(format!(
                    "{} times against {} topologies",
                    times.len(),
                    topologies.len()
                )),
            );
        }
        let samples = times
            .into_iter()
            .zip(topologies)
            .map(|(seconds, topology)| TopologySample::new(seconds, topology))
            .collect();
        Self::new(samples)
    }

    /// Returns the samples.
    ///
    /// 返回各样本。
    pub fn samples(&self) -> &[TopologySample] {
        &self.samples
    }

    /// Returns the number of samples.
    ///
    /// 返回样本的数量。
    pub fn len(&self) -> usize {
        self.samples.len()
    }

    /// Returns `true` when there are no samples.
    ///
    /// A series built through either constructor is never empty, so this
    /// always returns `false`; it exists because a collection type should
    /// offer it alongside [`TopologySeries::len`].
    ///
    /// 当没有样本时返回 `true`。
    ///
    /// 经任一构造函数构建的序列都不会为空，故这总是返回 `false`；它的
    /// 存在是因为集合类型应在 [`TopologySeries::len`] 之外一并提供它。
    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    /// Returns the first time in the series.
    ///
    /// 返回序列中的第一个时间。
    pub fn start_seconds(&self) -> f64 {
        self.samples.first().map_or(0.0, |sample| sample.seconds)
    }

    /// Returns the last time in the series.
    ///
    /// 返回序列中的最后一个时间。
    pub fn end_seconds(&self) -> f64 {
        self.samples.last().map_or(0.0, |sample| sample.seconds)
    }

    /// Returns the index of the sample that applies at a time.
    ///
    /// The index is the last entry whose time is at or before the query, so
    /// a time before the series returns zero and a time after it returns the
    /// last index.
    ///
    /// Returns `None` when the time is not finite or the series is empty.
    ///
    /// 返回某时间处适用的样本下标。
    ///
    /// 该下标是时间不晚于查询的最后一个项，故序列之前的时间返回零，
    /// 序列之后的时间返回最后的下标。
    ///
    /// 时间非有限或序列为空时返回 `None`。
    pub fn index_at(&self, seconds: f64) -> Option<usize> {
        if !seconds.is_finite() || self.samples.is_empty() {
            return None;
        }
        // The samples are ordered, so a binary search gives the insertion
        // point of the query, and the entry before it is the one in force.
        // 样本已排序，故二分查找给出查询的插入点，其前一项即生效的那项。
        match self
            .samples
            .binary_search_by(|sample| sample.seconds.partial_cmp(&seconds).expect("finite times"))
        {
            Ok(exact) => Some(exact),
            Err(0) => Some(0),
            Err(insert) => Some(insert - 1),
        }
    }

    /// Returns the topology in force at a time.
    ///
    /// Returns `None` when the time is not finite or the series is empty.
    ///
    /// 返回某时间处生效的拓扑。
    ///
    /// 时间非有限或序列为空时返回 `None`。
    pub fn topology_at(&self, seconds: f64) -> Option<&Topology> {
        Some(&self.samples.get(self.index_at(seconds)?)?.topology)
    }

    /// Returns the interval over which the sample at an index applies.
    ///
    /// The upper bound is the time of the next sample, or positive infinity
    /// for the last entry.
    ///
    /// Returns `None` when the index is out of range.
    ///
    /// 返回某下标处样本适用的区间。
    ///
    /// 上界是下一样本的时间，对最后一项为正无穷。
    ///
    /// 下标越界时返回 `None`。
    pub fn interval_of(&self, index: usize) -> Option<(f64, f64)> {
        let sample = self.samples.get(index)?;
        let end = self
            .samples
            .get(index + 1)
            .map_or(f64::INFINITY, |next| next.seconds);
        Some((sample.seconds, end))
    }

    /// Returns the number of times the topology changes.
    ///
    /// This is the number of samples less one, since each change introduces
    /// a new entry. It measures how dynamic the network is: a series with
    /// many changes needs routing that accounts for them.
    ///
    /// 返回拓扑变化的次数。
    ///
    /// 它是样本数减一，因为每次变化引入一个新项。它衡量网络的动态
    /// 程度：变化多的序列需要能考虑到这些变化的路由。
    pub fn change_count(&self) -> usize {
        self.samples.len().saturating_sub(1)
    }

    /// Returns the mean interval between changes, in seconds.
    ///
    /// The last entry has unbounded duration, so it is excluded from the
    /// mean and the result covers the changes that have a measured interval.
    ///
    /// Returns `None` when there is only one sample.
    ///
    /// 返回变化之间的平均间隔，单位为秒。
    ///
    /// 最后一项的持续时间无界，故它被排除在均值之外，结果涵盖有可测
    /// 间隔的那些变化。
    ///
    /// 只有一个样本时返回 `None`。
    pub fn mean_interval_seconds(&self) -> Option<f64> {
        if self.samples.len() < 2 {
            return None;
        }
        let span = self.end_seconds() - self.start_seconds();
        Some(span / self.change_count() as f64)
    }

    /// Returns the number of edge changes across the series.
    ///
    /// Each topology is compared with the one before it, counting the edges
    /// that appear and the edges that disappear. A topology that differs by
    /// one link contributes two, since one link changed.
    ///
    /// Returns zero for a series of one sample.
    ///
    /// 返回整个序列中边变化的次数。
    ///
    /// 每个拓扑与其前一个比较，统计出现与消失的边。相差一条链路的拓扑
    /// 贡献二，因为有一条链路发生了变化。
    ///
    /// 单样本序列返回零。
    pub fn edge_changes(&self) -> usize {
        let mut changes = 0;
        for pair in self.samples.windows(2) {
            let before = &pair[0].topology;
            let after = &pair[1].topology;
            // Edges present before but not after.
            // 之前存在而之后不存在的边。
            changes += before
                .edges()
                .iter()
                .filter(|edge| !after.has_edge(edge.from, edge.to))
                .count();
            // Edges present after but not before.
            // 之后存在而之前不存在的边。
            changes += after
                .edges()
                .iter()
                .filter(|edge| !before.has_edge(edge.from, edge.to))
                .count();
        }
        changes
    }

    /// Returns the number of nodes in the series.
    ///
    /// Every entry is expected to describe the same set of nodes, so the
    /// value is taken from the first entry. A caller that changes the node
    /// count between entries is comparing two different networks and should
    /// build two series instead.
    ///
    /// 返回序列中的节点数。
    ///
    /// 每个项都应描述同一组节点，故该值取自第一项。在各项之间改变节点
    /// 数的调用方是在比较两个不同的网络，应改为构建两个序列。
    pub fn node_count(&self) -> usize {
        self.samples
            .first()
            .map_or(0, |sample| sample.topology.node_count())
    }
}
