//! Resources, their availability, and their capacity.
//!
//! 资源、其可用性与容量。

use crate::error::{ResourceError, ResourceErrorKind};
use crate::window::{Interval, Window, merge_windows, subtract_window};

/// An identifier for a resource.
///
/// The identifier is an index chosen by the caller. It is deliberately opaque
/// to this crate so that a caller can use whatever numbering its own model
/// provides.
///
/// 资源的标识符。
///
/// 标识符是由调用方选定的下标。它对本 crate 刻意保持不透明，以便调用方
/// 使用其自身模型所提供的任何编号方式。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ResourceId(pub usize);

impl ResourceId {
    /// Creates an identifier.
    ///
    /// 创建一个标识符。
    #[must_use]
    pub const fn new(index: usize) -> Self {
        Self(index)
    }

    /// Returns the index.
    ///
    /// 返回该下标。
    #[must_use]
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
///
/// 带有可用窗口与容量的资源。
///
/// 容量是由调用方选定单位的正数。占用量超过某资源容量的任务根本无法在其上
/// 调度；当两个任务的需求之和在某瞬超出容量时，二者冲突。
#[derive(Debug, Clone, PartialEq)]
pub struct Resource {
    /// The identifier of this resource.
    ///
    /// 该资源的标识符。
    pub id: ResourceId,
    /// The capacity, in a unit the caller chooses.
    ///
    /// A capacity of one with unit demands gives an exclusive resource, where
    /// no two tasks overlap. A larger capacity, or fractional demands, gives
    /// a shared resource.
    ///
    /// 容量，单位由调用方选择。
    ///
    /// 容量为一且需求为单位量时得到独占资源，任意两个任务都不重叠。更大的
    /// 容量或分数需求则得到共享资源。
    capacity: f64,
    /// The availability windows, kept merged and in time order.
    ///
    /// 可用窗口，保持合并且按时间有序。
    windows: Vec<Window>,
}

impl Resource {
    /// Creates a resource with no availability and the given capacity.
    ///
    /// A resource with no windows is never available, so a caller adds
    /// windows before using it. That is the safe default: a resource whose
    /// availability was forgotten is unavailable rather than always
    /// available.
    ///
    /// 创建无可用性、容量给定的资源。
    ///
    /// 没有窗口的资源永不可用，故调用方须先添加窗口再使用。这是安全的
    /// 默认值：忘记设置可用性的资源是不可用，而非始终可用。
    #[must_use]
    pub const fn new(id: ResourceId, capacity: f64) -> Self {
        Self {
            id,
            capacity,
            windows: Vec::new(),
        }
    }

    /// Returns the capacity.
    ///
    /// 返回该容量。
    #[must_use]
    pub const fn capacity(&self) -> f64 {
        self.capacity
    }

    /// Sets the capacity.
    ///
    /// Returns an error when the capacity is not positive and finite.
    ///
    /// 设置容量。
    ///
    /// 当容量非正或非有限时返回错误。
    pub fn set_capacity(&mut self, capacity: f64) -> Result<(), ResourceError> {
        if !capacity.is_finite() || capacity <= 0.0 {
            return Err(ResourceError::new(ResourceErrorKind::InvalidCapacity)
                .with_detail(format!("capacity {capacity}")));
        }
        self.capacity = capacity;
        Ok(())
    }

    /// Returns `true` when the capacity is usable.
    ///
    /// 当该容量可用时返回 `true`。
    #[must_use]
    pub fn has_valid_capacity(&self) -> bool {
        self.capacity.is_finite() && self.capacity > 0.0
    }

    /// Returns the availability windows, merged and in time order.
    ///
    /// 返回可用窗口，已合并且按时间有序。
    #[must_use]
    pub fn windows(&self) -> &[Window] {
        &self.windows
    }

    /// Returns the number of availability windows.
    ///
    /// 返回可用窗口的数量。
    #[must_use]
    pub fn window_count(&self) -> usize {
        self.windows.len()
    }

    /// Returns `true` when the resource has no availability at all.
    ///
    /// 当该资源完全没有可用性时返回 `true`。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.windows.is_empty()
    }

    /// Replaces the availability with a list of windows.
    ///
    /// The windows are merged, so the stored form is canonical whatever order
    /// the caller supplies them in.
    ///
    /// Returns an error when a window is not usable.
    ///
    /// 用一组窗口替换可用性。
    ///
    /// 窗口会被合并，故无论调用方以何种顺序给出，存储形式都是规范的。
    ///
    /// 当某个窗口不可用时返回错误。
    pub fn set_windows(&mut self, windows: &[Window]) -> Result<(), ResourceError> {
        self.windows = merge_windows(windows)?;
        Ok(())
    }

    /// Adds one availability window.
    ///
    /// The window is merged into the existing set.
    ///
    /// Returns an error when the window is not usable.
    ///
    /// 添加一个可用窗口。
    ///
    /// 该窗口被合并进已有集合。
    ///
    /// 当该窗口不可用时返回错误。
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
    ///
    /// 移除一个可用窗口。
    ///
    /// 该窗口从已有集合中被减去，可能把某个已存窗口一分为二。
    ///
    /// 当该窗口不可用时返回错误。
    pub fn remove_window(&mut self, removed: Window) -> Result<(), ResourceError> {
        self.windows = subtract_window(&self.windows, removed)?;
        Ok(())
    }

    /// Returns `true` when the resource is available at an instant.
    ///
    /// An instantaneous query is a query of zero duration, which is satisfied
    /// when the instant is inside a window. At the end of a window the
    /// resource is not available, because the window is half-open.
    ///
    /// 当该资源在某瞬可用时返回 `true`。
    ///
    /// 瞬时查询即零时长查询，当该瞬落在某个窗口内即得到满足。在窗口
    /// 终点处该资源不可用，因为窗口是半开的。
    #[must_use]
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
    ///
    /// 当该资源在整段区间上可用时返回 `true`。
    ///
    /// 只要资源在所请求瞬时可用，即可满足零时长请求，故零时长按瞬时检查。
    #[must_use]
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
    ///
    /// 返回落在某区间内的可用性。
    ///
    /// 结果是已存窗口与该查询的交集，这正是调度器在询问资源的规划时域中
    /// 哪一段可用时所需的信息。
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
    ///
    /// 返回总可用时长，单位为秒。
    ///
    /// 各窗口互不相交，故时长可直接相加。
    pub fn total_availability(&self) -> f64 {
        self.windows.iter().map(Window::duration).sum()
    }

    /// Returns the first window that can hold a request.
    ///
    /// This is the earliest time at which the resource can satisfy a request
    /// of the given duration, which is what a greedy scheduler asks. Returns
    /// `None` when no window is long enough.
    ///
    /// 返回第一个能容纳某请求的窗口。
    ///
    /// 这是该资源能满足给定时长请求的最早时刻，即贪心调度器所询问的量。
    /// 没有足够长的窗口时返回 `None`。
    #[must_use]
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
    ///
    /// 返回第一个能容纳请求的窗口，且该请求的起点不早于给定时刻。
    ///
    /// 没有既够晚又够长的窗口时返回 `None`。
    #[must_use]
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
    ///
    /// 返回该资源不再可用的时刻。
    ///
    /// 该资源在所给瞬时不可用时返回 `None`；包含该瞬的窗口无界时返回无穷。
    #[must_use]
    pub fn busy_until(&self, time: f64) -> Option<f64> {
        self.windows
            .iter()
            .find(|window| window.interval.contains(time))
            .map(super::window::Window::end)
    }
}

/// A collection of resources, indexed by identifier.
///
/// The set keeps the resources in the order they were added and provides the
/// lookups a scheduler needs: by identifier, and by the resource that has a
/// window covering an instant.
///
/// 以标识符为索引的资源集合。
///
/// 该集合按添加顺序保存资源，并提供调度器所需的查找：按标识符，以及按
/// 某个瞬时上有窗口覆盖的资源。
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ResourceSet {
    resources: Vec<Resource>,
}

impl ResourceSet {
    /// Creates an empty set.
    ///
    /// 创建一个空集合。
    #[must_use]
    pub const fn new() -> Self {
        Self {
            resources: Vec::new(),
        }
    }

    /// Creates a set from a list of resources.
    ///
    /// Returns an error when two resources share an identifier, because a
    /// lookup would then be ambiguous.
    ///
    /// 由一组资源创建集合。
    ///
    /// 两个资源标识符相同时返回错误，因为查找将有歧义。
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
    ///
    /// 插入一个资源。
    ///
    /// 该标识符已存在时返回错误。
    pub fn insert(&mut self, resource: Resource) -> Result<(), ResourceError> {
        if self.get(resource.id).is_some() {
            return Err(ResourceError::new(ResourceErrorKind::DuplicateResource)
                .with_detail(format!("{}", resource.id)));
        }
        self.resources.push(resource);
        Ok(())
    }

    /// Returns the resource with the given identifier.
    ///
    /// 返回具有所给标识符的资源。
    #[must_use]
    pub fn get(&self, id: ResourceId) -> Option<&Resource> {
        self.resources.iter().find(|resource| resource.id == id)
    }

    /// Returns the resource with the given identifier for modification.
    ///
    /// 返回具有所给标识符的资源以供修改。
    pub fn get_mut(&mut self, id: ResourceId) -> Option<&mut Resource> {
        self.resources.iter_mut().find(|resource| resource.id == id)
    }

    /// Removes a resource.
    ///
    /// Returns an error when no resource has the identifier.
    ///
    /// 移除一个资源。
    ///
    /// 没有资源具有该标识符时返回错误。
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
    ///
    /// 返回按添加顺序排列的资源。
    #[must_use]
    pub fn resources(&self) -> &[Resource] {
        &self.resources
    }

    /// Returns the number of resources.
    ///
    /// 返回资源数量。
    #[must_use]
    pub fn len(&self) -> usize {
        self.resources.len()
    }

    /// Returns `true` when there are no resources.
    ///
    /// 当集合中没有资源时返回 `true`。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.resources.is_empty()
    }

    /// Returns the identifiers of the resources available at an instant.
    ///
    /// The result is in the order the resources were added.
    ///
    /// 返回在某瞬可用的资源标识符。
    ///
    /// 结果按资源添加顺序排列。
    #[must_use]
    pub fn available_at(&self, time: f64) -> Vec<ResourceId> {
        self.resources
            .iter()
            .filter(|resource| resource.is_available_at(time))
            .map(|resource| resource.id)
            .collect()
    }

    /// Returns the identifiers of the resources available for a whole
    /// interval.
    ///
    /// 返回在整段区间上可用的资源标识符。
    #[must_use]
    pub fn available_over(&self, start: f64, duration: f64) -> Vec<ResourceId> {
        self.resources
            .iter()
            .filter(|resource| resource.is_available_over(start, duration))
            .map(|resource| resource.id)
            .collect()
    }

    /// Returns the total availability of every resource, in seconds.
    ///
    /// 返回每个资源的总可用时长，单位为秒。
    pub fn total_availability(&self) -> f64 {
        self.resources
            .iter()
            .map(Resource::total_availability)
            .sum()
    }
}
