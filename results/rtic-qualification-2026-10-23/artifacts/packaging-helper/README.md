# Narrow offline C6 image correction

This unpublished Rust1.96.0/edition2024 host helper corrects the retained
espflash4.6.0 RTIC artifacts without modifying upstream source or original
images. It is not a flasher, ELF linker, general image editor or signature tool.
Archive this source, Cargo.toml, Cargo.lock, executable identity, inputs and
logs with the qualification provenance. Its output is a new artifact.

## Why an existing tool did not suffice

Both ordinary esptool5.1.0 ELF conversion and its `--use-segments` form reject
the unchanged old-HAL ELF's two flash segments within the same64KB map. The
original espflash output has valid XOR/SHA256 but frequency0xf and zero ELF hash.
The latest maintained release checked is4.6.0. Development
[`ac27e1f`](https://github.com/esp-rs/espflash/blob/ac27e1f463527929f133e2ffb34c481fa5a83e53/espflash/src/image_format/idf.rs#L448-L466)
still recognizes only `.flash.appdesc` for hash insertion, not this ELF's
`.rodata_desc`; its target frequency table still assigns the generic80MHz0xf.
The original binaries and failed conversion logs are retained separately.

The correct C6 frequency encoding comes from the installed, pinned
[esptool5.1.0 target](https://github.com/espressif/esptool/blob/v5.1.0/esptool/targets/esp32c6.py#L82-L87):
80MHz uses0 as a ROM workaround. Image structure and checksum/digest handling
follow the documented
[firmware format source](https://github.com/espressif/esptool/blob/v5.1.0/docs/en/advanced-topics/firmware-image-format.rst)
and the installed esptool bin_image.py. These source links specify field
meaning; independently running esptool image-info checks the resulting bytes.
For C6, encoding0 can describe80MHz or40MHz depending on MSPI clock source;
image-info's80m label is not runtime flash-clock readback.

## Interface and invariants

```sh
cargo +1.96.0 run --locked -- boot INPUT_BOOT_IMAGE NEW_OUTPUT
cargo +1.96.0 run --locked -- app INPUT_APP_IMAGE ORIGINAL_ELF NEW_OUTPUT
```

Inputs must already be intact C6/chip13,DIO,8MB,frequency0xf images with one to
sixteen nonempty aligned segments in the selected C6 RAM/flash ranges, no
loaded-address overlap, zero footer padding, valid segment XOR and appended
SHA256. Parsing is bounded by checked segment endpoints and actual file size.
Trailing/truncated/malformed data reject. Boot mode additionally requires a RAM
entry and RAM-only segments; app mode binds the RV32 little-endian ELF entry,
the actual256-byte descriptor mapping/magic and **every initialized allocated
ELF-section byte**, including a section split across adjacent RAM segments.
An unmapped gap, mismatching byte or overlapping mapping rejects.
The qualified ELF input is the pinned original executable with SHA-256
`12dc8bd4f079b9acdfda34db652e422cafddd9869c4b92493274cb56b47645a9`.
The helper checks RV32/endian, entry and data mappings; it does not additionally
classify Object's BinaryFormat::Elf/ObjectKind::Executable. Its qualification
therefore covers that known original ELF, without a general claim to reject
every other Object-supported file format/kind. The physical console preflight
checks ELF/executable identity separately.

The old-HAL descriptor occupies `.rodata_desc` at0x42000020, the first image
segment data at file0x20. Its ELF-hash field offset144 therefore maps to file
**0xb0**. The descriptor must match the original ELF and its old hash must be
zero. The helper writes the original ELF's actual32-byte SHA256 there.

The correction changes only image byte3 from0x3f to0x30, the app hash when in
app mode, the recomputed segment XOR, and the appended image SHA256. It checks
this byte-difference boundary and validates the new checksum/digest again.
Outputs use create_new, are written/synced explicitly and never overwrite an
existing path. Validation finishes before any output is created. A filesystem
write/sync error can leave a partial **new** output and returns failure; it
cannot damage a retained input or constitute a successful correction.

There is no unsafe Rust, FFI, device handle, interrupt, async cancellation,
thread or authored destructor. Host heap allocation and blocking file I/O are
outside any measured device path. Allocation failure is not claimed recoverable.

## Verification and retained implementation corrections

Initial helper compilation exposed object0.40's typed SectionFlags API; its
full failed test/Clippy logs are retained. The corrected use of the typed flag
and pattern compiled. The first actual app bind also rejected `.trap`: the
ELF section is legitimately split across two adjacent RAM image segments.
Binding now walks every byte across unique mappings, rather than skipping that
section or weakening the required byte agreement. A split-section regression
also rejects a changed byte and a gap. A later Clippy op_ref warning was fixed;
its failed log remains alongside the final passing checks.

Three meaningful unit tests pass: intact correction, eight malformed-image
counterexamples, and split-section mapping with corrupt-byte/gap controls.
Clippy all-targets with warnings denied and rustfmt pass. The final executable
was built and run against both actual inputs to exclusive final outputs.
Actual CLI controls reject a wrong ELF, app-as-boot role and existing output;
the invalid-input controls create no output. Independent esptool5.1.0 image-info
accepts both final checksums/digests and80MHz/DIO/8MB headers. The app reports
the exact original ELF SHA256. No physical operation is performed by these checks.

## All54 Rust API checklist dispositions

Reviewed against the repository's pinned coding standard and checklist. This
is a private one-shot application; no public library/package release API is
introduced. The following groups cover all54 active identifiers.

| Identifiers | Applicability and evidence |
| --- | --- |
| C-CASE, C-WORD-ORDER | Applies: ordinary Rust names; address/data/checksum/digest and original/output distinguish byte roles. |
| C-COMMON-TRAITS, C-DEBUG, C-DEBUG-NONEMPTY | Applies at private scope: field-bearing Debug for parsed Image/Segment; no live resource is Clone/Copy or given an invalid Default. |
| C-GOOD-ERR, C-FAILURE, C-VALIDATE | Applies: contextual Results for malformed inputs/ELF/identity/mappings and exclusive output/write failures, with validation before mutation/output and deliberate negatives. |
| C-CRATE-DOC, C-EXAMPLE, C-QUESTION-MARK, C-LINK | Applies at application scope: module docs plus these runnable fallible CLI forms, exact source links and actual-result limits. |
| C-METADATA | Exception M1 reused from docs/reviews/benchctl.md: accurate unpublished package/compiler/edition/description, pinned dependencies and retained lock; publication metadata and project licensing remain incomplete, not fully compliant. No authors/remote/license is invented. Revisit before publication. |
| C-SEND-SYNC | Applies by containment: single-threaded owned host data and file, no unsafe sharing or device/task lifetime. No extra unsafe trait implementations. |
| C-NO-OUT, C-INTERMEDIATE, C-CALLER-CONTROL | Applies: parsing/correction return values, borrow input bytes, retain originals; output ownership is chosen by caller and protected by create_new. |
| C-CUSTOM-TYPE, C-NEWTYPE | Applies at private scope: parsed Image and Segment retain validated data/address boundaries. Explicit CLI app/boot roles are checked; no interchangeable public duration/byte API is exposed. |
| C-PERMISSIVE | Exception L1 reused from docs/reviews/benchctl.md: project license remains unselected; this unpublished internal helper does not choose one. Pinned anyhow/object/sha2 retain their existing MIT OR Apache-2.0 licenses. Revisit project licensing before redistribution/publication; dependency licensing does not license this source by inference. |
| C-CONV, C-CONV-TRAITS, C-CONV-SPECIFIC, C-GETTER, C-METHOD, C-CTOR | Not applicable: no authored conversion/accessor/method/constructor API; named private fallible free functions perform the operation. |
| C-ITER, C-ITER-TY, C-COLLECT | Not applicable: no authored iterator or custom collection; Vec/Range use standard interfaces. |
| C-FEATURE | Not applicable to new names: no authored Cargo features; existing dependency read/std features are selected unchanged. |
| C-SERDE | Not applicable: no exported interchange Rust data type; private parser views are not serialized. The fixed vendor binary format is validated explicitly. |
| C-NUM-FMT, C-BITFLAG | Not applicable: no authored numeric/flag wrapper; standard numeric formatting and upstream typed ELF flags are used. |
| C-RW-VALUE, C-GENERIC, C-OBJECT, C-STRUCT-BOUNDS | Not applicable: no generic I/O, generic data structure or dynamic trait API is authored. |
| C-EVOCATIVE, C-MACRO-ATTR, C-ANYWHERE, C-MACRO-VIS, C-MACRO-TY | Not applicable: no authored macros. Standard/upstream macros do not create a new macro API here. |
| C-HIDDEN, C-STRUCT-PRIVATE, C-NEWTYPE-HIDE, C-SEALED, C-STABLE | No public library representation/trait/stability surface. Parsed fields/functions remain private and no extensibility promise is made. |
| C-SMART-PTR, C-BUILDER | Not applicable: no authored smart pointer or builder API. |
| C-DEREF, C-OVERLOAD | Not applicable: no authored Deref or operator-overloading implementation. |
| C-DTOR-FAIL, C-DTOR-BLOCK | No authored destructor; required write/sync failures are explicit Results before standard File cleanup. |
| C-RELNOTES | Not applicable: no published/versioned release; provenance records this experiment-specific source and executable. |
