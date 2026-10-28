//! Validate private runtime exchanges and emit only fixed consistency findings.

use runtime_protocol::{
    BASE_OVERRIDE, BLUETOOTH_OVERRIDE, IEEE802154_OVERRIDE, MAX_REQUEST_BYTES, MAX_RESPONSE_BYTES,
    Phase, Request, Response, Snapshot, WIFI_AP_OVERRIDE, WIFI_STATION_OVERRIDE,
};
use serde::{Deserialize, Serialize};

pub(crate) const MAX_TRANSCRIPT_BYTES: usize = 16 * 1024;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Exchange {
    pub(crate) request: Request,
    pub(crate) response: Response,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum Kind {
    RuntimeTableSummary,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum Observer {
    UartApplicationApi,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Report {
    schema_version: u32,
    kind: Kind,
    observer: Observer,
    phase_count: u32,
    factory_unchanged: bool,
    base_override_matches: bool,
    cached_interfaces_unchanged_after_base_override: bool,
    interface_overrides_match: bool,
    initial_table_state_restored: bool,
}

fn interfaces_equal(left: &Snapshot, right: &Snapshot) -> bool {
    left.wifi_station == right.wifi_station
        && left.wifi_ap == right.wifi_ap
        && left.bluetooth == right.bluetooth
        && left.ieee802154 == right.ieee802154
}

pub(crate) fn decode(bytes: &[u8]) -> Result<[Exchange; 4], &'static str> {
    if bytes.len() > MAX_TRANSCRIPT_BYTES || !bytes.ends_with(b"\n") {
        return Err("incomplete or oversized runtime transcript");
    }
    let mut exchanges = Vec::new();
    let mut first_id = None;
    for (index, line) in bytes[..bytes.len() - 1]
        .split(|&byte| byte == b'\n')
        .enumerate()
    {
        if index >= Phase::ALL.len()
            || line.is_empty()
            || line.len() > MAX_REQUEST_BYTES + MAX_RESPONSE_BYTES + 64
        {
            return Err("invalid runtime exchange framing");
        }
        let exchange: Exchange =
            serde_json::from_slice(line).map_err(|_| "invalid runtime exchange JSON")?;
        let initial = *first_id.get_or_insert(exchange.request.id);
        if initial == 0 {
            return Err("runtime correlation token must be nonzero");
        }
        let id = initial
            .checked_add(index as u32)
            .ok_or("runtime correlation token overflow")?;
        let expected = Request::new(id, Phase::ALL[index]);
        if exchange.request != expected {
            return Err("runtime request sequence mismatch");
        }
        exchange
            .response
            .validate_for(&exchange.request)
            .map_err(|_| "runtime response validation failed")?;
        exchanges.push(exchange);
    }
    exchanges
        .try_into()
        .map_err(|_| "runtime transcript must contain four exchanges")
}

pub(crate) fn summarize(bytes: &[u8]) -> Result<Report, &'static str> {
    let exchanges = decode(bytes)?;
    let mut snapshots = Vec::with_capacity(4);
    for exchange in exchanges {
        snapshots.push(
            exchange
                .response
                .snapshot
                .ok_or("runtime success is missing its snapshot")?,
        );
    }
    let defaults = &snapshots[0];
    let base = &snapshots[1];
    let interfaces = &snapshots[2];
    let restored = &snapshots[3];

    if defaults.base == BASE_OVERRIDE
        || defaults.wifi_station == WIFI_STATION_OVERRIDE
        || defaults.wifi_ap == WIFI_AP_OVERRIDE
        || defaults.bluetooth == BLUETOOTH_OVERRIDE
        || defaults.ieee802154 == IEEE802154_OVERRIDE
    {
        return Err("runtime control collides with the initial table state");
    }
    if snapshots
        .iter()
        .any(|snapshot| snapshot.factory != defaults.factory)
    {
        return Err("factory field changed during runtime sequence");
    }
    if base.base != BASE_OVERRIDE || interfaces.base != BASE_OVERRIDE {
        return Err("runtime base override does not match the control");
    }
    if !interfaces_equal(defaults, base) {
        return Err("cached interface table changed after the base-only override");
    }
    if interfaces.wifi_station != WIFI_STATION_OVERRIDE
        || interfaces.wifi_ap != WIFI_AP_OVERRIDE
        || interfaces.bluetooth != BLUETOOTH_OVERRIDE
        || interfaces.ieee802154 != IEEE802154_OVERRIDE
    {
        return Err("runtime interface overrides do not match the controls");
    }
    if restored != defaults {
        return Err("runtime table restoration does not match the initial state");
    }

    Ok(Report {
        schema_version: 1,
        kind: Kind::RuntimeTableSummary,
        observer: Observer::UartApplicationApi,
        phase_count: 4,
        factory_unchanged: true,
        base_override_matches: true,
        cached_interfaces_unchanged_after_base_override: true,
        interface_overrides_match: true,
        initial_table_state_restored: true,
    })
}

#[cfg(test)]
pub(crate) fn synthetic_exchanges() -> [Exchange; 4] {
    let defaults = Snapshot {
        factory: [48, 52, 56, 60, 64, 68, 72, 76],
        base: [48, 52, 56, 68, 72, 76],
        wifi_station: [48, 52, 56, 68, 72, 76],
        wifi_ap: [48, 52, 56, 68, 72, 77],
        bluetooth: [48, 52, 56, 68, 72, 78],
        ieee802154: [48, 52, 56, 60, 64, 68, 72, 76],
    };
    let base = Snapshot {
        base: BASE_OVERRIDE,
        ..defaults
    };
    let interfaces = Snapshot {
        wifi_station: WIFI_STATION_OVERRIDE,
        wifi_ap: WIFI_AP_OVERRIDE,
        bluetooth: BLUETOOTH_OVERRIDE,
        ieee802154: IEEE802154_OVERRIDE,
        ..base
    };
    let snapshots = [defaults, base, interfaces, defaults];
    core::array::from_fn(|index| {
        let request = Request::new(index as u32 + 1, Phase::ALL[index]);
        Exchange {
            request,
            response: Response::new(request.id, request.phase, 0, Some(snapshots[index])),
        }
    })
}

#[cfg(test)]
mod tests {
    use super::{Exchange, summarize, synthetic_exchanges};

    fn transcript(exchanges: &[Exchange]) -> Vec<u8> {
        let mut bytes = Vec::new();
        for exchange in exchanges {
            serde_json::to_writer(&mut bytes, exchange).unwrap();
            bytes.push(b'\n');
        }
        bytes
    }

    #[test]
    fn complete_sequence_is_sanitized_and_repeatable() {
        let exchanges = synthetic_exchanges();
        let bytes = transcript(&exchanges);
        let first = summarize(&bytes).unwrap();
        assert_eq!(first, summarize(&bytes).unwrap());
        let output = serde_json::to_string(&first).unwrap();
        assert!(!output.contains("snapshot"));
        for exchange in exchanges {
            assert!(!output.contains(&serde_json::to_string(&exchange.response.snapshot).unwrap()));
        }
    }

    #[test]
    fn mutated_late_fields_and_failed_restoration_are_rejected() {
        for phase in 1..4 {
            for field in [
                "factory",
                "base",
                "wifi_station",
                "wifi_ap",
                "bluetooth",
                "ieee802154",
            ] {
                let bytes = transcript(&synthetic_exchanges());
                let mut entries: Vec<serde_json::Value> = bytes
                    .split(|&byte| byte == b'\n')
                    .filter(|line| !line.is_empty())
                    .map(|line| serde_json::from_slice(line).unwrap())
                    .collect();
                let value = entries[phase]["response"]["snapshot"][field][0]
                    .as_u64()
                    .unwrap();
                entries[phase]["response"]["snapshot"][field][0] = (value ^ 4).into();
                let mut changed = Vec::new();
                for entry in entries {
                    serde_json::to_writer(&mut changed, &entry).unwrap();
                    changed.push(b'\n');
                }
                assert!(summarize(&changed).is_err(), "phase {phase} field {field}");
            }
        }
    }

    #[test]
    fn missing_extra_out_of_order_and_uncorrelated_records_fail() {
        let mut exchanges = synthetic_exchanges();
        assert!(summarize(&transcript(&exchanges[..3])).is_err());
        let mut extra = transcript(&exchanges);
        extra.extend_from_slice(&transcript(&exchanges[..1]));
        assert!(summarize(&extra).is_err());
        exchanges.swap(1, 2);
        assert!(summarize(&transcript(&exchanges)).is_err());
        exchanges = synthetic_exchanges();
        exchanges[3].response.id = 99;
        assert!(summarize(&transcript(&exchanges)).is_err());
        let mut incomplete = transcript(&synthetic_exchanges());
        incomplete.pop();
        assert!(summarize(&incomplete).is_err());
    }
}
