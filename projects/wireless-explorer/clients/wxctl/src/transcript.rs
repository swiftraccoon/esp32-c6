//! Live and archived captures share the same acceptance path.
use anyhow::{Result, bail, ensure};
use wireless_protocol::{LineDecoder, PREFIX, Request, Response, decode_request, decode_response};

pub(crate) const MAX_CAPTURE_BYTES: usize = 64 * 1024;

#[derive(Debug)]
pub(crate) struct Transcript {
    decoder: LineDecoder,
    request: Request,
    response: Option<Response>,
}

impl Transcript {
    pub(crate) fn new(request: Request) -> Self {
        Self {
            decoder: LineDecoder::new(),
            request,
            response: None,
        }
    }

    pub(crate) fn has_response(&self) -> bool {
        self.response.is_some()
    }

    pub(crate) fn feed(&mut self, bytes: &[u8]) -> Result<()> {
        for &byte in bytes {
            if let Some(line) = self.decoder.push(byte) {
                let line = line?;
                // A clean control capture admits only frames and blank lines.
                // Boot messages, panic text and other diagnostics fail closed.
                ensure!(
                    line.starts_with(PREFIX),
                    "unexpected device output: {}",
                    String::from_utf8_lossy(line)
                );
                ensure!(self.response.is_none(), "duplicate response");
                let response = decode_response(line)?;
                response.validate_for(&self.request)?;
                self.response = Some(response);
            }
        }
        Ok(())
    }

    pub(crate) fn finish(self) -> Result<Response> {
        ensure!(!self.decoder.is_pending(), "incomplete trailing line");
        self.response
            .ok_or_else(|| anyhow::anyhow!("missing response"))
    }
}

pub(crate) fn request_frame(bytes: &[u8]) -> Result<Request> {
    let Some(line) = bytes.strip_suffix(b"\n") else {
        bail!("request lacks LF");
    };
    ensure!(!line.contains(&b'\n'), "multiple request lines");
    Ok(decode_request(line)?)
}
