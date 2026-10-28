//! A separate strict station-probe consumer over the complete existing parser.

use serde::{Deserialize, Serialize};

use crate::{
    Error, FrameConsumer, InformationElements, Summary, analyze_consumers, validate_target,
};

/// Whole-capture accounting plus exact nonce-SSID/transmitter presence counts.
/// No address, payload, timestamp, input string or input-derived hash is exposed.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StationProbeSummary {
    /// Format version for this independent station-probe report.
    pub schema_version: u32,
    /// The complete container's existing summary, including other/bad/unsupported frames.
    pub capture: Summary,
    /// Supported probe requests with included, successfully verified FCS.
    pub qualified_probe_count: u64,
    /// Probe requests with exact target SSID/transmitter and checked FCS.
    pub matching_probe_count: u64,
    /// Exact target matches without included FCS; never witnesses.
    pub matching_unqualified_fcs_probe_count: u64,
    /// Checked probes with the target SSID but another transmitter.
    pub matching_ssid_wrong_mac_count: u64,
    /// Checked probes with the target transmitter but another SSID.
    pub matching_mac_wrong_ssid_count: u64,
    /// Probe variants outside the supported header profile; never witnesses.
    pub unsupported_probe_count: u64,
}

struct Target<'a> {
    ssid: &'a [u8],
    station: [u8; 6],
    report: StationProbeSummary,
}

impl FrameConsumer for Target<'_> {
    fn observe(&mut self, frame: &[u8], has_fcs: bool) -> Result<(), Error> {
        // The shared decoder has already checked length>=2, protocol version,
        // CRC/error metadata and (when present) the actual complete frame FCS.
        let control = u16::from_le_bytes([frame[0], frame[1]]);
        if control & 0x00ff != 0x0040 {
            return Ok(());
        }
        if control & 0xc700 != 0 {
            self.report.unsupported_probe_count += 1;
            return Ok(());
        }
        if frame.len() < 24
            || frame[22] & 15 != 0
            || frame[4..10] != [255; 6]
            || frame[16..22] != [255; 6]
            || frame[10] & 1 != 0
            || frame[10..16] == [0; 6]
        {
            return Err(Error::StationProbe);
        }
        let mut ssid = None;
        for element in InformationElements(&frame[24..]) {
            let element = element.map_err(|_| Error::StationProbe)?;
            if element.kind == 0 {
                if ssid.is_some() || element.value.len() > 32 {
                    return Err(Error::StationProbe);
                }
                ssid = Some(element.value);
            }
        }
        let ssid = ssid.ok_or(Error::StationProbe)?;
        let same_ssid = ssid == self.ssid;
        let same_mac = frame[10..16] == self.station;
        if has_fcs {
            self.report.qualified_probe_count += 1;
            match (same_ssid, same_mac) {
                (true, true) => self.report.matching_probe_count += 1,
                (true, false) => self.report.matching_ssid_wrong_mac_count += 1,
                (false, true) => self.report.matching_mac_wrong_ssid_count += 1,
                _ => {}
            }
        } else if same_ssid && same_mac {
            self.report.matching_unqualified_fcs_probe_count += 1;
        }
        Ok(())
    }
}

/// Check exact station probe presence while retaining whole-container accounting.
///
/// A target SSID must be nonempty, nonzero and at most32 bytes; the target MAC
/// must be nonzero unicast. The caller owns nonce generation/freshness. Only
/// version-zero unfragmented broadcast-DA/BSSID probe requests with a base
/// 24-byte management header, clear DS/MoreFrag/Protected/Order bits and exactly
/// one SSID IE are supported. Retry/sequence/duration and other well-framed IEs
/// remain opaque. DATAPAD variants remain unsupported in the shared decoder.
///
/// No association, peer authentication, full IEEE conformance, radio timing,
/// completeness or absence is established. Existing beacon comparison rules
/// remain unchanged; a target subset cannot repair an Unqualified comparison.
///
/// # Errors
/// Returns the shared container/framing/metadata errors, [`Error::WitnessTarget`]
/// for invalid private targets, or [`Error::StationProbe`] for malformed
/// supported probe contents. Any late malformed record fails the entire call.
///
/// ```
/// use wifi_observer::{station_probe_witness, Error};
/// assert_eq!(station_probe_witness(&[], &[], [0;6]), Err(Error::WitnessTarget));
/// ```
pub fn station_probe_witness(
    bytes: &[u8],
    expected_ssid: &[u8],
    expected_station: [u8; 6],
) -> Result<StationProbeSummary, Error> {
    validate_target(expected_ssid, expected_station)?;
    let mut target = Target {
        ssid: expected_ssid,
        station: expected_station,
        report: StationProbeSummary {
            schema_version: 1,
            ..StationProbeSummary::default()
        },
    };
    target.report.capture = analyze_consumers(bytes, None, Some(&mut target))?;
    Ok(target.report)
}
