//! The schedule itself.
//!
//! 调度本身。

use crate::error::{ScheduleError, ScheduleErrorKind};
use crate::placement::{Evaluation, Placement, ResourceUsage};
use apsis_resource::ResourceId;
use apsis_task::TaskId;
use std::collections::BTreeMap;

/// A set of task placements.
///
/// The schedule stores placements in the order they were made and keeps an
/// index from task to placement, so that a lookup by task is direct. A task
/// may be placed at most once: placing it again is an error rather than a
/// silent overwrite, because a schedule that contains two placements for one
/// task is not a schedule.
///
/// 一组任务放置。
///
/// 该计划按放置发生的顺序保存各项，并维护从任务到放置的索引，故按任务查找
/// 是直接的。一个任务至多放置一次：重复放置是错误而非静默覆盖，因为含有
/// 同一任务两处放置的集合已不是一份计划。
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Schedule {
    placements: Vec<Placement>,
    index: BTreeMap<usize, usize>,
}

impl Schedule {
    /// Creates an empty schedule.
    ///
    /// 创建一份空计划。
    #[must_use]
    pub fn new() -> Self {
        Self {
            placements: Vec::new(),
            index: BTreeMap::new(),
        }
    }

    /// Returns the placements in the order they were made.
    ///
    /// 返回按放置顺序排列的各项。
    #[must_use]
    pub fn placements(&self) -> &[Placement] {
        &self.placements
    }

    /// Returns the number of placements.
    ///
    /// 返回放置的数量。
    #[must_use]
    pub fn len(&self) -> usize {
        self.placements.len()
    }

    /// Returns `true` when there are no placements.
    ///
    /// 当没有放置时返回 `true`。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.placements.is_empty()
    }

    /// Returns `true` when a task has been placed.
    ///
    /// 当某任务已被放置时返回 `true`。
    #[must_use]
    pub fn is_placed(&self, task: TaskId) -> bool {
        self.index.contains_key(&task.index())
    }

    /// Returns the placement of a task.
    ///
    /// 返回某任务的放置。
    #[must_use]
    pub fn placement_of(&self, task: TaskId) -> Option<&Placement> {
        self.index
            .get(&task.index())
            .map(|position| &self.placements[*position])
    }

    /// Places a task at a time.
    ///
    /// The placement is checked against the task's window. The resources of
    /// the task are recorded with the placement through the task itself, so
    /// this method does not need them; the capacity check is performed by
    /// [`Schedule::validate_capacity`], which needs the resources.
    ///
    /// Returns an error when the task is already placed, when the start time
    /// is not finite, or when the placement falls outside the task's window.
    ///
    /// 把某任务放置在某时刻。
    ///
    /// 放置会对照该任务的窗口校验。任务的资源通过任务本身随放置一同记录，
    /// 故本方法无需它们；容量检查由需要资源的
    /// [`Schedule::validate_capacity`] 完成。
    ///
    /// 该任务已被放置、起始时刻非有限，或放置落在任务窗口之外时返回错误。
    ///
    /// # Panics
    ///
    /// Panics only if the internal index disagrees with the placement list,
    /// which cannot happen.
    ///
    /// 仅当内部索引与放置列表不一致时 panic，这不会发生。
    pub fn place(
        &mut self,
        task: &apsis_task::Task,
        start: f64,
    ) -> Result<&Placement, ScheduleError> {
        if !start.is_finite() {
            return Err(ScheduleError::new(ScheduleErrorKind::InvalidStart)
                .with_detail(format!("start {start}")));
        }
        if self.is_placed(task.id) {
            return Err(ScheduleError::new(ScheduleErrorKind::AlreadyPlaced)
                .with_detail(format!("{}", task.id)));
        }
        if start < task.earliest_start {
            return Err(
                ScheduleError::new(ScheduleErrorKind::BeforeWindow).with_detail(format!(
                    "task {} starts at {start}, before its window opens at {}",
                    task.id, task.earliest_start
                )),
            );
        }
        if start + task.duration > task.latest_end {
            return Err(
                ScheduleError::new(ScheduleErrorKind::AfterWindow).with_detail(format!(
                    "task {} ends at {}, after its window closes at {}",
                    task.id,
                    start + task.duration,
                    task.latest_end
                )),
            );
        }
        let placement = Placement::new(task.id, start, task.duration);
        self.index.insert(task.id.index(), self.placements.len());
        self.placements.push(placement);
        Ok(self
            .placements
            .last()
            .expect("the placement was just pushed"))
    }

    /// Removes the placement of a task.
    ///
    /// Returns an error when the task is not placed.
    ///
    /// 移除某任务的放置。
    ///
    /// 该任务未被放置时返回错误。
    pub fn unplace(&mut self, task: TaskId) -> Result<Placement, ScheduleError> {
        let position = self.index.remove(&task.index()).ok_or_else(|| {
            ScheduleError::new(ScheduleErrorKind::NotPlaced).with_detail(format!("{task}"))
        })?;
        let removed = self.placements.remove(position);
        // Every placement after the removed one shifts down by one, so the
        // index must be rebuilt. The schedule is small enough that this is
        // cheaper than maintaining a linked structure.
        // 被移除项之后的每项都前移一位，故索引须重建。计划规模足够小，
        // 这比维护链式结构更划算。
        self.rebuild_index();
        Ok(removed)
    }

    /// Rebuilds the task to position index.
    ///
    /// 重建从任务到位置的索引。
    fn rebuild_index(&mut self) {
        self.index.clear();
        for (position, placement) in self.placements.iter().enumerate() {
            self.index.insert(placement.task.index(), position);
        }
    }

    /// Returns the placements that use a resource, in time order.
    ///
    /// 返回使用某资源的放置，按时间排序。
    #[must_use]
    pub fn placements_on(
        &self,
        resource: ResourceId,
        tasks: &[apsis_task::Task],
    ) -> Vec<Placement> {
        let mut selected: Vec<Placement> = self
            .placements
            .iter()
            .filter(|placement| {
                tasks
                    .iter()
                    .find(|task| task.id == placement.task)
                    .is_some_and(|task| task.requires(resource))
            })
            .copied()
            .collect();
        selected.sort_by(|first, second| {
            first
                .start
                .partial_cmp(&second.start)
                .unwrap_or(core::cmp::Ordering::Equal)
        });
        selected
    }

    /// Returns `true` when two placements of different tasks overlap.
    ///
    /// This checks the schedule against itself, ignoring the resources. Two
    /// tasks on disjoint resources may legitimately overlap, so a caller that
    /// wants to know whether a resource is overbooked should use
    /// [`Schedule::validate_capacity`] instead.
    ///
    /// 当两个不同任务的放置重叠时返回 `true`。
    ///
    /// 这是计划与自身的比较，忽略资源。位于不相交资源上的两个任务本就可
    /// 以合法重叠，故欲知某资源是否超订的调用方应改用
    /// [`Schedule::validate_capacity`]。
    #[must_use]
    pub fn has_overlapping_placements(&self) -> bool {
        for (index, placement) in self.placements.iter().enumerate() {
            if self.placements[..index]
                .iter()
                .any(|earlier| earlier.overlaps(placement))
            {
                return true;
            }
        }
        false
    }

    /// Returns the total placed duration, in seconds.
    ///
    /// 返回已放置的总时长，单位为秒。
    #[must_use]
    pub fn scheduled_seconds(&self) -> f64 {
        self.placements
            .iter()
            .map(|placement| placement.duration)
            .sum()
    }

    /// Returns the earliest start of any placement.
    ///
    /// Returns `None` when the schedule is empty.
    ///
    /// 返回所有放置中最早的开始时刻。
    ///
    /// 计划为空时返回 `None`。
    #[must_use]
    pub fn earliest_start(&self) -> Option<f64> {
        self.placements
            .iter()
            .map(|placement| placement.start)
            .min_by(|first, second| {
                first
                    .partial_cmp(second)
                    .unwrap_or(core::cmp::Ordering::Equal)
            })
    }

    /// Returns the latest end of any placement.
    ///
    /// Returns `None` when the schedule is empty.
    ///
    /// 返回所有放置中最晚的结束时刻。
    ///
    /// 计划为空时返回 `None`。
    pub fn latest_end(&self) -> Option<f64> {
        self.placements
            .iter()
            .map(Placement::end)
            .max_by(|first, second| {
                first
                    .partial_cmp(second)
                    .unwrap_or(core::cmp::Ordering::Equal)
            })
    }

    /// Validates a schedule against a set of resources.
    ///
    /// The check has two parts. First every placement must agree with the
    /// requirements of its task, which catches a schedule built with a
    /// different task list. Then, for each resource, the total demand placed
    /// at any instant must not exceed the capacity.
    ///
    /// The second check walks the placement boundaries of each resource in
    /// time order, which is exact because the total demand is constant
    /// between boundaries.
    ///
    /// Returns an error describing the first violation.
    ///
    /// 对照一组资源校验该计划。
    ///
    /// 检查分两部分。其一，每处放置须与其任务的需求一致，这可捕获用另
    /// 一份任务列表建成的计划。其二，对每个资源，任意瞬时的总需求不得超
    /// 过容量。
    ///
    /// 第二部分按时间序遍历各资源的放置边界，由于相邻边界之间总需求恒定，
    /// 该遍历是精确的。
    ///
    /// 返回描述首个违规的错误。
    pub fn validate_capacity(
        &self,
        tasks: &[apsis_task::Task],
        resources: &apsis_resource::ResourceSet,
    ) -> Result<(), ScheduleError> {
        for placement in &self.placements {
            let task = tasks
                .iter()
                .find(|task| task.id == placement.task)
                .ok_or_else(|| {
                    ScheduleError::new(ScheduleErrorKind::RequirementMismatch)
                        .with_detail(format!("{} is not in the task list", placement.task))
                })?;
            if (placement.duration - task.duration).abs() > 1e-12 {
                return Err(ScheduleError::new(ScheduleErrorKind::RequirementMismatch)
                    .with_detail(format!(
                        "{} is placed for {} s but takes {} s",
                        task.id, placement.duration, task.duration
                    )));
            }
        }

        for resource in resources.resources() {
            // The boundaries at which the total demand can change.
            // 总需求可能发生变化的各边界。
            let mut events: Vec<(f64, f64)> = Vec::new();
            for placement in &self.placements {
                let Some(task) = tasks.iter().find(|task| task.id == placement.task) else {
                    continue;
                };
                if let Some(demand) = task.demand_for(resource.id) {
                    events.push((placement.start, demand));
                    events.push((placement.end(), -demand));
                }
            }
            if events.is_empty() {
                continue;
            }
            // Sort by time, with a release before an acquisition at the same
            // instant so that two placements that touch are not counted as
            // overlapping.
            // 按时间排序；同一瞬先释放后获取，使两个相接的放置不被计为
            // 重叠。
            events.sort_by(|first, second| {
                first
                    .0
                    .partial_cmp(&second.0)
                    .unwrap_or(core::cmp::Ordering::Equal)
                    .then_with(|| {
                        first
                            .1
                            .partial_cmp(&second.1)
                            .unwrap_or(core::cmp::Ordering::Equal)
                    })
            });

            let mut current = 0.0;
            for (time, change) in events {
                current += change;
                if current > resource.capacity() + 1e-9 {
                    return Err(ScheduleError::new(ScheduleErrorKind::CapacityExceeded)
                        .with_detail(format!(
                            "{} carries demand {} against capacity {} at {time}",
                            resource.id,
                            current,
                            resource.capacity()
                        )));
                }
            }
        }
        Ok(())
    }

    /// Evaluates the schedule against a task list and a resource set.
    ///
    /// The evaluation reports the counts, the makespan, and the usage of each
    /// resource. The peak demand of a resource is found by the same event
    /// sweep that the capacity check uses, so the two agree.
    ///
    /// 对照任务列表与资源集合评估该计划。
    ///
    /// 评估给出各项计数、makespan 以及每个资源的使用情况。资源的峰值
    /// 需求由与容量检查相同的事件扫掠求得，故二者一致。
    #[must_use]
    pub fn evaluate(
        &self,
        tasks: &[apsis_task::Task],
        resources: &apsis_resource::ResourceSet,
    ) -> Evaluation {
        let mut resource_usage = Vec::with_capacity(resources.len());
        for resource in resources.resources() {
            let mut events: Vec<(f64, f64)> = Vec::new();
            let mut placement_count = 0;
            for placement in &self.placements {
                let Some(task) = tasks.iter().find(|task| task.id == placement.task) else {
                    continue;
                };
                if let Some(demand) = task.demand_for(resource.id) {
                    events.push((placement.start, demand));
                    events.push((placement.end(), -demand));
                    placement_count += 1;
                }
            }
            events.sort_by(|first, second| {
                first
                    .0
                    .partial_cmp(&second.0)
                    .unwrap_or(core::cmp::Ordering::Equal)
                    .then_with(|| {
                        first
                            .1
                            .partial_cmp(&second.1)
                            .unwrap_or(core::cmp::Ordering::Equal)
                    })
            });

            let mut current = 0.0;
            let mut peak = 0.0_f64;
            let mut busy = 0.0_f64;
            let mut previous: Option<f64> = None;
            for (time, change) in events {
                if let Some(previous) = previous {
                    if current > 0.0 {
                        busy += time - previous;
                    }
                }
                current += change;
                peak = peak.max(current);
                previous = Some(time);
            }

            resource_usage.push(ResourceUsage {
                resource: resource.id,
                peak_demand: peak,
                busy_seconds: busy,
                placement_count,
            });
        }

        Evaluation {
            placed_count: self.placements.len(),
            offered_count: tasks.len(),
            scheduled_seconds: self.scheduled_seconds(),
            makespan_start: self.earliest_start().unwrap_or(0.0),
            makespan_end: self.latest_end().unwrap_or(0.0),
            resource_usage,
        }
    }

    /// Returns the schedule sorted by start time, leaving the original alone.
    ///
    /// A schedule is stored in the order it was built, which is the order a
    /// scheduler considered the tasks. A caller that wants to read the plan
    /// in time order uses this.
    ///
    /// 返回按开始时刻排序的计划副本，原计划不变。
    ///
    /// 计划按其构建顺序存储，即调度器考虑任务的顺序。欲按时间阅读该计
    /// 划的调用方使用本方法。
    #[must_use]
    pub fn in_time_order(&self) -> Vec<Placement> {
        let mut ordered = self.placements.clone();
        ordered.sort_by(|first, second| {
            first
                .start
                .partial_cmp(&second.start)
                .unwrap_or(core::cmp::Ordering::Equal)
                .then_with(|| first.task.cmp(&second.task))
        });
        ordered
    }

    /// Returns the time at which a resource becomes free if it is busy at a
    /// given instant.
    ///
    /// Returns `None` when the resource is idle there. This is what a
    /// sequential scheduler asks when it looks for the next gap.
    ///
    /// 若某资源在给定瞬时忙碌，返回其变为空闲的时刻。
    ///
    /// 该资源在彼处空闲时返回 `None`。顺序调度器寻找下一个空隙时正是
    /// 询问这一点。
    pub fn resource_busy_until(
        &self,
        resource: ResourceId,
        tasks: &[apsis_task::Task],
        time: f64,
    ) -> Option<f64> {
        self.placements
            .iter()
            .filter(|placement| {
                tasks
                    .iter()
                    .find(|task| task.id == placement.task)
                    .is_some_and(|task| task.requires(resource) && placement.covers(time))
            })
            .map(Placement::end)
            .max_by(|first, second| {
                first
                    .partial_cmp(second)
                    .unwrap_or(core::cmp::Ordering::Equal)
            })
    }
}
