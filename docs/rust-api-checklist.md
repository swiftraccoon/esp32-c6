# Rust API review inventory

This inventory covers every active checklist entry at
[rust-lang/api-guidelines `19f0f62ecd0f56a0b7adaa332c352bc2476506ea`](https://github.com/rust-lang/api-guidelines/blob/19f0f62ecd0f56a0b7adaa332c352bc2476506ea/src/checklist.md).
There are **54** entries. Links below open the pinned explanatory sections.
Read those sections for the rule and its qualifications; the trigger column is
only a repository review prompt.

All entries in this generic inventory start **pending**. Actual implementation
decisions and evidence are recorded separately for [bench-core](reviews/bench-core.md),
[benchctl](reviews/benchctl.md), [the HAL adapters](reviews/esp-hal.md), and
[ESP-IDF](reviews/esp-idf.md). This inventory is not itself a claim of compliance.
The governing authority and embedded mappings are in
[coding-standard.md](coding-standard.md).

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
| [C-CASE](https://github.com/rust-lang/api-guidelines/blob/19f0f62ecd0f56a0b7adaa332c352bc2476506ea/src/naming.md#user-content-c-case) | Authored Rust identifiers | Pending |
| [C-CONV](https://github.com/rust-lang/api-guidelines/blob/19f0f62ecd0f56a0b7adaa332c352bc2476506ea/src/naming.md#user-content-c-conv) | Conversion methods | Pending |
| [C-GETTER](https://github.com/rust-lang/api-guidelines/blob/19f0f62ecd0f56a0b7adaa332c352bc2476506ea/src/naming.md#user-content-c-getter) | Accessors | Pending |
| [C-ITER](https://github.com/rust-lang/api-guidelines/blob/19f0f62ecd0f56a0b7adaa332c352bc2476506ea/src/naming.md#user-content-c-iter) | Collection iteration | Pending |
| [C-ITER-TY](https://github.com/rust-lang/api-guidelines/blob/19f0f62ecd0f56a0b7adaa332c352bc2476506ea/src/naming.md#user-content-c-iter-ty) | Named iterator types | Pending |
| [C-FEATURE](https://github.com/rust-lang/api-guidelines/blob/19f0f62ecd0f56a0b7adaa332c352bc2476506ea/src/naming.md#user-content-c-feature) | Cargo feature definitions | Pending |
| [C-WORD-ORDER](https://github.com/rust-lang/api-guidelines/blob/19f0f62ecd0f56a0b7adaa332c352bc2476506ea/src/naming.md#user-content-c-word-order) | Related identifier families | Pending |
| [C-COMMON-TRAITS](https://github.com/rust-lang/api-guidelines/blob/19f0f62ecd0f56a0b7adaa332c352bc2476506ea/src/interoperability.md#user-content-c-common-traits) | Exposed types; review resource ownership | Pending |
| [C-CONV-TRAITS](https://github.com/rust-lang/api-guidelines/blob/19f0f62ecd0f56a0b7adaa332c352bc2476506ea/src/interoperability.md#user-content-c-conv-traits) | Conversions between types | Pending |
| [C-COLLECT](https://github.com/rust-lang/api-guidelines/blob/19f0f62ecd0f56a0b7adaa332c352bc2476506ea/src/interoperability.md#user-content-c-collect) | Collections; review capacity failures | Pending |
| [C-SERDE](https://github.com/rust-lang/api-guidelines/blob/19f0f62ecd0f56a0b7adaa332c352bc2476506ea/src/interoperability.md#user-content-c-serde) | Data structures, configuration, result records | Pending |
| [C-SEND-SYNC](https://github.com/rust-lang/api-guidelines/blob/19f0f62ecd0f56a0b7adaa332c352bc2476506ea/src/interoperability.md#user-content-c-send-sync) | Exposed types and execution contexts | Pending |
| [C-GOOD-ERR](https://github.com/rust-lang/api-guidelines/blob/19f0f62ecd0f56a0b7adaa332c352bc2476506ea/src/interoperability.md#user-content-c-good-err) | Public failures; review `no_std` mapping | Pending |
| [C-NUM-FMT](https://github.com/rust-lang/api-guidelines/blob/19f0f62ecd0f56a0b7adaa332c352bc2476506ea/src/interoperability.md#user-content-c-num-fmt) | Numeric/bit-oriented wrapper types | Pending |
| [C-RW-VALUE](https://github.com/rust-lang/api-guidelines/blob/19f0f62ecd0f56a0b7adaa332c352bc2476506ea/src/interoperability.md#user-content-c-rw-value) | Generic I/O interfaces; inspect actual traits | Pending |
| [C-EVOCATIVE](https://github.com/rust-lang/api-guidelines/blob/19f0f62ecd0f56a0b7adaa332c352bc2476506ea/src/macros.md#user-content-c-evocative) | Authored macro interfaces | Pending |
| [C-MACRO-ATTR](https://github.com/rust-lang/api-guidelines/blob/19f0f62ecd0f56a0b7adaa332c352bc2476506ea/src/macros.md#user-content-c-macro-attr) | Macros producing items | Pending |
| [C-ANYWHERE](https://github.com/rust-lang/api-guidelines/blob/19f0f62ecd0f56a0b7adaa332c352bc2476506ea/src/macros.md#user-content-c-anywhere) | Item macros and supported invocation scopes | Pending |
| [C-MACRO-VIS](https://github.com/rust-lang/api-guidelines/blob/19f0f62ecd0f56a0b7adaa332c352bc2476506ea/src/macros.md#user-content-c-macro-vis) | Macro-generated public/private items | Pending |
| [C-MACRO-TY](https://github.com/rust-lang/api-guidelines/blob/19f0f62ecd0f56a0b7adaa332c352bc2476506ea/src/macros.md#user-content-c-macro-ty) | Macro inputs containing type expressions | Pending |
| [C-CRATE-DOC](https://github.com/rust-lang/api-guidelines/blob/19f0f62ecd0f56a0b7adaa332c352bc2476506ea/src/documentation.md#user-content-c-crate-doc) | Library entry points | Pending |
| [C-EXAMPLE](https://github.com/rust-lang/api-guidelines/blob/19f0f62ecd0f56a0b7adaa332c352bc2476506ea/src/documentation.md#user-content-c-example) | Public APIs and relevant hardware examples | Pending |
| [C-QUESTION-MARK](https://github.com/rust-lang/api-guidelines/blob/19f0f62ecd0f56a0b7adaa332c352bc2476506ea/src/documentation.md#user-content-c-question-mark) | Examples with fallible operations | Pending |
| [C-FAILURE](https://github.com/rust-lang/api-guidelines/blob/19f0f62ecd0f56a0b7adaa332c352bc2476506ea/src/documentation.md#user-content-c-failure) | API errors, panics, unsafe preconditions | Pending |
| [C-LINK](https://github.com/rust-lang/api-guidelines/blob/19f0f62ecd0f56a0b7adaa332c352bc2476506ea/src/documentation.md#user-content-c-link) | Related API/target documentation | Pending |
| [C-METADATA](https://github.com/rust-lang/api-guidelines/blob/19f0f62ecd0f56a0b7adaa332c352bc2476506ea/src/documentation.md#user-content-c-metadata) | Package manifests and publication intent | Pending |
| [C-RELNOTES](https://github.com/rust-lang/api-guidelines/blob/19f0f62ecd0f56a0b7adaa332c352bc2476506ea/src/documentation.md#user-content-c-relnotes) | Versioned releases | Pending |
| [C-HIDDEN](https://github.com/rust-lang/api-guidelines/blob/19f0f62ecd0f56a0b7adaa332c352bc2476506ea/src/documentation.md#user-content-c-hidden) | Exported documentation surface | Pending |
| [C-SMART-PTR](https://github.com/rust-lang/api-guidelines/blob/19f0f62ecd0f56a0b7adaa332c352bc2476506ea/src/predictability.md#user-content-c-smart-ptr) | Authored smart pointers | Pending |
| [C-CONV-SPECIFIC](https://github.com/rust-lang/api-guidelines/blob/19f0f62ecd0f56a0b7adaa332c352bc2476506ea/src/predictability.md#user-content-c-conv-specific) | Placement of conversion APIs | Pending |
| [C-METHOD](https://github.com/rust-lang/api-guidelines/blob/19f0f62ecd0f56a0b7adaa332c352bc2476506ea/src/predictability.md#user-content-c-method) | Operations associated with a value | Pending |
| [C-NO-OUT](https://github.com/rust-lang/api-guidelines/blob/19f0f62ecd0f56a0b7adaa332c352bc2476506ea/src/predictability.md#user-content-c-no-out) | Return values and caller-owned buffers | Pending |
| [C-OVERLOAD](https://github.com/rust-lang/api-guidelines/blob/19f0f62ecd0f56a0b7adaa332c352bc2476506ea/src/predictability.md#user-content-c-overload) | Operator implementations | Pending |
| [C-DEREF](https://github.com/rust-lang/api-guidelines/blob/19f0f62ecd0f56a0b7adaa332c352bc2476506ea/src/predictability.md#user-content-c-deref) | `Deref`/`DerefMut` implementations | Pending |
| [C-CTOR](https://github.com/rust-lang/api-guidelines/blob/19f0f62ecd0f56a0b7adaa332c352bc2476506ea/src/predictability.md#user-content-c-ctor) | Initialization interfaces | Pending |
| [C-INTERMEDIATE](https://github.com/rust-lang/api-guidelines/blob/19f0f62ecd0f56a0b7adaa332c352bc2476506ea/src/flexibility.md#user-content-c-intermediate) | Reusable calculations and partial outcomes | Pending |
| [C-CALLER-CONTROL](https://github.com/rust-lang/api-guidelines/blob/19f0f62ecd0f56a0b7adaa332c352bc2476506ea/src/flexibility.md#user-content-c-caller-control) | Ownership and buffer APIs | Pending |
| [C-GENERIC](https://github.com/rust-lang/api-guidelines/blob/19f0f62ecd0f56a0b7adaa332c352bc2476506ea/src/flexibility.md#user-content-c-generic) | Reusable parameter contracts and code size | Pending |
| [C-OBJECT](https://github.com/rust-lang/api-guidelines/blob/19f0f62ecd0f56a0b7adaa332c352bc2476506ea/src/flexibility.md#user-content-c-object) | Traits intended for dynamic dispatch | Pending |
| [C-NEWTYPE](https://github.com/rust-lang/api-guidelines/blob/19f0f62ecd0f56a0b7adaa332c352bc2476506ea/src/type-safety.md#user-content-c-newtype) | Units, identifiers, modes, semantic wrappers | Pending |
| [C-CUSTOM-TYPE](https://github.com/rust-lang/api-guidelines/blob/19f0f62ecd0f56a0b7adaa332c352bc2476506ea/src/type-safety.md#user-content-c-custom-type) | Ambiguous parameters/configuration | Pending |
| [C-BITFLAG](https://github.com/rust-lang/api-guidelines/blob/19f0f62ecd0f56a0b7adaa332c352bc2476506ea/src/type-safety.md#user-content-c-bitflag) | Combinable flags | Pending |
| [C-BUILDER](https://github.com/rust-lang/api-guidelines/blob/19f0f62ecd0f56a0b7adaa332c352bc2476506ea/src/type-safety.md#user-content-c-builder) | Complex construction contracts | Pending |
| [C-VALIDATE](https://github.com/rust-lang/api-guidelines/blob/19f0f62ecd0f56a0b7adaa332c352bc2476506ea/src/dependability.md#user-content-c-validate) | Input boundaries and invariants | Pending |
| [C-DTOR-FAIL](https://github.com/rust-lang/api-guidelines/blob/19f0f62ecd0f56a0b7adaa332c352bc2476506ea/src/dependability.md#user-content-c-dtor-fail) | Resource cleanup | Pending |
| [C-DTOR-BLOCK](https://github.com/rust-lang/api-guidelines/blob/19f0f62ecd0f56a0b7adaa332c352bc2476506ea/src/dependability.md#user-content-c-dtor-block) | Cleanup with transfer/wait/shutdown behavior | Pending |
| [C-DEBUG](https://github.com/rust-lang/api-guidelines/blob/19f0f62ecd0f56a0b7adaa332c352bc2476506ea/src/debuggability.md#user-content-c-debug) | Public types; review peripheral side effects | Pending |
| [C-DEBUG-NONEMPTY](https://github.com/rust-lang/api-guidelines/blob/19f0f62ecd0f56a0b7adaa332c352bc2476506ea/src/debuggability.md#user-content-c-debug-nonempty) | Public debug representations | Pending |
| [C-SEALED](https://github.com/rust-lang/api-guidelines/blob/19f0f62ecd0f56a0b7adaa332c352bc2476506ea/src/future-proofing.md#user-content-c-sealed) | Deliberately closed trait implementation sets | Pending |
| [C-STRUCT-PRIVATE](https://github.com/rust-lang/api-guidelines/blob/19f0f62ecd0f56a0b7adaa332c352bc2476506ea/src/future-proofing.md#user-content-c-struct-private) | Public representations and mutation invariants | Pending |
| [C-NEWTYPE-HIDE](https://github.com/rust-lang/api-guidelines/blob/19f0f62ecd0f56a0b7adaa332c352bc2476506ea/src/future-proofing.md#user-content-c-newtype-hide) | Exposed implementation-dependent types | Pending |
| [C-STRUCT-BOUNDS](https://github.com/rust-lang/api-guidelines/blob/19f0f62ecd0f56a0b7adaa332c352bc2476506ea/src/future-proofing.md#user-content-c-struct-bounds) | Generic data structure definitions | Pending |
| [C-STABLE](https://github.com/rust-lang/api-guidelines/blob/19f0f62ecd0f56a0b7adaa332c352bc2476506ea/src/necessities.md#user-content-c-stable) | Stable public API commitments | Pending |
| [C-PERMISSIVE](https://github.com/rust-lang/api-guidelines/blob/19f0f62ecd0f56a0b7adaa332c352bc2476506ea/src/necessities.md#user-content-c-permissive) | Project/dependency licensing and distribution | Pending |

## Pinned-source anomaly

The pinned `checklist.md` also contains a link definition named `C-HTML-ROOT`.
It is not an active checklist entry, and the pinned documentation chapter has no
matching section. It is recorded here for completeness and creates no extra
requirement. Do not invent an HTML-root configuration obligation from that
unused definition.
