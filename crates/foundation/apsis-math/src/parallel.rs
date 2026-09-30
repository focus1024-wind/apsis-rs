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

use core::hint;

/// The number of threads to use for a piece of work.
///
/// This is the machine's parallelism as reported by the standard library, at
/// least one. It is the number the callers use to decide whether a parallel
/// run is worth starting.
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
pub fn for_each_index<F>(len: usize, operation: F)
where
    F: Fn(usize) + Sync,
{
    let threads = threads_for(len);
    if threads <= 1 {
        // A single thread would spawn and join one thread to do exactly what
        // this loop does, so it runs directly.
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
