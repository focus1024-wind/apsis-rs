//! Tests for the topology time series.

use apsis_topology::{Topology, TopologyErrorKind, TopologySample, TopologySeries};

/// A topology with the given directed edges, all of unit cost.
fn topology_with(node_count: usize, edges: &[(usize, usize)]) -> Topology {
    let mut topology = Topology::new(node_count);
    for &(from, to) in edges {
        topology.add_edge(from, to, 1.0).expect("the nodes exist");
    }
    topology
}

/// A series of three topologies at 0, 10, and 25 seconds.
fn series() -> TopologySeries {
    TopologySeries::new(vec![
        TopologySample::new(0.0, topology_with(3, &[(0, 1)])),
        TopologySample::new(10.0, topology_with(3, &[(0, 1), (1, 2)])),
        TopologySample::new(25.0, topology_with(3, &[(0, 2)])),
    ])
    .expect("the times increase")
}

#[test]
fn a_series_holds_its_samples() {
    let series = series();
    assert_eq!(series.len(), 3);
    assert!(!series.is_empty());
    assert_eq!(series.start_seconds(), 0.0);
    assert_eq!(series.end_seconds(), 25.0);
    assert_eq!(series.node_count(), 3);
}

#[test]
fn a_time_selects_the_sample_in_force() {
    let series = series();
    // At the exact times, the entry itself applies.
    assert_eq!(series.index_at(0.0), Some(0));
    assert_eq!(series.index_at(10.0), Some(1));
    assert_eq!(series.index_at(25.0), Some(2));

    // Between entries, the earlier one applies.
    assert_eq!(series.index_at(5.0), Some(0));
    assert_eq!(series.index_at(9.999), Some(0));
    assert_eq!(series.index_at(10.001), Some(1));
    assert_eq!(series.index_at(24.999), Some(1));

    // A time after the last entry keeps the last topology.
    assert_eq!(series.index_at(1_000.0), Some(2));
    // A time before the first entry uses the first topology.
    assert_eq!(series.index_at(-50.0), Some(0));
}

#[test]
fn the_topology_in_force_reflects_the_changes() {
    let series = series();
    // At one second only the first link exists.
    let early = series.topology_at(1.0).expect("a topology exists");
    assert!(early.has_edge(0, 1));
    assert!(!early.has_edge(1, 2));

    // At fifteen seconds the second link has been added.
    let middle = series.topology_at(15.0).expect("a topology exists");
    assert!(middle.has_edge(0, 1));
    assert!(middle.has_edge(1, 2));

    // At thirty seconds the topology has changed again.
    let late = series.topology_at(30.0).expect("a topology exists");
    assert!(!late.has_edge(0, 1));
    assert!(late.has_edge(0, 2));
}

#[test]
fn the_interval_of_a_sample_is_bounded_by_the_next() {
    let series = series();
    assert_eq!(series.interval_of(0), Some((0.0, 10.0)));
    assert_eq!(series.interval_of(1), Some((10.0, 25.0)));
    // The last interval is unbounded.
    let (start, end) = series.interval_of(2).expect("the index exists");
    assert_eq!(start, 25.0);
    assert!(end.is_infinite());
    assert_eq!(series.interval_of(3), None);
}

#[test]
fn the_change_count_excludes_the_first_entry() {
    let series = series();
    assert_eq!(series.change_count(), 2);
    assert_eq!(series.mean_interval_seconds(), Some(12.5));
}

#[test]
fn the_edge_changes_count_appearances_and_disappearances() {
    let series = series();
    // From the first to the second topology, the edge 1 to 2 appears, one
    // change. From the second to the third, the edges 0 to 1 and 1 to 2
    // disappear and 0 to 2 appears, three changes.
    assert_eq!(series.edge_changes(), 4);
}

#[test]
fn a_single_sample_series_has_no_changes() {
    let single = TopologySeries::new(vec![TopologySample::new(0.0, topology_with(2, &[(0, 1)]))])
        .expect("valid");
    assert_eq!(single.len(), 1);
    assert_eq!(single.change_count(), 0);
    assert_eq!(single.mean_interval_seconds(), None);
    assert_eq!(single.edge_changes(), 0);
}

#[test]
fn an_empty_series_is_rejected() {
    let error = TopologySeries::new(Vec::new()).expect_err("no samples");
    assert_eq!(error.kind(), TopologyErrorKind::EmptySeries);
}

#[test]
fn a_non_finite_time_is_rejected() {
    for time in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let error = TopologySeries::new(vec![TopologySample::new(time, Topology::new(1))])
            .expect_err("the time is not finite");
        assert_eq!(error.kind(), TopologyErrorKind::InvalidTime, "time {time}");
    }
}

#[test]
fn unordered_times_are_rejected() {
    let error = TopologySeries::new(vec![
        TopologySample::new(10.0, Topology::new(1)),
        TopologySample::new(5.0, Topology::new(1)),
    ])
    .expect_err("the times decrease");
    assert_eq!(error.kind(), TopologyErrorKind::UnorderedTimes);

    // Equal times are also rejected, because the topology at that instant
    // would be ambiguous.
    let error = TopologySeries::new(vec![
        TopologySample::new(10.0, Topology::new(1)),
        TopologySample::new(10.0, Topology::new(1)),
    ])
    .expect_err("the times are equal");
    assert_eq!(error.kind(), TopologyErrorKind::UnorderedTimes);
}

#[test]
fn a_series_can_be_built_from_separate_lists() {
    let series = TopologySeries::from_parts(
        vec![0.0, 10.0],
        vec![topology_with(2, &[(0, 1)]), topology_with(2, &[])],
    )
    .expect("the lists match");
    assert_eq!(series.len(), 2);
    assert!(
        series
            .topology_at(5.0)
            .expect("a topology exists")
            .has_edge(0, 1)
    );
    assert!(
        !series
            .topology_at(15.0)
            .expect("a topology exists")
            .has_edge(0, 1)
    );
}

#[test]
fn mismatched_list_lengths_are_rejected() {
    let error = TopologySeries::from_parts(vec![0.0, 10.0], vec![topology_with(2, &[(0, 1)])])
        .expect_err("the lengths differ");
    assert_eq!(error.kind(), TopologyErrorKind::EmptySeries);
}

#[test]
fn a_non_finite_query_time_is_rejected() {
    let series = series();
    assert_eq!(series.index_at(f64::NAN), None);
    assert_eq!(series.index_at(f64::INFINITY), None);
    assert_eq!(series.topology_at(f64::NAN), None);
}

#[test]
fn a_series_with_negative_times_is_accepted() {
    // The origin of the time axis is the caller's choice, so negative times
    // are as valid as positive ones.
    let series = TopologySeries::new(vec![
        TopologySample::new(-100.0, topology_with(2, &[(0, 1)])),
        TopologySample::new(-50.0, topology_with(2, &[])),
    ])
    .expect("the times increase");
    assert_eq!(series.start_seconds(), -100.0);
    assert_eq!(series.end_seconds(), -50.0);
    assert!(
        series
            .topology_at(-75.0)
            .expect("a topology exists")
            .has_edge(0, 1)
    );
    assert!(
        !series
            .topology_at(0.0)
            .expect("a topology exists")
            .has_edge(0, 1)
    );
}

#[test]
fn the_samples_are_accessible() {
    let series = series();
    assert_eq!(series.samples().len(), 3);
    assert_eq!(series.samples()[0].seconds, 0.0);
    assert_eq!(series.samples()[1].seconds, 10.0);
}

#[test]
fn a_change_at_the_boundary_is_immediate() {
    // The topology changes exactly at the recorded time, not after it.
    let series = series();
    let before = series.topology_at(10.0 - 1e-9).expect("a topology exists");
    let after = series.topology_at(10.0).expect("a topology exists");
    assert!(!before.has_edge(1, 2));
    assert!(after.has_edge(1, 2));
}
