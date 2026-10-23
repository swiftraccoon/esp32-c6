//! Validate one complete boot before making its measurements reportable.

use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use bench_core::{DEFAULT_CHECKSUM, DEFAULT_ROUNDS, DEFAULT_SAMPLES, DEFAULT_SEED, WORKLOAD_ID};
use serde::ser::SerializeMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

const MESSAGE_SAMPLES: u32 = 1_000;
const MESSAGE_CHECKSUM_MASK: u32 = 0xa5a5_5a5a;

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "event", rename_all = "kebab-case")]
pub(crate) enum Record {
    Hello(Hello),
    Sample(Sample),
    Complete(Complete),
}

#[derive(Debug, Deserialize)]
pub(crate) struct Complete {
    protocol: u32,
    clocks_after: Option<Clocks>,
    samples: Option<u32>,
    compute_samples: Option<u32>,
    timer_overhead_samples: Option<u32>,
    message_roundtrip_samples: Option<u32>,
}

impl Serialize for Complete {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("protocol", &self.protocol)?;
        // Legacy parsed records intentionally discarded completion metadata.
        // Keep its archival output stable; protocol 2 preserves these fields.
        if self.protocol == 2 {
            if let Some(value) = &self.clocks_after {
                map.serialize_entry("clocks_after", value)?;
            }
            if let Some(value) = self.samples {
                map.serialize_entry("samples", &value)?;
            }
            if let Some(value) = self.compute_samples {
                map.serialize_entry("compute_samples", &value)?;
            }
            if let Some(value) = self.timer_overhead_samples {
                map.serialize_entry("timer_overhead_samples", &value)?;
            }
            if let Some(value) = self.message_roundtrip_samples {
                map.serialize_entry("message_roundtrip_samples", &value)?;
            }
        }
        map.end()
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct Hello {
    protocol: u32,
    platform: String,
    cpu_mhz: u32,
    sample_count: u32,
    seed: u32,
    rounds: u32,
    workload: String,
    warmup_iterations: u32,
    timer_resolution_ns: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    benchmarks: Option<BTreeMap<String, u32>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    clocks: Option<Clocks>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    message: Option<Value>,
    // Protocol 1 historically accepted and discarded additive metadata. Keep
    // that replay behavior while rejecting unknown protocol 2 configuration.
    #[serde(flatten, skip_serializing)]
    extra: BTreeMap<String, Value>,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Clocks {
    source: String,
    cpu_mhz: u32,
    flash_source_mhz: u32,
    flash_mhz: u32,
    flash_divider: u32,
    registers: ClockRegisters,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct ClockRegisters {
    pcr_sysclk_conf: u32,
    pcr_cpu_freq_conf: u32,
    pcr_mspi_clk_conf: u32,
    spi0_clock: u32,
}

impl Clocks {
    /// Decodes the C6 PLL configuration, independently of firmware's numbers.
    ///
    /// This is configuration readback, not an external frequency measurement.
    /// Offsets/fields follow the pinned esp32c6 0.24.0 PAC PCR/SPI0 registers.
    pub(crate) fn validate(&self) -> Result<()> {
        ensure!(
            self.source == "register-readback",
            "clocks lack register-readback evidence"
        );
        let registers = &self.registers;
        ensure!(
            (registers.pcr_sysclk_conf >> 16) & 3 == 1,
            "unsupported CPU clock source"
        );
        let system_divider = ((registers.pcr_sysclk_conf >> 8) & 255) + 1;
        ensure!(system_divider == 3, "unexpected PLL system divider");
        let cpu_divider_field = (registers.pcr_cpu_freq_conf >> 8) & 255;
        ensure!(
            matches!(cpu_divider_field, 0 | 1 | 3),
            "unsupported CPU divider"
        );
        let cpu_mhz = if cpu_divider_field == 0 && registers.pcr_cpu_freq_conf & (1 << 16) != 0 {
            120
        } else {
            480 / system_divider / (cpu_divider_field + 1)
        };
        let flash_divider_field = (registers.pcr_mspi_clk_conf >> 8) & 255;
        ensure!(
            matches!(flash_divider_field, 3..=5),
            "unsupported flash source divider"
        );
        let flash_source_mhz = 480 / (flash_divider_field + 1);
        let flash_divider = if registers.spi0_clock & (1 << 31) != 0 {
            1
        } else {
            ((registers.spi0_clock >> 16) & 255) + 1
        };
        ensure!(
            flash_source_mhz.is_multiple_of(flash_divider),
            "fractional flash MHz is unsupported"
        );
        let flash_mhz = flash_source_mhz / flash_divider;
        ensure!(
            self.cpu_mhz == cpu_mhz && cpu_mhz == 160,
            "CPU readback differs from 160 MHz baseline"
        );
        ensure!(
            self.flash_source_mhz == flash_source_mhz && flash_source_mhz == 80,
            "flash source readback differs from 80 MHz baseline"
        );
        ensure!(
            self.flash_divider == flash_divider && flash_divider == 1,
            "flash divider readback differs from baseline"
        );
        ensure!(
            self.flash_mhz == flash_mhz && flash_mhz == 80,
            "flash readback differs from 80 MHz baseline"
        );
        Ok(())
    }
}

impl Hello {
    fn benchmark_counts(&self, expected_platform: &str) -> Result<BTreeMap<String, u32>> {
        ensure!(matches!(self.protocol, 1 | 2), "unsupported hello protocol");
        ensure!(
            self.platform == expected_platform,
            "unexpected firmware platform"
        );
        ensure!(
            self.cpu_mhz == 160,
            "CPU configuration differs from baseline"
        );
        ensure!(
            self.sample_count == DEFAULT_SAMPLES,
            "unexpected sample count"
        );
        ensure!(self.seed == DEFAULT_SEED, "unexpected input seed");
        ensure!(self.rounds == DEFAULT_ROUNDS, "unexpected workload length");
        ensure!(self.workload == WORKLOAD_ID, "unexpected workload revision");
        ensure!(self.warmup_iterations == 8, "unexpected warmup count");
        ensure!(
            self.timer_resolution_ns == 1_000,
            "unexpected timer resolution"
        );
        let mut counts = BTreeMap::from([
            ("compute".into(), DEFAULT_SAMPLES),
            ("timer-overhead".into(), DEFAULT_SAMPLES),
        ]);
        if self.protocol == 2 {
            ensure!(
                self.extra.is_empty(),
                "unknown protocol 2 hello configuration"
            );
            match self.platform.as_str() {
                "bare-metal" => {}
                "embassy" | "esp-idf-freertos" => {
                    counts.insert("message-roundtrip".into(), MESSAGE_SAMPLES);
                }
                _ => anyhow::bail!("unknown protocol 2 platform"),
            }
            ensure!(
                self.benchmarks.as_ref() == Some(&counts),
                "benchmark declaration differs from platform contract"
            );
            self.clocks
                .as_ref()
                .context("missing initial clock readback")?
                .validate()?;
            if self.platform == "bare-metal" {
                ensure!(
                    self.message.is_none(),
                    "bare-metal cannot declare scheduler message metadata"
                );
            } else {
                let message = self.message.as_ref().context("missing message metadata")?;
                ensure!(message.is_object(), "message metadata must be an object");
                ensure!(
                    message["transport"]
                        .as_str()
                        .is_some_and(|value| !value.trim().is_empty()),
                    "missing message transport description"
                );
                ensure!(
                    message["queue_capacity"] == 1,
                    "unexpected message queue capacity"
                );
                ensure!(
                    message["warmup_iterations"] == 8,
                    "unexpected message warmup count"
                );
            }
        }
        Ok(counts)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct Sample {
    protocol: u32,
    benchmark: String,
    index: u32,
    iterations: u32,
    elapsed_ns: u64,
    checksum: u32,
}

#[derive(Debug, Serialize)]
pub(crate) struct Summary {
    sample_count: usize,
    min_ns: u64,
    p50_ns: u64,
    p95_ns: u64,
    p99_ns: u64,
    max_ns: u64,
    mean_ns: f64,
}

impl Summary {
    fn from_samples(samples: &[Sample]) -> Self {
        // Only called after validating a nonempty, complete sample set.
        let mut times: Vec<u64> = samples.iter().map(|sample| sample.elapsed_ns).collect();
        times.sort_unstable();
        let nearest_rank = |percent: usize| times[(percent * times.len()).div_ceil(100) - 1];
        Self {
            sample_count: times.len(),
            min_ns: times[0],
            p50_ns: nearest_rank(50),
            p95_ns: nearest_rank(95),
            p99_ns: nearest_rank(99),
            max_ns: times[times.len() - 1],
            mean_ns: times.iter().map(|&time| time as f64).sum::<f64>() / times.len() as f64,
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct Boot {
    hello: Option<Hello>,
    clocks_after: Option<Clocks>,
    expected_counts: BTreeMap<String, u32>,
    samples: BTreeMap<String, Vec<Sample>>,
    complete: bool,
    rejected: bool,
}

impl Boot {
    /// Permanently revokes numeric summaries after a framing, I/O or text failure.
    pub(crate) fn reject(&mut self) {
        self.rejected = true;
    }

    /// Returns true only when this record closes a valid, complete boot.
    ///
    /// Any rejected record permanently invalidates this boot, even if a caller
    /// attempts to continue feeding records after an error.
    ///
    /// # Errors
    ///
    /// Rejects incompatible configuration, invalid checksums, missing/duplicate
    /// indices, timer-unit inconsistencies, premature completion and rebooted or
    /// already rejected runs. Zero compute duration is invalid; zero observer
    /// or message-roundtrip duration remains valid at microsecond resolution.
    pub(crate) fn accept(&mut self, record: Record, expected_platform: &str) -> Result<bool> {
        ensure!(!self.rejected, "boot was previously rejected");
        let result = self.accept_record(record, expected_platform);
        if result.is_err() {
            self.rejected = true;
        }
        result
    }

    fn accept_record(&mut self, record: Record, expected_platform: &str) -> Result<bool> {
        ensure!(!self.complete, "record arrived after completion");
        match record {
            Record::Hello(hello) => {
                ensure!(
                    self.hello.is_none(),
                    "duplicate hello or unexpected device reboot"
                );
                self.expected_counts = hello.benchmark_counts(expected_platform)?;
                self.hello = Some(hello);
            }
            Record::Sample(sample) => {
                let hello = self.hello.as_ref().context("sample arrived before hello")?;
                ensure!(
                    sample.protocol == hello.protocol,
                    "inconsistent sample protocol"
                );
                let count = self
                    .expected_counts
                    .get(&sample.benchmark)
                    .context("undeclared benchmark")?;
                ensure!(sample.index < *count, "sample index out of range");
                match sample.benchmark.as_str() {
                    "compute" => {
                        ensure!(sample.elapsed_ns > 0, "compute timer did not advance");
                        ensure!(
                            sample.iterations == DEFAULT_ROUNDS,
                            "wrong compute round count"
                        );
                        ensure!(
                            sample.checksum == DEFAULT_CHECKSUM,
                            "workload checksum mismatch"
                        );
                    }
                    "timer-overhead" => {
                        ensure!(
                            sample.iterations == 0,
                            "calibration contains workload iterations"
                        );
                        ensure!(sample.checksum == 0, "unexpected calibration checksum");
                    }
                    "message-roundtrip" => {
                        ensure!(sample.iterations == 1, "wrong message roundtrip count");
                        ensure!(
                            sample.checksum == sample.index ^ MESSAGE_CHECKSUM_MASK,
                            "message checksum mismatch"
                        );
                    }
                    _ => anyhow::bail!("unknown benchmark {:?}", sample.benchmark),
                }
                ensure!(
                    sample.elapsed_ns % 1_000 == 0,
                    "time disagrees with reported resolution"
                );
                let samples = self.samples.entry(sample.benchmark.clone()).or_default();
                ensure!(
                    samples.len() == sample.index as usize,
                    "duplicate or missing sample index"
                );
                samples.push(sample);
            }
            Record::Complete(Complete {
                protocol,
                clocks_after,
                samples,
                compute_samples,
                timer_overhead_samples,
                message_roundtrip_samples,
            }) => {
                let hello = self
                    .hello
                    .as_ref()
                    .context("completion arrived before hello")?;
                ensure!(
                    protocol == hello.protocol,
                    "inconsistent completion protocol"
                );
                if protocol == 2 {
                    let after = clocks_after
                        .as_ref()
                        .context("missing final clock readback")?;
                    after.validate()?;
                    ensure!(
                        Some(after) == hello.clocks.as_ref(),
                        "clock registers changed during measurement"
                    );
                    ensure!(
                        samples == Some(self.expected_counts.values().sum()),
                        "wrong completion total"
                    );
                    ensure!(
                        compute_samples == Some(DEFAULT_SAMPLES),
                        "wrong completion compute count"
                    );
                    ensure!(
                        timer_overhead_samples == Some(DEFAULT_SAMPLES),
                        "wrong completion calibration count"
                    );
                    ensure!(
                        message_roundtrip_samples
                            == self.expected_counts.get("message-roundtrip").copied(),
                        "wrong completion message count"
                    );
                }
                for (benchmark, expected_count) in &self.expected_counts {
                    let count = self.samples.get(benchmark).map_or(0, Vec::len);
                    ensure!(
                        count == *expected_count as usize,
                        "incomplete {benchmark}: {count} samples"
                    );
                }
                self.clocks_after = clocks_after;
                self.complete = true;
            }
        }
        Ok(self.complete)
    }

    pub(crate) fn hello(&self) -> Option<&Hello> {
        self.hello.as_ref()
    }

    /// Returns final register readback only after accepted protocol completion.
    pub(crate) fn clocks_after(&self) -> Option<&Clocks> {
        self.clocks_after.as_ref()
    }

    /// Returns nearest-rank statistics only for a complete, unrejected boot.
    ///
    /// # Errors
    ///
    /// An incomplete or rejected boot has no reportable numeric summary.
    pub(crate) fn summaries(&self) -> Result<BTreeMap<String, Summary>> {
        ensure!(!self.rejected, "rejected boot has no reportable summary");
        ensure!(self.complete, "incomplete boot has no reportable summary");
        Ok(self
            .samples
            .iter()
            .map(|(name, samples)| (name.clone(), Summary::from_samples(samples)))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn hello() -> Record {
        Record::Hello(Hello {
            protocol: 1,
            platform: "test".into(),
            cpu_mhz: 160,
            sample_count: DEFAULT_SAMPLES,
            seed: DEFAULT_SEED,
            rounds: DEFAULT_ROUNDS,
            workload: WORKLOAD_ID.into(),
            warmup_iterations: 8,
            timer_resolution_ns: 1_000,
            benchmarks: None,
            clocks: None,
            message: None,
            extra: BTreeMap::new(),
        })
    }

    fn complete() -> Record {
        Record::Complete(Complete {
            protocol: 1,
            clocks_after: None,
            samples: None,
            compute_samples: None,
            timer_overhead_samples: None,
            message_roundtrip_samples: None,
        })
    }

    fn clocks() -> Value {
        json!({
            "source": "register-readback", "cpu_mhz": 160,
            "flash_source_mhz": 80, "flash_mhz": 80, "flash_divider": 1,
            "registers": {
                "pcr_sysclk_conf": 0x0001_0200_u32,
                "pcr_cpu_freq_conf": 0,
                "pcr_mspi_clk_conf": 0x0000_0500_u32,
                "spi0_clock": 0x8000_0000_u32
            }
        })
    }

    fn v2_hello(platform: &str) -> Value {
        let mut hello = serde_json::to_value(hello()).unwrap();
        hello["protocol"] = json!(2);
        hello["platform"] = json!(platform);
        hello["clocks"] = clocks();
        hello["benchmarks"] = json!({"compute": 64, "timer-overhead": 64});
        if platform != "bare-metal" {
            hello["benchmarks"]["message-roundtrip"] = json!(1_000);
            hello["message"] =
                json!({"transport": "test", "queue_capacity": 1, "warmup_iterations": 8});
        }
        hello
    }

    fn v2_complete(platform: &str) -> Value {
        let mut complete = json!({
            "event": "complete", "protocol": 2, "clocks_after": clocks(),
            "samples": 128, "compute_samples": 64, "timer_overhead_samples": 64
        });
        if platform != "bare-metal" {
            complete["samples"] = json!(1_128);
            complete["message_roundtrip_samples"] = json!(1_000);
        }
        complete
    }

    fn v2_sample(benchmark: &str, index: u32) -> Record {
        let mut sample = serde_json::to_value(sample(benchmark, index)).unwrap();
        sample["protocol"] = json!(2);
        if benchmark == "message-roundtrip" {
            sample["iterations"] = json!(1);
            sample["checksum"] = json!(index ^ 0xa5a5_5a5a);
            sample["elapsed_ns"] = json!(u64::from(index) * 1_000);
        }
        serde_json::from_value(sample).unwrap()
    }

    fn v2_boot(platform: &str, message_count: u32) -> Boot {
        let mut boot = Boot::default();
        boot.accept(
            serde_json::from_value(v2_hello(platform)).unwrap(),
            platform,
        )
        .unwrap();
        for benchmark in ["compute", "timer-overhead"] {
            for index in 0..DEFAULT_SAMPLES {
                boot.accept(v2_sample(benchmark, index), platform).unwrap();
            }
        }
        for index in 0..message_count {
            boot.accept(v2_sample("message-roundtrip", index), platform)
                .unwrap();
        }
        boot
    }

    #[test]
    fn v2_supports_complete_bare_and_scheduler_boots_without_inventing_results() {
        for platform in ["bare-metal", "embassy", "esp-idf-freertos"] {
            let message_count = if platform == "bare-metal" { 0 } else { 1_000 };
            let mut boot = v2_boot(platform, message_count);
            let complete = v2_complete(platform);
            let record = serde_json::from_value(complete.clone()).unwrap();
            assert_eq!(serde_json::to_value(&record).unwrap(), complete);
            boot.accept(record, platform).unwrap();
            let summary = boot.summaries().unwrap();
            assert_eq!(summary["compute"].sample_count, 64);
            assert_eq!(summary["timer-overhead"].sample_count, 64);
            if platform == "bare-metal" {
                assert!(!summary.contains_key("message-roundtrip"));
            } else {
                assert_eq!(summary["message-roundtrip"].sample_count, 1_000);
                assert_eq!(summary["message-roundtrip"].min_ns, 0);
                assert_eq!(summary["message-roundtrip"].p99_ns, 989_000);
                assert_eq!(summary["message-roundtrip"].max_ns, 999_000);
                assert_eq!(summary["message-roundtrip"].mean_ns, 499_500.0);
            }
        }
    }

    #[test]
    fn v2_rejects_false_or_unsupported_clock_claims() {
        let mut changes = Vec::new();
        let mut nominal_only = v2_hello("bare-metal");
        nominal_only["clocks"]
            .as_object_mut()
            .unwrap()
            .remove("registers");
        changes.push(nominal_only);
        let mut absent = v2_hello("bare-metal");
        absent.as_object_mut().unwrap().remove("clocks");
        changes.push(absent);
        for (field, wrong) in [
            ("source", json!("configuration")),
            ("cpu_mhz", json!(120)),
            ("flash_mhz", json!(40)),
            ("flash_divider", json!(2)),
        ] {
            let mut hello = v2_hello("bare-metal");
            hello["clocks"][field] = wrong;
            changes.push(hello);
        }
        for (field, wrong) in [
            ("pcr_sysclk_conf", 0x0000_0200_u32),
            ("pcr_cpu_freq_conf", 0x0001_0000_u32),
            ("pcr_mspi_clk_conf", 0x0000_0300_u32),
            ("spi0_clock", 0x0001_0000_u32),
        ] {
            let mut hello = v2_hello("bare-metal");
            hello["clocks"]["registers"][field] = json!(wrong);
            changes.push(hello);
        }
        for hello in changes {
            // Missing typed register fields fail at decoding; inconsistent
            // values fail when accepting the otherwise well-formed record.
            if let Ok(record) = serde_json::from_value(hello) {
                let mut boot = Boot::default();
                assert!(boot.accept(record, "bare-metal").is_err());
                assert!(boot.summaries().is_err());
            }
        }
    }

    #[test]
    fn v2_requires_stable_clock_registers_and_truthful_completion_counts() {
        let mut changes = Vec::new();
        let mut no_readback = v2_complete("embassy");
        no_readback.as_object_mut().unwrap().remove("clocks_after");
        changes.push(no_readback);
        let mut changed = v2_complete("embassy");
        // Unrelated LS-divider bit: decoded baseline still agrees, but the
        // complete raw configuration must remain unchanged across timing.
        changed["clocks_after"]["registers"]["pcr_sysclk_conf"] = json!(0x0001_0201);
        changes.push(changed);
        for field in [
            "samples",
            "compute_samples",
            "timer_overhead_samples",
            "message_roundtrip_samples",
        ] {
            let mut wrong = v2_complete("embassy");
            wrong[field] = json!(0);
            changes.push(wrong);
        }
        for complete in changes {
            let mut boot = v2_boot("embassy", 1_000);
            assert!(
                boot.accept(serde_json::from_value(complete).unwrap(), "embassy")
                    .is_err()
            );
            assert!(boot.summaries().is_err());
        }
        let mut missing_sample = v2_boot("embassy", 999);
        assert!(
            missing_sample
                .accept(
                    serde_json::from_value(v2_complete("embassy")).unwrap(),
                    "embassy"
                )
                .is_err()
        );
    }

    #[test]
    fn v2_declaration_is_exact_for_the_selected_platform() {
        let mut changes = Vec::new();
        for platform in ["bare-metal", "embassy"] {
            for (benchmark, count) in [("compute", 63), ("timer-overhead", 65), ("unknown", 1)] {
                let mut hello = v2_hello(platform);
                hello["benchmarks"][benchmark] = json!(count);
                changes.push((platform, hello));
            }
        }
        let mut omitted = v2_hello("embassy");
        omitted["benchmarks"]
            .as_object_mut()
            .unwrap()
            .remove("message-roundtrip");
        changes.push(("embassy", omitted));
        let mut fabricated = v2_hello("bare-metal");
        fabricated["benchmarks"]["message-roundtrip"] = json!(1_000);
        changes.push(("bare-metal", fabricated));
        let mut unknown_config = v2_hello("bare-metal");
        unknown_config["clock_nominal_mhz"] = json!(160);
        changes.push(("bare-metal", unknown_config));
        for (platform, hello) in changes {
            let mut boot = Boot::default();
            assert!(
                boot.accept(serde_json::from_value(hello).unwrap(), platform)
                    .is_err()
            );
        }
    }

    #[test]
    fn v2_requires_bounded_message_metadata_only_for_schedulers() {
        for replacement in [
            Value::Null,
            json!({}),
            json!({
                "transport": " ", "queue_capacity": 1, "warmup_iterations": 8
            }),
            json!({
                "transport": "channel", "queue_capacity": 2, "warmup_iterations": 8
            }),
            json!({
                "transport": "channel", "queue_capacity": 1, "warmup_iterations": 0
            }),
        ] {
            let mut hello = v2_hello("embassy");
            hello["message"] = replacement;
            assert!(
                Boot::default()
                    .accept(serde_json::from_value(hello).unwrap(), "embassy")
                    .is_err()
            );
        }
        let mut hello = v2_hello("bare-metal");
        hello["message"] =
            json!({"transport": "channel", "queue_capacity": 1, "warmup_iterations": 8});
        assert!(
            Boot::default()
                .accept(serde_json::from_value(hello).unwrap(), "bare-metal")
                .is_err()
        );
    }

    #[test]
    fn v1_completion_metadata_keeps_its_original_parsed_output() {
        let record: Record = serde_json::from_value(json!({
            "event": "complete", "protocol": 1, "samples": 999,
            "compute_samples": 0, "timer_overhead_samples": 0,
            "upstream_optional_diagnostic": true
        }))
        .unwrap();
        assert_eq!(
            serde_json::to_value(record).unwrap(),
            json!({"event": "complete", "protocol": 1})
        );
    }

    #[test]
    fn v2_rejects_message_corruption_ordering_protocol_mismatch_and_reboot() {
        for (field, wrong) in [
            ("checksum", json!(0)),
            ("iterations", json!(2)),
            ("index", json!(1)),
            ("protocol", json!(1)),
            ("elapsed_ns", json!(999)),
            ("benchmark", json!("unknown")),
        ] {
            let mut boot = v2_boot("embassy", 0);
            let mut sample = serde_json::to_value(v2_sample("message-roundtrip", 0)).unwrap();
            sample[field] = wrong;
            assert!(
                boot.accept(serde_json::from_value(sample).unwrap(), "embassy")
                    .is_err()
            );
            assert!(
                boot.accept(v2_sample("message-roundtrip", 0), "embassy")
                    .is_err()
            );
            assert!(boot.summaries().is_err());
        }
        let mut boot = v2_boot("embassy", 1_000);
        assert!(
            boot.accept(v2_sample("message-roundtrip", 1_000), "embassy")
                .is_err()
        );
        let mut boot = v2_boot("embassy", 1);
        assert!(
            boot.accept(v2_sample("message-roundtrip", 0), "embassy")
                .is_err()
        );
        let mut boot = v2_boot("embassy", 0);
        assert!(
            boot.accept(
                serde_json::from_value(v2_hello("embassy")).unwrap(),
                "embassy"
            )
            .is_err()
        );
    }

    fn sample(benchmark: &str, index: u32) -> Record {
        Record::Sample(Sample {
            protocol: 1,
            benchmark: benchmark.into(),
            index,
            iterations: if benchmark == "compute" {
                DEFAULT_ROUNDS
            } else {
                0
            },
            elapsed_ns: 1_000,
            checksum: if benchmark == "compute" {
                DEFAULT_CHECKSUM
            } else {
                0
            },
        })
    }

    #[test]
    fn rejects_incomplete_duplicate_and_rebooted_runs() {
        let mut boot = Boot::default();
        assert!(boot.accept(sample("compute", 0), "test").is_err());
        assert!(boot.accept(hello(), "test").is_err());
        let mut boot = Boot::default();
        boot.accept(hello(), "test").unwrap();
        boot.accept(sample("compute", 0), "test").unwrap();
        assert!(boot.accept(sample("compute", 0), "test").is_err());
        assert!(boot.summaries().is_err());
        let mut boot = Boot::default();
        boot.accept(hello(), "test").unwrap();
        assert!(boot.accept(hello(), "test").is_err());
        let mut boot = Boot::default();
        boot.accept(hello(), "test").unwrap();
        assert!(boot.accept(complete(), "test").is_err());
        assert!(boot.summaries().is_err());
    }

    #[test]
    fn wrong_checksum_never_becomes_a_result() {
        let mut boot = Boot::default();
        boot.accept(hello(), "test").unwrap();
        let Record::Sample(mut bad) = sample("compute", 0) else {
            unreachable!()
        };
        bad.checksum ^= 1;
        assert!(boot.accept(Record::Sample(bad), "test").is_err());
        assert!(boot.accept(sample("compute", 0), "test").is_err());
        assert!(boot.summaries().is_err());
    }

    #[test]
    fn requires_both_complete_sample_sets() {
        let mut boot = Boot::default();
        boot.accept(hello(), "test").unwrap();
        for benchmark in ["compute", "timer-overhead"] {
            for index in 0..DEFAULT_SAMPLES {
                assert!(!boot.accept(sample(benchmark, index), "test").unwrap());
            }
        }
        assert!(boot.accept(complete(), "test").unwrap());
        let summary = boot.summaries().unwrap();
        assert_eq!(summary["compute"].sample_count, DEFAULT_SAMPLES as usize);
        assert_eq!(summary["timer-overhead"].mean_ns, 1_000.0);
    }

    #[test]
    fn nearest_rank_percentiles_have_explicit_small_sample_behavior() {
        let samples: Vec<Sample> = [3, 0, 2, 1]
            .into_iter()
            .map(|elapsed_ns| Sample {
                protocol: 1,
                benchmark: "statistic-test".into(),
                index: 0,
                iterations: 0,
                elapsed_ns,
                checksum: 0,
            })
            .collect();
        let summary = Summary::from_samples(&samples);
        assert_eq!(summary.min_ns, 0);
        assert_eq!(summary.p50_ns, 1);
        assert_eq!(summary.p95_ns, 3);
        assert_eq!(summary.p99_ns, 3);
        assert_eq!(summary.max_ns, 3);
        assert_eq!(summary.mean_ns, 1.5);
    }

    #[test]
    fn nearest_rank_at_64_samples_keeps_the_tail_and_outlier() {
        let samples: Vec<Sample> = (1..=64)
            .rev()
            .map(|value| Sample {
                protocol: 1,
                benchmark: "statistic-test".into(),
                index: 0,
                iterations: 0,
                elapsed_ns: value * 1_000,
                checksum: 0,
            })
            .collect();
        let summary = Summary::from_samples(&samples);
        assert_eq!(summary.p50_ns, 32_000);
        assert_eq!(summary.p95_ns, 61_000);
        assert_eq!(summary.p99_ns, 64_000);
        assert_eq!(summary.max_ns, 64_000);
        assert_eq!(summary.mean_ns, 32_500.0);
    }

    #[test]
    fn enforces_timer_units_but_preserves_zero_overhead() {
        for elapsed_ns in [0, 999, 1_001] {
            let mut boot = Boot::default();
            boot.accept(hello(), "test").unwrap();
            let Record::Sample(mut bad) = sample("compute", 0) else {
                unreachable!()
            };
            bad.elapsed_ns = elapsed_ns;
            assert!(boot.accept(Record::Sample(bad), "test").is_err());
            assert!(boot.summaries().is_err());
        }
        let mut boot = Boot::default();
        boot.accept(hello(), "test").unwrap();
        let Record::Sample(mut overhead) = sample("timer-overhead", 0) else {
            unreachable!()
        };
        overhead.elapsed_ns = 0;
        assert!(!boot.accept(Record::Sample(overhead), "test").unwrap());
    }

    #[test]
    fn rejects_a_record_after_completion_and_revokes_summary() {
        let mut boot = Boot::default();
        boot.accept(hello(), "test").unwrap();
        for benchmark in ["compute", "timer-overhead"] {
            for index in 0..DEFAULT_SAMPLES {
                boot.accept(sample(benchmark, index), "test").unwrap();
            }
        }
        boot.accept(complete(), "test").unwrap();
        assert!(boot.summaries().is_ok());
        assert!(boot.accept(hello(), "test").is_err());
        assert!(boot.summaries().is_err());
    }
}
