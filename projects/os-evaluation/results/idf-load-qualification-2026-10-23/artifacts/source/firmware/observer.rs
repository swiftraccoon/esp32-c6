//! Fixed-size task/heap observations outside the offered-load windows.
//!
//! Both profiles retain the same Rust record layout. Their generated SDK ABI
//! differs: enabled runtime counters are u64; disabled counters are u32 zeros
//! which are represented as unavailable, never as measured zero CPU time.

use std::io::{self, Write};

use esp_idf_sys as sys;

use crate::timer;

const TASK_CAPACITY: usize = 16;

/// Numeric SDK identity, used only while the corresponding task stays alive.
#[derive(Clone, Copy, Debug)]
pub(super) struct Identity {
    pub(super) handle: usize,
    pub(super) priority: u32,
}

impl Identity {
    pub(super) fn current() -> Self {
        // SAFETY: Called from initialized IDF task context. Null selects the
        // current task for the priority query. The returned handle is retained
        // numerically and never dereferenced by Rust; the owner governs lifetime
        // before passing it back to later SDK task queries.
        unsafe {
            Self {
                handle: sys::xTaskGetCurrentTaskHandle() as usize,
                priority: sys::uxTaskPriorityGet(std::ptr::null_mut()),
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct Task {
    handle: u32,
    number: u32,
    state: u32,
    priority: u32,
    runtime_us: Option<u64>,
    stack_free_bytes: u32,
}

#[derive(Clone, Copy, Debug, Default)]
struct Heap {
    free_bytes: usize,
    allocated_bytes: usize,
    largest_free_block: usize,
    minimum_free_bytes: usize,
}

/// A completed capture is numeric owned storage, with no borrowed SDK pointers.
///
/// Default reserves an unfilled slot; it is not a valid observed snapshot.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct Snapshot {
    phase: u32,
    boundary: &'static str,
    producer_phase: u32,
    consumer_phase: u32,
    bracket_start_us: u64,
    bracket_end_us: u64,
    tasks_start_us: u64,
    tasks_end_us: u64,
    heap_start_us: u64,
    heap_end_us: u64,
    total_runtime_us: Option<u64>,
    count: usize,
    tasks: [Task; TASK_CAPACITY],
    heap: Heap,
}

fn error(message: &str) -> io::Error {
    io::Error::other(message)
}

/// Calls the configured ABI directly; no U32/U64 pointer cast is permitted.
fn collect_tasks(raw: &mut [sys::TaskStatus_t; TASK_CAPACITY]) -> io::Result<(usize, Option<u64>)> {
    #[cfg(feature = "load-runtime-stats")]
    {
        let mut total = 0_u64;
        // SAFETY: The array is live, aligned and writable for TASK_CAPACITY
        // generated records. The SDK retains no pointer. Passing an actual
        // *mut u64 is a compile-time check of the regenerated enabled ABI.
        let count = unsafe {
            sys::uxTaskGetSystemState(raw.as_mut_ptr(), TASK_CAPACITY as u32, &mut total)
        };
        Ok((count as usize, Some(total)))
    }
    #[cfg(not(feature = "load-runtime-stats"))]
    {
        let mut total = 0_u32;
        // SAFETY: Same array lifetime/capacity contract as above. This branch
        // passes *mut u32 to the separately generated disabled-profile ABI.
        // Its documented zero is checked, then discarded as unavailable.
        let count = unsafe {
            sys::uxTaskGetSystemState(raw.as_mut_ptr(), TASK_CAPACITY as u32, &mut total)
        };
        if total != 0 {
            return Err(error(
                "disabled runtime profile returned an active total counter",
            ));
        }
        Ok((count as usize, None))
    }
}

fn task_runtime(status: &sys::TaskStatus_t) -> io::Result<Option<u64>> {
    #[cfg(feature = "load-runtime-stats")]
    {
        // Direct assignment deliberately fails against a U32 field; widening
        // a stale binding would hide the required native-configuration check.
        let runtime: u64 = status.ulRunTimeCounter;
        Ok(Some(runtime))
    }
    #[cfg(not(feature = "load-runtime-stats"))]
    {
        let disabled: u32 = status.ulRunTimeCounter;
        if disabled != 0 {
            return Err(error(
                "disabled runtime profile returned an active task counter",
            ));
        }
        Ok(None)
    }
}

impl Snapshot {
    /// Captures a quiescent boundary chosen/proved by the calling coordinator.
    ///
    /// This function does not wait for actors or sample their shared phase
    /// atomics. It retains the caller's counter observations alongside SDK task
    /// states; the caller and protocol admission enforce live blocked actors.
    pub(super) fn capture(
        phase: u32,
        boundary: &'static str,
        producer_phase: u32,
        consumer_phase: u32,
    ) -> io::Result<Self> {
        if phase >= 6 || !matches!(boundary, "before" | "after") {
            return Err(error("invalid load snapshot phase or boundary"));
        }
        // SAFETY: Generated C records contain only integer/raw-pointer fields
        // with valid zero representations. No Rust references or owned handles
        // are fabricated. Only the SDK-returned initialized prefix is copied;
        // name/stack pointers are neither retained nor dereferenced.
        let mut raw: [sys::TaskStatus_t; TASK_CAPACITY] = unsafe { std::mem::zeroed() };
        let mut info: sys::multi_heap_info_t = unsafe { std::mem::zeroed() };
        let mut result = Self {
            phase,
            boundary,
            producer_phase,
            consumer_phase,
            ..Self::default()
        };
        result.bracket_start_us = timer::legacy_micros()?;
        result.bracket_end_us = timer::legacy_micros()?;
        result.tasks_start_us = timer::legacy_micros()?;
        let (count, total_runtime_us) = collect_tasks(&mut raw)?;
        result.tasks_end_us = timer::legacy_micros()?;
        if count == 0 || count > TASK_CAPACITY {
            return Err(error("load task snapshot empty or insufficient capacity"));
        }
        result.count = count;
        result.total_runtime_us = total_runtime_us;
        for (destination, source) in result.tasks.iter_mut().zip(&raw[..count]) {
            *destination = Task {
                handle: source.xHandle as u32,
                number: source.xTaskNumber,
                state: source.eCurrentState,
                priority: source.uxCurrentPriority,
                runtime_us: task_runtime(source)?,
                stack_free_bytes: source.usStackHighWaterMark,
            };
        }
        result.heap_start_us = timer::legacy_micros()?;
        // SAFETY: Called in initialized allocator/task context, outside offered
        // load. The aligned writable info record outlives the synchronous call;
        // the explicit 8-bit-capability class is not added to overlapping ones.
        unsafe { sys::heap_caps_get_info(&mut info, sys::MALLOC_CAP_8BIT) };
        result.heap_end_us = timer::legacy_micros()?;
        result.heap = Heap {
            free_bytes: info.total_free_bytes,
            allocated_bytes: info.total_allocated_bytes,
            largest_free_block: info.largest_free_block,
            minimum_free_bytes: info.minimum_free_bytes,
        };
        Ok(result)
    }

    /// Emits a retained snapshot after collection, with explicit null counters.
    pub(super) fn write_json(&self, mut output: impl Write) -> io::Result<()> {
        write!(
            output,
            "{{\"event\":\"snapshot\",\"phase\":{},\"boundary\":\"{}\",\"producer_phase\":{},\"consumer_phase\":{},\"bracket_start_us\":{},\"bracket_end_us\":{},\"tasks_start_us\":{},\"tasks_end_us\":{},\"heap_start_us\":{},\"heap_end_us\":{},\"total_runtime_us\":",
            self.phase,
            self.boundary,
            self.producer_phase,
            self.consumer_phase,
            self.bracket_start_us,
            self.bracket_end_us,
            self.tasks_start_us,
            self.tasks_end_us,
            self.heap_start_us,
            self.heap_end_us,
        )?;
        write_counter(&mut output, self.total_runtime_us)?;
        write!(output, ",\"tasks\":[")?;
        for (index, task) in self.tasks[..self.count].iter().enumerate() {
            if index > 0 {
                write!(output, ",")?;
            }
            write!(
                output,
                "{{\"handle\":{},\"number\":{},\"state\":{},\"priority\":{},\"runtime_us\":",
                task.handle, task.number, task.state, task.priority,
            )?;
            write_counter(&mut output, task.runtime_us)?;
            write!(output, ",\"stack_free_bytes\":{}}}", task.stack_free_bytes)?;
        }
        let heap = self.heap;
        writeln!(
            output,
            "],\"heap\":{{\"free_bytes\":{},\"allocated_bytes\":{},\"largest_free_block\":{},\"minimum_free_bytes\":{}}}}}",
            heap.free_bytes, heap.allocated_bytes, heap.largest_free_block, heap.minimum_free_bytes,
        )
    }
}

fn write_counter(mut output: impl Write, counter: Option<u64>) -> io::Result<()> {
    match counter {
        Some(value) => write!(output, "{value}"),
        None => write!(output, "null"),
    }
}
