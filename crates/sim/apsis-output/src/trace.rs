//! The trace and its summaries.
//!
//! 轨迹及其各汇总。

use crate::record::{Record, RecordKind};
use std::collections::BTreeMap;

/// A list of records in the order they were made.
///
/// The trace preserves insertion order rather than sorting by time, because
/// the order a model recorded in is part of what it decided. A caller that
/// needs time order asks for [`Trace::in_time_order`], which leaves the
/// original alone.
///
/// 按作出次序排列的记录列表。
///
/// 轨迹保留插入次序而不按时间排序，因为模型记录的次序是其决策的一部分。
/// 需要时间次序的调用方索取 [`Trace::in_time_order`]，它不改变原轨迹。
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Trace {
    records: Vec<Record>,
}

impl Trace {
    /// Creates an empty trace.
    ///
    /// 创建一个空轨迹。
    #[must_use]
    pub const fn new() -> Self {
        Self {
            records: Vec::new(),
        }
    }

    /// Creates a trace from a list of records.
    ///
    /// 由记录列表创建轨迹。
    #[must_use]
    pub fn from_records(records: Vec<Record>) -> Self {
        Self { records }
    }

    /// Returns the number of records.
    ///
    /// 返回记录的数量。
    #[must_use]
    pub fn len(&self) -> usize {
        self.records.len()
    }

    /// Returns `true` when there are no records.
    ///
    /// 当没有记录时返回 `true`。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    /// Returns the records in the order they were made.
    ///
    /// 按作出次序返回各记录。
    #[must_use]
    pub fn records(&self) -> &[Record] {
        &self.records
    }

    /// Adds a record, returning `false` when it is not usable.
    ///
    /// A record whose time or value is not finite is refused rather than
    /// stored, so a trace never holds a value that cannot be summarized.
    ///
    /// 添加一条记录，不可用时返回 `false`。
    ///
    /// 时间或值非有限的记录被拒绝而非存储，故轨迹绝不保存无法汇总的
    /// 值。
    pub fn push(&mut self, record: Record) -> bool {
        if !record.is_valid() {
            return false;
        }
        self.records.push(record);
        true
    }

    /// Returns `true` when no record has a time or value that is not finite.
    ///
    /// 当没有任何记录的时间或值非有限时返回 `true`。
    pub fn is_stable(&self) -> bool {
        self.records.iter().all(Record::is_valid)
    }

    /// Returns the records in time order, earliest first.
    ///
    /// Records at the same instant keep their insertion order, so the sort is
    /// stable and the result does not depend on the sort algorithm.
    ///
    /// 按时间次序返回各记录，最早在前。
    ///
    /// 同一瞬时的记录保持其插入次序，故该排序是稳定的，结果不依赖于
    /// 排序算法。
    #[must_use]
    pub fn in_time_order(&self) -> Vec<Record> {
        let mut ordered = self.records.clone();
        ordered.sort_by(|left, right| {
            left.time
                .partial_cmp(&right.time)
                .unwrap_or(core::cmp::Ordering::Equal)
        });
        ordered
    }

    /// Returns the records of one kind, in order.
    ///
    /// 按次序返回某一类别的记录。
    #[must_use]
    pub fn of_kind(&self, kind: RecordKind) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.kind == kind)
            .collect()
    }

    /// Returns the records with a label, in order.
    ///
    /// 按次序返回具有某标签的记录。
    #[must_use]
    pub fn with_label(&self, label: &str) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.label == label)
            .collect()
    }

    /// Returns the value of the last record with a label.
    ///
    /// This is what a caller asks for after a run: the final value a series
    /// reached. Returns `None` when no record has the label.
    ///
    /// 返回具有某标签的最后一条记录的值。
    ///
    /// 这是调用方在运行结束后所索取的：某序列达到的最终值。没有记录
    /// 具有该标签时返回 `None`。
    #[must_use]
    pub fn value_of(&self, label: &str) -> Option<f64> {
        self.records
            .iter()
            .rev()
            .find(|record| record.label == label)
            .map(|record| record.value)
    }

    /// Returns the time of the last record with a label.
    ///
    /// 返回具有某标签的最后一条记录的时间。
    #[must_use]
    pub fn last_time_of(&self, label: &str) -> Option<f64> {
        self.records
            .iter()
            .rev()
            .find(|record| record.label == label)
            .map(|record| record.time)
    }

    /// Returns the labels present, in sorted order.
    ///
    /// The order is sorted rather than first seen, so that two traces with
    /// the same labels list them the same way.
    ///
    /// 按排序次序返回出现的标签。
    ///
    /// 次序是按排序而非按首次出现，使具有相同标签的两个轨迹以相同
    /// 方式列出它们。
    #[must_use]
    pub fn labels(&self) -> Vec<&str> {
        let mut labels: Vec<&str> = self
            .records
            .iter()
            .map(|record| record.label.as_str())
            .collect();
        labels.sort_unstable();
        labels.dedup();
        labels
    }

    /// Returns a summary of the series recorded under a label.
    ///
    /// Returns `None` when no record has the label.
    ///
    /// 返回在某标签下所记录序列的汇总。
    ///
    /// 没有记录具有该标签时返回 `None`。
    pub fn summarize(&self, label: &str) -> Option<SeriesSummary> {
        let values: Vec<f64> = self
            .records
            .iter()
            .filter(|record| record.label == label)
            .map(|record| record.value)
            .collect();
        if values.is_empty() {
            return None;
        }
        let count = values.len();
        let sum: f64 = values.iter().sum();
        let min = values.iter().copied().fold(f64::INFINITY, f64::min);
        let max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let first = values[0];
        let last = values[count - 1];
        Some(SeriesSummary {
            label: label.to_string(),
            count,
            min,
            max,
            mean: sum / count as f64,
            first,
            last,
            change: last - first,
        })
    }

    /// Returns the number of warnings in the trace.
    ///
    /// 返回轨迹中警告的数量。
    #[must_use]
    pub fn warning_count(&self) -> usize {
        self.records
            .iter()
            .filter(|record| record.kind.is_warning())
            .count()
    }

    /// Returns a summary of every label, in sorted label order.
    ///
    /// 按标签排序次序返回每个标签的汇总。
    #[must_use]
    pub fn summaries(&self) -> Vec<SeriesSummary> {
        self.labels()
            .into_iter()
            .filter_map(|label| self.summarize(label))
            .collect()
    }

    /// Returns the greatest number of records that share a label.
    ///
    /// This is the length of the longest series, which a caller uses to size
    /// a table of the trace.
    ///
    /// 返回共享同一标签的记录数的最大值。
    ///
    /// 它是最长序列的长度，调用方据此确定轨迹表格的大小。
    #[must_use]
    pub fn longest_series(&self) -> usize {
        let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
        for record in &self.records {
            *counts.entry(record.label.as_str()).or_insert(0) += 1;
        }
        counts.values().copied().max().unwrap_or(0)
    }

    /// Removes every record and returns how many were discarded.
    ///
    /// 移除所有记录并返回被丢弃的数量。
    pub fn clear(&mut self) -> usize {
        let count = self.records.len();
        self.records.clear();
        count
    }
}

/// A summary of one series of records.
///
/// 某一条记录序列的汇总。
#[derive(Debug, Clone, PartialEq)]
pub struct SeriesSummary {
    /// The label the series was recorded under.
    ///
    /// 该序列所记录用的标签。
    pub label: String,
    /// The number of records in the series.
    ///
    /// 该序列中记录的数量。
    pub count: usize,
    /// The smallest value recorded.
    ///
    /// 所记录的最小值。
    pub min: f64,
    /// The greatest value recorded.
    ///
    /// 所记录的最大值。
    pub max: f64,
    /// The mean of the values.
    ///
    /// 各值的均值。
    pub mean: f64,
    /// The first value recorded.
    ///
    /// 所记录的第一个值。
    pub first: f64,
    /// The last value recorded.
    ///
    /// 所记录的最后一个值。
    pub last: f64,
    /// The difference between the last and first values.
    ///
    /// 最后一个值与第一个值之差。
    pub change: f64,
}

impl SeriesSummary {
    /// Returns the spread between the greatest and smallest values.
    ///
    /// 返回最大值与最小值之间的跨度。
    #[must_use]
    pub fn range(&self) -> f64 {
        self.max - self.min
    }

    /// Returns `true` when the series never changed.
    ///
    /// 当该序列从未变化时返回 `true`。
    #[must_use]
    pub fn is_constant(&self) -> bool {
        self.max == self.min
    }
}
