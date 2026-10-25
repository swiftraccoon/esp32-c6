//! Bounded native queue, one-shot notification generator and persistent workers.

use std::ffi::c_void;
use std::hint::black_box;
use std::io;
use std::ptr::NonNull;
use std::sync::Arc;
use std::sync::atomic::{AtomicI32, AtomicU32, Ordering};
use std::sync::mpsc::{Receiver, SyncSender, sync_channel};
use std::thread::JoinHandle;
use std::time::Duration;

use esp_idf_sys as sys;

use super::observer::Identity;
use super::timer;

pub(super) const RECORDS: usize = 2_016;
pub(super) const CAPACITY: u32 = 8;
pub(super) const TASK_STACK: usize = 8_192;
const NOTIFICATION: u32 = 1;
const WAIT_TICKS: u32 = 10;
const WORK: u32 = 0;
const WARMUP: u32 = 1;
const END_PHASE: u32 = 2;
const STOP: u32 = 3;

#[derive(Clone, Copy, Debug)]
pub(super) struct Spec {
    pub index: u32,
    pub id: &'static str,
    pub period_us: u32,
    pub slots: u32,
    pub rounds: u32,
    pub initial_delay_us: u32,
    pub base_index: u32,
}

impl Spec {
    pub(super) const fn window_us(self) -> u64 {
        self.period_us as u64 * self.slots as u64
    }
}

pub(super) const SPECS: [Spec; 6] = [
    Spec {
        index: 0,
        id: "control-no-work",
        period_us: 2_000,
        slots: 125,
        rounds: 0,
        initial_delay_us: 0,
        base_index: 0,
    },
    Spec {
        index: 1,
        id: "control-delayed",
        period_us: 500,
        slots: 16,
        rounds: 0,
        initial_delay_us: 2_000,
        base_index: 125,
    },
    Spec {
        index: 2,
        id: "period-2000-us",
        period_us: 2_000,
        slots: 125,
        rounds: 4_096,
        initial_delay_us: 0,
        base_index: 141,
    },
    Spec {
        index: 3,
        id: "period-1000-us",
        period_us: 1_000,
        slots: 250,
        rounds: 4_096,
        initial_delay_us: 0,
        base_index: 266,
    },
    Spec {
        index: 4,
        id: "period-500-us",
        period_us: 500,
        slots: 500,
        rounds: 4_096,
        initial_delay_us: 0,
        base_index: 516,
    },
    Spec {
        index: 5,
        id: "period-250-us",
        period_us: 250,
        slots: 1_000,
        rounds: 4_096,
        initial_delay_us: 0,
        base_index: 1_016,
    },
];

#[derive(Debug, Default)]
pub(super) struct Record {
    pub attempt: AtomicU32,
    pub start: AtomicU32,
    pub finish: AtomicU32,
    pub checksum: AtomicU32,
    pub status: AtomicU32,
    pub queue_depth: AtomicU32,
}

const _: () = assert!(std::mem::size_of::<Record>() == 24);

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct PhaseResult {
    pub epoch_us: u64,
    pub generation_done_us: u64,
    pub drain_done_us: u64,
    pub callback_count: u32,
    pub early_wakes: u32,
}

#[derive(Debug)]
struct Queue(NonNull<sys::QueueDefinition>);

// SAFETY: Ordinary FreeRTOS task-context queues synchronize copied items. Every
// operation borrows an Arc owner, preserving allocation through all waits. Each
// item is six initialized aligned u32 words (24 bytes), never a borrowed pointer
// or padded Rust struct. No reset/ISR/cancellation operation is exposed.
unsafe impl Send for Queue {}
// SAFETY: The same kernel synchronization and Arc lifetime permit shared task
// calls; no Rust reference into queue storage or unsynchronized mutable view.
unsafe impl Sync for Queue {}

impl Queue {
    fn new() -> io::Result<Self> {
        // SAFETY: Fixed positive capacity and item width, ordinary base queue.
        let pointer = unsafe { sys::xQueueGenericCreate(CAPACITY, 24, 0) };
        NonNull::new(pointer)
            .map(Self)
            .ok_or_else(|| error("native queue allocation failed"))
    }

    fn send(&self, words: &[u32; 6], ticks: u32) -> io::Result<bool> {
        // SAFETY: Live Arc-backed queue copies exactly the aligned initialized
        // 24-byte array. Borrow remains live across a bounded wait; no pointer
        // is retained after return. Hot-path calls always use ticks=0.
        let result =
            unsafe { sys::xQueueGenericSend(self.0.as_ptr(), words.as_ptr().cast(), ticks, 0) };
        match result {
            1 => Ok(true),
            0 => Ok(false),
            other => Err(error(&format!("unexpected native send status {other}"))),
        }
    }

    fn recv(&self, ticks: u32) -> io::Result<Option<[u32; 6]>> {
        let mut words = [0_u32; 6];
        // SAFETY: The live queue writes at most its configured 24 bytes into
        // this initialized aligned array, which remains live through the wait.
        let result =
            unsafe { sys::xQueueReceive(self.0.as_ptr(), words.as_mut_ptr().cast(), ticks) };
        match result {
            1 => Ok(Some(words)),
            0 => Ok(None),
            other => Err(error(&format!("unexpected native receive status {other}"))),
        }
    }

    fn depth(&self) -> u32 {
        // SAFETY: Read-only task-context occupancy of a live Arc-backed queue.
        unsafe { sys::uxQueueMessagesWaiting(self.0.as_ptr()) }
    }

    fn qualify(&self) -> io::Result<()> {
        for i in 0..CAPACITY {
            if !self.send(&[WARMUP, i, 0, 0, 0, 0], 0)? {
                return Err(error("queue did not admit its declared capacity"));
            }
        }
        if self.send(&[WARMUP, 99, 0, 0, 0, 0], 0)? || self.depth() != CAPACITY {
            return Err(error("queue full control failed"));
        }
        for i in 0..CAPACITY {
            if self.recv(0)? != Some([WARMUP, i, 0, 0, 0, 0]) {
                return Err(error("queue FIFO control failed"));
            }
        }
        if self.recv(0)?.is_some() || self.depth() != 0 {
            return Err(error("queue empty control failed"));
        }
        Ok(())
    }
}

impl Drop for Queue {
    fn drop(&mut self) {
        // SAFETY: Final Arc owner only, hence no borrowed call or live waiter.
        // Explicit successful task shutdown precedes final resource release.
        unsafe { sys::vQueueDelete(self.0.as_ptr()) };
    }
}

#[derive(Debug)]
struct WakeContext {
    producer: usize,
    task: AtomicU32,
    priority: AtomicU32,
    count: AtomicU32,
    notify_result: AtomicI32,
}

unsafe extern "C" fn wake(argument: *mut c_void) {
    // SAFETY: TimerOwner retains the Arc allocation through successful blocking
    // quiescence and deletion; on failure it leaks that ownership until reset.
    // Producer TCB remains live throughout that interval. Only immutable data
    // and target-supported atomics are accessed. Task-dispatch context permits
    // these short, nonblocking SDK queries/notification; no panic/format/alloc.
    let context = unsafe { &*argument.cast::<WakeContext>() };
    let identity = Identity::current();
    context
        .task
        .store(identity.handle as u32, Ordering::Relaxed);
    context.priority.store(identity.priority, Ordering::Relaxed);
    let result = unsafe {
        sys::xTaskGenericNotify(
            context.producer as sys::TaskHandle_t,
            NOTIFICATION,
            0,
            sys::eNotifyAction_eIncrement,
            std::ptr::null_mut(),
        )
    };
    context.notify_result.store(result, Ordering::Relaxed);
    context.count.fetch_add(1, Ordering::Release);
}

struct TimerOwner {
    handle: Option<NonNull<sys::esp_timer>>,
    context: Option<Arc<WakeContext>>,
}

impl TimerOwner {
    fn new(context: Arc<WakeContext>) -> io::Result<Self> {
        let mut handle = std::ptr::null_mut();
        let args = sys::esp_timer_create_args_t {
            callback: Some(wake),
            arg: Arc::as_ptr(&context).cast_mut().cast(),
            dispatch_method: sys::esp_timer_dispatch_t_ESP_TIMER_TASK,
            name: c"load-release".as_ptr(),
            skip_unhandled_events: false,
        };
        // SAFETY: Arguments are initialized; library copies them. Static name
        // and retained Arc-backed arg outlive all callbacks, as explained above.
        sdk(
            unsafe { sys::esp_timer_create(&args, &mut handle) },
            "timer creation failed",
        )?;
        Ok(Self {
            handle: Some(NonNull::new(handle).ok_or_else(|| error("timer returned null handle"))?),
            context: Some(context),
        })
    }

    fn numeric(&self) -> io::Result<usize> {
        self.handle
            .map(|h| h.as_ptr() as usize)
            .ok_or_else(|| error("timer is already closed"))
    }

    fn close(&mut self) -> io::Result<()> {
        let handle = self
            .handle
            .ok_or_else(|| error("timer is already closed"))?;
        // SAFETY: Live timer, ordinary coordinator task, no concurrent rearming:
        // producer is blocked on its command after its own quiescence receipt.
        sdk(
            unsafe { sys::esp_timer_stop_blocking(handle.as_ptr(), WAIT_TICKS) },
            "timer final quiescence failed",
        )?;
        sdk(
            unsafe { sys::esp_timer_delete(handle.as_ptr()) },
            "timer deletion failed",
        )?;
        self.handle = None;
        self.context.take();
        Ok(())
    }
}

impl Drop for TimerOwner {
    fn drop(&mut self) {
        if self.handle.is_some()
            && let Some(context) = self.context.take()
        {
            // Failed-boot safety fallback, not successful cleanup. Destructors
            // cannot block or prove callback quiescence. Retain the bounded Arc
            // and SDK timer until host reset rather than free an in-flight arg.
            std::mem::forget(context);
        }
    }
}

#[derive(Debug)]
enum ProducerCommand {
    Prime {
        timer: usize,
        context: Arc<WakeContext>,
    },
    Phase {
        timer: usize,
        context: Arc<WakeContext>,
        spec: Spec,
        epoch: u64,
    },
    Stop,
}

#[derive(Debug)]
enum ConsumerReply {
    Warm,
    Ended(u64),
}

fn error(message: &str) -> io::Error {
    io::Error::other(message)
}

fn sdk(result: i32, message: &str) -> io::Result<()> {
    if result == 0 {
        Ok(())
    } else {
        Err(error(&format!("{message}: SDK status {result}")))
    }
}

fn offset(time: u64, epoch: u64) -> io::Result<u32> {
    time.checked_sub(epoch)
        .and_then(|d| d.try_into().ok())
        .ok_or_else(|| error("job clock backstep or offset overflow"))
}

fn park_failure(message: &str) -> ! {
    // Retain all task handles/Arc resources on a failed boot. No later success
    // can be emitted; host timeout/reset is recovery. Error logging is outside
    // the successful measured path and is never a favorable sample.
    eprintln!("load qualification failed: {message}");
    loop {
        std::thread::park();
    }
}

fn wait_notification() -> io::Result<()> {
    // SAFETY: Current producer task exclusively owns reserved index1; the
    // common SDK config has two entries. Pthread uses index0. Bounded task wait.
    let count = unsafe { sys::ulTaskGenericNotifyTake(NOTIFICATION, 1, WAIT_TICKS) };
    if count == 1 {
        Ok(())
    } else {
        Err(error("timer wake count was not exactly one"))
    }
}

fn clear_check() -> io::Result<()> {
    // SAFETY: Same current-task/index ownership, zero timeout. Called only when
    // the owned timer is quiescent, so a nonzero value is a stale-notify failure.
    if unsafe { sys::ulTaskGenericNotifyTake(NOTIFICATION, 1, 0) } == 0 {
        Ok(())
    } else {
        Err(error("stale producer notification"))
    }
}

fn arm(timer: usize, target: u64) -> io::Result<()> {
    let delay = target.saturating_sub(timer::legacy_micros()?).max(1);
    // SAFETY: Main's live owner outlives the persistent producer. Exactly one
    // one-shot is armed after the previous notification, never periodic mode.
    sdk(
        unsafe { sys::esp_timer_start_once(timer as sys::esp_timer_handle_t, delay) },
        "one-shot arm failed",
    )
}

fn quiesce(timer: usize, context: &WakeContext) -> io::Result<()> {
    // SAFETY: Live timer, ordinary producer task (not callback/ISR), no other
    // actor rearms. ESP_OK guarantees disarmed and no in-flight callback.
    sdk(
        unsafe { sys::esp_timer_stop_blocking(timer as sys::esp_timer_handle_t, WAIT_TICKS) },
        "producer timer quiescence failed",
    )?;
    clear_check()?;
    let callbacks = context.count.load(Ordering::Acquire);
    let notification_status = context.notify_result.load(Ordering::Acquire);
    if notification_status != 1 {
        return Err(error(&format!(
            "timer notification status {notification_status}"
        )));
    }
    if callbacks == 0
        || context.priority.load(Ordering::Acquire) != 22
        || context.task.load(Ordering::Acquire) == 0
    {
        return Err(error("timer callback identity or notification failed"));
    }
    Ok(())
}

fn generate(
    timer: usize,
    context: &WakeContext,
    spec: Spec,
    epoch: u64,
    queue: &Queue,
    records: &[Record],
) -> io::Result<PhaseResult> {
    clear_check()?;
    let initial_callbacks = context.count.load(Ordering::Acquire);
    let cutoff = epoch
        .checked_add(spec.window_us())
        .ok_or_else(|| error("phase cutoff overflow"))?;
    let mut next = 0_u32;
    let mut early = 0_u32;
    arm(timer, epoch + u64::from(spec.initial_delay_us))?;
    loop {
        wait_notification()?;
        let entry = timer::legacy_micros()?;
        if entry >= cutoff {
            for index in next..spec.slots {
                records[(spec.base_index + index) as usize]
                    .status
                    .store(1, Ordering::Release);
            }
            break;
        }
        let next_target = epoch + u64::from(next) * u64::from(spec.period_us);
        if entry < next_target {
            early += 1;
            if early > spec.slots + 8 {
                return Err(error("too many early timer wakes"));
            }
            arm(timer, next_target)?;
            continue;
        }
        let index = ((entry - epoch) / u64::from(spec.period_us)) as u32;
        for missed in next..index {
            records[(spec.base_index + missed) as usize]
                .status
                .store(1, Ordering::Release);
        }
        let global = spec.base_index + index;
        let nominal = epoch + u64::from(index) * u64::from(spec.period_us);
        let record = &records[global as usize];
        record
            .attempt
            .store(offset(entry, epoch)?, Ordering::Relaxed);
        let words = [
            WORK,
            global,
            nominal as u32,
            (nominal >> 32) as u32,
            epoch as u32,
            (epoch >> 32) as u32,
        ];
        let accepted = queue.send(&words, 0)?;
        record.queue_depth.store(queue.depth(), Ordering::Relaxed);
        // Producer6 cannot be preempted by consumer5 before this publication.
        // Timer22 preemption only notifies this producer; no other sender exists.
        record
            .status
            .store(if accepted { 3 } else { 2 }, Ordering::Release);
        next = index + 1;
        let target = if next == spec.slots {
            cutoff
        } else {
            epoch + u64::from(next) * u64::from(spec.period_us)
        };
        arm(timer, target)?;
    }
    quiesce(timer, context)?;
    Ok(PhaseResult {
        epoch_us: epoch,
        generation_done_us: timer::legacy_micros()?,
        drain_done_us: 0,
        callback_count: context
            .count
            .load(Ordering::Acquire)
            .wrapping_sub(initial_callbacks),
        early_wakes: early,
    })
}

fn producer_loop(
    commands: Receiver<ProducerCommand>,
    replies: SyncSender<PhaseResult>,
    ready: SyncSender<Identity>,
    queue: Arc<Queue>,
    records: Arc<Vec<Record>>,
    phase: Arc<AtomicU32>,
) {
    // SAFETY: Initialized current task; only this producer changes its own
    // priority during setup, before any timer or measured phase exists.
    unsafe { sys::vTaskPrioritySet(std::ptr::null_mut(), 6) };
    if ready.send(Identity::current()).is_err() {
        park_failure("producer identity disconnected");
    }
    drop(ready);
    loop {
        match commands.recv() {
            Ok(ProducerCommand::Prime { timer, context }) => {
                let result = (|| {
                    clear_check()?;
                    arm(timer, timer::legacy_micros()? + 1_000)?;
                    wait_notification()?;
                    quiesce(timer, &context)?;
                    Ok::<_, io::Error>(PhaseResult::default())
                })();
                match result {
                    Ok(result) => {
                        if replies.send(result).is_err() {
                            park_failure("prime reply disconnected");
                        }
                    }
                    Err(error) => park_failure(&error.to_string()),
                }
            }
            Ok(ProducerCommand::Phase {
                timer,
                context,
                spec,
                epoch,
            }) => match generate(timer, &context, spec, epoch, &queue, &records) {
                Ok(result) => {
                    phase.store(spec.index + 1, Ordering::Release);
                    if replies.send(result).is_err() {
                        park_failure("producer reply disconnected");
                    }
                }
                Err(error) => park_failure(&error.to_string()),
            },
            Ok(ProducerCommand::Stop) => return,
            Err(_) => park_failure("producer command disconnected"),
        }
    }
}

fn process_job(words: [u32; 6], current_phase: u32, records: &[Record]) -> io::Result<()> {
    let global = words[1];
    let spec = *SPECS
        .get(current_phase as usize)
        .ok_or_else(|| error("consumer phase outside contract"))?;
    if global < spec.base_index || global >= spec.base_index + spec.slots {
        return Err(error("job belongs to another phase"));
    }
    let record = records
        .get(global as usize)
        .ok_or_else(|| error("job index out of range"))?;
    let nominal = u64::from(words[2]) | (u64::from(words[3]) << 32);
    let epoch = u64::from(words[4]) | (u64::from(words[5]) << 32);
    if nominal != epoch + u64::from(global - spec.base_index) * u64::from(spec.period_us)
        || record.status.load(Ordering::Acquire) != 3
    {
        return Err(error("job release or publication invalid"));
    }
    let start = timer::legacy_micros()?;
    let value = black_box(bench_core::workload(
        black_box(bench_core::DEFAULT_SEED ^ global),
        black_box(spec.rounds),
    ));
    let finish = timer::legacy_micros()?;
    record.start.store(offset(start, epoch)?, Ordering::Relaxed);
    record
        .finish
        .store(offset(finish, epoch)?, Ordering::Relaxed);
    record.checksum.store(value, Ordering::Relaxed);
    record.status.store(4, Ordering::Release);
    Ok(())
}

fn consumer_loop(
    queue: Arc<Queue>,
    records: Arc<Vec<Record>>,
    replies: SyncSender<ConsumerReply>,
    ready: SyncSender<Identity>,
    phase: Arc<AtomicU32>,
) {
    if ready.send(Identity::current()).is_err() {
        park_failure("consumer identity disconnected");
    }
    drop(ready);
    let mut current = 0_u32;
    loop {
        let words = match queue.recv(u32::MAX) {
            Ok(Some(words)) => words,
            Ok(None) => park_failure("indefinite consumer receive unexpectedly expired"),
            Err(error) => park_failure(&error.to_string()),
        };
        match words[0] {
            WORK => {
                if let Err(error) = process_job(words, current, &records) {
                    park_failure(&error.to_string());
                }
            }
            WARMUP => {
                let value = black_box(bench_core::workload(
                    black_box(bench_core::DEFAULT_SEED),
                    black_box(bench_core::DEFAULT_ROUNDS),
                ));
                if value != bench_core::DEFAULT_CHECKSUM
                    || replies.send(ConsumerReply::Warm).is_err()
                {
                    park_failure("warmup checksum/reply failed");
                }
            }
            END_PHASE => {
                if words[1] != current || queue.depth() != 0 {
                    park_failure("phase-end marker order/queue failed");
                }
                current += 1;
                phase.store(current, Ordering::Release);
                match timer::legacy_micros() {
                    Ok(now) => {
                        if replies.send(ConsumerReply::Ended(now)).is_err() {
                            park_failure("consumer end reply failed");
                        }
                    }
                    Err(error) => park_failure(&error.to_string()),
                }
            }
            STOP => return,
            _ => park_failure("unknown queue message kind"),
        }
    }
}

pub(super) struct Harness {
    pub records: Arc<Vec<Record>>,
    pub producer: Identity,
    pub consumer: Identity,
    pub producer_phase: Arc<AtomicU32>,
    pub consumer_phase: Arc<AtomicU32>,
    queue: Arc<Queue>,
    context: Arc<WakeContext>,
    timer: TimerOwner,
    commands: SyncSender<ProducerCommand>,
    produced: Receiver<PhaseResult>,
    consumed: Receiver<ConsumerReply>,
    producer_join: Option<JoinHandle<()>>,
    consumer_join: Option<JoinHandle<()>>,
}

impl Harness {
    pub(super) fn new() -> io::Result<Self> {
        let mut data = Vec::new();
        data.try_reserve_exact(RECORDS)
            .map_err(|_| error("record reservation failed"))?;
        for _ in 0..RECORDS {
            data.push(Record::default());
        }
        let records = Arc::new(data);
        let queue = Arc::new(Queue::new()?);
        queue.qualify()?;
        let producer_phase = Arc::new(AtomicU32::new(0));
        let consumer_phase = Arc::new(AtomicU32::new(0));
        let (commands, command_rx) = sync_channel(1);
        let (reply_tx, produced) = sync_channel(1);
        let (consumer_tx, consumed) = sync_channel(1);
        let (producer_ready, producer_identity) = sync_channel(1);
        let (consumer_ready, consumer_identity) = sync_channel(1);
        let (q, r, p) = (
            Arc::clone(&queue),
            Arc::clone(&records),
            Arc::clone(&consumer_phase),
        );
        let consumer_join = std::thread::Builder::new()
            .name("load-consumer".into())
            .stack_size(TASK_STACK)
            .spawn(move || consumer_loop(q, r, consumer_tx, consumer_ready, p))?;
        let (q, r, p) = (
            Arc::clone(&queue),
            Arc::clone(&records),
            Arc::clone(&producer_phase),
        );
        let producer_join = std::thread::Builder::new()
            .name("load-producer".into())
            .stack_size(TASK_STACK)
            .spawn(move || producer_loop(command_rx, reply_tx, producer_ready, q, r, p))?;
        let producer = producer_identity
            .recv()
            .map_err(|_| error("producer identity missing"))?;
        let consumer = consumer_identity
            .recv()
            .map_err(|_| error("consumer identity missing"))?;
        if producer.priority != 6 || consumer.priority != 5 {
            return Err(error("workload task priorities invalid"));
        }
        let context = Arc::new(WakeContext {
            producer: producer.handle,
            task: AtomicU32::new(0),
            priority: AtomicU32::new(0),
            count: AtomicU32::new(0),
            notify_result: AtomicI32::new(1),
        });
        let timer = TimerOwner::new(Arc::clone(&context))?;
        let result = Self {
            records,
            producer,
            consumer,
            producer_phase,
            consumer_phase,
            queue,
            context,
            timer,
            commands,
            produced,
            consumed,
            producer_join: Some(producer_join),
            consumer_join: Some(consumer_join),
        };
        result
            .commands
            .send(ProducerCommand::Prime {
                timer: result.timer.numeric()?,
                context: Arc::clone(&result.context),
            })
            .map_err(|_| error("timer prime command failed"))?;
        result
            .produced
            .recv_timeout(Duration::from_secs(1))
            .map_err(|_| error("timer prime timeout"))?;
        result.wait_ready(0)?;
        Ok(result)
    }

    pub(super) fn timer_identity(&self) -> (u32, u32) {
        (
            self.context.task.load(Ordering::Acquire),
            self.context.priority.load(Ordering::Acquire),
        )
    }

    pub(super) fn wait_ready(&self, expected: u32) -> io::Result<()> {
        let start = timer::legacy_micros()?;
        loop {
            let phases = self.producer_phase.load(Ordering::Acquire) == expected
                && self.consumer_phase.load(Ordering::Acquire) == expected;
            // SAFETY: Both persistent TCBs live until explicit Stop after timer
            // deletion and the final observation. Error exits park them alive.
            let blocked = unsafe {
                sys::eTaskGetState(self.producer.handle as sys::TaskHandle_t)
                    == sys::eTaskState_eBlocked
                    && sys::eTaskGetState(self.consumer.handle as sys::TaskHandle_t)
                        == sys::eTaskState_eBlocked
            };
            if phases && blocked && self.queue.depth() == 0 {
                return Ok(());
            }
            if timer::legacy_micros()?.saturating_sub(start) > 200_000 {
                return Err(error("workload tasks did not settle"));
            }
            std::thread::yield_now();
        }
    }

    pub(super) fn warmup(&self, expected: u32) -> io::Result<()> {
        self.wait_ready(expected)?;
        for _ in 0..8 {
            if !self.queue.send(&[WARMUP, 0, 0, 0, 0, 0], WAIT_TICKS)? {
                return Err(error("warmup send failed"));
            }
            if !matches!(
                self.consumed.recv_timeout(Duration::from_secs(1)),
                Ok(ConsumerReply::Warm)
            ) {
                return Err(error("warmup reply failed"));
            }
        }
        self.wait_ready(expected)
    }

    pub(super) fn run_phase(&self, spec: Spec, epoch: u64) -> io::Result<PhaseResult> {
        self.commands
            .send(ProducerCommand::Phase {
                timer: self.timer.numeric()?,
                context: Arc::clone(&self.context),
                spec,
                epoch,
            })
            .map_err(|_| error("phase command failed"))?;
        let mut result = self
            .produced
            .recv_timeout(Duration::from_secs(1))
            .map_err(|_| error("producer phase timeout"))?;
        if !self
            .queue
            .send(&[END_PHASE, spec.index, 0, 0, 0, 0], WAIT_TICKS)?
        {
            return Err(error("phase-end marker timed out"));
        }
        result.drain_done_us = match self.consumed.recv_timeout(Duration::from_millis(100)) {
            Ok(ConsumerReply::Ended(now)) => now,
            _ => return Err(error("consumer bounded drain failed")),
        };
        if result.drain_done_us > epoch + spec.window_us() + 100_000 {
            return Err(error("consumer exceeded drain bound"));
        }
        self.wait_ready(spec.index + 1)?;
        Ok(result)
    }

    pub(super) fn finish(&mut self) -> io::Result<()> {
        self.wait_ready(6)?;
        self.timer.close()?;
        self.commands
            .send(ProducerCommand::Stop)
            .map_err(|_| error("producer stop failed"))?;
        if !self.queue.send(&[STOP, 0, 0, 0, 0, 0], 0)? {
            return Err(error("consumer stop failed"));
        }
        self.producer_join
            .take()
            .ok_or_else(|| error("producer already joined"))?
            .join()
            .map_err(|_| error("producer join failed"))?;
        self.consumer_join
            .take()
            .ok_or_else(|| error("consumer already joined"))?
            .join()
            .map_err(|_| error("consumer join failed"))?;
        Ok(())
    }
}
