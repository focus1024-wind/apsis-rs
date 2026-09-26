//! Tests for resources and the set that holds them.

use apsis_resource::{Interval, Resource, ResourceErrorKind, ResourceId, ResourceSet, Window};

/// A resource with one window from 100 to 200 seconds.
fn station() -> Resource {
    let mut resource = Resource::new(ResourceId::new(0), 1.0);
    resource
        .add_window(Window::new(100.0, 200.0))
        .expect("the window is usable");
    resource
}

#[test]
fn a_new_resource_has_no_availability() {
    let resource = Resource::new(ResourceId::new(3), 1.0);
    assert!(resource.is_empty());
    assert_eq!(resource.window_count(), 0);
    assert_eq!(resource.total_availability(), 0.0);
    assert!(!resource.is_available_at(0.0));
    assert_eq!(resource.id, ResourceId::new(3));
    assert!(!resource.is_available_at(f64::NAN));
}

#[test]
fn adding_a_window_makes_the_resource_available() {
    let resource = station();
    assert_eq!(resource.window_count(), 1);
    assert_eq!(resource.total_availability(), 100.0);
    assert!(resource.is_available_at(150.0));
    assert!(!resource.is_available_at(50.0));
    assert!(!resource.is_available_at(250.0));
}

#[test]
fn availability_is_half_open_at_the_edges() {
    let resource = station();
    assert!(resource.is_available_at(100.0));
    assert!(resource.is_available_at(199.999));
    // The end instant belongs to the next interval, so the resource stops
    // being available exactly there.
    assert!(!resource.is_available_at(200.0));
}

#[test]
fn availability_over_a_duration_requires_the_whole_span() {
    let resource = station();
    assert!(resource.is_available_over(100.0, 100.0));
    assert!(resource.is_available_over(120.0, 80.0));
    assert!(resource.is_available_over(150.0, 50.0));
    // A request that reaches past the window fails, even by a little.
    assert!(!resource.is_available_over(150.0, 50.001));
    assert!(!resource.is_available_over(99.0, 2.0));
}

#[test]
fn a_zero_duration_request_is_an_instantaneous_query() {
    let resource = station();
    assert!(resource.is_available_over(150.0, 0.0));
    assert!(!resource.is_available_over(250.0, 0.0));
    // The half-open boundary applies here too.
    assert!(!resource.is_available_over(200.0, 0.0));
}

#[test]
fn a_request_with_an_invalid_duration_is_refused() {
    let resource = station();
    assert!(!resource.is_available_over(100.0, -1.0));
    assert!(!resource.is_available_over(100.0, f64::NAN));
    assert!(!resource.is_available_over(f64::NAN, 10.0));
    assert!(!resource.is_available_over(100.0, f64::INFINITY));
}

#[test]
fn adding_overlapping_windows_merges_them() {
    let mut resource = Resource::new(ResourceId::new(0), 1.0);
    resource.add_window(Window::new(0.0, 10.0)).expect("usable");
    resource.add_window(Window::new(5.0, 15.0)).expect("usable");
    // The two overlap, so the stored form is one window.
    assert_eq!(resource.window_count(), 1);
    assert_eq!(resource.windows()[0], Window::new(0.0, 15.0));
    assert_eq!(resource.total_availability(), 15.0);
}

#[test]
fn adding_disjoint_windows_keeps_them_separate() {
    let mut resource = Resource::new(ResourceId::new(0), 1.0);
    resource
        .add_window(Window::new(30.0, 40.0))
        .expect("usable");
    resource.add_window(Window::new(0.0, 10.0)).expect("usable");
    assert_eq!(resource.window_count(), 2);
    // The stored order is time order, not insertion order.
    assert_eq!(resource.windows()[0].start(), 0.0);
    assert_eq!(resource.windows()[1].start(), 30.0);
}

#[test]
fn removing_a_window_splits_availability() {
    let mut resource = station();
    resource
        .remove_window(Window::new(140.0, 160.0))
        .expect("usable");
    assert_eq!(resource.window_count(), 2);
    assert_eq!(resource.windows()[0], Window::new(100.0, 140.0));
    assert_eq!(resource.windows()[1], Window::new(160.0, 200.0));
    assert!(!resource.is_available_at(150.0));
    assert!(resource.is_available_at(130.0));
    assert!(resource.is_available_at(180.0));
}

#[test]
fn setting_the_windows_replaces_them() {
    let mut resource = station();
    resource
        .set_windows(&[Window::new(0.0, 10.0), Window::new(20.0, 30.0)])
        .expect("usable");
    assert_eq!(resource.window_count(), 2);
    assert!(!resource.is_available_at(150.0));
    assert!(resource.is_available_at(5.0));

    // Setting nothing removes all availability.
    resource.set_windows(&[]).expect("usable");
    assert!(resource.is_empty());
}

#[test]
fn availability_within_a_query_is_the_intersection() {
    let resource = station();
    let within = resource.availability_within(Interval::new(150.0, 250.0));
    assert_eq!(within, vec![Window::new(150.0, 200.0)]);

    let outside = resource.availability_within(Interval::new(300.0, 400.0));
    assert!(outside.is_empty());

    let covering = resource.availability_within(Interval::new(0.0, 1000.0));
    assert_eq!(covering, resource.windows().to_vec());
}

#[test]
fn the_earliest_fit_is_the_first_window_long_enough() {
    let mut resource = Resource::new(ResourceId::new(0), 1.0);
    resource
        .set_windows(&[Window::new(0.0, 10.0), Window::new(30.0, 60.0)])
        .expect("usable");

    // A request of five seconds fits in the first window.
    assert_eq!(resource.earliest_fit(5.0), Some(Window::new(0.0, 5.0)));
    // A request of twenty does not, so it takes the second.
    assert_eq!(resource.earliest_fit(20.0), Some(Window::new(30.0, 50.0)));
    // A request longer than any window has no answer.
    assert_eq!(resource.earliest_fit(40.0), None);
    // An instantaneous request fits at the start of the first window.
    assert_eq!(resource.earliest_fit(0.0), Some(Window::new(0.0, 0.0)));
    assert_eq!(resource.earliest_fit(-1.0), None);
    assert_eq!(resource.earliest_fit(f64::NAN), None);
}

#[test]
fn the_earliest_fit_after_a_time_respects_the_lower_bound() {
    let mut resource = Resource::new(ResourceId::new(0), 1.0);
    resource
        .set_windows(&[Window::new(0.0, 10.0), Window::new(30.0, 60.0)])
        .expect("usable");

    // Starting no earlier than five, the first window can still hold it.
    assert_eq!(
        resource.earliest_fit_after(5.0, 4.0),
        Some(Window::new(5.0, 9.0))
    );
    // A request of eight seconds starting at five does not fit, because the
    // window ends at ten, so the second window is used.
    assert_eq!(
        resource.earliest_fit_after(5.0, 8.0),
        Some(Window::new(30.0, 38.0))
    );
    // Starting after the second window begins, the request starts there.
    assert_eq!(
        resource.earliest_fit_after(40.0, 10.0),
        Some(Window::new(40.0, 50.0))
    );
    assert_eq!(resource.earliest_fit_after(100.0, 1.0), None);
    assert_eq!(resource.earliest_fit_after(f64::NAN, 1.0), None);
}

#[test]
fn the_busy_until_time_is_the_end_of_the_containing_window() {
    let resource = station();
    assert_eq!(resource.busy_until(150.0), Some(200.0));
    assert_eq!(resource.busy_until(50.0), None);
}

#[test]
fn the_capacity_is_validated() {
    let mut resource = Resource::new(ResourceId::new(0), 1.0);
    assert!(resource.has_valid_capacity());
    assert!(resource.set_capacity(2.5).is_ok());
    assert_eq!(resource.capacity(), 2.5);

    for capacity in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        let error = resource
            .set_capacity(capacity)
            .expect_err("the capacity is not usable");
        assert_eq!(error.kind(), ResourceErrorKind::InvalidCapacity);
    }
    // The capacity is unchanged after a failed set.
    assert_eq!(resource.capacity(), 2.5);

    let invalid = Resource::new(ResourceId::new(0), -1.0);
    assert!(!invalid.has_valid_capacity());
}

#[test]
fn a_set_holds_resources_by_identifier() {
    let mut set = ResourceSet::new();
    assert!(set.is_empty());
    set.insert(station()).expect("the identifier is new");
    set.insert(Resource::new(ResourceId::new(1), 2.0))
        .expect("the identifier is new");
    assert_eq!(set.len(), 2);

    let found = set.get(ResourceId::new(0)).expect("the resource exists");
    assert_eq!(found.capacity(), 1.0);
    assert!(set.get(ResourceId::new(9)).is_none());
    assert_eq!(set.resources().len(), 2);
}

#[test]
fn a_duplicate_identifier_is_rejected() {
    let mut set = ResourceSet::new();
    set.insert(station()).expect("the identifier is new");
    let error = set
        .insert(Resource::new(ResourceId::new(0), 5.0))
        .expect_err("the identifier is taken");
    assert_eq!(error.kind(), ResourceErrorKind::DuplicateResource);
    assert_eq!(set.len(), 1);
}

#[test]
fn a_set_can_be_built_from_a_list() {
    let set = ResourceSet::from_resources(vec![station(), Resource::new(ResourceId::new(1), 1.0)])
        .expect("the identifiers are distinct");
    assert_eq!(set.len(), 2);

    let error =
        ResourceSet::from_resources(vec![station(), Resource::new(ResourceId::new(0), 1.0)])
            .expect_err("the identifiers collide");
    assert_eq!(error.kind(), ResourceErrorKind::DuplicateResource);
}

#[test]
fn a_resource_can_be_removed() {
    let mut set = ResourceSet::new();
    set.insert(station()).expect("new");
    let removed = set.remove(ResourceId::new(0)).expect("the resource exists");
    assert_eq!(removed.id, ResourceId::new(0));
    assert!(set.is_empty());
    assert_eq!(
        set.remove(ResourceId::new(0)).expect_err("gone").kind(),
        ResourceErrorKind::NoSuchResource
    );
}

#[test]
fn a_resource_can_be_modified_through_the_set() {
    let mut set = ResourceSet::new();
    set.insert(station()).expect("new");
    let resource = set
        .get_mut(ResourceId::new(0))
        .expect("the resource exists");
    resource
        .add_window(Window::new(300.0, 400.0))
        .expect("usable");
    assert_eq!(
        set.get(ResourceId::new(0)).expect("exists").window_count(),
        2
    );
    assert!(set.get_mut(ResourceId::new(9)).is_none());
}

#[test]
fn the_set_reports_availability() {
    let mut set = ResourceSet::new();
    set.insert(station()).expect("new");
    set.insert(Resource::new(ResourceId::new(1), 1.0))
        .expect("new");

    // Only the first resource has any availability.
    assert_eq!(set.available_at(150.0), vec![ResourceId::new(0)]);
    assert!(set.available_at(50.0).is_empty());
    assert_eq!(set.available_over(120.0, 50.0), vec![ResourceId::new(0)]);
    assert!(set.available_over(120.0, 200.0).is_empty());
    assert_eq!(set.total_availability(), 100.0);
}

#[test]
fn the_identifier_displays_its_index() {
    assert_eq!(ResourceId::new(4).to_string(), "resource 4");
    assert_eq!(ResourceId::new(4).index(), 4);
    assert_eq!(ResourceId::new(4), ResourceId(4));
}

#[test]
fn identifiers_are_ordered() {
    // Ordering lets a caller sort a list of resources deterministically.
    let mut ids = [ResourceId::new(3), ResourceId::new(1), ResourceId::new(2)];
    ids.sort();
    assert_eq!(
        ids,
        [ResourceId::new(1), ResourceId::new(2), ResourceId::new(3)]
    );
}
