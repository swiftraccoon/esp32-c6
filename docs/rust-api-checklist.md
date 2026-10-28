# Rust API review inventory

This inventory covers every active entry in the
[local Rust API checklist](standards/rust-api/src/checklist.md), at the revision
recorded in the [source lock](standards/sources.json).
There are **54** entries. Links below open the pinned explanatory sections.
Read those sections for the rule and its qualifications; the trigger column is
only a repository review prompt.

This inventory is a review aid, not a claim of compliance. Current component
decisions cover all 54 entries:

- [Workspace utilities](../tools/workspace-tools/docs/api.md).
- [OS workload](../projects/os-evaluation/crates/bench-core/docs/api.md) and
  [collector](../projects/os-evaluation/tools/benchctl/docs/api.md).
- [HAL firmware](../projects/os-evaluation/firmware/esp-hal/docs/api.md) and
  [IDF firmware](../projects/os-evaluation/firmware/esp-idf/docs/api.md).
- [Wireless explorer](../projects/wireless-explorer/docs/api.md).
- [Device identity host tools, protocol and firmware](../projects/device-identity/docs/api.md).

The authority and embedded mappings are in [coding-standard.md](coding-standard.md).

## Review procedure

For a new crate, assess every row. For a later change, reuse the crate's record,
review the affected rows, and reconsider prior applicability decisions when its
public API, target, features, dependencies, or release status changes. Record
the crate/change identifier and supported build configurations with the review.

Use **applies** with implementation/check evidence, **not applicable** with the
specific absent construct or scope, **exception** with the narrow reason and
consequence, or **pending** when unresolved. Do not turn an inconvenient
applicable rule into “not applicable.” An exception needs the source, target,
alternatives, consequence, and revisit condition; the standard describes this
process. These decisions belong in a review record or crate documentation,
linked from the change. Do not mutate this initial inventory into a claim about
all future code.

| Guideline and pinned explanation | Applicability trigger in this repository | Initial state / evidence |
| --- | --- | --- |
| [C-CASE](standards/rust-api/src/naming.md#c-case) | Authored Rust identifiers | Pending |
| [C-CONV](standards/rust-api/src/naming.md#c-conv) | Conversion methods | Pending |
| [C-GETTER](standards/rust-api/src/naming.md#c-getter) | Accessors | Pending |
| [C-ITER](standards/rust-api/src/naming.md#c-iter) | Collection iteration | Pending |
| [C-ITER-TY](standards/rust-api/src/naming.md#c-iter-ty) | Named iterator types | Pending |
| [C-FEATURE](standards/rust-api/src/naming.md#c-feature) | Cargo feature definitions | Pending |
| [C-WORD-ORDER](standards/rust-api/src/naming.md#c-word-order) | Related identifier families | Pending |
| [C-COMMON-TRAITS](standards/rust-api/src/interoperability.md#c-common-traits) | Exposed types; review resource ownership | Pending |
| [C-CONV-TRAITS](standards/rust-api/src/interoperability.md#c-conv-traits) | Conversions between types | Pending |
| [C-COLLECT](standards/rust-api/src/interoperability.md#c-collect) | Collections; review capacity failures | Pending |
| [C-SERDE](standards/rust-api/src/interoperability.md#c-serde) | Data structures, configuration, result records | Pending |
| [C-SEND-SYNC](standards/rust-api/src/interoperability.md#c-send-sync) | Exposed types and execution contexts | Pending |
| [C-GOOD-ERR](standards/rust-api/src/interoperability.md#c-good-err) | Public failures; review `no_std` mapping | Pending |
| [C-NUM-FMT](standards/rust-api/src/interoperability.md#c-num-fmt) | Numeric/bit-oriented wrapper types | Pending |
| [C-RW-VALUE](standards/rust-api/src/interoperability.md#c-rw-value) | Generic I/O interfaces; inspect actual traits | Pending |
| [C-EVOCATIVE](standards/rust-api/src/macros.md#c-evocative) | Authored macro interfaces | Pending |
| [C-MACRO-ATTR](standards/rust-api/src/macros.md#c-macro-attr) | Macros producing items | Pending |
| [C-ANYWHERE](standards/rust-api/src/macros.md#c-anywhere) | Item macros and supported invocation scopes | Pending |
| [C-MACRO-VIS](standards/rust-api/src/macros.md#c-macro-vis) | Macro-generated public/private items | Pending |
| [C-MACRO-TY](standards/rust-api/src/macros.md#c-macro-ty) | Macro inputs containing type expressions | Pending |
| [C-CRATE-DOC](standards/rust-api/src/documentation.md#c-crate-doc) | Library entry points | Pending |
| [C-EXAMPLE](standards/rust-api/src/documentation.md#c-example) | Public APIs and relevant hardware examples | Pending |
| [C-QUESTION-MARK](standards/rust-api/src/documentation.md#c-question-mark) | Examples with fallible operations | Pending |
| [C-FAILURE](standards/rust-api/src/documentation.md#c-failure) | API errors, panics, unsafe preconditions | Pending |
| [C-LINK](standards/rust-api/src/documentation.md#c-link) | Related API/target documentation | Pending |
| [C-METADATA](standards/rust-api/src/documentation.md#c-metadata) | Package manifests and publication intent | Pending |
| [C-RELNOTES](standards/rust-api/src/documentation.md#c-relnotes) | Versioned releases | Pending |
| [C-HIDDEN](standards/rust-api/src/documentation.md#c-hidden) | Exported documentation surface | Pending |
| [C-SMART-PTR](standards/rust-api/src/predictability.md#c-smart-ptr) | Authored smart pointers | Pending |
| [C-CONV-SPECIFIC](standards/rust-api/src/predictability.md#c-conv-specific) | Placement of conversion APIs | Pending |
| [C-METHOD](standards/rust-api/src/predictability.md#c-method) | Operations associated with a value | Pending |
| [C-NO-OUT](standards/rust-api/src/predictability.md#c-no-out) | Return values and caller-owned buffers | Pending |
| [C-OVERLOAD](standards/rust-api/src/predictability.md#c-overload) | Operator implementations | Pending |
| [C-DEREF](standards/rust-api/src/predictability.md#c-deref) | `Deref`/`DerefMut` implementations | Pending |
| [C-CTOR](standards/rust-api/src/predictability.md#c-ctor) | Initialization interfaces | Pending |
| [C-INTERMEDIATE](standards/rust-api/src/flexibility.md#c-intermediate) | Reusable calculations and partial outcomes | Pending |
| [C-CALLER-CONTROL](standards/rust-api/src/flexibility.md#c-caller-control) | Ownership and buffer APIs | Pending |
| [C-GENERIC](standards/rust-api/src/flexibility.md#c-generic) | Reusable parameter contracts and code size | Pending |
| [C-OBJECT](standards/rust-api/src/flexibility.md#c-object) | Traits intended for dynamic dispatch | Pending |
| [C-NEWTYPE](standards/rust-api/src/type-safety.md#c-newtype) | Units, identifiers, modes, semantic wrappers | Pending |
| [C-CUSTOM-TYPE](standards/rust-api/src/type-safety.md#c-custom-type) | Ambiguous parameters/configuration | Pending |
| [C-BITFLAG](standards/rust-api/src/type-safety.md#c-bitflag) | Combinable flags | Pending |
| [C-BUILDER](standards/rust-api/src/type-safety.md#c-builder) | Complex construction contracts | Pending |
| [C-VALIDATE](standards/rust-api/src/dependability.md#c-validate) | Input boundaries and invariants | Pending |
| [C-DTOR-FAIL](standards/rust-api/src/dependability.md#c-dtor-fail) | Resource cleanup | Pending |
| [C-DTOR-BLOCK](standards/rust-api/src/dependability.md#c-dtor-block) | Cleanup with transfer/wait/shutdown behavior | Pending |
| [C-DEBUG](standards/rust-api/src/debuggability.md#c-debug) | Public types; review peripheral side effects | Pending |
| [C-DEBUG-NONEMPTY](standards/rust-api/src/debuggability.md#c-debug-nonempty) | Public debug representations | Pending |
| [C-SEALED](standards/rust-api/src/future-proofing.md#c-sealed) | Deliberately closed trait implementation sets | Pending |
| [C-STRUCT-PRIVATE](standards/rust-api/src/future-proofing.md#c-struct-private) | Public representations and mutation invariants | Pending |
| [C-NEWTYPE-HIDE](standards/rust-api/src/future-proofing.md#c-newtype-hide) | Exposed implementation-dependent types | Pending |
| [C-STRUCT-BOUNDS](standards/rust-api/src/future-proofing.md#c-struct-bounds) | Generic data structure definitions | Pending |
| [C-STABLE](standards/rust-api/src/necessities.md#c-stable) | Stable public API commitments | Pending |
| [C-PERMISSIVE](standards/rust-api/src/necessities.md#c-permissive) | Project/dependency licensing and distribution | Pending |

## Pinned-source anomaly

The pinned `checklist.md` also contains a link definition named `C-HTML-ROOT`.
It is not an active checklist entry, and the pinned documentation chapter has no
matching section. It is recorded here for completeness and creates no extra
requirement. Do not invent an HTML-root configuration obligation from that
unused definition.
