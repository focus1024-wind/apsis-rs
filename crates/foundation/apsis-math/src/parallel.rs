//! Scoped parallelism built on the standard library alone.
//!
//! The workspace forbids third party crates, so a threading pool would have to
//! be written here. It is not: the operations in this workspace that benefit
//! from several cores are all embarrassingly parallel loops over independent
//! items, and the standard library's scoped threads express those directly.
//! `std::thread::scope` borrows its data, so a caller can share a slice or a
//! reader without an atomic reference count, and joins every thread before returning, so no
//! result escapes the scope still being written.
//!
//! The entry point is [`for_each_index`], which splits a range into one chunk
//! per thread and runs a closure over each element. The closure must be
//! callable from another thread, so anything it captures has to be `Sync`;
//! in practice that means reading shared input and writing to a disjoint
//! slot of the output, which is the shape every parallel loop here has.
//!
//! Because there is no pool, the cost of a parallel call includes starting
//! the threads. That is worth paying only when each thread has real work, so
//! [`thread_count`] reports what the machine offers and the callers in this
//! workspace use it to skip the parallel path for small inputs.
//!
//! ```
//! use apsis_math::parallel::map_index;
//!
//! // Square every element, one value per index.
//! let input: Vec<f64> = (0..1000).map(f64::from).collect();
//! let output = map_index(input.len(), |index| input[index] * input[index]);
//! assert_eq!(output[10], 100.0);
//! ```
//!
//! 仅建立在标准库之上的作用域并行。
//!
//! 工作区禁止第三方 crate，故线程池须在此编写。但它并未如此：本工作区中
//! 受益于多核的操作，全都是对独立项做可并行化的循环，而标准库的作用域线程
//! 直接表达这些。`std::thread::scope` 借用其数据，故调用方无需原子引用计数
//! 即可共享切片或读取器，并在返回前 join 每个线程，故没有任何结果会在仍被
//! 写入时逸出该作用域。
//!
//! 入口是 [`for_each_index`]，它把区间按线程切成若干块，并对每个元素运行
//! 一个闭包。该闭包必须可从另一线程调用，故它所捕获的任何东西都必须是
//! `Sync` 的；实践中这意味着读取共享输入并写入输出的互不相交槽位，这正是
//! 此处每个并行循环所具有的形状。
//!
//! 因为没有池，一次并行调用的成本包含启动线程。只有当每个线程都有实打实的
//! 工作时才值得付出，故 [`thread_count`] 报告机器所提供的数量，本工作区中
//! 的调用方用它来对小的输入跳过并行路径。

use core::hint;

/// The number of threads to use for a piece of work.
///
/// This is the machine's parallelism as reported by the standard library, at
/// least one. It is the number the callers use to decide whether a parallel
/// run is worth starting.
///
/// 某项工作应使用的线程数。
///
/// 这是标准库报告的机器并行度，至少为一。调用方据此决定并行运行是否值得启动。
pub fn thread_count() -> usize {
    std::thread::available_parallelism()
        .map(|count| count.get())
        .unwrap_or(1)
}

/// The number of threads that should run a loop over `len` items.
///
/// A loop shorter than the thread count would leave most threads with nothing
/// to do while still paying to start them, so this returns one thread until
/// there is enough work for a second. The threshold is a per item cost of
/// roughly a microsecond, below which starting a thread cannot pay for
/// itself.
///
/// 对 `len` 个元素的循环应使用的线程数。
///
/// 比线程数还短的循环会让多数线程无所事事，却仍要付出启动代价，因此在工作量
/// 足以分给第二个线程之前本函数返回一个线程。阈值是每项约一微秒的开销，低于
/// 此值启动线程无法收回成本。
pub fn threads_for(len: usize) -> usize {
    const ITEMS_PER_THREAD: usize = 1_024;
    thread_count().min(len.div_ceil(ITEMS_PER_THREAD)).max(1)
}

/// Runs a closure once for each index of a range, spread over several threads.
///
/// The range is split into one contiguous chunk per thread and each thread
/// runs the closure for the indices in its chunk. The closure may be called
/// from any thread, so it must be `Send` and `Sync`; anything it captures is
/// shared rather than copied. The call returns only after every thread has
/// finished, so the data the closure wrote is safe to read afterwards.
///
/// The closure is handed the index rather than the element, which keeps the
/// signature independent of the collection: the caller reads the input and
/// writes the output itself, and the two never overlap.
///
/// # Panics
///
/// If the closure panics, the panic is resumed on the calling thread after
/// the other threads have been joined, so a failure surfaces at the call
/// rather than in a detached thread.
///
/// 对一段范围内的每个下标各调用一次闭包，分摊到多个线程上执行。
///
/// 该范围被切分为每线程一段连续区间，每个线程对落在自己区间内的下标调用闭包。
/// 闭包可能在任意线程上被调用，故必须是 `Send` 与 `Sync`；它捕获的数据是被共享
/// 而非复制的。本次调用只有在全部线程结束后才返回，因此闭包写入的数据此后可安全
/// 读取。
///
/// 闭包收到的是下标而非元素，这让签名与具体集合无关：调用方自行读取输入并
/// 写出输出，两者不会重叠。
///
/// # Panics
///
/// 若闭包发生 panic，该 panic 会在其它线程 join 之后于调用线程上重新抛出，
/// 因此失败出现在调用处，而不是某个已分离的线程中。
pub fn for_each_index<F>(len: usize, operation: F)
where
    F: Fn(usize) + Sync,
{
    let threads = threads_for(len);
    if threads <= 1 {
        // A single thread would spawn and join one thread to do exactly what
        // this loop does, so it runs directly.
        // 单线程情况下，若走作用域仍要生成并 join 一个线程来做与本循环完全
        // 相同的事，故直接执行。
        for index in 0..len {
            operation(index);
        }
        return;
    }
    let chunk = len.div_ceil(threads);
    std::thread::scope(|scope| {
        for worker in 0..threads {
            let first = worker * chunk;
            if first >= len {
                break;
            }
            let last = hint::black_box((first + chunk).min(len));
            let operation = &operation;
            scope.spawn(move || {
                for index in first..last {
                    operation(index);
                }
            });
        }
    });
}

/// Collects one value for each index of a range, spread over several threads.
///
/// This is [`for_each_index`] for the common case where the result of the
/// work is a value rather than a write into a shared array. The output is
/// returned in index order regardless of which thread produced which element,
/// so it does not depend on the scheduling.
///
/// The closure is handed the index and returns the value for that index. As
/// with [`for_each_index`], it may run on any thread, so captured data must be
/// `Sync`.
///
/// 为一段范围内的每个下标各收集一个值，分摊到多个线程上执行。
///
/// 当工作的结果是值而非写入共享数组时，本函数是 [`for_each_index`] 的对应形式。
/// 输出按下标顺序返回，与哪个线程产生了哪个元素无关，故不依赖调度。
///
/// 闭包收到下标并返回该下标对应的值。与 [`for_each_index`] 一样，它可能在任意
/// 线程上运行，故捕获的数据必须为 `Sync`。
pub fn map_index<T, F>(len: usize, operation: F) -> Vec<T>
where
    T: Send,
    F: Fn(usize) -> T + Sync,
{
    let threads = threads_for(len);
    if threads <= 1 {
        return (0..len).map(operation).collect();
    }
    let chunk = len.div_ceil(threads);
    // Each thread writes a contiguous run of the output, so the buffer is
    // split into disjoint mutable slices up front and no slot is written
    // twice. `Vec<Option<T>>` is filled in parallel and unwrapped after the
    // scope, which is where the values are moved into their final order.
    // 每个线程写入输出的一段连续区间，故缓冲区先被切分为互不相交的可变切片，
    // 没有槽位被写两次。`Vec<Option<T>>` 并行填充，作用域结束后解包，值在此被
    // 移入最终顺序。
    let mut slots: Vec<Option<T>> = (0..len).map(|_| None).collect();
    let operation = &operation;
    std::thread::scope(|scope| {
        let mut rest = &mut slots[..];
        let mut offset = 0;
        while offset < len {
            let take = chunk.min(len - offset);
            let (this, tail) = core::mem::take(&mut rest).split_at_mut(take);
            let base = offset;
            scope.spawn(move || {
                for (index, slot) in this.iter_mut().enumerate() {
                    *slot = Some(operation(base + index));
                }
            });
            rest = tail;
            offset += take;
        }
    });
    slots
        .into_iter()
        .map(|slot| slot.expect("every index was assigned"))
        .collect()
}
