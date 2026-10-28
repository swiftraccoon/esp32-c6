//! Fixed read-only Kismet transport; credentials and response headers stay here.
#![forbid(unsafe_code)]

use std::fs;
use std::io::{self, Read, Write};
use std::process::{Command, ExitCode, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use kismet_worker::{
    BYTE_LIMIT, PACKET_LIMIT, Ready, Request, SECONDS, STDERR_LIMIT, Status, Stop, credential,
    curl_quote,
};

#[path = "../../../crates/wifi-observer/src/framing.rs"]
mod framing;

fn empty() -> Status {
    Status {
        stop: Stop::Setup,
        saved_bytes: 0,
        observed_bytes: 0,
        epbs: 0,
        http_status: 0,
        curl_exit: -1,
        curl_signal: 0,
        intentional_kill: false,
        cleanup_success: true,
        stderr_truncated: false,
        request_sha256: "0".repeat(64),
        binary_sha256: "0".repeat(64),
        build_inputs_sha256: env!("WORKER_INPUTS_SHA256").into(),
    }
}

fn expected_process(pid: u32) -> bool {
    let Ok(entries) = fs::read_dir("/proc") else {
        return false;
    };
    let matches: Vec<_> = entries
        .filter_map(Result::ok)
        .filter_map(|e| {
            let id = e.file_name().to_str()?.parse::<u32>().ok()?;
            (fs::read_to_string(e.path().join("comm")).ok()?.trim() == "kismet").then_some(id)
        })
        .collect();
    matches == [pid]
}

fn http_status(headers: &[u8]) -> u16 {
    // No raw header field is forwarded. A single response status is required;
    // redirects, interim/multiple responses and authentication retries fail.
    let mut codes = headers.split(|b| *b == b'\n').filter_map(|line| {
        let line = std::str::from_utf8(line).ok()?;
        let mut fields = line.split_ascii_whitespace();
        let protocol = fields.next()?;
        if !matches!(protocol, "HTTP/1.0" | "HTTP/1.1" | "HTTP/2" | "HTTP/3") {
            return None;
        }
        fields.next()?.parse::<u16>().ok()
    });
    match (codes.next(), codes.next()) {
        (Some(code), None) => code,
        _ => 0,
    }
}

#[derive(Clone, Copy)]
struct Limits {
    packets: usize,
    bytes: usize,
    hard: Duration,
    idle: Duration,
}

#[cfg(test)]
fn collect(
    url: &str,
    username: &str,
    password: &str,
    output: &mut impl Write,
    limits: Limits,
) -> Status {
    collect_ready(
        url,
        username,
        password,
        output,
        limits,
        None,
        &mut io::sink(),
    )
}

#[derive(Clone)]
struct ReadyBindings {
    request: String,
    binary: String,
}

fn emit_ready(
    bindings: Option<&ReadyBindings>,
    prefix: &[u8],
    complete: bool,
    http: u16,
    sent: &mut bool,
    writer: &mut impl Write,
) -> Result<(), ()> {
    if *sent || !complete || http != 200 {
        return Ok(());
    }
    if let Some(bindings) = bindings {
        let prefix_sha256 = hash_bytes(prefix).map_err(|_| ())?;
        let ready = Ready {
            http_status: http,
            prefix_bytes: prefix.len(),
            prefix_sha256,
            request_sha256: bindings.request.clone(),
            binary_sha256: bindings.binary.clone(),
            build_inputs_sha256: env!("WORKER_INPUTS_SHA256").into(),
        };
        writer.write_all(ready.line().as_bytes()).map_err(|_| ())?;
        writer.flush().map_err(|_| ())?;
        *sent = true;
    }
    Ok(())
}

fn collect_ready(
    url: &str,
    username: &str,
    password: &str,
    output: &mut impl Write,
    limits: Limits,
    bindings: Option<&ReadyBindings>,
    ready_output: &mut impl Write,
) -> Status {
    let mut status = empty();
    let Ok(auth) = curl_quote(&format!("{username}:{password}")) else {
        return status;
    };
    let Ok(url) = curl_quote(url) else {
        return status;
    };
    if username.contains(':') {
        return status;
    }
    let configuration = format!(
        "url = {url}\nuser = {auth}\nbasic\nsilent\nshow-error\nno-buffer\nnoproxy = \"*\"\nproto = \"=http\"\nmax-redirs = 0\nconnect-timeout = 3\nmax-time = {}\ndump-header = \"/dev/stderr\"\n",
        limits.hard.as_secs_f64()
    );
    let Ok(mut child) = Command::new("/usr/bin/curl")
        .args(["--disable", "--config", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    else {
        return status;
    };
    let sent = child
        .stdin
        .take()
        .is_some_and(|mut input| input.write_all(configuration.as_bytes()).is_ok());
    if !sent {
        let _ = child.kill();
        status.cleanup_success = child.wait().is_ok();
        return status;
    }
    let Some(mut stdout) = child.stdout.take() else {
        let _ = child.kill();
        status.cleanup_success = child.wait().is_ok();
        return status;
    };
    let Some(stderr) = child.stderr.take() else {
        let _ = child.kill();
        status.cleanup_success = child.wait().is_ok();
        return status;
    };
    let (tx, rx) = mpsc::sync_channel(1);
    let body_thread = thread::spawn(move || {
        loop {
            let mut chunk = vec![0; 8192];
            match stdout.read(&mut chunk) {
                Ok(0) => {
                    let _ = tx.send(Ok(Vec::new()));
                    break;
                }
                Ok(n) => {
                    chunk.truncate(n);
                    if tx.send(Ok(chunk)).is_err() {
                        break;
                    }
                }
                Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                Err(_) => {
                    let _ = tx.send(Err(()));
                    break;
                }
            }
        }
    });
    let (header_tx, header_rx) = mpsc::sync_channel(1);
    let error_thread = thread::spawn(move || {
        let mut bytes = Vec::new();
        let mut reader = stderr.take((STDERR_LIMIT + 1) as u64);
        let mut chunk = [0; 1024];
        let mut announced = false;
        loop {
            match reader.read(&mut chunk) {
                Ok(0) => return (bytes, true),
                Ok(n) => {
                    bytes.extend_from_slice(&chunk[..n]);
                    if !announced && bytes.windows(4).any(|w| w == b"\r\n\r\n") {
                        let _ = header_tx.send(http_status(&bytes));
                        announced = true;
                    }
                }
                Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                Err(_) => return (bytes, false),
            }
        }
    });
    let started = Instant::now();
    let mut activity = started;
    let mut framer = match framing::BlockFramer::new(limits.bytes) {
        Ok(f) => f,
        Err(_) => {
            let _ = child.kill();
            let _ = child.wait();
            drop(rx);
            let _ = body_thread.join();
            let _ = error_thread.join();
            return status;
        }
    };
    status.stop = Stop::Eof;
    let mut prefix = Vec::new();
    let mut prefix_complete = false;
    let mut ready_sent = false;
    let mut ready_http = 0;
    'capture: loop {
        if let Ok(code) = header_rx.try_recv() {
            ready_http = code;
        }
        if emit_ready(
            bindings,
            &prefix,
            prefix_complete,
            ready_http,
            &mut ready_sent,
            ready_output,
        )
        .is_err()
        {
            status.stop = Stop::Write;
            break;
        }
        let now = Instant::now();
        if now.duration_since(started) >= limits.hard {
            status.stop = Stop::Deadline;
            break;
        }
        if now.duration_since(activity) >= limits.idle {
            status.stop = Stop::Idle;
            break;
        }
        let wait = (limits.hard - now.duration_since(started))
            .min(limits.idle - now.duration_since(activity));
        let wait = if bindings.is_some() {
            wait.min(Duration::from_millis(10))
        } else {
            wait
        };
        let chunk = match rx.recv_timeout(wait) {
            Ok(Ok(chunk)) if !chunk.is_empty() => chunk,
            Ok(Ok(_)) => {
                status.stop = Stop::Eof;
                break;
            }
            Ok(Err(_)) => {
                status.stop = Stop::Read;
                break;
            }
            Err(mpsc::RecvTimeoutError::Timeout) => continue,
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                status.stop = Stop::Read;
                break;
            }
        };
        activity = Instant::now();
        status.observed_bytes += chunk.len();
        let retain = chunk.len().min(limits.bytes - status.saved_bytes);
        let mut cursor = 0;
        let mut written = 0;
        while cursor < retain {
            let previous = cursor;
            match framer.push(&chunk[cursor..retain]) {
                Ok((used, boundary)) => {
                    cursor += used;
                    if bindings.is_some() && !prefix_complete {
                        prefix.extend_from_slice(&chunk[previous..cursor]);
                        if boundary.is_some_and(|b| b.block_type == 0x0a0d0d0a) {
                            prefix_complete = true;
                            if output.write_all(&chunk[written..cursor]).is_err()
                                || output.flush().is_err()
                            {
                                status.stop = Stop::Write;
                                break 'capture;
                            }
                            status.saved_bytes += cursor - written;
                            written = cursor;
                            if let Ok(code) = header_rx.try_recv() {
                                ready_http = code;
                            }
                            if emit_ready(
                                bindings,
                                &prefix,
                                prefix_complete,
                                ready_http,
                                &mut ready_sent,
                                ready_output,
                            )
                            .is_err()
                            {
                                status.stop = Stop::Write;
                                break 'capture;
                            }
                        }
                    }
                    if boundary.is_some_and(|b| b.block_type == 6) {
                        status.epbs += 1;
                    }
                    if status.epbs == limits.packets {
                        if output.write_all(&chunk[written..cursor]).is_err() {
                            status.stop = Stop::Write;
                        } else {
                            status.saved_bytes += cursor - written;
                            status.stop = Stop::Boundary;
                        }
                        break 'capture;
                    }
                }
                Err(_) => {
                    if output.write_all(&chunk[written..retain]).is_err() {
                        status.stop = Stop::Write;
                    } else {
                        status.saved_bytes += retain - written;
                        status.stop = Stop::Framing;
                    }
                    break 'capture;
                }
            }
        }
        if output.write_all(&chunk[written..retain]).is_err() {
            status.stop = Stop::Write;
            break;
        }
        status.saved_bytes += retain - written;
        if status.saved_bytes == limits.bytes {
            status.stop = Stop::ByteLimit;
            break;
        }
    }
    if status.stop == Stop::Boundary && framer.finish().is_err() {
        status.stop = Stop::Framing;
    }
    if output.flush().is_err() {
        status.stop = Stop::Write;
    }
    match child.try_wait() {
        Ok(Some(_)) => {}
        Ok(None) => {
            status.intentional_kill = status.stop == Stop::Boundary && child.kill().is_ok();
            if !status.intentional_kill {
                let _ = child.kill();
            }
        }
        Err(_) => {
            status.cleanup_success = false;
            let _ = child.kill();
        }
    }
    let result = child.wait();
    status.cleanup_success &= result.is_ok();
    if let Ok(exit) = result {
        status.curl_exit = exit.code().unwrap_or(-1);
        #[cfg(unix)]
        {
            use std::os::unix::process::ExitStatusExt;
            status.curl_signal = exit.signal().unwrap_or(0);
        }
    }
    // After a failed attempt, retain all already-delivered body bytes through
    // EOF, up to the declared artifact cap. A successful quota deliberately
    // retains only the prefix ending at its last complete block.
    if status.stop != Stop::Boundary {
        loop {
            match rx.recv_timeout(Duration::from_secs(1)) {
                Ok(Ok(chunk)) if !chunk.is_empty() => {
                    status.observed_bytes += chunk.len();
                    let retain = chunk.len().min(limits.bytes - status.saved_bytes);
                    if output.write_all(&chunk[..retain]).is_ok() {
                        status.saved_bytes += retain;
                    } else {
                        status.stop = Stop::Write;
                    }
                }
                Ok(Ok(_)) => break,
                Ok(Err(_)) => {
                    status.cleanup_success = false;
                    break;
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    status.cleanup_success = false;
                    break;
                }
            }
        }
    }
    drop(rx);
    status.cleanup_success &= body_thread.join().is_ok();
    match error_thread.join() {
        Ok((headers, read_ok)) => {
            status.http_status = http_status(&headers);
            status.stderr_truncated = headers.len() > STDERR_LIMIT;
            status.cleanup_success &= read_ok;
        }
        Err(_) => status.cleanup_success = false,
    }
    status
}

fn run(ready_enabled: bool) -> Status {
    let mut encoded = Vec::new();
    if io::stdin()
        .lock()
        .take(16 * 1024 + 1)
        .read_to_end(&mut encoded)
        .is_err()
        || encoded.len() > 16 * 1024
    {
        return empty();
    }
    let Ok(request) = Request::decode(&*encoded) else {
        return empty();
    };
    if !expected_process(request.expected_pid) {
        return empty();
    }
    let Ok(username) = credential(&request.username, "httpd_username") else {
        return empty();
    };
    let Ok(password) = credential(&request.password, "httpd_password") else {
        return empty();
    };
    let url = format!(
        "http://127.0.0.1:2501/datasource/pcap/by-uuid/{}/packets.pcapng",
        request.source_uuid
    );
    let bindings = if ready_enabled {
        match (hash_bytes(&encoded), binary_hash()) {
            (Ok(request), Ok(binary)) => Some(ReadyBindings { request, binary }),
            _ => return empty(),
        }
    } else {
        None
    };
    let mut status = collect_ready(
        &url,
        &username,
        &password,
        &mut io::stdout().lock(),
        Limits {
            packets: PACKET_LIMIT,
            bytes: BYTE_LIMIT,
            hard: Duration::from_secs(SECONDS),
            idle: Duration::from_secs(SECONDS),
        },
        bindings.as_ref(),
        &mut io::stderr().lock(),
    );
    match (hash_bytes(&encoded), binary_hash()) {
        (Ok(request), Ok(binary)) => {
            status.request_sha256 = request;
            status.binary_sha256 = binary;
        }
        _ => status.cleanup_success = false,
    }
    status
}

fn hash_command() -> Command {
    if cfg!(target_os = "macos") {
        let mut c = Command::new("/usr/bin/shasum");
        c.args(["-a", "256"]);
        c
    } else {
        Command::new("/usr/bin/sha256sum")
    }
}

fn hash_result(result: std::process::Output) -> Result<String, &'static str> {
    if !result.status.success() {
        return Err("worker provenance unavailable");
    }
    let digest = std::str::from_utf8(&result.stdout)
        .map_err(|_| "worker provenance unavailable")?
        .split_ascii_whitespace()
        .next()
        .ok_or("worker provenance unavailable")?;
    if digest.len() != 64 || !digest.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("worker provenance unavailable");
    }
    Ok(digest.into())
}

fn hash_bytes(bytes: &[u8]) -> Result<String, &'static str> {
    let mut child = hash_command()
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|_| "worker hash unavailable")?;
    let written = child
        .stdin
        .take()
        .is_some_and(|mut input| input.write_all(bytes).is_ok());
    if !written {
        let _ = child.kill();
        let _ = child.wait();
        return Err("worker hash unavailable");
    }
    hash_result(
        child
            .wait_with_output()
            .map_err(|_| "worker hash unavailable")?,
    )
}

fn binary_hash() -> Result<String, &'static str> {
    let executable = std::env::current_exe().map_err(|_| "worker provenance unavailable")?;
    hash_result(
        hash_command()
            .arg(executable)
            .output()
            .map_err(|_| "worker provenance unavailable")?,
    )
}

fn provenance() -> Result<(), &'static str> {
    let digest = binary_hash()?;
    let curl = Command::new("/usr/bin/curl")
        .arg("--version")
        .output()
        .map_err(|_| "worker provenance unavailable")?;
    let curl = std::str::from_utf8(&curl.stdout)
        .map_err(|_| "worker provenance unavailable")?
        .split_ascii_whitespace()
        .nth(1)
        .ok_or("worker provenance unavailable")?;
    if !curl.bytes().all(|b| b.is_ascii_digit() || b == b'.') {
        return Err("worker provenance unavailable");
    }
    println!(
        "{{\"schema_version\":1,\"binary_sha256\":\"{digest}\",\"build_inputs_sha256\":\"{}\",\"rustc_version\":\"{}\",\"curl_version\":\"{curl}\"}}",
        env!("WORKER_INPUTS_SHA256"),
        env!("WORKER_RUSTC_VERSION")
    );
    Ok(())
}

fn main() -> ExitCode {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args == ["--provenance"] {
        return if provenance().is_ok() {
            ExitCode::SUCCESS
        } else {
            eprintln!("worker provenance unavailable");
            ExitCode::FAILURE
        };
    }
    let ready = args == ["--ready"];
    if !args.is_empty() && !ready {
        eprintln!("invalid worker arguments");
        return ExitCode::FAILURE;
    }
    let status = run(ready);
    let written = io::stderr()
        .lock()
        .write_all(status.line().as_bytes())
        .is_ok();
    if status.accepted() && written {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;

    fn body(packets: usize) -> Vec<u8> {
        let mut b = vec![
            0x0a, 0x0d, 0x0d, 0x0a, 28, 0, 0, 0, 0x4d, 0x3c, 0x2b, 0x1a, 1, 0, 0, 0,
        ];
        b.extend_from_slice(&u64::MAX.to_le_bytes());
        b.extend_from_slice(&28_u32.to_le_bytes());
        for _ in 0..packets {
            b.extend_from_slice(&6_u32.to_le_bytes());
            b.extend_from_slice(&32_u32.to_le_bytes());
            b.extend_from_slice(&[0; 20]);
            b.extend_from_slice(&32_u32.to_le_bytes());
        }
        b
    }

    fn mock(response: Vec<u8>, delay: Duration) -> (String, thread::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/synthetic", listener.local_addr().unwrap());
        let handle = thread::spawn(move || {
            let (mut socket, _) = listener.accept().unwrap();
            socket
                .set_read_timeout(Some(Duration::from_secs(2)))
                .unwrap();
            let mut request = Vec::new();
            let mut byte = [0; 1];
            while !request.ends_with(b"\r\n\r\n") {
                if socket.read_exact(&mut byte).is_err() {
                    return;
                }
                request.push(byte[0]);
                if request.len() > 8192 {
                    return;
                }
            }
            // Credentials reach only the private loopback HTTP request.
            assert!(
                request
                    .windows(b"Authorization: Basic".len())
                    .any(|w| w == b"Authorization: Basic")
            );
            let _ = socket.write_all(&response);
            thread::sleep(delay);
        });
        (url, handle)
    }

    fn response(code: u16, body: &[u8]) -> Vec<u8> {
        let mut r=format!("HTTP/1.1 {code} Synthetic\r\nContent-Length: {}\r\nSet-Cookie: PRIVATE_COOKIE_SENTINEL\r\nConnection: close\r\n\r\n",body.len()).into_bytes();
        r.extend_from_slice(body);
        r
    }
    fn limits() -> Limits {
        Limits {
            packets: 32,
            bytes: BYTE_LIMIT,
            hard: Duration::from_secs(2),
            idle: Duration::from_secs(1),
        }
    }

    #[test]
    fn real_curl_complete_prefix_http_failures_and_private_headers() {
        for code in [200, 401, 302] {
            let expected = body(32);
            let mut sent = expected.clone();
            sent.extend_from_slice(b"ignored after successful boundary");
            let (url, server) = mock(response(code, &sent), Duration::ZERO);
            let mut output = Vec::new();
            let status = collect(
                &url,
                "synthetic-user",
                "SENTINEL-password\\\"colon:ok",
                &mut output,
                limits(),
            );
            server.join().unwrap();
            assert_eq!(output, expected);
            assert_eq!(status.http_status, code);
            assert_eq!(status.accepted(), code == 200);
            assert!(!status.line().contains("SENTINEL"));
            assert!(!status.line().contains("COOKIE"));
            assert!(!status.line().contains(&url));
        }
    }

    #[test]
    fn real_curl_truncation_malformed_body_deadline_and_cap_remain_failed() {
        let mut partial = body(31);
        partial.extend_from_slice(&[6, 0, 0]);
        for raw in [partial, b"<html>not PCAPNG</html>".to_vec()] {
            let (url, server) = mock(response(200, &raw), Duration::ZERO);
            let mut output = Vec::new();
            let status = collect(&url, "synthetic", "private", &mut output, limits());
            server.join().unwrap();
            assert!(!status.accepted());
            assert_eq!(output, raw);
            assert!(status.cleanup_success);
        }
        let raw = body(32);
        let (url, server) = mock(response(200, &raw), Duration::ZERO);
        let mut output = Vec::new();
        let mut bound = limits();
        bound.bytes = 128;
        let status = collect(&url, "synthetic", "private", &mut output, bound);
        server.join().unwrap();
        assert_eq!(status.stop, Stop::ByteLimit);
        assert_eq!(output.len(), 128);
        assert!(!status.accepted());
        let raw = body(1);
        let mut streamed = b"HTTP/1.1 200 OK\r\nConnection: close\r\n\r\n".to_vec();
        streamed.extend_from_slice(&raw);
        let (url, server) = mock(streamed, Duration::from_millis(250));
        let mut output = Vec::new();
        let mut bound = limits();
        bound.hard = Duration::from_millis(100);
        bound.idle = Duration::from_millis(100);
        let status = collect(&url, "synthetic", "private", &mut output, bound);
        server.join().unwrap();
        assert!(!status.accepted());
        assert_eq!(output, raw);
        assert!(matches!(
            status.stop,
            Stop::Deadline | Stop::Idle | Stop::Eof
        ));
    }

    #[test]
    fn invalid_credentials_never_spawn_curl() {
        for (u, p) in [
            ("bad:user", "private"),
            ("synthetic", "bad\noption"),
            ("synthetic", "bad\0value"),
        ] {
            let mut output = Vec::new();
            let status = collect("http://127.0.0.1:1", u, p, &mut output, limits());
            assert_eq!(status.stop, Stop::Setup);
            assert!(output.is_empty());
        }
    }
    #[test]
    fn opt_in_ready_precedes_packets_and_binds_the_saved_prefix() {
        struct Signal(mpsc::Sender<Vec<u8>>);
        impl Write for Signal {
            fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
                self.0
                    .send(bytes.to_vec())
                    .map_err(|_| io::ErrorKind::BrokenPipe)?;
                Ok(bytes.len())
            }
            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/synthetic", listener.local_addr().unwrap());
        let raw = body(32);
        let prefix = raw[..28].to_vec();
        let expected = prefix.clone();
        let (continue_tx, continue_rx) = mpsc::channel();
        let server = thread::spawn(move || {
            let (mut socket, _) = listener.accept().unwrap();
            socket
                .set_read_timeout(Some(Duration::from_secs(2)))
                .unwrap();
            let mut request = Vec::new();
            let mut byte = [0];
            while !request.ends_with(b"\r\n\r\n") {
                socket.read_exact(&mut byte).unwrap();
                request.push(byte[0]);
            }
            socket
                .write_all(
                    b"HTTP/1.1 200 OK\r\nSet-Cookie: PRIVATE_SENTINEL\r\nConnection: close\r\n\r\n",
                )
                .unwrap();
            socket.write_all(&raw[..28]).unwrap();
            continue_rx.recv_timeout(Duration::from_secs(2)).unwrap();
            socket.write_all(&raw[28..]).unwrap();
        });
        let (ready_tx, ready_rx) = mpsc::channel();
        let worker = thread::spawn(move || {
            let mut raw = Vec::new();
            let bindings = ReadyBindings {
                request: "a".repeat(64),
                binary: "b".repeat(64),
            };
            let status = collect_ready(
                &url,
                "synthetic",
                "private",
                &mut raw,
                limits(),
                Some(&bindings),
                &mut Signal(ready_tx),
            );
            (status, raw)
        });
        let line = ready_rx.recv_timeout(Duration::from_secs(1)).unwrap();
        let ready = Ready::parse_line(&line).unwrap();
        assert_eq!(ready.http_status, 200);
        assert_eq!(ready.prefix_bytes, 28);
        assert_eq!(ready.prefix_sha256, hash_bytes(&prefix).unwrap());
        assert_eq!(ready.request_sha256, "a".repeat(64));
        assert!(!line.windows(b"PRIVATE".len()).any(|w| w == b"PRIVATE"));
        continue_tx.send(()).unwrap();
        server.join().unwrap();
        let (status, raw) = worker.join().unwrap();
        assert!(status.accepted());
        assert_eq!(&raw[..28], expected);
        assert!(ready_rx.try_recv().is_err());
    }
    #[test]
    fn status_parser_does_not_forward_headers_or_accept_multiple_responses() {
        assert_eq!(
            http_status(b"HTTP/1.1 200 OK\r\nSet-Cookie: PRIVATE_SENTINEL\r\n\r\n"),
            200
        );
        assert_eq!(http_status(b"HTTP/1.1 401 Unauthorized\r\n"), 401);
        assert_eq!(
            http_status(b"HTTP/1.1 100 Continue\r\nHTTP/1.1 200 OK\r\n"),
            0
        );
        assert_eq!(http_status(b"not a response"), 0);
    }
}
