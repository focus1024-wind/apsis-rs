//! The trace and its summaries.

use crate::record::{Record, RecordKind};
use std::collections::BTreeMap;

/// A list of records in the order they were made.
///
/// The trace preserves insertion order rather than sorting by time, because
/// the order a model recorded in is part of what it decided. A caller that
/// needs time order asks for [`Trace::in_time_order`], which leaves the
/// original alone.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Trace {
    records: Vec<Record>,
}

impl Trace {
    /// Creates an empty trace.
    pub const fn new() -> Self {
        Self {
            records: Vec::new(),
        }
    }

    /// Creates a trace from a list of records.
    pub fn from_records(records: Vec<Record>) -> Self {
        Self { records }
    }

    /// Returns the number of records.
    pub fn len(&self) -> usize {
        self.records.len()
    }

    /// Returns `true` when there are no records.
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    /// Returns the records in the order they were made.
    pub fn records(&self) -> &[Record] {
        &self.records
    }

    /// Adds a record, returning `false` when it is not usable.
    ///
    /// A record whose time or value is not finite is refused rather than
    /// stored, so a trace never holds a value that cannot be summarized.
    pub fn push(&mut self, record: Record) -> bool {
        if !record.is_valid() {
            return false;
        }
        self.records.push(record);
        true
    }

    /// Returns `true` when no record has a time or value that is not finite.
    pub fn is_stable(&self) -> bool {
        self.records.iter().all(Record::is_valid)
    }

    /// Returns the records in time order, earliest first.
    ///
    /// Records at the same instant keep their insertion order, so the sort is
    /// stable and the result does not depend on the sort algorithm.
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
    pub fn of_kind(&self, kind: RecordKind) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.kind == kind)
            .collect()
    }

    /// Returns the records with a label, in order.
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
    pub fn value_of(&self, label: &str) -> Option<f64> {
        self.records
            .iter()
            .rev()
            .find(|record| record.label == label)
            .map(|record| record.value)
    }

    /// Returns the time of the last record with a label.
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
    pub fn warning_count(&self) -> usize {
        self.records
            .iter()
            .filter(|record| record.kind.is_warning())
            .count()
    }

    /// Returns a summary of every label, in sorted label order.
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
    pub fn longest_series(&self) -> usize {
        let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
        for record in &self.records {
            *counts.entry(record.label.as_str()).or_insert(0) += 1;
        }
        counts.values().copied().max().unwrap_or(0)
    }

    /// Removes every record and returns how many were discarded.
    pub fn clear(&mut self) -> usize {
        let count = self.records.len();
        self.records.clear();
        count
    }
}

/// A summary of one series of records.
#[derive(Debug, Clone, PartialEq)]
pub struct SeriesSummary {
    /// The label the series was recorded under.
    pub label: String,
    /// The number of records in the series.
    pub count: usize,
    /// The smallest value recorded.
    pub min: f64,
    /// The greatest value recorded.
    pub max: f64,
    /// The mean of the values.
    pub mean: f64,
    /// The first value recorded.
    pub first: f64,
    /// The last value recorded.
    pub last: f64,
    /// The difference between the last and first values.
    pub change: f64,
}

impl SeriesSummary {
    /// Returns the spread between the greatest and smallest values.
    pub fn range(&self) -> f64 {
        self.max - self.min
    }

    /// Returns `true` when the series never changed.
    pub fn is_constant(&self) -> bool {
        self.max == self.min
    }
}
