# Identity construction

`c6-anonymization-identity` converts caller-provided random bytes into a local
unicast MAC and a bounded ASCII SSID. It is dependency-free, `no_std`, uses no
allocation and forbids unsafe code. It performs no hardware access, entropy
collection, persistence or interface initialization.

`LocalMac::from_random_bytes` sets the locally administered bit and clears the
multicast bit. `Identity::from_random_bytes` consumes fourteen bytes: six for
that MAC and eight encoded as the sixteen lowercase hexadecimal digits after
the `anon-` SSID prefix. The SSID is always 21 ASCII bytes.

An entirely zero fourteen-byte sample is rejected as an input sentinel. This
check does not establish entropy quality. The caller must supply randomness
appropriate for the intended identity lifetime; repeated input produces the
same identity. A fixed random identity can still correlate sessions.

The values are observable protocol identifiers, not authentication credentials.
Their `Debug` output is redacted to avoid adding correlation data to ordinary
diagnostics. Explicit accessors provide bytes for applying SDK controls. Data
equality or successful construction does not establish fresh entropy, applied
hardware state, transmitted identity or anonymity.

The crate-level Rust documentation contains a synthetic construction example.
Stack-specific initialization, readback and external observation belong to the
integration that uses these values.
