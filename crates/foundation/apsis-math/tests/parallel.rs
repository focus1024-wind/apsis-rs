//! Tests for the scoped parallelism helpers.
//!
//! The central property is that a parallel run and a serial run produce the
//! same values, because the whole point of the helpers is to be a drop-in for
//! the sequential loop they replace. Each test therefore compares against the
//! sequential answer rather than against a constant, and forces both the
//! single and multi thread paths by choosing lengths that fall on each side
//! of the threshold.
//!
//! 作用域并行辅助函数的测试。
//!
//! 核心性质是并行运行与串行运行产生相同的值，因为这些辅助函数的全部意义就是作为
//! 它们所替代的顺序循环的替代品。故每个测试都与顺序答案比较而非与常量比较，并
//! 通过选取落在阈值两侧的长度，强制走过单线程与多线程两条路径。

use std::sync::atomic::Ordering;

use apsis_math::parallel::{for_each_index, map_index, thread_count, threads_for};

#[test]
fn the_thread_count_is_at_least_one() {
    assert!(thread_count() >= 1);
}

#[test]
fn small_work_uses_a_single_thread() {
    // Below the per thread threshold there is nothing to gain from starting
    // a thread, so the helper says so.
    // 低于每线程阈值时，启动线程没有任何收益，故该辅助函数如此表示。
    assert_eq!(threads_for(0), 1);
    assert_eq!(threads_for(1), 1);
    assert_eq!(threads_for(64), 1);
}

#[test]
fn large_work_uses_more_than_one_thread() {
    let count = thread_count();
    if count > 1 {
        assert!(threads_for(1_000_000) > 1);
        // Never more threads than there are items.
        // 线程数绝不超过项数。
        assert!(threads_for(10) <= 10);
    }
}

#[test]
fn every_index_is_visited_exactly_once() {
    use std::sync::atomic::AtomicUsize;
    for &len in &[0_usize, 1, 7, 1_000, 5_000, 100_000] {
        let visits = (0..len).map(|_| AtomicUsize::new(0)).collect::<Vec<_>>();
        for_each_index(len, |index| {
            visits[index].fetch_add(1, Ordering::Relaxed);
        });
        assert!(
            visits
                .iter()
                .all(|count| count.load(Ordering::Relaxed) == 1),
            "length {len} visited an index more than once"
        );
    }
}

#[test]
fn the_parallel_result_matches_the_sequential_one() {
    for &len in &[0_usize, 1, 1_000, 50_000, 300_000] {
        let sequential: Vec<f64> = (0..len)
            .map(|index| (index as f64).sin() * 1_000.0)
            .collect();
        let parallel = map_index(len, |index| (index as f64).sin() * 1_000.0);
        assert_eq!(parallel, sequential, "length {len} differs");
    }
}

#[test]
fn the_output_is_in_index_order() {
    // Threads finish out of order, so this checks that the helper restores
    // index order rather than completion order.
    // 各线程完成次序不定，故这检查该辅助函数恢复的是下标次序而非完成次序。
    let len = 200_000;
    let values = map_index(len, |index| index);
    for (position, value) in values.iter().enumerate() {
        assert_eq!(*value, position);
    }
}

#[test]
fn a_closure_may_capture_shared_input() {
    // The shape every caller in the workspace uses: read a shared slice and
    // produce one value per element. `map_index` owns the output buffer, so
    // no shared mutable state is needed.
    // 本工作区中每个调用方所用的形状：读取共享切片并为每个元素产生一个值。
    // `map_index` 拥有输出缓冲，故无需共享可变状态。
    let input: Vec<f64> = (0..100_000).map(f64::from).collect();
    let output = map_index(input.len(), |index| input[index] * input[index]);
    for index in [0, 1, 999, 50_000, 99_999] {
        assert_eq!(output[index], input[index] * input[index]);
    }
}

#[test]
fn a_closure_may_accumulate_atomically() {
    // Writing to a shared counter needs atomics; this checks the helper
    // tolerates that, since summing a large range is a common pattern.
    // 写共享计数器需要原子操作；这检查该辅助函数容忍它，因为对大区间求和是常见
    // 模式。
    use std::sync::atomic::AtomicU64;
    let total = AtomicU64::new(0);
    let len = 200_000_u64;
    for_each_index(len as usize, |index| {
        total.fetch_add(index as u64, Ordering::Relaxed);
    });
    let expected: u64 = (0..len).sum();
    assert_eq!(total.load(Ordering::Relaxed), expected);
}

#[test]
fn an_empty_range_does_nothing() {
    let empty: Vec<f64> = map_index(0, |_| 1.0);
    assert!(empty.is_empty());
    // The index form accepts an empty range without spawning anything.
    // 索引形式接受空区间而不创建任何线程。
    for_each_index(0, |_| unreachable!("an empty range runs no closure"));
}

#[test]
fn a_range_that_does_not_divide_evenly_covers_everything() {
    // A length that is not a multiple of the thread count is where an
    // off by one in the chunking would drop the tail.
    // 长度不是线程数的整数倍之处，正是分块中差一会丢掉尾部的地方。
    use std::sync::atomic::AtomicUsize;
    for &len in &[1_001_usize, 4_097, 65_537, 100_003] {
        let marks = (0..len).map(|_| AtomicUsize::new(0)).collect::<Vec<_>>();
        for_each_index(len, |index| {
            marks[index].store(1, Ordering::Relaxed);
        });
        assert!(
            marks.iter().all(|mark| mark.load(Ordering::Relaxed) == 1),
            "length {len} missed an index"
        );
    }
}

#[test]
fn values_of_a_non_copy_type_survive_the_collect() {
    // A string is not `Copy`, so this exercises the unwrap-and-move path that
    // the parallel branch uses.
    // 字符串不是 `Copy`，故这检验并行分支所用的解包并移动路径。
    let values = map_index(10_000, |index| format!("item-{index}"));
    assert_eq!(values[0], "item-0");
    assert_eq!(values[9_999], "item-9999");
    assert_eq!(values.len(), 10_000);
}
