//! Qualification-only admission for finite offered-load protocol 5.
//!
//! Nominal demand, generator misses and queue rejection remain separate from
//! conditional completion latencies. Settled attribution includes drain and
//! boundary work; it is never a physical CPU percentage or sustainable rate.

use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail, ensure};
use clap::ValueEnum;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{Map, Value, json};

use crate::qualification::{Admission, CapturePurpose};
use crate::queue_protocol::Configuration;
use crate::{boot_text, compiler_version, digest, write_json};

pub(crate) const EXPERIMENT: &str = "idf-load-qualification-v1";
const RECORDS: u32 = 2_016;
const CAPACITY: u32 = 8;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Profile {
    RuntimeStatsDisabled,
    RuntimeStatsEnabled,
}

impl Profile {
    fn enabled(self) -> bool {
        self == Self::RuntimeStatsEnabled
    }
}

/// Unlike an Option field, this transparent wrapper requires an explicit field.
#[derive(Clone, Copy, Debug, Serialize)]
#[serde(transparent)]
struct Nullable<T>(Option<T>);

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Nullable<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        // Value requests deserialize_any, so Serde's missing-field deserializer
        // rejects absence rather than silently supplying Option::None.
        let value = Value::deserialize(deserializer)?;
        if value.is_null() {
            Ok(Self(None))
        } else {
            T::deserialize(value)
                .map(|value| Self(Some(value)))
                .map_err(serde::de::Error::custom)
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "event", rename_all = "kebab-case", deny_unknown_fields)]
enum Record {
    Hello(Hello),
    QueueControl(QueueControl),
    Phase(Phase),
    Snapshot(Snapshot),
    Sample(Sample),
    PhaseEnd(PhaseEnd),
    Complete(Complete),
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Hello {
    protocol: u32,
    experiment: String,
    profile: Profile,
    runtime_stats_enabled: bool,
    counter_bits: u32,
    counter_clock: Nullable<String>,
    task_capacity: u32,
    coordinator_handle: u32,
    producer_handle: u32,
    consumer_handle: u32,
    timer_handle: u32,
    coordinator_priority: u32,
    producer_priority: u32,
    consumer_priority: u32,
    timer_priority: u32,
    coordinator_stack_bytes: u32,
    producer_stack_bytes: u32,
    consumer_stack_bytes: u32,
    notification_index: u32,
    notification_entries: u32,
    queue_capacity: u32,
    record_count: u32,
    record_storage_bytes: u32,
    snapshot_count: u32,
    snapshot_storage_bytes: u32,
    warmups_per_phase: u32,
    config_before: Configuration,
}

impl Hello {
    fn validate(&self, expected: Profile) -> Result<()> {
        ensure!(
            self.protocol == 5 && self.experiment == EXPERIMENT && self.profile == expected,
            "wrong load protocol, experiment or requested profile"
        );
        ensure!(
            self.runtime_stats_enabled == expected.enabled()
                && self.counter_bits == if expected.enabled() { 64 } else { 32 }
                && self.counter_clock.0.as_deref()
                    == expected.enabled().then_some("esp_timer_get_time"),
            "wrong load counter width, availability or clock"
        );
        let roles = [
            self.coordinator_handle,
            self.producer_handle,
            self.consumer_handle,
            self.timer_handle,
        ];
        ensure!(
            roles.iter().all(|&handle| handle != 0)
                && roles.into_iter().collect::<BTreeSet<_>>().len() == 4,
            "load role task handles must be distinct and nonzero"
        );
        ensure!(
            self.coordinator_priority == 1
                && self.producer_priority == 6
                && self.consumer_priority == 5
                && self.timer_priority == 22
                && self.coordinator_stack_bytes == 32_768
                && self.producer_stack_bytes == 8_192
                && self.consumer_stack_bytes == 8_192,
            "wrong load role priority or stack reservation"
        );
        ensure!(
            self.task_capacity == 16
                && self.notification_index == 1
                && self.notification_entries == 2
                && self.queue_capacity == CAPACITY
                && self.record_count == RECORDS
                && self.record_storage_bytes == 48_384
                && self.snapshot_count == 12
                && self.snapshot_storage_bytes > 0
                && self.warmups_per_phase == 8,
            "wrong load notification, queue or recording configuration"
        );
        self.config_before.validate()
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct QueueControl {
    capacity: u32,
    accepted: u32,
    full_rejected: u32,
    fifo_pass: bool,
    empty_pass: bool,
}

#[derive(Clone, Copy, Debug)]
struct Definition {
    id: &'static str,
    period: u32,
    slots: u32,
    rounds: u32,
    delay: u32,
    base: u32,
}

const DEFINITIONS: [Definition; 6] = [
    Definition {
        id: "control-no-work",
        period: 2_000,
        slots: 125,
        rounds: 0,
        delay: 0,
        base: 0,
    },
    Definition {
        id: "control-delayed",
        period: 500,
        slots: 16,
        rounds: 0,
        delay: 2_000,
        base: 125,
    },
    Definition {
        id: "period-2000-us",
        period: 2_000,
        slots: 125,
        rounds: 4_096,
        delay: 0,
        base: 141,
    },
    Definition {
        id: "period-1000-us",
        period: 1_000,
        slots: 250,
        rounds: 4_096,
        delay: 0,
        base: 266,
    },
    Definition {
        id: "period-500-us",
        period: 500,
        slots: 500,
        rounds: 4_096,
        delay: 0,
        base: 516,
    },
    Definition {
        id: "period-250-us",
        period: 250,
        slots: 1_000,
        rounds: 4_096,
        delay: 0,
        base: 1_016,
    },
];

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Phase {
    index: u32,
    id: String,
    period_us: u32,
    slots: u32,
    window_us: u32,
    rounds: u32,
    initial_delay_us: u32,
    base_index: u32,
    epoch_us: u64,
    cutoff_us: u64,
    deadline_us: u32,
}

impl Phase {
    fn validate(&self, index: usize) -> Result<()> {
        let definition = DEFINITIONS.get(index).context("extra load phase")?;
        ensure!(
            self.index as usize == index
                && self.id == definition.id
                && self.period_us == definition.period
                && self.slots == definition.slots
                && self.window_us == definition.period * definition.slots
                && self.rounds == definition.rounds
                && self.initial_delay_us == definition.delay
                && self.base_index == definition.base
                && self.deadline_us == 2_000,
            "wrong load phase definition or order"
        );
        ensure!(
            self.epoch_us.checked_add(u64::from(self.window_us)) == Some(self.cutoff_us),
            "load cutoff differs from checked epoch plus window"
        );
        self.cutoff_us
            .checked_add(100_000)
            .context("drain bound overflow")?;
        Ok(())
    }

    fn absolute(&self, offset: u32) -> Result<u64> {
        self.epoch_us
            .checked_add(u64::from(offset))
            .context("load timestamp overflow")
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Task {
    handle: u32,
    number: u32,
    state: u32,
    priority: u32,
    runtime_us: Nullable<u64>,
    stack_free_bytes: u32,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Heap {
    free_bytes: u32,
    allocated_bytes: u32,
    largest_free_block: u32,
    minimum_free_bytes: u32,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
enum Boundary {
    Before,
    After,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Snapshot {
    phase: u32,
    boundary: Boundary,
    producer_phase: u32,
    consumer_phase: u32,
    bracket_start_us: u64,
    bracket_end_us: u64,
    tasks_start_us: u64,
    tasks_end_us: u64,
    heap_start_us: u64,
    heap_end_us: u64,
    total_runtime_us: Nullable<u64>,
    tasks: Vec<Task>,
    heap: Heap,
}

impl Snapshot {
    fn task(&self, handle: u32) -> Result<&Task> {
        self.tasks
            .iter()
            .find(|task| task.handle == handle)
            .context("snapshot missing load role/task")
    }

    fn validate(&self, hello: &Hello, phase: &Phase, boundary: Boundary) -> Result<()> {
        let actor_phase = phase.index + u32::from(boundary == Boundary::After);
        ensure!(
            self.phase == phase.index
                && self.boundary == boundary
                && self.producer_phase == actor_phase
                && self.consumer_phase == actor_phase,
            "wrong load snapshot phase, boundary or actor epoch"
        );
        ensure!(
            self.bracket_start_us <= self.bracket_end_us
                && self.bracket_end_us <= self.tasks_start_us
                && self.tasks_start_us <= self.tasks_end_us
                && self.tasks_end_us <= self.heap_start_us
                && self.heap_start_us <= self.heap_end_us,
            "unordered load observer endpoints"
        );
        if hello.profile.enabled() {
            let total = self
                .total_runtime_us
                .0
                .context("enabled load total is null")?;
            ensure!(
                self.tasks_start_us <= total && total <= self.tasks_end_us,
                "load runtime total is outside enumeration bracket"
            );
        } else {
            ensure!(
                self.total_runtime_us.0.is_none(),
                "disabled load total is not null"
            );
        }
        ensure!(
            !self.tasks.is_empty() && self.tasks.len() <= hello.task_capacity as usize,
            "empty or over-capacity load task snapshot"
        );
        let mut handles = BTreeSet::new();
        let mut numbers = BTreeSet::new();
        for task in &self.tasks {
            ensure!(
                task.handle != 0
                    && task.number != 0
                    && handles.insert(task.handle)
                    && numbers.insert(task.number),
                "zero or duplicate load task identity"
            );
            ensure!(task.state <= 3, "deleted or invalid load task state");
            match (hello.profile.enabled(), task.runtime_us.0) {
                (true, Some(runtime)) => ensure!(
                    runtime <= self.total_runtime_us.0.context("missing total")?,
                    "load task runtime exceeds total clock"
                ),
                (false, None) => {}
                _ => bail!("load task counter availability disagrees with profile"),
            }
        }
        let coordinator = self.task(hello.coordinator_handle)?;
        let producer = self.task(hello.producer_handle)?;
        let consumer = self.task(hello.consumer_handle)?;
        let timer = self.task(hello.timer_handle)?;
        ensure!(
            coordinator.state == 0 && producer.state == 2 && consumer.state == 2,
            "load roles are not running/blocked at settled boundaries"
        );
        ensure!(
            coordinator.priority == 1
                && producer.priority == 6
                && consumer.priority == 5
                && timer.priority == 22,
            "load role priority changed"
        );
        ensure!(
            coordinator.stack_free_bytes <= hello.coordinator_stack_bytes
                && producer.stack_free_bytes <= hello.producer_stack_bytes
                && consumer.stack_free_bytes <= hello.consumer_stack_bytes,
            "load stack-fill indicator exceeds reservation"
        );
        ensure!(
            self.heap.largest_free_block <= self.heap.free_bytes
                && self.heap.minimum_free_bytes <= self.heap.free_bytes,
            "load heap largest/minimum exceeds current free"
        );
        if boundary == Boundary::Before {
            ensure!(
                self.heap_end_us
                    .checked_add(20_000)
                    .is_some_and(|end| end <= phase.epoch_us),
                "load epoch is less than 20ms after baseline snapshot"
            );
        }
        Ok(())
    }

    fn follows(&self, before: &Self, profile: Profile) -> Result<()> {
        ensure!(
            before.heap_end_us <= self.bracket_start_us,
            "overlapping load snapshots"
        );
        ensure!(
            self.tasks.len() == before.tasks.len(),
            "load task lifecycle changed set"
        );
        if profile.enabled() {
            ensure!(
                self.total_runtime_us.0 > before.total_runtime_us.0,
                "load total runtime clock did not advance"
            );
        }
        for previous in &before.tasks {
            let task = self.task(previous.handle)?;
            ensure!(
                task.number == previous.number,
                "load task handle reused with changed number"
            );
            if profile.enabled() {
                ensure!(
                    task.runtime_us.0 >= previous.runtime_us.0,
                    "load runtime counter decreased"
                );
            }
        }
        Ok(())
    }

    fn gross(&self) -> Value {
        json!({"empty_timer_bracket_us": self.bracket_end_us-self.bracket_start_us,
            "task_enumeration_us": self.tasks_end_us-self.tasks_start_us,
            "heap_info_us": self.heap_end_us-self.heap_start_us})
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
enum Status {
    GeneratorMissed,
    QueueFull,
    Completed,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Sample {
    phase: u32,
    index: u32,
    global_index: u32,
    status: Status,
    attempt_offset_us: u32,
    start_offset_us: u32,
    finish_offset_us: u32,
    checksum: u32,
    queue_depth: u32,
}

impl Sample {
    fn validate(&self, phase: &Phase, index: usize, previous_finish: Option<u32>) -> Result<()> {
        ensure!(
            self.phase == phase.index
                && self.index as usize == index
                && phase.base_index.checked_add(self.index) == Some(self.global_index),
            "wrong, missing or duplicate nominal load sample identity"
        );
        if self.status == Status::GeneratorMissed {
            ensure!(
                self.attempt_offset_us == 0
                    && self.start_offset_us == 0
                    && self.finish_offset_us == 0
                    && self.checksum == 0
                    && self.queue_depth == 0,
                "generator-missed load row carries invented observations"
            );
            return Ok(());
        }
        let release = self
            .index
            .checked_mul(phase.period_us)
            .context("release offset overflow")?;
        let next = release
            .checked_add(phase.period_us)
            .context("next slot overflow")?;
        ensure!(
            release <= self.attempt_offset_us && self.attempt_offset_us < next,
            "load offer is not in its latest nominal slot"
        );
        phase.absolute(self.attempt_offset_us)?;
        if self.status == Status::QueueFull {
            ensure!(
                self.queue_depth == CAPACITY
                    && self.start_offset_us == 0
                    && self.finish_offset_us == 0
                    && self.checksum == 0,
                "queue-full load row has wrong depth or invented service"
            );
            return Ok(());
        }
        ensure!(
            (1..=CAPACITY).contains(&self.queue_depth),
            "load post-send queue depth outside capacity"
        );
        ensure!(
            self.attempt_offset_us <= self.start_offset_us
                && self.start_offset_us <= self.finish_offset_us
                && (phase.rounds == 0 || self.start_offset_us < self.finish_offset_us),
            "load completion has impossible service ordering"
        );
        ensure!(
            previous_finish.is_none_or(|finish| finish <= self.start_offset_us),
            "load FIFO consumer services overlap or reorder"
        );
        ensure!(
            phase.absolute(self.finish_offset_us)? <= phase.cutoff_us + 100_000,
            "load completion exceeds drain bound"
        );
        let answer =
            bench_core::workload(bench_core::DEFAULT_SEED ^ self.global_index, phase.rounds);
        ensure!(self.checksum == answer, "wrong load workload checksum");
        Ok(())
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct PhaseEnd {
    index: u32,
    generation_done_us: u64,
    drain_done_us: u64,
    timer_stop_pass: bool,
    queue_empty: bool,
    notification_strays: u32,
    callback_count: u32,
    early_wakes: u32,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Complete {
    phases: u32,
    nominal_samples: u32,
    producer_joined: bool,
    consumer_joined: bool,
    timer_closed: bool,
    config_after: Configuration,
}

#[derive(Debug)]
struct PhaseData {
    definition: Phase,
    before: Option<Snapshot>,
    samples: Vec<Sample>,
    end: Option<PhaseEnd>,
    after: Option<Snapshot>,
}

#[derive(Clone, Copy, Debug, Default)]
enum Stage {
    #[default]
    Hello,
    QueueControl,
    Phase,
    Before,
    Samples,
    PhaseEnd,
    After,
    Complete,
    Finished,
}

#[derive(Debug, Default)]
struct Boot {
    hello: Option<Hello>,
    queue_control: Option<QueueControl>,
    phases: Vec<PhaseData>,
    complete: Option<Complete>,
    stage: Stage,
    rejected: bool,
}

impl Boot {
    fn accept_line(&mut self, line: &[u8], profile: Profile) -> Result<bool> {
        ensure!(!self.rejected, "load boot was already rejected");
        let result = (|| {
            let line = line.trim_ascii();
            if !line.starts_with(b"{") {
                return Ok(false);
            }
            self.accept(
                serde_json::from_slice(line).context("malformed load record")?,
                profile,
            )
        })();
        if result.is_err() {
            self.rejected = true;
        }
        result
    }

    fn current(&self) -> Result<&PhaseData> {
        self.phases.last().context("load record preceded phase")
    }
    fn current_mut(&mut self) -> Result<&mut PhaseData> {
        self.phases.last_mut().context("load record preceded phase")
    }

    fn accept(&mut self, record: Record, profile: Profile) -> Result<bool> {
        match (self.stage, record) {
            (Stage::Hello, Record::Hello(hello)) => {
                hello.validate(profile)?;
                self.hello = Some(hello);
                self.stage = Stage::QueueControl;
            }
            (Stage::QueueControl, Record::QueueControl(control)) => {
                ensure!(
                    control.capacity == 8
                        && control.accepted == 8
                        && control.full_rejected == 1
                        && control.fifo_pass
                        && control.empty_pass,
                    "load queue full/FIFO control failed"
                );
                self.queue_control = Some(control);
                self.stage = Stage::Phase;
            }
            (Stage::Phase, Record::Phase(phase)) => {
                phase.validate(self.phases.len())?;
                if let Some(previous) = self.phases.last() {
                    ensure!(
                        phase.epoch_us >= previous.definition.cutoff_us,
                        "load phase windows overlap"
                    );
                }
                self.phases.push(PhaseData {
                    definition: phase,
                    before: None,
                    samples: Vec::new(),
                    end: None,
                    after: None,
                });
                self.stage = Stage::Before;
            }
            (Stage::Before, Record::Snapshot(snapshot)) => {
                snapshot.validate(
                    self.hello.as_ref().context("missing hello")?,
                    &self.current()?.definition,
                    Boundary::Before,
                )?;
                if self.phases.len() > 1 {
                    snapshot.follows(
                        self.phases[self.phases.len() - 2]
                            .after
                            .as_ref()
                            .context("missing earlier snapshot")?,
                        profile,
                    )?;
                }
                self.current_mut()?.before = Some(snapshot);
                self.stage = Stage::Samples;
            }
            (Stage::Samples, Record::Sample(sample)) => {
                let phase = self.current_mut()?;
                let finish = phase
                    .samples
                    .iter()
                    .rev()
                    .find(|row| row.status == Status::Completed)
                    .map(|row| row.finish_offset_us);
                sample.validate(&phase.definition, phase.samples.len(), finish)?;
                // A prior start strictly before this send-entry proves that
                // job was already dequeued. Equal clocks remain ambiguous at
                // microsecond resolution and stay in this conservative upper
                // bound; no exact queue insertion instant is inferred.
                let possible_queued = phase
                    .samples
                    .iter()
                    .filter(|row| {
                        row.status == Status::Completed
                            && row.start_offset_us >= sample.attempt_offset_us
                    })
                    .count() as u32;
                match sample.status {
                    Status::QueueFull => ensure!(
                        possible_queued >= CAPACITY,
                        "queue-full contradicts possible queued accepted history"
                    ),
                    Status::Completed => ensure!(
                        sample.queue_depth <= possible_queued + 1,
                        "post-send depth exceeds possible queued accepted history"
                    ),
                    Status::GeneratorMissed => {}
                }
                phase.samples.push(sample);
                if phase.samples.len() == phase.definition.slots as usize {
                    self.stage = Stage::PhaseEnd;
                }
            }
            (Stage::PhaseEnd, Record::PhaseEnd(end)) => {
                let phase = self.current_mut()?;
                ensure!(
                    end.index == phase.definition.index
                        && end.timer_stop_pass
                        && end.queue_empty
                        && end.notification_strays == 0,
                    "load timer quiescence/queue/notification control failed"
                );
                ensure!(
                    phase.definition.cutoff_us <= end.generation_done_us
                        && end.generation_done_us <= end.drain_done_us
                        && end.drain_done_us <= phase.definition.cutoff_us + 100_000,
                    "load generation/drain chronology exceeds contract"
                );
                let attempted = phase
                    .samples
                    .iter()
                    .filter(|row| row.status != Status::GeneratorMissed)
                    .count() as u32;
                ensure!(
                    end.callback_count > 0
                        && end.callback_count <= 2 * phase.definition.slots + 16
                        && end.early_wakes <= phase.definition.slots + 8
                        && u64::from(end.callback_count)
                            == u64::from(attempted) + u64::from(end.early_wakes) + 1,
                    "load callback count disagrees with attempts, early wakes and terminating wake"
                );
                for row in phase
                    .samples
                    .iter()
                    .filter(|row| row.status == Status::Completed)
                {
                    ensure!(
                        phase.definition.absolute(row.finish_offset_us)? <= end.drain_done_us,
                        "load job finishes after declared drain completion"
                    );
                }
                validate_control(phase)?;
                phase.end = Some(end);
                self.stage = Stage::After;
            }
            (Stage::After, Record::Snapshot(snapshot)) => {
                let hello = self.hello.as_ref().context("missing hello")?;
                let phase = self.current()?;
                snapshot.validate(hello, &phase.definition, Boundary::After)?;
                snapshot.follows(
                    phase.before.as_ref().context("missing before snapshot")?,
                    profile,
                )?;
                ensure!(
                    snapshot.bracket_start_us
                        >= phase
                            .end
                            .as_ref()
                            .context("missing phase end")?
                            .drain_done_us,
                    "load post snapshot precedes drained/quiescent boundary"
                );
                self.current_mut()?.after = Some(snapshot);
                self.stage = if self.phases.len() == DEFINITIONS.len() {
                    Stage::Complete
                } else {
                    Stage::Phase
                };
            }
            (Stage::Complete, Record::Complete(complete)) => {
                ensure!(
                    complete.phases == 6
                        && complete.nominal_samples == RECORDS
                        && complete.producer_joined
                        && complete.consumer_joined
                        && complete.timer_closed,
                    "load completion lacks all phases, jobs or cleanup"
                );
                complete.config_after.validate()?;
                ensure!(
                    complete.config_after
                        == self.hello.as_ref().context("missing hello")?.config_before,
                    "load clock/timer configuration changed"
                );
                self.complete = Some(complete);
                self.stage = Stage::Finished;
                return Ok(true);
            }
            _ => bail!("load record is missing, extra, duplicated or out of order"),
        }
        Ok(false)
    }

    fn report_fields(&self) -> Result<Value> {
        ensure!(
            !self.rejected && matches!(self.stage, Stage::Finished),
            "incomplete or rejected load boot has no qualification summary"
        );
        let hello = self.hello.as_ref().context("missing load hello")?;
        let phases = self
            .phases
            .iter()
            .map(|phase| phase_report(phase, hello))
            .collect::<Result<Vec<_>>>()?;
        Ok(
            json!({"hello": hello, "queue_control": self.queue_control, "phases": phases,
            "config_after": self.complete.as_ref().context("missing completion")?.config_after,
            "qualification": {"status": "controls_passed", "phases": 6, "nominal_samples": RECORDS,
                "limits": ["Finite-pulse qualification only; no comparative score or sustainable capacity",
                    "Missed/full rows remain offered-work failures; quantiles are conditional",
                    "Attribution spans settled generation+drain+boundary work; not exact in-window or physical CPU utilization",
                    "Gross observer durations are not subtracted; heap/stack indicators are not global peaks",
                    "Statistics configurations change whole images; no universal per-switch cost is inferred"]}}),
        )
    }
}

fn validate_control(phase: &PhaseData) -> Result<()> {
    if phase.definition.index == 0 {
        ensure!(
            phase
                .samples
                .iter()
                .all(|row| row.status == Status::Completed
                    && row.attempt_offset_us - row.index * phase.definition.period_us
                        <= phase.definition.period_us / 4),
            "no-work generator control lost work or exceeded lateness bound"
        );
    } else if phase.definition.index == 1 {
        ensure!(
            phase
                .samples
                .iter()
                .take(4)
                .all(|row| row.status == Status::GeneratorMissed)
                && phase
                    .samples
                    .iter()
                    .any(|row| row.status == Status::Completed)
                && phase
                    .samples
                    .iter()
                    .all(|row| row.status != Status::QueueFull),
            "forced-delay latest-slot control did not preserve missed releases"
        );
    }
    Ok(())
}

fn distribution(mut values: Vec<u64>) -> Value {
    if values.is_empty() {
        return Value::Null;
    }
    values.sort_unstable();
    let n = values.len();
    let rank = |percent: usize| values[(percent * n).div_ceil(100) - 1];
    let sum = values.iter().map(|&value| u128::from(value)).sum::<u128>();
    json!({"count": n, "min_us": values[0], "p50_us": rank(50), "p95_us": rank(95),
        "p99_us": rank(99), "max_us": values[n-1], "sum_us": sum,
        "mean_us": sum as f64/n as f64})
}

fn attributed(before: &Snapshot, after: &Snapshot, hello: &Hello) -> Result<Value> {
    if !hello.profile.enabled() {
        return Ok(Value::Null);
    }
    let elapsed = after
        .total_runtime_us
        .0
        .context("missing after total")?
        .checked_sub(before.total_runtime_us.0.context("missing before total")?)
        .context("runtime total decreased")?;
    let mut deltas = BTreeMap::new();
    let mut sum = 0_u64;
    for old in &before.tasks {
        let task = after.task(old.handle)?;
        let delta = task
            .runtime_us
            .0
            .context("missing after task runtime")?
            .checked_sub(old.runtime_us.0.context("missing before task runtime")?)
            .context("runtime decreased")?;
        sum = sum
            .checked_add(delta)
            .context("summed load task runtime overflow")?;
        deltas.insert(
            old.handle,
            json!({"number": old.number, "runtime_us": delta}),
        );
    }
    let residual = i64::try_from(i128::from(elapsed) - i128::from(sum))
        .context("load residual exceeds signed range")?;
    Ok(
        json!({"whole_phase_total_elapsed_us": elapsed, "task_deltas": deltas,
        "summed_task_runtime_us": sum, "unattributed_boundary_residual_us": residual,
        "scope": "Settled before to post-drain snapshots including boundary work, not exact offer-window CPU"}),
    )
}

fn phase_report(data: &PhaseData, hello: &Hello) -> Result<Value> {
    let phase = &data.definition;
    let before = data
        .before
        .as_ref()
        .context("missing load before snapshot")?;
    let after = data.after.as_ref().context("missing load after snapshot")?;
    let end = data.end.as_ref().context("missing load phase end")?;
    let mut missed = 0_u32;
    let mut full = 0_u32;
    let mut accepted = 0_u32;
    let mut in_window = 0_u32;
    let mut in_drain = 0_u32;
    let mut met = 0_u32;
    let mut lateness = Vec::new();
    let mut latency = Vec::new();
    let mut to_start = Vec::new();
    let mut to_complete = Vec::new();
    let mut peak = None;
    for row in &data.samples {
        if row.status == Status::GeneratorMissed {
            missed += 1;
            continue;
        }
        let release = row.index * phase.period_us;
        lateness.push(u64::from(row.attempt_offset_us - release));
        peak = Some(peak.unwrap_or(0).max(row.queue_depth));
        if row.status == Status::QueueFull {
            full += 1;
            continue;
        }
        accepted += 1;
        if phase.absolute(row.finish_offset_us)? <= phase.cutoff_us {
            in_window += 1;
        } else {
            in_drain += 1;
        }
        met += u32::from(
            u64::from(row.finish_offset_us) <= u64::from(release) + u64::from(phase.deadline_us),
        );
        latency.push(u64::from(row.finish_offset_us - release));
        to_start.push(u64::from(row.start_offset_us - row.attempt_offset_us));
        to_complete.push(u64::from(row.finish_offset_us - row.attempt_offset_us));
    }
    ensure!(
        phase.slots == missed + full + accepted && accepted == in_window + in_drain,
        "load conservation failed"
    );
    let qualified = missed == 0
        && lateness
            .iter()
            .max()
            .is_some_and(|&maximum| maximum <= u64::from(phase.period_us / 4));
    Ok(
        json!({"phase": phase, "before": before, "samples": data.samples, "phase_end": end,
        "after": after, "counts": {"offered": phase.slots, "generator_missed": missed,
            "attempted": full+accepted, "queue_full": full, "accepted": accepted,
            "completed_in_window": in_window, "completed_in_drain": in_drain,
            "deadline_met": met, "deadline_missed_accepted": accepted-met,
            "offered_deadline_failures": phase.slots-met, "unfinished": 0},
        "generator": {"qualified": qualified,
            "classification": if qualified { "generator-qualified" } else { "generator-limited" },
            "attempt_lateness_us": distribution(lateness)},
        "conditional_latencies": {"nominal_to_completion_us": distribution(latency),
            "send_entry_to_start_us": distribution(to_start), "send_entry_to_completion_us": distribution(to_complete)},
        "post_send_queue_peak": peak, "accounting": attributed(before, after, hello)?,
        "gross_observer_us": {"before": before.gross(), "after": after.gross()},
        "memory_indicators": {"capabilities": 4, "before": before.heap, "after": after.heap},
        "spans": {"offer_window_us": phase.window_us,
            "generation_done_after_cutoff_us": end.generation_done_us-phase.cutoff_us,
            "drain_after_cutoff_us": end.drain_done_us-phase.cutoff_us,
            "settled_snapshot_span_us": after.tasks_end_us-before.tasks_start_us}}),
    )
}

pub(crate) fn validate_bytes(bytes: &[u8], profile: Profile) -> Result<Value> {
    validate(io::Cursor::new(bytes), profile)
}

pub(crate) fn validate_file(path: &Path, profile: Profile) -> Result<Value> {
    validate(File::open(path)?, profile)
}

fn validate(input: impl Read, profile: Profile) -> Result<Value> {
    let mut boot = Boot::default();
    boot_text::replay(input, |line| boot.accept_line(line, profile))?;
    boot.report_fields()
}

#[derive(Debug)]
pub(crate) struct Capture<'a> {
    pub(crate) purpose: CapturePurpose,
    pub(crate) profile: Profile,
    pub(crate) port: &'a str,
    pub(crate) output: &'a Path,
    pub(crate) elf: &'a Path,
    pub(crate) provenance: &'a Path,
    pub(crate) boot_id: &'a str,
    pub(crate) run_order: u32,
    pub(crate) timeout: Duration,
}

/// Retains one qualification attempt; it cannot supply a comparative score.
pub(crate) fn capture(config: Capture<'_>) -> Result<()> {
    ensure!(
        config.purpose == CapturePurpose::Qualification,
        "load capture is qualification only; comparison is unsupported"
    );
    ensure!(
        !config.timeout.is_zero(),
        "capture timeout must be positive"
    );
    ensure!(
        !config.output.exists(),
        "result directory already exists; refusing overwrite"
    );
    let firmware = fs::read(config.elf).context("reading load firmware ELF")?;
    let provenance_bytes = fs::read(config.provenance).context("reading load provenance")?;
    let admission = Admission {
        purpose: CapturePurpose::Qualification,
        elf_sha256: digest(&firmware),
        provenance_sha256: digest(&provenance_bytes),
        method: None,
    };
    let provenance: Value = serde_json::from_slice(&provenance_bytes)?;
    ensure!(provenance.is_object(), "build provenance must be an object");
    let collector = fs::read(std::env::current_exe()?)?;
    let compiler = compiler_version()?;
    if let Some(parent) = config.output.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::create_dir(config.output)?;
    fs::write(config.output.join("firmware.elf"), &firmware)?;
    fs::write(config.output.join("provenance.json"), &provenance_bytes)?;
    let mut report = json!({"schema_version": 4, "admission": admission, "status": "running",
        "platform": "esp-idf-freertos", "experiment": EXPERIMENT, "profile": config.profile,
        "timestamp_unix_ms": SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis(),
        "independent_boot_id": config.boot_id, "run_order_index": config.run_order,
        "reset_kind": "RTS warm-reset request; DTR deasserted",
        "serial": {"baud": 115200, "data_bits": 8, "parity": "none", "stop_bits": 1},
        "firmware_sha256": digest(&firmware), "collector_version": env!("CARGO_PKG_VERSION"),
        "collector_executable_sha256": digest(&collector), "host_rustc_at_capture": compiler,
        "provenance": provenance, "hello": null, "qualification": null,
        "benchmarks": null, "comparative_score": null, "failure": null,
        "limitations": ["Qualification-only finite pulses; no comparative estimator, sustainable rate or OS score",
            "Common timestamps and recording remain enabled in both profiles",
            "Counters cover settled generation+drain+boundary intervals, not physical CPU busy time",
            "No global runtime peak, external timing or energy measurement",
            "Warm-reset metadata does not authenticate a board or establish independent trials",
            "Observation ends after completion and its received chunk; later diagnostics remain unobserved"]});
    write_json(File::create(config.output.join("run.json"))?, &report)?;
    let mut raw = File::create(config.output.join("serial.raw"))?;
    let mut records = File::create(config.output.join("records.jsonl"))?;
    let mut boot = Boot::default();
    let result = (|| {
        admission.check_inputs(&fs::read(config.elf)?, &fs::read(config.provenance)?)?;
        let mut port = serialport::new(config.port, 115_200)
            .timeout(Duration::from_millis(100))
            .open()
            .context("opening load serial port")?;
        port.write_data_terminal_ready(false)?;
        port.write_request_to_send(true)?;
        std::thread::sleep(Duration::from_millis(100));
        port.clear(serialport::ClearBuffer::Input)?;
        port.write_request_to_send(false)?;
        capture_fields(
            &mut port,
            &mut raw,
            &mut records,
            &mut boot,
            config.profile,
            config.timeout,
        )
    })();
    raw.flush()?;
    records.flush()?;
    report["hello"] = serde_json::to_value(&boot.hello)?;
    record_outcome(&mut report, &result);
    write_json(File::create(config.output.join("run.json"))?, &report)?;
    result?;
    println!(
        "Validated load qualification boot {}: {}",
        config.boot_id,
        config.output.display()
    );
    Ok(())
}

fn record_outcome(report: &mut Value, result: &Result<Map<String, Value>>) {
    match result {
        Ok(fields) => {
            for (key, value) in fields {
                report[key] = value.clone();
            }
            report["status"] = json!("pass");
        }
        Err(error) => {
            report["status"] = json!("failed");
            report["failure"] = json!(format!("{error:#}"));
            report["qualification"] = Value::Null;
            report["phases"] = Value::Null;
            report["benchmarks"] = Value::Null;
            report["comparative_score"] = Value::Null;
        }
    }
}

/// Derivation belongs to admission: a completed stream can still fail checked
/// arithmetic. Its error must follow the same retained-failure report path.
fn capture_fields(
    input: impl Read,
    raw: impl Write,
    records: impl Write,
    boot: &mut Boot,
    profile: Profile,
    timeout: Duration,
) -> Result<Map<String, Value>> {
    let result = (|| {
        capture_stream(input, raw, records, boot, profile, timeout)?;
        boot.report_fields()?
            .as_object()
            .cloned()
            .context("load report must be an object")
    })();
    if result.is_err() {
        boot.rejected = true;
    }
    result
}

fn capture_stream(
    mut input: impl Read,
    mut raw: impl Write,
    mut records: impl Write,
    boot: &mut Boot,
    profile: Profile,
    timeout: Duration,
) -> Result<()> {
    let result = boot_text::capture(&mut input, &mut raw, timeout, |line| {
        let complete = boot.accept_line(line, profile)?;
        if line.trim_ascii().starts_with(b"{") {
            records.write_all(line.trim_ascii())?;
            records.write_all(b"\n")?;
        }
        Ok(complete)
    });
    if result.is_err() {
        boot.rejected = true;
    }
    result
}

/// Verifies report-only admission and retained original inputs before replay.
pub(crate) fn verify_archive_inputs(directory: &Path, report: &Value) -> Result<Profile> {
    let admission: Admission = serde_json::from_value(report["admission"].clone())?;
    let profile: Profile = serde_json::from_value(report["profile"].clone())?;
    ensure!(
        report["schema_version"] == 4
            && report["experiment"] == EXPERIMENT
            && admission.purpose == CapturePurpose::Qualification
            && admission.method.is_none()
            && report["benchmarks"].is_null()
            && report["comparative_score"].is_null()
            && report["firmware_sha256"] == admission.elf_sha256,
        "load archive must retain qualification-only admission"
    );
    let provenance_bytes = fs::read(directory.join("provenance.json"))?;
    admission.check_inputs(
        &fs::read(directory.join("firmware.elf"))?,
        &provenance_bytes,
    )?;
    ensure!(
        report["provenance"] == serde_json::from_slice::<Value>(&provenance_bytes)?,
        "load provenance differs from retained input bytes"
    );
    if report["status"] == "failed" {
        ensure!(
            report["qualification"].is_null() && report["phases"].is_null(),
            "failed load archive must not retain favorable phase summaries"
        );
    }
    Ok(profile)
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    fn configuration() -> Value {
        json!({"clocks": {"source": "register-readback", "cpu_mhz": 160,
            "flash_source_mhz": 80, "flash_mhz": 80, "flash_divider": 1,
            "registers": {"pcr_sysclk_conf": 0x0001_0200_u32,
                "pcr_cpu_freq_conf": 0, "pcr_mspi_clk_conf": 0x500,
                "spi0_clock": 0x8000_0000_u32}},
            "timer": {"source": "xtal", "enabled": true, "unit0_enabled": true,
                "unit0_stalls_cpu": true, "systimer_conf": 0x5000_0000_u32,
                "pcr_systimer_conf": 1, "pcr_systimer_func_clk_conf": 1 << 22}})
    }

    fn snapshot(profile: Profile, phase: usize, after: bool, epoch: u64, offset: i64) -> Value {
        let start = u64::try_from(i128::from(epoch) + i128::from(offset)).unwrap();
        let step = (phase * 2 + usize::from(after)) as u64;
        let mut tasks = vec![];
        for (handle, number, state, priority, runtime, stack) in [
            (1, 11, 0, 1, 1_000 + step * 1_000, 20_000),
            (2, 12, 2, 6, 2_000 + step * 2_000, 6_000),
            (3, 13, 2, 5, 4_300_000_000 + step * 20_000, 6_000),
            (4, 14, 3, 22, 3_000 + step * 500, 4_000),
            (5, 15, 1, 0, 4_000 + step * 5_000, 1_000),
        ] {
            tasks.push(json!({"handle": handle, "number": number, "state": state,
                "priority": priority, "runtime_us": if profile.enabled() { json!(runtime) } else { Value::Null },
                "stack_free_bytes": stack}));
        }
        json!({"event": "snapshot", "phase": phase,
            "boundary": if after { "after" } else { "before" },
            "producer_phase": phase+usize::from(after), "consumer_phase": phase+usize::from(after),
            "bracket_start_us": start, "bracket_end_us": start+1,
            "tasks_start_us": start+2, "tasks_end_us": start+4,
            "heap_start_us": start+5, "heap_end_us": start+6,
            "total_runtime_us": if profile.enabled() { json!(start+3) } else { Value::Null },
            "tasks": tasks, "heap": {"free_bytes": 300_000, "allocated_bytes": 100_000,
                "largest_free_block": 280_000, "minimum_free_bytes": 290_000}})
    }

    /// Complete host-only observations with deterministic queue saturation.
    fn transcript(profile: Profile) -> Vec<Value> {
        let mut records = vec![
            json!({"event": "hello", "protocol": 5,
            "experiment": EXPERIMENT, "profile": profile,
            "runtime_stats_enabled": profile.enabled(),
            "counter_bits": if profile.enabled() {64} else {32},
            "counter_clock": if profile.enabled() {json!("esp_timer_get_time")} else {Value::Null},
            "task_capacity": 16, "coordinator_handle": 1, "producer_handle": 2,
            "consumer_handle": 3, "timer_handle": 4, "coordinator_priority": 1,
            "producer_priority": 6, "consumer_priority": 5, "timer_priority": 22,
            "coordinator_stack_bytes": 32_768, "producer_stack_bytes": 8_192,
            "consumer_stack_bytes": 8_192, "notification_index": 1,
            "notification_entries": 2, "queue_capacity": 8, "record_count": RECORDS,
            "record_storage_bytes": 48_384, "snapshot_count": 12,
            "snapshot_storage_bytes": 10_000, "warmups_per_phase": 8,
            "config_before": configuration()}),
            json!({"event": "queue-control", "capacity": 8, "accepted": 8,
                "full_rejected": 1, "fifo_pass": true, "empty_pass": true}),
        ];
        for (phase_index, definition) in DEFINITIONS.iter().enumerate() {
            let epoch = 5_000_000_000_u64 + phase_index as u64 * 1_000_000;
            let window = definition.period * definition.slots;
            records.push(json!({"event": "phase", "index": phase_index, "id": definition.id,
                "period_us": definition.period, "slots": definition.slots, "window_us": window,
                "rounds": definition.rounds, "initial_delay_us": definition.delay,
                "base_index": definition.base, "epoch_us": epoch, "cutoff_us": epoch+u64::from(window),
                "deadline_us": 2_000}));
            records.push(snapshot(profile, phase_index, false, epoch, -30_000));
            let mut starts = Vec::new();
            let mut finish = 0_u32;
            let mut attempts = 0;
            for index in 0..definition.slots {
                let global = definition.base + index;
                let mut row = json!({"event": "sample", "phase": phase_index, "index": index,
                    "global_index": global, "status": "generator-missed", "attempt_offset_us": 0,
                    "start_offset_us": 0, "finish_offset_us": 0, "checksum": 0, "queue_depth": 0});
                if phase_index != 1 || index >= 4 {
                    attempts += 1;
                    let entry = index * definition.period + 5;
                    let waiting = starts.iter().filter(|&&start| start > entry).count() as u32;
                    row["attempt_offset_us"] = json!(entry);
                    if waiting == CAPACITY {
                        row["status"] = json!("queue-full");
                        row["queue_depth"] = json!(CAPACITY);
                    } else {
                        let start = (entry + 7).max(finish + 7);
                        finish = start + if definition.rounds == 0 { 0 } else { 300 };
                        starts.push(start);
                        row["status"] = json!("completed");
                        row["queue_depth"] = json!(waiting + 1);
                        row["start_offset_us"] = json!(start);
                        row["finish_offset_us"] = json!(finish);
                        row["checksum"] = json!(bench_core::workload(
                            bench_core::DEFAULT_SEED ^ global,
                            definition.rounds
                        ));
                    }
                }
                records.push(row);
            }
            let generation = epoch + u64::from(window) + 20;
            let drain = generation.max(epoch + u64::from(finish) + 10);
            records.push(json!({"event": "phase-end", "index": phase_index,
                "generation_done_us": generation, "drain_done_us": drain,
                "timer_stop_pass": true, "queue_empty": true, "notification_strays": 0,
                "callback_count": attempts+1, "early_wakes": 0}));
            records.push(snapshot(
                profile,
                phase_index,
                true,
                epoch,
                (drain - epoch + 1_000) as i64,
            ));
        }
        records.push(
            json!({"event": "complete", "phases": 6, "nominal_samples": RECORDS,
            "producer_joined": true, "consumer_joined": true, "timer_closed": true,
            "config_after": configuration()}),
        );
        records
    }

    fn bytes(records: &[Value]) -> Vec<u8> {
        let mut raw = b"ESP-ROM:esp32c6-20220919\nrst:0x1\n".to_vec();
        for record in records {
            serde_json::to_writer(&mut raw, record).unwrap();
            raw.push(b'\n');
        }
        raw
    }

    fn row(records: &mut [Value], phase: u32, index: u32) -> &mut Value {
        records
            .iter_mut()
            .find(|record| {
                record["event"] == "sample" && record["phase"] == phase && record["index"] == index
            })
            .unwrap()
    }

    fn event<'a>(records: &'a mut [Value], name: &str, phase: u32) -> &'a mut Value {
        records
            .iter_mut()
            .find(|record| {
                record["event"] == name && (record["index"] == phase || record["phase"] == phase)
            })
            .unwrap()
    }

    fn rejects(records: &[Value], profile: Profile, expected: &str) {
        let error = validate_bytes(&bytes(records), profile).unwrap_err();
        assert!(
            format!("{error:#}").contains(expected),
            "expected {expected}: {error:#}"
        );
    }

    #[test]
    fn full_profiles_retain_all_jobs_failures_and_nullable_attribution() {
        for profile in [Profile::RuntimeStatsDisabled, Profile::RuntimeStatsEnabled] {
            let records = transcript(profile);
            assert_eq!(records.len(), 2_043);
            let output = validate_bytes(&bytes(&records), profile).unwrap();
            assert_eq!(output["phases"].as_array().unwrap().len(), 6);
            assert_eq!(output["phases"][1]["counts"]["generator_missed"], 4);
            assert!(
                output["phases"][5]["counts"]["queue_full"]
                    .as_u64()
                    .unwrap()
                    > 0
            );
            assert!(
                output["phases"][5]["counts"]["completed_in_drain"]
                    .as_u64()
                    .unwrap()
                    > 0
            );
            assert!(
                output["phases"][5]["counts"]["offered_deadline_failures"]
                    .as_u64()
                    .unwrap()
                    > 0
            );
            assert_eq!(output["phases"][5]["post_send_queue_peak"], 8);
            assert_eq!(
                output["phases"][0]["samples"][0]["start_offset_us"],
                output["phases"][0]["samples"][0]["finish_offset_us"]
            );
            assert_eq!(
                output["phases"][0]["accounting"].is_null(),
                !profile.enabled()
            );
            if profile.enabled() {
                assert!(
                    output["phases"][0]["before"]["tasks"][2]["runtime_us"]
                        .as_u64()
                        .unwrap()
                        > u64::from(u32::MAX)
                );
            }
        }
    }

    #[test]
    fn requested_profile_and_explicit_null_fields_are_required() {
        let mut records = transcript(Profile::RuntimeStatsDisabled);
        rejects(&records, Profile::RuntimeStatsEnabled, "requested profile");
        records[0].as_object_mut().unwrap().remove("counter_clock");
        rejects(&records, Profile::RuntimeStatsDisabled, "missing field");
        let mut records = transcript(Profile::RuntimeStatsDisabled);
        records[3]
            .as_object_mut()
            .unwrap()
            .remove("total_runtime_us");
        rejects(&records, Profile::RuntimeStatsDisabled, "missing field");
        let mut records = transcript(Profile::RuntimeStatsDisabled);
        records[3]["tasks"][0]
            .as_object_mut()
            .unwrap()
            .remove("runtime_us");
        rejects(&records, Profile::RuntimeStatsDisabled, "missing field");
        records[3]["tasks"][0]["runtime_us"] = json!(0);
        rejects(&records, Profile::RuntimeStatsDisabled, "availability");
    }

    #[test]
    fn conditional_quantiles_preserve_real_zeros_and_empty_subsets() {
        let summary = distribution(vec![10, 0, 1, 0]);
        assert_eq!(summary["count"], 4);
        assert_eq!(summary["min_us"], 0);
        assert_eq!(summary["p50_us"], 0);
        assert_eq!(summary["p95_us"], 10);
        assert_eq!(summary["p99_us"], 10);
        assert_eq!(summary["sum_us"], 11);
        assert_eq!(summary["mean_us"], 2.75);
        assert!(distribution(vec![]).is_null());
    }

    #[test]
    fn entirely_missed_work_pulse_remains_adverse_with_no_invented_latency() {
        let mut records = transcript(Profile::RuntimeStatsDisabled);
        for record in records
            .iter_mut()
            .filter(|r| r["event"] == "sample" && r["phase"] == 5)
        {
            record["status"] = json!("generator-missed");
            for field in [
                "attempt_offset_us",
                "start_offset_us",
                "finish_offset_us",
                "checksum",
                "queue_depth",
            ] {
                record[field] = json!(0);
            }
        }
        event(&mut records, "phase-end", 5)["callback_count"] = json!(1);
        let output = validate_bytes(&bytes(&records), Profile::RuntimeStatsDisabled).unwrap();
        let phase = &output["phases"][5];
        assert_eq!(phase["counts"]["generator_missed"], 1000);
        assert_eq!(phase["counts"]["offered_deadline_failures"], 1000);
        assert_eq!(phase["generator"]["classification"], "generator-limited");
        assert!(phase["generator"]["attempt_lateness_us"].is_null());
        assert!(phase["conditional_latencies"]["nominal_to_completion_us"].is_null());
        assert!(phase["post_send_queue_peak"].is_null());
    }

    #[test]
    fn signed_boundary_residual_is_not_clamped_or_named_idle() {
        let mut records = transcript(Profile::RuntimeStatsEnabled);
        let mut first = true;
        for snapshot in records.iter_mut().filter(|r| r["event"] == "snapshot") {
            if first {
                first = false;
                continue;
            }
            let current = snapshot["tasks"][0]["runtime_us"].as_u64().unwrap();
            snapshot["tasks"][0]["runtime_us"] = json!(current + 1_000_000);
        }
        let output = validate_bytes(&bytes(&records), Profile::RuntimeStatsEnabled).unwrap();
        assert!(
            output["phases"][0]["accounting"]["unattributed_boundary_residual_us"]
                .as_i64()
                .unwrap()
                < 0
        );
        assert!(
            output["phases"][0]["accounting"]
                .get("cpu_busy_percent")
                .is_none()
        );
    }

    #[test]
    fn derived_arithmetic_failure_is_retained_as_failed_after_complete_stream() {
        let mut records = transcript(Profile::RuntimeStatsEnabled);
        let shift = 10_000_000_000_000_000_000_u64;
        let mut first_snapshot = true;
        for record in &mut records {
            let fields: &[&str] = match record["event"].as_str().unwrap() {
                "phase" => &["epoch_us", "cutoff_us"],
                "phase-end" => &["generation_done_us", "drain_done_us"],
                "snapshot" => &[
                    "bracket_start_us",
                    "bracket_end_us",
                    "tasks_start_us",
                    "tasks_end_us",
                    "heap_start_us",
                    "heap_end_us",
                    "total_runtime_us",
                ],
                _ => &[],
            };
            for &field in fields {
                record[field] = json!(record[field].as_u64().unwrap() + shift);
            }
            if record["event"] == "snapshot" {
                if !first_snapshot {
                    for task in record["tasks"].as_array_mut().unwrap() {
                        task["runtime_us"] =
                            json!(task["runtime_us"].as_u64().unwrap() + 6_000_000_000_000_000_000);
                    }
                }
                first_snapshot = false;
            }
        }
        let raw = bytes(&records);
        let mut boot = Boot::default();
        let mut retained = Vec::new();
        let result = capture_fields(
            io::Cursor::new(&raw),
            &mut retained,
            &mut Vec::new(),
            &mut boot,
            Profile::RuntimeStatsEnabled,
            Duration::from_secs(5),
        );
        assert!(
            format!("{:#}", result.as_ref().unwrap_err())
                .contains("summed load task runtime overflow")
        );
        assert_eq!(retained, raw);
        assert!(boot.report_fields().is_err());
        let mut report = json!({"status": "running", "qualification": null});
        record_outcome(&mut report, &result);
        assert_eq!(report["status"], "failed");
        assert!(report["qualification"].is_null() && report["phases"].is_null());
        assert!(report["benchmarks"].is_null() && report["comparative_score"].is_null());
    }

    #[test]
    fn rejects_hidden_releases_fifo_checksum_and_impossible_timelines() {
        let original = transcript(Profile::RuntimeStatsEnabled);
        for (kind, expected) in [
            (0, "identity"),
            (1, "latest nominal slot"),
            (2, "checksum"),
            (3, "service ordering"),
            (4, "overlap"),
            (5, "depth"),
            (6, "invented"),
            (7, "no-work"),
            (8, "forced-delay"),
            (9, "drain bound"),
            (10, "accepted history"),
            (11, "accepted history"),
            (12, "accepted history"),
        ] {
            let mut records = original.clone();
            match kind {
                0 => row(&mut records, 2, 0)["global_index"] = json!(0),
                1 => row(&mut records, 2, 1)["attempt_offset_us"] = json!(0),
                2 => row(&mut records, 2, 0)["checksum"] = json!(0),
                3 => {
                    let r = row(&mut records, 2, 0);
                    r["finish_offset_us"] = r["start_offset_us"].clone();
                }
                4 => row(&mut records, 2, 0)["finish_offset_us"] = json!(2013),
                5 => row(&mut records, 2, 0)["queue_depth"] = json!(9),
                6 => row(&mut records, 1, 0)["checksum"] = json!(1),
                7 => {
                    let r = row(&mut records, 0, 0);
                    r["status"] = json!("generator-missed");
                    for field in [
                        "attempt_offset_us",
                        "start_offset_us",
                        "finish_offset_us",
                        "checksum",
                        "queue_depth",
                    ] {
                        r[field] = json!(0);
                    }
                    let end = event(&mut records, "phase-end", 0);
                    end["callback_count"] = json!(end["callback_count"].as_u64().unwrap() - 1);
                }
                8 => {
                    let r = row(&mut records, 1, 0);
                    r["status"] = json!("completed");
                    r["queue_depth"] = json!(1);
                    r["checksum"] = json!(bench_core::DEFAULT_SEED ^ 125);
                    let end = event(&mut records, "phase-end", 1);
                    end["callback_count"] = json!(end["callback_count"].as_u64().unwrap() + 1);
                }
                9 => row(&mut records, 2, 0)["finish_offset_us"] = json!(350_001),
                10 => {
                    let r = row(&mut records, 2, 0);
                    r["status"] = json!("queue-full");
                    r["queue_depth"] = json!(8);
                    r["start_offset_us"] = json!(0);
                    r["finish_offset_us"] = json!(0);
                    r["checksum"] = json!(0);
                }
                11 => {
                    let r = row(&mut records, 2, 10);
                    r["status"] = json!("queue-full");
                    r["queue_depth"] = json!(8);
                    r["start_offset_us"] = json!(0);
                    r["finish_offset_us"] = json!(0);
                    r["checksum"] = json!(0);
                }
                12 => row(&mut records, 2, 10)["queue_depth"] = json!(8),
                _ => unreachable!(),
            }
            rejects(&records, Profile::RuntimeStatsEnabled, expected);
        }
        let mut missing = original.clone();
        missing.remove(4);
        rejects(&missing, Profile::RuntimeStatsEnabled, "identity");
        let mut extra = original;
        extra.insert(4, extra[4].clone());
        rejects(&extra, Profile::RuntimeStatsEnabled, "identity");
    }

    #[test]
    fn rejects_callback_quiescence_and_post_drain_lies() {
        let original = transcript(Profile::RuntimeStatsEnabled);
        for (field, value, expected) in [
            ("timer_stop_pass", json!(false), "quiescence"),
            ("queue_empty", json!(false), "quiescence"),
            ("notification_strays", json!(1), "quiescence"),
            ("callback_count", json!(1), "callback count"),
            ("early_wakes", json!(5000), "callback count"),
            ("generation_done_us", json!(0), "chronology"),
            ("drain_done_us", json!(u64::MAX), "chronology"),
        ] {
            let mut records = original.clone();
            event(&mut records, "phase-end", 0)[field] = value;
            rejects(&records, Profile::RuntimeStatsEnabled, expected);
        }
        let mut records = original;
        let drain = event(&mut records, "phase-end", 0)["drain_done_us"]
            .as_u64()
            .unwrap();
        let after = records
            .iter_mut()
            .find(|r| r["event"] == "snapshot" && r["phase"] == 0 && r["boundary"] == "after")
            .unwrap();
        for (offset, key) in [
            "bracket_start_us",
            "bracket_end_us",
            "tasks_start_us",
            "total_runtime_us",
            "tasks_end_us",
            "heap_start_us",
            "heap_end_us",
        ]
        .into_iter()
        .enumerate()
        {
            after[key] = json!(drain - 100 + offset as u64);
        }
        rejects(&records, Profile::RuntimeStatsEnabled, "precedes");
    }

    #[test]
    fn rejects_counter_identity_epoch_and_observer_corruption() {
        let original = transcript(Profile::RuntimeStatsEnabled);
        for (kind, expected) in [
            (0, "width"),
            (1, "handles"),
            (2, "duplicate"),
            (3, "settled"),
            (4, "priority"),
            (5, "reservation"),
            (6, "total clock"),
            (7, "epoch"),
            (8, "20ms"),
            (9, "largest"),
            (10, "unordered"),
            (11, "null"),
        ] {
            let mut records = original.clone();
            match kind {
                0 => records[0]["counter_bits"] = json!(32),
                1 => records[0]["timer_handle"] = json!(2),
                2 => records[3]["tasks"][1]["number"] = json!(11),
                3 => records[3]["tasks"][1]["state"] = json!(1),
                4 => records[3]["tasks"][3]["priority"] = json!(5),
                5 => records[3]["tasks"][1]["stack_free_bytes"] = json!(8193),
                6 => records[3]["tasks"][0]["runtime_us"] = json!(u64::MAX),
                7 => records[3]["producer_phase"] = json!(1),
                8 => {
                    let epoch = records[3]["heap_end_us"].as_u64().unwrap();
                    records[2]["epoch_us"] = json!(epoch);
                    records[2]["cutoff_us"] = json!(epoch + 250_000);
                }
                9 => records[3]["heap"]["largest_free_block"] = json!(u32::MAX),
                10 => records[3]["tasks_end_us"] = json!(0),
                11 => records[3]["total_runtime_us"] = Value::Null,
                _ => unreachable!(),
            }
            rejects(&records, Profile::RuntimeStatsEnabled, expected);
        }
        let mut records = original.clone();
        let after = records
            .iter_mut()
            .find(|r| r["event"] == "snapshot" && r["boundary"] == "after")
            .unwrap();
        after["tasks"][1]["number"] = json!(90);
        rejects(&records, Profile::RuntimeStatsEnabled, "reused");
        let mut records = original;
        let after = records
            .iter_mut()
            .find(|r| r["event"] == "snapshot" && r["boundary"] == "after")
            .unwrap();
        after["tasks"][1]["runtime_us"] = json!(0);
        rejects(&records, Profile::RuntimeStatsEnabled, "decreased");
    }

    #[test]
    fn rejects_unknown_fields_bad_configuration_and_incomplete_cleanup() {
        let original = transcript(Profile::RuntimeStatsDisabled);
        let mut records = original.clone();
        records[4]["pending"] = json!(false);
        rejects(&records, Profile::RuntimeStatsDisabled, "unknown field");
        let mut records = original.clone();
        records[0]["config_before"]["clocks"]["cpu_mhz"] = json!(80);
        rejects(&records, Profile::RuntimeStatsDisabled, "CPU");
        let mut records = original.clone();
        records.last_mut().unwrap()["timer_closed"] = json!(false);
        rejects(&records, Profile::RuntimeStatsDisabled, "cleanup");
        let mut records = original;
        records.pop();
        rejects(&records, Profile::RuntimeStatsDisabled, "incomplete");
    }

    struct Chunked<'a> {
        bytes: &'a [u8],
        width: usize,
    }
    impl Read for Chunked<'_> {
        fn read(&mut self, target: &mut [u8]) -> io::Result<usize> {
            let n = target.len().min(self.width).min(self.bytes.len());
            target[..n].copy_from_slice(&self.bytes[..n]);
            self.bytes = &self.bytes[n..];
            Ok(n)
        }
    }

    #[test]
    fn actual_capture_and_replay_reject_fatal_reboots_at_split_boundaries() {
        let valid = bytes(&transcript(Profile::RuntimeStatsDisabled));
        for (fault, expected) in [
            ("Guru Meditation Error: Core0 panic'ed\n", "fatal or reboot"),
            ("Rebooting...\n", "fatal or reboot"),
            ("ESP-ROM:esp32c6-20220919\n", "additional ROM"),
        ] {
            let altered = [fault.as_bytes(), &valid].concat();
            for width in [1, 7, 1024] {
                let mut boot = Boot::default();
                let mut raw = Vec::new();
                let error = capture_stream(
                    &mut Chunked {
                        bytes: &altered,
                        width,
                    },
                    &mut raw,
                    &mut Vec::new(),
                    &mut boot,
                    Profile::RuntimeStatsDisabled,
                    Duration::from_secs(5),
                )
                .unwrap_err();
                assert!(format!("{error:#}").contains(expected));
                assert!(boot.report_fields().is_err());
                assert!(!raw.is_empty());
            }
            let error = validate_bytes(&altered, Profile::RuntimeStatsDisabled).unwrap_err();
            assert!(format!("{error:#}").contains(expected));
        }
        let mut boot = Boot::default();
        let mut raw = Vec::new();
        capture_stream(
            &mut io::Cursor::new(&valid),
            &mut raw,
            &mut Vec::new(),
            &mut boot,
            Profile::RuntimeStatsDisabled,
            Duration::from_secs(5),
        )
        .unwrap();
        assert_eq!(raw, valid);
        let trailing = [valid.as_slice(), b"panic after complete\n"].concat();
        assert!(validate_bytes(&trailing, Profile::RuntimeStatsDisabled).is_err());
    }

    static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);
    struct Temporary(PathBuf);
    impl Temporary {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "esp32c6-load-host-{}-{}-{}",
                std::process::id(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos(),
                NEXT_TEMP.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&path).unwrap();
            Self(path)
        }
    }
    impl Drop for Temporary {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn capture_preflight_and_cli_require_qualification_and_profile() {
        let temporary = Temporary::new();
        let output = temporary.0.join("run");
        let config = Capture {
            purpose: CapturePurpose::Comparison,
            profile: Profile::RuntimeStatsDisabled,
            port: "invalid-device",
            output: &output,
            elf: Path::new("missing-elf"),
            provenance: Path::new("missing-provenance"),
            boot_id: "fixture",
            run_order: 1,
            timeout: Duration::from_secs(1),
        };
        assert!(format!("{:#}", capture(config).unwrap_err()).contains("qualification only"));
        assert!(!output.exists());
        let options = [
            "benchctl",
            "capture-load",
            "--port",
            "none",
            "--output",
            "none",
            "--elf",
            "none",
            "--provenance",
            "none",
            "--boot-id",
            "none",
            "--run-order",
            "1",
        ];
        assert!(crate::Cli::try_parse_from(options).is_err());
        let mut purpose_only = options.to_vec();
        purpose_only.extend(["--purpose", "qualification"]);
        assert!(crate::Cli::try_parse_from(purpose_only).is_err());
        assert!(
            crate::Cli::try_parse_from(["benchctl", "validate-load", "--input", "none"]).is_err()
        );
    }

    #[test]
    fn started_failed_port_attempt_retains_original_inputs_and_no_scores() {
        let temporary = Temporary::new();
        let elf = temporary.0.join("input.elf");
        let provenance = temporary.0.join("input.json");
        let output = temporary.0.join("run");
        fs::write(&elf, b"structural host fixture").unwrap();
        fs::write(&provenance, b"{ \"host_fixture\" : true }\n").unwrap();
        let result = capture(Capture {
            purpose: CapturePurpose::Qualification,
            profile: Profile::RuntimeStatsEnabled,
            port: "/does/not/exist/esp32c6-load-fixture",
            output: &output,
            elf: &elf,
            provenance: &provenance,
            boot_id: "host-fixture",
            run_order: 1,
            timeout: Duration::from_secs(1),
        });
        assert!(format!("{:#}", result.unwrap_err()).contains("opening load serial port"));
        let report: Value =
            serde_json::from_slice(&fs::read(output.join("run.json")).unwrap()).unwrap();
        assert_eq!(report["status"], "failed");
        assert!(report["qualification"].is_null() && report["benchmarks"].is_null());
        assert!(report["comparative_score"].is_null());
        assert_eq!(
            fs::read(output.join("provenance.json")).unwrap(),
            fs::read(provenance).unwrap()
        );
        assert_eq!(
            fs::read(output.join("firmware.elf")).unwrap(),
            fs::read(elf).unwrap()
        );
        assert!(output.join("serial.raw").exists());
    }

    fn archived_capture(root: &Path) -> (PathBuf, Value) {
        let directory = root.join("results/host-only");
        fs::create_dir_all(&directory).unwrap();
        let profile = Profile::RuntimeStatsEnabled;
        let raw = bytes(&transcript(profile));
        let value = validate_bytes(&raw, profile).unwrap();
        let elf = b"synthetic host ELF identity";
        let provenance = b"{ \"host_fixture\": true }\n";
        let admission = Admission {
            purpose: CapturePurpose::Qualification,
            elf_sha256: digest(elf),
            provenance_sha256: digest(provenance),
            method: None,
        };
        let mut report = json!({"schema_version": 4, "experiment": EXPERIMENT, "profile": profile,
            "status": "pass", "admission": admission, "firmware_sha256": digest(elf),
            "collector_executable_sha256": "host-fixture", "benchmarks": null,
            "comparative_score": null, "provenance": serde_json::from_slice::<Value>(provenance).unwrap()});
        for (key, field) in value.as_object().unwrap() {
            report[key] = field.clone();
        }
        fs::write(directory.join("firmware.elf"), elf).unwrap();
        fs::write(directory.join("provenance.json"), provenance).unwrap();
        fs::write(directory.join("serial.raw"), raw).unwrap();
        write_json(File::create(directory.join("replay.json")).unwrap(), &value).unwrap();
        write_json(File::create(directory.join("run.json")).unwrap(), &report).unwrap();
        (directory, report)
    }

    #[test]
    fn archive_replay_binds_profile_scores_provenance_and_actual_replay() {
        let temporary = Temporary::new();
        let (directory, original) = archived_capture(&temporary.0);
        assert_eq!(
            crate::qualification::replay_archives(&temporary.0).unwrap()["successful_protocol_transcripts"],
            1
        );
        for kind in 0..5 {
            let mut report = original.clone();
            match kind {
                0 => report["admission"]["purpose"] = json!("comparison"),
                1 => report["profile"] = json!("runtime-stats-disabled"),
                2 => report["benchmarks"] = json!({"unsupported_score": 1}),
                3 => report["comparative_score"] = json!(1),
                4 => report["phases"][0]["counts"]["accepted"] = json!(0),
                _ => unreachable!(),
            }
            write_json(File::create(directory.join("run.json")).unwrap(), &report).unwrap();
            assert!(
                crate::qualification::replay_archives(&temporary.0).is_err(),
                "case {kind}"
            );
        }
        write_json(File::create(directory.join("run.json")).unwrap(), &original).unwrap();
        fs::write(directory.join("provenance.json"), b"{}").unwrap();
        assert!(crate::qualification::replay_archives(&temporary.0).is_err());
    }

    #[test]
    fn failed_archive_is_never_replayed_into_success_or_kept_with_phase_scores() {
        let temporary = Temporary::new();
        let (directory, mut report) = archived_capture(&temporary.0);
        report["status"] = json!("failed");
        report["failure"] = json!("host injected failure");
        report["qualification"] = Value::Null;
        write_json(File::create(directory.join("run.json")).unwrap(), &report).unwrap();
        assert!(crate::qualification::replay_archives(&temporary.0).is_err());
        report["phases"] = Value::Null;
        write_json(File::create(directory.join("run.json")).unwrap(), &report).unwrap();
        fs::write(
            directory.join("serial.raw"),
            b"panic retained host failure\n",
        )
        .unwrap();
        let output = crate::qualification::replay_archives(&temporary.0).unwrap();
        assert_eq!(output["failed_captures_retained"], 1);
        assert_eq!(output["successful_protocol_transcripts"], 0);
    }
}
