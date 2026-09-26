//! Resources, their availability, and their capacity.

use crate::error::{ResourceError, ResourceErrorKind};
use crate::window::{Interval, Window, merge_windows, subtract_window};

/// An identifier for a resource.
///
/// The identifier is an index chosen by the caller. It is deliberately opaque
/// to this crate so that a caller can use whatever numbering its own model
/// provides.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ResourceId(pub usize);

impl ResourceId {
    /// Creates an identifier.
    pub const fn new(index: usize) -> Self {
        Self(index)
    }

    /// Returns the index.
    pub const fn index(self) -> usize {
        self.0
    }
}

impl core::fmt::Display for ResourceId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "resource {}", self.0)
    }
}

/// A resource with availability windows and a capacity.
///
/// The capacity is a positive number whose unit the caller chooses. A task
/// that occupies more than the capacity of a resource cannot be scheduled on
/// it at all; two tasks conflict when their demands together exceed the
/// capacity at some instant.
#[derive(Debug, Clone, PartialEq)]
pub struct Resource {
    /// The identifier of this resource.
    pub id: ResourceId,
    /// The capacity, in a unit the caller chooses.
    ///
    /// A capacity of one with unit demands gives an exclusive resource, where
    /// no two tasks overlap. A larger capacity, or fractional demands, gives
    /// a shared resource.
    capacity: f64,
    /// The availability windows, kept merged and in time order.
    windows: Vec<Window>,
}

impl Resource {
    /// Creates a resource with no availability and the given capacity.
    ///
    /// A resource with no windows is never available, so a caller adds
    /// windows before using it. That is the safe default: a resource whose
    /// availability was forgotten is unavailable rather than always
    /// available.
    pub const fn new(id: ResourceId, capacity: f64) -> Self {
        Self {
            id,
            capacity,
            windows: Vec::new(),
        }
    }

    /// Returns the capacity.
    pub const fn capacity(&self) -> f64 {
        self.capacity
    }

    /// Sets the capacity.
    ///
    /// Returns an error when the capacity is not positive and finite.
    pub fn set_capacity(&mut self, capacity: f64) -> Result<(), ResourceError> {
        if !capacity.is_finite() || capacity <= 0.0 {
            return Err(ResourceError::new(ResourceErrorKind::InvalidCapacity)
                .with_detail(format!("capacity {capacity}")));
        }
        self.capacity = capacity;
        Ok(())
    }

    /// Returns `true` when the capacity is usable.
    pub fn has_valid_capacity(&self) -> bool {
        self.capacity.is_finite() && self.capacity > 0.0
    }

    /// Returns the availability windows, merged and in time order.
    pub fn windows(&self) -> &[Window] {
        &self.windows
    }

    /// Returns the number of availability windows.
    pub fn window_count(&self) -> usize {
        self.windows.len()
    }

    /// Returns `true` when the resource has no availability at all.
    pub fn is_empty(&self) -> bool {
        self.windows.is_empty()
    }

    /// Replaces the availability with a list of windows.
    ///
    /// The windows are merged, so the stored form is canonical whatever order
    /// the caller supplies them in.
    ///
    /// Returns an error when a window is not usable.
    pub fn set_windows(&mut self, windows: &[Window]) -> Result<(), ResourceError> {
        self.windows = merge_windows(windows)?;
        Ok(())
    }

    /// Adds one availability window.
    ///
    /// The window is merged into the existing set.
    ///
    /// Returns an error when the window is not usable.
    pub fn add_window(&mut self, window: Window) -> Result<(), ResourceError> {
        window.validate()?;
        let mut all = self.windows.clone();
        all.push(window);
        self.windows = merge_windows(&all)?;
        Ok(())
    }

    /// Removes an availability window.
    ///
    /// The window is subtracted from the existing set, which may split a
    /// stored window in two.
    ///
    /// Returns an error when the window is not usable.
    pub fn remove_window(&mut self, removed: Window) -> Result<(), ResourceError> {
        self.windows = subtract_window(&self.windows, removed)?;
        Ok(())
    }

    /// Returns `true` when the resource is available at an instant.
    ///
    /// An instantaneous query is a query of zero duration, which is satisfied
    /// when the instant is inside a window. At the end of a window the
    /// resource is not available, because the window is half-open.
    pub fn is_available_at(&self, time: f64) -> bool {
        self.windows
            .iter()
            .any(|window| window.interval.contains(time))
    }

    /// Returns `true` when the resource is available for a whole interval.
    ///
    /// A resource can satisfy a request of zero duration whenever it is
    /// available at the requested instant, so a zero duration is checked as
    /// an instant.
    pub fn is_available_over(&self, start: f64, duration: f64) -> bool {
        if !start.is_finite() || !duration.is_finite() || duration < 0.0 {
            return false;
        }
        if duration == 0.0 {
            return self.is_available_at(start);
        }
        let requested = Interval::new(start, start + duration);
        self.windows
            .iter()
            .any(|window| window.interval.contains_interval(&requested))
    }

    /// Returns the availability that falls inside an interval.
    ///
    /// The result is the intersection of the stored windows with the query,
    /// which is what a scheduler needs to know when it asks what part of its
    /// planning horizon a resource is usable for.
    pub fn availability_within(&self, query: Interval) -> Vec<Window> {
        self.windows
            .iter()
            .filter_map(|window| window.interval.intersection(&query))
            .map(Window::from_interval)
            .collect()
    }

    /// Returns the total availability, in seconds.
    ///
    /// The windows are disjoint, so the durations add.
    pub fn total_availability(&self) -> f64 {
        self.windows.iter().map(Window::duration).sum()
    }

    /// Returns the first window that can hold a request.
    ///
    /// This is the earliest time at which the resource can satisfy a request
    /// of the given duration, which is what a greedy scheduler asks. Returns
    /// `None` when no window is long enough.
    pub fn earliest_fit(&self, duration: f64) -> Option<Window> {
        if !duration.is_finite() || duration < 0.0 {
            return None;
        }
        self.windows
            .iter()
            .find(|window| window.duration() >= duration)
            .map(|window| Window::new(window.start(), window.start() + duration))
    }

    /// Returns the first window that can hold a request starting no earlier
    /// than a given time.
    ///
    /// Returns `None` when no window is both late enough and long enough.
    pub fn earliest_fit_after(&self, not_before: f64, duration: f64) -> Option<Window> {
        if !not_before.is_finite() || !duration.is_finite() || duration < 0.0 {
            return None;
        }
        self.windows.iter().find_map(|window| {
            let start = window.start().max(not_before);
            let end = start + duration;
            (end <= window.end()).then(|| Window::new(start, end))
        })
    }

    /// Returns the time at which the resource stops being available.
    ///
    /// Returns `None` when the resource is not available at the given
    /// instant, and infinity when the window that contains it is unbounded.
    pub fn busy_until(&self, time: f64) -> Option<f64> {
        self.windows
            .iter()
            .find(|window| window.interval.contains(time))
            .map(|window| window.end())
    }
}

/// A collection of resources, indexed by identifier.
///
/// The set keeps the resources in the order they were added and provides the
/// lookups a scheduler needs: by identifier, and by the resource that has a
/// window covering an instant.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ResourceSet {
    resources: Vec<Resource>,
}

impl ResourceSet {
    /// Creates an empty set.
    pub const fn new() -> Self {
        Self {
            resources: Vec::new(),
        }
    }

    /// Creates a set from a list of resources.
    ///
    /// Returns an error when two resources share an identifier, because a
    /// lookup would then be ambiguous.
    pub fn from_resources(resources: Vec<Resource>) -> Result<Self, ResourceError> {
        let mut set = Self::new();
        for resource in resources {
            set.insert(resource)?;
        }
        Ok(set)
    }

    /// Inserts a resource.
    ///
    /// Returns an error when the identifier is already present.
    pub fn insert(&mut self, resource: Resource) -> Result<(), ResourceError> {
        if self.get(resource.id).is_some() {
            return Err(ResourceError::new(ResourceErrorKind::DuplicateResource)
                .with_detail(format!("{}", resource.id)));
        }
        self.resources.push(resource);
        Ok(())
    }

    /// Returns the resource with the given identifier.
    pub fn get(&self, id: ResourceId) -> Option<&Resource> {
        self.resources.iter().find(|resource| resource.id == id)
    }

    /// Returns the resource with the given identifier for modification.
    pub fn get_mut(&mut self, id: ResourceId) -> Option<&mut Resource> {
        self.resources.iter_mut().find(|resource| resource.id == id)
    }

    /// Removes a resource.
    ///
    /// Returns an error when no resource has the identifier.
    pub fn remove(&mut self, id: ResourceId) -> Result<Resource, ResourceError> {
        let index = self
            .resources
            .iter()
            .position(|resource| resource.id == id)
            .ok_or_else(|| {
                ResourceError::new(ResourceErrorKind::NoSuchResource).with_detail(format!("{id}"))
            })?;
        Ok(self.resources.remove(index))
    }

    /// Returns the resources in the order they were added.
    pub fn resources(&self) -> &[Resource] {
        &self.resources
    }

    /// Returns the number of resources.
    pub fn len(&self) -> usize {
        self.resources.len()
    }

    /// Returns `true` when there are no resources.
    pub fn is_empty(&self) -> bool {
        self.resources.is_empty()
    }

    /// Returns the identifiers of the resources available at an instant.
    ///
    /// The result is in the order the resources were added.
    pub fn available_at(&self, time: f64) -> Vec<ResourceId> {
        self.resources
            .iter()
            .filter(|resource| resource.is_available_at(time))
            .map(|resource| resource.id)
            .collect()
    }

    /// Returns the identifiers of the resources available for a whole
    /// interval.
    pub fn available_over(&self, start: f64, duration: f64) -> Vec<ResourceId> {
        self.resources
            .iter()
            .filter(|resource| resource.is_available_over(start, duration))
            .map(|resource| resource.id)
            .collect()
    }

    /// Returns the total availability of every resource, in seconds.
    pub fn total_availability(&self) -> f64 {
        self.resources
            .iter()
            .map(Resource::total_availability)
            .sum()
    }
}
