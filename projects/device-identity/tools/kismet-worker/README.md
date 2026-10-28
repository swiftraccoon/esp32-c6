# Kismet packet worker

Read one bounded PCAPNG prefix from an existing Kismet datasource, keeping HTTP
credentials on the receiver host. The worker issues the fixed source-scoped GET
on loopback port 2501. It performs no source-control, radio tuning, interface
probing, service or credential-management operation.

This independent std-only Rust workspace pins Rust 1.90.0, edition/style 2024,
and the `x86_64-unknown-linux-gnu` target used by the existing receiver host. It
uses installed curl for HTTP and SHA256 utilities for build/runtime artifact
identity. The host controller builds its shared library with Rust 1.96.0. No
ESP-IDF SDK is needed. Project licensing remains undecided; this is an internal,
unpublished tool.

Build and qualify with the pinned toolchain from this directory:

```sh
cargo +1.90.0 test --locked --offline
cargo +1.90.0 build --release --locked --offline
```

Use `identityctl wifi-capture` with a private prequalified receiver profile;
manual credential values do not belong in command arguments. The worker reads
a bounded binary request from stdin and emits raw packet bytes on stdout plus
fixed terminal status on stderr. Request paths and source UUID remain private.
The no-network `--provenance` operation reports the executable/build-input and
tool identities expected by the profile.

The opt-in `--ready` operation adds one fixed readiness event on stderr after a
single HTTP 200 response and a complete section header have been saved to
stdout. The event binds the exact prefix offset/hash, request, executable and
build inputs. It does not establish packet reception. The AP trial controller
also validates its local saved prefix and checks the receiver process before
sending the start command. The ordinary no-argument stream behavior is unchanged.

The [capture method](../../experiments/wifi-observer.md) defines sample size,
deadlines, failure retention and source freezing. The
[API/safety contract](../../docs/api.md) covers credential custody, child-process
cleanup, shared framing and privacy. Only completed controller captures with
successful exact replay are eligible observations; standalone packet summaries
do not authenticate a capture or repair a failure.
