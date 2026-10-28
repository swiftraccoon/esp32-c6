# Rust API and execution contract

The internal `c6-anonymization-identity`, `c6-anonymization-esp-idf` and
`c6-anonymization-softap-example` packages pin Rust 1.96.0, edition/style 2024
and their own workspace lockfiles. They are unpublished 0.1 packages with no
selected distribution license. The identity crate is dependency-free `no_std`;
the ESP-IDF adapter and example use `std` on the C6 target. Host tests do not
initialize an SDK or open devices.

## Identity values

`LocalMac` constructs a locally administered unicast value from six caller
bytes. `Identity` constructs an AP MAC and 21-byte ASCII SSID from fourteen
caller bytes. Its all-zero rejection is a sentinel check, not entropy testing.
The constructor does not obtain randomness or guarantee freshness; equal
samples produce equal identities. Copyable values confer no hardware authority.
Their fields are private, explicit accessors expose observable values, and
Debug redacts them. Core construction allocates nothing and uses no unsafe code.
SSID UTF-8 conversion relies on its private ASCII-only construction invariant.

## SDK adapter and example

The [adapter contract](../esp-idf/README.md) specifies the unsafe early-startup
preconditions, one-attempt latch, SAR enable/fill/disable sequence and exact
34-byte sample layout. BASE/AP table selection precedes interface reads and
driver initialization. BASE remains fixed for the boot; two AP identities can
be selected across explicit checked Stop. Known sentinels and collisions reject.
The actual build must exclude the SDK randomness bypass and persistent storage.

No safe API can attest that another application component did not already use
ADC, RF or the unsynchronized SDK MAC table. The public unsafe constructor
therefore states those exclusivity/pre-initialization requirements. Subsequent
methods retain one concrete driver and expose no mutable driver escape hatch.
`Cell` makes the handle not Sync; no unsafe Send/Sync implementation is authored.
The methods run in normal task context and can allocate, lock, sleep or block.

FFI uses generated SDK bindings, fixed scalar enums and live bounded buffers;
foreign pointers are not retained. An atomic acquire/release latch consumes
initialization before operations begin. SAR is disabled before fallible Rust
validation or Wi-Fi initialization. SDK errors retain numeric codes and fixed
context. Later errors stick and prohibit further Starts, while Stop can retry.
The driver/event loop use `ManuallyDrop` to avoid fallible vendor cleanup;
dropping the handle does not stop RF or release those resources before reset.

SDK active/inactive observations are cached event state, not independent
RF evidence or fresh command-correlated Stop attribution. After failed/in-flight
Start, an already-inactive cached state can satisfy a later observation. The
sticky failure prevents a successful session claim; failed physical trials
still require external reset/restoration recovery.

The [example](../examples/softap/README.md) owns increasing UART request IDs,
one attempt per epoch, explicit-Stop admission and prearmed software deadlines.
Its main loop handles partial writes, bounded frames, transport errors and
throttled cleanup retries. A late Start is rejected. Requested timers do not
bound blocking SDK calls or physical RF duration. Panic abort/reboot invalidates
the session. UART replies are deliberately private identity-bearing records;
ordinary SDK consoles/logs are disabled. Malformed input uses fixed errors and
never creates an admitted absence/silence result.

## API guideline applicability

The [complete 54-entry inventory](../../../docs/rust-api-checklist.md) and its
pinned explanatory chapters govern this record. Decisions cover all three
packages, their host controls and the selected C6 build; hardware claims require
the [separate observation method](../experiments/softap-identity.md).

| Entry | Decision and evidence |
| --- | --- |
| C-CASE | Applies: standard Rust casing and explicit UART enum names. |
| C-CONV | Applies: random-byte constructors name their source and normalization; raw accessors do not imply entropy generation. |
| C-GETTER | Applies: `mac`, `ssid`, `identity`, `failure` and `active` use conventional names. |
| C-ITER | Not applicable: no authored collection iteration API. |
| C-ITER-TY | Not applicable: no iterator-returning collection method. |
| C-FEATURE | Applies: example `firmware` selects the target binary; host protocol tests need no SDK. |
| C-WORD-ORDER | Applies: package names consistently identify anonymization identity, adapter and example. |
| C-COMMON-TRAITS | Applies: value types have Copy/Clone and equality; Epoch also has Hash. Diagnostics have Debug/Display/Error. Live SDK ownership is not Clone/Copy/Default. |
| C-CONV-TRAITS | Not applicable: normalization and fallible identity construction are named operations, not general lossless conversions. |
| C-COLLECT | Not applicable: no authored collection. |
| C-SERDE | Applies to bounded UART records. Core identities are private-representation values, not serialized configuration; explicit raw accessors support the example's private interchange. |
| C-SEND-SYNC | Applies: normal automatic traits for data; the live adapter is not Sync and requires sole task ownership. No authored unsafe auto-trait implementations. |
| C-GOOD-ERR | Applies with E1: core/adapter errors have fixed variants, Display and core Error; SDK codes are retained. Example-local i32 errors intentionally match its bounded private status protocol. |
| C-NUM-FMT | Not applicable: no public binary numeric wrapper. MAC Debug intentionally redacts rather than printing addresses. |
| C-RW-VALUE | Not applicable: no public generic reader/writer abstraction. Target UART I/O is private and concrete. |
| C-EVOCATIVE | Not applicable: no authored macro API. |
| C-MACRO-ATTR | Not applicable: no item-producing macro API. |
| C-ANYWHERE | Not applicable: no authored item macros. |
| C-MACRO-VIS | Not applicable: no macro-generated public surface. |
| C-MACRO-TY | Not applicable: no macro type-fragment interface. |
| C-CRATE-DOC | Applies: each library documents purpose, execution and claim boundaries. |
| C-EXAMPLE | Applies: core rustdoc demonstrates fallible construction; the adapter/example READMEs show the related hardware integration and startup conditions. |
| C-QUESTION-MARK | Applies: fallible core rustdoc and integration sketch propagate errors. Test assertions are confined to controls. |
| C-FAILURE | Applies: constructors/operations document errors, startup safety, retained resources, blocking and explicit Stop. The example states its invariant assertion and panic/reset behavior. |
| C-LINK | Applies: component READMEs, integration design, evidence inputs and observation method link to their owners. |
| C-METADATA | Exception M1: accurate internal names/descriptions/compiler/publish=false; license/authors/distribution URLs remain undecided rather than invented. |
| C-RELNOTES | Not applicable: no public release. Meaningful behavior/checks are recorded in commits. |
| C-HIDDEN | Applies: identity representation and SDK backend are private; only integration operations and example wire records are public. |
| C-SMART-PTR | Not applicable: no authored smart pointer. ManuallyDrop is a standard ownership mechanism with documented retention. |
| C-CONV-SPECIFIC | Applies: byte construction stays on the specific identity/value types. |
| C-METHOD | Applies: identity construction/access, SDK transitions and session admission are inherent methods. |
| C-NO-OUT | Applies: operations return complete values/results. The bounded reply encoder deliberately borrows caller storage and returns its used length. |
| C-OVERLOAD | Not applicable: no overloaded operators. |
| C-DEREF | Not applicable: no authored Deref/DerefMut implementation. |
| C-CTOR | Applies: constructors establish private representations; the unsafe SDK constructor consumes one startup attempt. |
| C-INTERMEDIATE | Applies: selected identities are explicitly available before Start for independent observation; no claim is inferred from selection alone. |
| C-CALLER-CONTROL | Applies: caller owns startup, identity lifetimes, buffers, deadlines and cleanup. Core values do not allocate. |
| C-GENERIC | Not applicable: no public generic abstraction is needed for this first concrete SDK adapter. |
| C-OBJECT | Not applicable: no authored trait intended for dynamic dispatch. |
| C-NEWTYPE | Applies: LocalMac and Identity distinguish normalized addresses and coupled observable identities from random bytes. |
| C-CUSTOM-TYPE | Applies: Epoch, Operation, Domain and typed errors prevent unrelated boolean/byte meanings. |
| C-BITFLAG | Not applicable: no combinable public flags. |
| C-BUILDER | Not applicable: no complex public builder is introduced. SDK configuration is fixed for this integration. |
| C-VALIDATE | Applies: sample/collision guards, exact SDK configuration/address readback, order/failure checks, bounded complete JSON and deadline controls. No statistical entropy-health or RF-absence claim follows. |
| C-DTOR-FAIL | Applies: no authored fallible destructor. SDK resources are deliberately retained; checked Stop is explicit and failures remain visible. |
| C-DTOR-BLOCK | Applies: no authored blocking destructor; explicit Stop/state waits can block and have no hard RF bound. |
| C-DEBUG | Applies: identity values and private replies redact content; adapter Debug shows only fixed state/failure. |
| C-DEBUG-NONEMPTY | Applies: redacted Debug retains type and meaningful nonprivate state. |
| C-SEALED | Not applicable: no public trait implementation set. |
| C-STRUCT-PRIVATE | Applies: identity, adapter and session fields are private. Public request/reply fields are passive example interchange validated at admission. |
| C-NEWTYPE-HIDE | Applies: MAC/identity byte representations are private behind explicit accessors. |
| C-STRUCT-BOUNDS | Not applicable: no authored generic struct bounds. |
| C-STABLE | Exception S1: internal 0.1 API and pinned Rust/SDK; target std uses scoped bootstrap. No cross-SDK or stable API guarantee. |
| C-PERMISSIVE | Exception L1: project licensing is undecided and packages are unpublished. Vendor/wrapper dependencies retain their own licenses. |

E1 is confined to the example-local wire status layer, not a general-purpose
library error contract. Revisit it if that protocol becomes reusable. M1/L1/S1
require deliberate metadata, licensing, compatibility and release decisions
before distribution. Host property/failure controls and independent source review
support only the exercised representations and sequencing, not SDK fault
injection, UART operation, receiver delivery, entropy quality or full anonymity.
