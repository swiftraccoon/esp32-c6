use std::env;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

fn command() -> Command {
    if cfg!(target_os = "macos") {
        let mut c = Command::new("shasum");
        c.args(["-a", "256"]);
        c
    } else {
        Command::new("sha256sum")
    }
}

fn hash(path: &Path) -> String {
    let output = command()
        .arg(path)
        .output()
        .expect("source hash tool must run");
    assert!(output.status.success(), "source hash tool must succeed");
    let text = String::from_utf8(output.stdout).expect("source hash must be UTF-8");
    let digest = text
        .split_ascii_whitespace()
        .next()
        .expect("source hash must exist");
    assert!(
        digest.len() == 64 && digest.bytes().all(|b| b.is_ascii_hexdigit()),
        "source hash must have SHA256 shape"
    );
    digest.to_owned()
}

fn main() {
    let inputs = [
        "Cargo.toml",
        "Cargo.lock",
        "rust-toolchain.toml",
        "build.rs",
        "src/lib.rs",
        "src/main.rs",
        "src/readiness.rs",
        "../../crates/wifi-observer/src/framing.rs",
    ];
    let mut manifest = String::new();
    for input in inputs {
        println!("cargo:rerun-if-changed={input}");
        manifest.push_str(&format!("{input}={}\n", hash(Path::new(input))));
    }
    let mut child = command()
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("input hash tool must run");
    child
        .stdin
        .take()
        .expect("input hash stdin must exist")
        .write_all(manifest.as_bytes())
        .expect("input hash must be written");
    let output = child
        .wait_with_output()
        .expect("input hash tool must finish");
    assert!(output.status.success(), "input hash tool must succeed");
    let text = String::from_utf8(output.stdout).expect("input hash must be UTF-8");
    let digest = text
        .split_ascii_whitespace()
        .next()
        .expect("input hash must exist");
    println!("cargo:rustc-env=WORKER_INPUTS_SHA256={digest}");
    let version = Command::new(env::var_os("RUSTC").expect("Cargo must supply RUSTC"))
        .arg("--version")
        .output()
        .expect("rustc version must run");
    assert!(version.status.success(), "rustc version must succeed");
    let version = String::from_utf8(version.stdout).expect("rustc version must be UTF-8");
    println!("cargo:rustc-env=WORKER_RUSTC_VERSION={}", version.trim());
}
