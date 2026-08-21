# Nsld Executable Finalizer Registry

Status: implemented early-beta contract, incomplete platform-writer parity.

## Purpose

`nuis-nsld-executable-finalizer-registry-v1` is the static boundary between
Nsld's object-format-neutral final executable pipeline and an OS-native
executable shell writer. The final-stage, writer, and emit paths select a
registered provider; they do not branch on Mach-O, ELF, or PE/COFF themselves.

This keeps the verified NSB image, lifecycle metadata, and deterministic link
plan as semantic authority. An OS executable is a compatibility shell around
that authority, not a replacement binary model.

## Selection Contract

Every selection uses a canonical target key:

```text
<machine-architecture>-<operating-system>-<object-format>
```

The current normalizations include:

* `arm64` to `aarch64`
* `darwin` and `apple-darwin` to `macos`
* `linux-gnu` and `gnu-linux` to `linux`
* `win32` and `win64` to `windows`
* `macho` to `mach-o`
* `coff`, `pe`, and `pe/coff` to `pe-coff`

Registrations may use `*` only for a target component they intentionally leave
open. The most specific match wins. Missing providers and equal-specificity
ambiguity fail closed, so registration order never becomes hidden authority.

The registry validates unique provider and target identities, verifies that a
`ready` provider owns an executor, and hashes its canonical registration set.
Dry-run JSON and the persisted host invoke plan expose:

* `finalizer_contract`
* `finalizer_registry_hash`
* `finalizer_registry_valid`
* `finalizer_target_key`
* `finalizer_provider_id`
* `finalizer_provider_status`
* `finalizer_execution_kind`

## Current Providers

Provider routes include the canonical target key and packaging mode. This lets
one platform keep a narrow internal route alongside a compatibility fallback
without relying on registration order.

The current static set is deliberately asymmetric:

| Provider | Target | Packaging | Input | Status |
| --- | --- | --- | --- | --- |
| `nsld.finalizer.elf.amd64.artifact-image-v1` | `x86_64-linux-elf` | `native-cpu-llvm` | compiled-artifact native handoff | `ready` |
| `nsld.finalizer.mach-o.arm64.artifact-image-v1` | `aarch64-macos-mach-o` | `native-cpu-llvm` | compiled-artifact native handoff | `ready` |
| `nsld.finalizer.mach-o.arm64.host-command-shell-v1` | `aarch64-macos-mach-o` | any | native object | `ready` |
| `nsld.finalizer.mach-o.registered-v1` | `*-macos-mach-o` | any | native object | `registered-not-implemented` |
| `nsld.finalizer.elf.registered-v1` | `*-linux-elf` | any | native object | `registered-not-implemented` |
| `nsld.finalizer.pe-coff.registered-v1` | `*-windows-pe-coff` | any | native object | `registered-not-implemented` |

For `native-cpu-llvm`, Nuisc compiles the LLVM program and runtime shim into
separate relocatable objects. The compiled artifact carries them in the
versioned `NHOB` bundle under `host_objects_binary`, with stable object id,
role, object format, and bytes. Artifact reports and LinkPlan additionally
project byte counts and content hashes.

The internal Mach-O provider parses that compiled-artifact native handoff,
checks target and ABI identity, requires exactly one `program-llvm` and one
`runtime-shim` role, and checks every object against its LinkPlan identity and
hash. Each payload must be an arm64 `MH_OBJECT` with a structurally valid load
command span. The provider now parses every `LC_SEGMENT_64` section record,
the single `LC_SYMTAB` plus its `nlist_64` and string tables, and every ARM64
relocation table. All spans, section ordinals, symbol indexes, relocation
widths, and `ADDEND` payload semantics are checked before admission.

The provider returns `nuis-nsld-macho-host-object-linkage-v1` through the
generic finalizer summary callback. That report counts objects, sections,
symbols, and relocations; identifies cross-object references resolved inside
the program/runtime pair; and lists the remaining C/system compatibility
symbols without treating them as internally resolved. JSON, CLI text, and the
persisted invoke plan consume the same typed summary. The provider then
validates either a thin arm64
`MH_EXECUTE` compatibility image or the arm64 slice of a universal Mach-O and
atomically materializes it with executable permissions. This path requires no
host process or environment gate, so Nsld does not invoke clang a second time.

The host-command provider remains a compatibility fallback. It owns both
command planning and actual process invocation, while
`final_executable_emit` stays provider-neutral. The policy and explicit-allow
gates still apply before that provider may run. It must execute the exact
driver path resolved during the verified dry-run boundary and may not repeat a
`PATH` lookup after admission.

The first ELF route uses the same internal command evidence and provider-neutral
atomic executable writer. The `x86_64-linux-elf + native-cpu-llvm` provider
requires exact plan/artifact target and ABI identity, one `program-llvm` and one
`runtime-shim` object, matching object ids, roles, sizes, formats, and FNV
hashes. Each object must be a little-endian ELF64 x86-64 `ET_REL` with a bounded
section table, one bounded `SHT_SYMTAB`, valid section and symbol string tables,
and explicit-addend `SHT_RELA` records using the registered `R_X86_64_NONE`,
`64`, `PC32`, `PLT32`, `32`, or `32S` subset. The provider rejects malformed
local/global symbol boundaries, duplicate strong definitions, unsupported
relocations, and out-of-range patch sites, then derives the internally resolved
and external compatibility symbol sets across the program/runtime pair. The
compatibility image must be ELF64 `ET_EXEC` or PIE `ET_DYN`, own a bounded
program-header table and at least one `PT_LOAD`, and place its nonzero entry
inside a file-backed executable load segment. Rejection occurs before output
mutation; accepted bytes are installed atomically with executable permissions
without invoking Clang or LLD.

This closes a registered Linux compatibility-image finalizer, not a pure Nsld
ELF linker. The accepted executable is still the host-toolchain-linked image
embedded by Nuisc. Nsld now owns object parsing and cross-object symbol closure,
deterministic final addresses, merged-image reconstruction, direct write-once
relocation application, and `nuis-nsld-elf-amd64-platform-structure-plan-v1`.
That plan validates the applied-image ledger, groups external PLT32 targets
through a static provider rule registry, assigns shared nonlazy PLT/GOT slots
plus dynamic symbol/string and `R_X86_64_JUMP_SLOT` records, and binds every
deferred source to one non-mutating patch preview. The platform application
stage now materializes those records and deferred patches once under a second
write ledger. `nuis-nsld-elf-amd64-shell-layout-plan-v1` maps that exact ledger
into deterministic ELF/program/section tables, non-overlapping permission
segments, an optional dynamic table, and a file-backed executable entry. Nsld
does not yet serialize or load-admit the provider-owned ELF shell.

This is not yet a pure Nsld linker claim. Nsld now understands the real input
tables, assigns deterministic final sections and addresses, applies registered
direct/platform relocations, emits stub/GOT and dyld metadata, and serializes a
private final-address Mach-O shell through
`nuis-nsld-macho-arm64-shell-image-serialization-v2`. The serializer appends a
standard SHA-256 ad-hoc SuperBlob/CodeDirectory through
`nuis-nsld-macho-arm64-ad-hoc-signature-v1`; its independent validator reparses
all Mach-O command boundaries, the deterministic `LC_UUID`, `__LINKEDIT`
coverage, signature fields, padding, and code slots before emitting
`nuis-nsld-macho-arm64-signed-image-validation-v1`.
`nuis-nsld-macho-arm64-os-loader-probe-v1` now provides an explicit,
non-publishing loader gate. It defaults to plan-only and, under `--apply`, only
materializes signed zero-unresolved/zero-bind inputs in a private temporary
file, checks exact-byte identity, bounds process time and output, and proves
cleanup. A fully internal ARM64 fixture is accepted by the real macOS kernel
and dyld and exits zero. Successful apply now persists a provider-owned,
SHA-256-bound `nuis-nsld-macho-arm64-publication-admission-v1` receipt at one
stable relative filename. The independent validation contract strictly reparses
the canonical flat TOML, rebuilds the current private product, and replays the
finalizer registry, target, image, signature, and loader-probe identities. The
ordinary compiled-artifact route has a positive internally closed fixture;
receipt tamper and rebuilt-product drift fail closed. Nuisc still embeds the
host-toolchain-linked compatibility executable used for default runnable
publication. The narrow AMD64 ELF compatibility route is now ready; other ELF
architectures and PE/COFF remain visible, selectable, and honestly blocked
rather than silently falling through a generic linker.

The artifact-image registration also declares the unique capability
`nsld.finalizer.mach-o.arm64.private-image-publication-v1`. Capability identity
and presence participate in the registry hash. The provider-neutral
`final-executable-private-image-publication` frontdoor only invokes that
registered callback. Plan-only mode does not mutate output; explicit apply
replays admission immediately before atomically installing the exact held image.
Invalid admission preserves the compatibility output, while the positive
compiled-artifact fixture publishes an owner-executable Mach-O and runs it
through the OS loader.

## Extension Rule

A future provider must:

1. register one stable provider identity and canonical target pattern;
2. consume the format-independent command/request context;
3. preserve the verified NSB entry, lifecycle, clock, and data-order metadata;
4. return an error before output mutation when its target or relocation input
   is unsupported;
5. pass the registry conformance and finalizer CLI regressions.
6. register any private-image publication capability with one unique stable ID
   and keep the callback absent when the provider cannot honor replay admission.

The deterministic non-section-symbol milestone is now closed by
`nuis-nsld-macho-placement-binding-v3`. The Mach-O provider coalesces tentative
definitions into provider-owned zero-fill storage, preserves `N_ABS` values as
absolute coordinates, and resolves `N_INDR` aliases through one deterministic
graph. Missing targets and cycles fail before mutation. Relocation,
materialization, platform GOT construction, and shell symbols consume the same
binding evidence; aliases lower to their terminal section or absolute form and
the shell emits real `N_ABS` records. A real alias/absolute fixture still passes
signature, admission, registered publication, and execution without adding
Mach-O branches to Nuisc or the generic emit frontdoor.

`nuis-nsld-final-output-selection-registry-v1` now closes the ordinary-output
selection boundary without adding object-format logic to Nuisc. Its sole
default, `compatibility-default`, only observes the existing output. The
explicit `admitted-private-image` policy delegates through the selected
finalizer capability and may install only bytes accepted by receipt replay.
Plan-only requests never mutate output; `--apply` is rejected without an
explicit apply-capable policy. `nuis-nsld-final-output-selection-evidence-v1`
binds the policy-registry hash, provider/target/capability, admission receipt and
verification hashes, publication ledger, candidate image, and installed output
size/SHA-256/executable identity into the ordinary final-output report. Receipt
tamper remains fail-closed and the compatibility executable remains the
byte-for-byte default.

`nuis-nsld-elf-amd64-placement-binding-v1` now closes the first provider-owned
Linux layout milestone. It merges `SHF_ALLOC` contributions into deterministic
text, read-only data, data, zero-fill, and common classes; page-separates their
permission boundaries; orders program before runtime; assigns checked aligned
file, image, and virtual coordinates; coalesces common declarations by maximum
size/alignment with strong-definition
precedence; preserves absolute values; maps unmatched weak references to zero;
and binds internal references while retaining unresolved system names as an
explicit compatibility boundary. TLS,
compressed, writable-executable, malformed zero-fill, excessive-alignment, and
overflow cases fail before mutation. Reversing object input preserves the same
plan hash.

`nuis-nsld-elf-amd64-relocation-application-v1` now maps every registered
`R_X86_64_NONE/64/PC32/PLT32/32/32S` record to one placed source and bound
target. It computes `S+A` or `S+A-P`, checks signed or unsigned width semantics,
and emits canonical little-endian patch previews. Placement-hash drift,
unsupported shapes, and overflow fail before mutation; unresolved compatibility
targets remain explicit platform-structure work. Reversing object input
preserves the relocation plan hash.

`nuis-nsld-elf-amd64-materialization-preview-v1` now closes the read-only image
construction milestone. It reparses hash- and size-bound `ET_REL` inputs,
copies every verified file-backed placement into a deterministic merged memory
image, proves zero-fill ranges separately, and binds source-object, file-image,
memory-image, placement, and relocation hashes. Every direct relocation becomes
one non-overlapping patch span with source-byte and encoded-byte hashes;
unresolved compatibility targets stay deferred. Preview construction proves it
did not mutate or publish the merged image, and reversed object input preserves
the plan hash.

`nuis-nsld-elf-amd64-patch-application-v1` now closes the direct-write
milestone. It independently rebuilds both the source image and materialization
preview, rejects plan/source/order/width/hash drift, and applies every direct
span exactly once in an isolated buffer. The resulting ledger binds source and
applied file/memory hashes plus each preview and write audit. Deferred
compatibility targets remain untouched, and no applied bytes are serialized or
published.

`nuis-nsld-elf-amd64-platform-structure-plan-v1` now closes the non-mutating
platform layout milestone. It consumes that applied-image ledger and deferred
relocation set, groups external PLT32 targets through a registration-owned rule
table, assigns shared nonlazy PLT/GOT slots plus deduplicated dynamic symbols,
strings, and `R_X86_64_JUMP_SLOT` records, and binds each deferred source to one
checked patch preview.

`nuis-nsld-elf-amd64-platform-patch-application-v1` now closes the platform
byte milestone. It rebuilds the exact plan and base image, reserves inherited
direct patches, emits checked nonlazy PLT/GOT, `Elf64_Sym`, dynamic string, and
`Elf64_Rela` records into an isolated extended image, and commits every deferred
source once. A deterministic ledger binds all source/encoded/post-write hashes,
unresolved dynamic-bind records, and final file/memory hashes.

`nuis-nsld-elf-amd64-shell-layout-plan-v1` now closes the non-mutating shell
coordinate milestone. It rebuilds placement and relocation plans, validates the
platform application ledger and image hashes, maps base and platform RX/RW/RO
regions into checked `PT_LOAD` records, emits `PT_PHDR` plus optional
`PT_DYNAMIC` coordinates, plans section names/headers and dynamic tags, and
selects a registered source entry inside a file-backed executable segment.
Every section, program header, and dynamic entry audit includes the upstream
application ledger. Static and external-boundary plans are deterministic under
reversed object order; ledger drift and missing entry definitions fail closed.

`nuis-nsld-elf-amd64-shell-image-serialization-v1` now closes the isolated byte
image milestone. It rebuilds the exact upstream plan, copies the verified
platform file image, encodes ELF64/program/dynamic/section-name/section-header
tables at planned coordinates, and permits shell writes only in non-overlapping
zero-reserved spans. Every write binds source, encoded, and post-write hashes;
file-backed source sections remain byte-identical, `SHT_NOBITS` sources remain
zero-fill, and one canonical ledger binds the final private image. Static and
external-boundary images are deterministic, but remain unpublished.

`nuis-nsld-elf-amd64-shell-image-validation-v1` now closes the independent
private-image boundary. A parser separate from the encoder discovers and checks
the ELF header, program/dynamic/name/section tables, source spans, write audits,
unexplained platform-prefix changes, and final ledger. Both static and external
images remain unpublished. The next native milestone is an explicit bounded,
validation-ledger-bound OS-loader probe for static closure only; registered
external interpreter and dependency provenance remain separate. Nuisc remains
free of ELF branches.

## Validation

```sh
CARGO_INCREMENTAL=0 cargo test -q -j 1 -p nsld executable_finalizer
CARGO_INCREMENTAL=0 cargo test -q -j 1 -p nsld final_executable_elf_artifact
CARGO_INCREMENTAL=0 cargo test -q -j 1 -p nsld final_executable_macho_input
CARGO_INCREMENTAL=0 cargo test -q -j 1 -p nsld final_executable_host
CARGO_INCREMENTAL=0 cargo test -q -j 1 -p nsld --test host_finalizer_cli
```
