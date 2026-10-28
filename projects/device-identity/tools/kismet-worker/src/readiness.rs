//! Explicit opt-in receiver readiness, without headers or credential values.

use std::fmt;

use crate::{BYTE_LIMIT, Error};

const MARKER: &str = "KISMET_WIFI_READY_V1";

/// HTTP success and the first completely saved section header, bound to inputs.
#[derive(Clone, Eq, PartialEq)]
pub struct Ready {
    /// Observed single HTTP response status; readiness requires 200.
    pub http_status: u16,
    /// Exclusive byte offset through the first complete section header.
    pub prefix_bytes: usize,
    /// SHA256 of exactly that HTTP-body prefix, never its contents.
    pub prefix_sha256: String,
    /// SHA256 of the exact private binary request supplied on stdin.
    pub request_sha256: String,
    /// Runtime worker executable SHA256.
    pub binary_sha256: String,
    /// Aggregate identity of compiled worker and shared framing inputs.
    pub build_inputs_sha256: String,
}

impl fmt::Debug for Ready {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Ready")
            .field("http_status", &self.http_status)
            .field("prefix_bytes", &self.prefix_bytes)
            .field("private_bindings", &"<redacted>")
            .finish()
    }
}

impl Ready {
    /// Encode exactly one fixed readiness line, including LF.
    pub fn line(&self) -> String {
        format!(
            "{MARKER} {} {} {} {} {} {}\n",
            self.http_status,
            self.prefix_bytes,
            self.prefix_sha256,
            self.request_sha256,
            self.binary_sha256,
            self.build_inputs_sha256
        )
    }

    /// Decode one complete readiness line, with strict version and field bounds.
    ///
    /// # Errors
    /// Returns a static error for malformed or unsupported readiness metadata.
    pub fn parse_line(bytes: &[u8]) -> Result<Self, Error> {
        let line = std::str::from_utf8(bytes).map_err(|_| Error("invalid receiver readiness"))?;
        if !line.ends_with('\n') || line[..line.len() - 1].contains(['\r', '\n']) {
            return Err(Error("invalid receiver readiness"));
        }
        let fields: Vec<_> = line.split_ascii_whitespace().collect();
        if fields.len() != 7
            || fields[0] != MARKER
            || fields[3..].iter().any(|hash| {
                hash.len() != 64
                    || !hash
                        .bytes()
                        .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            })
        {
            return Err(Error("invalid receiver readiness"));
        }
        let http_status = fields[1]
            .parse()
            .map_err(|_| Error("invalid receiver readiness"))?;
        let prefix_bytes = fields[2]
            .parse()
            .map_err(|_| Error("invalid receiver readiness"))?;
        if http_status != 200 || !(28..=BYTE_LIMIT).contains(&prefix_bytes) || prefix_bytes % 4 != 0
        {
            return Err(Error("invalid receiver readiness"));
        }
        Ok(Self {
            http_status,
            prefix_bytes,
            prefix_sha256: fields[3].into(),
            request_sha256: fields[4].into(),
            binary_sha256: fields[5].into(),
            build_inputs_sha256: fields[6].into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn readiness_roundtrip_versions_bounds_and_debug_privacy() {
        let ready = Ready {
            http_status: 200,
            prefix_bytes: 48,
            prefix_sha256: "a".repeat(64),
            request_sha256: "b".repeat(64),
            binary_sha256: "c".repeat(64),
            build_inputs_sha256: "d".repeat(64),
        };
        assert_eq!(Ready::parse_line(ready.line().as_bytes()).unwrap(), ready);
        for line in [
            ready.line().replace(MARKER, "KISMET_WIFI_READY_V1FUTURE"),
            ready.line().replace("200 48", "401 48"),
            ready.line().replace("200 48", "200 47"),
            format!("{}extra\n", ready.line()),
        ] {
            assert!(Ready::parse_line(line.as_bytes()).is_err());
        }
        assert!(!format!("{ready:?}").contains(&ready.request_sha256));
        assert!(!format!("{ready:?}").contains(&ready.prefix_sha256));
    }
}
