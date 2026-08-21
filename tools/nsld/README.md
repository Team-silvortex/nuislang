# Nsld

`nsld` is the Nuis linker front-door.

In the current `beta-0.3.*` line it is still a CLI wrapper over repository-owned
linker contract logic, including `nuisc::linker` helpers. That is intentional:
the tool exists before the final self-owned linker core so the toolchain can
start exercising linker plans, clock ordering, section/container metadata, and
heterogeneous binary contracts early.

## Boundary

The long-term shape is:

```text
nsld core capability -> CLI adapter
```

The CLI should never become the only durable protocol. New behavior should be
modeled as structured linker data first, then rendered for terminal output.

Nsld's native linker contract is not required to be a traditional `.o`-first
pipeline. Object files, Mach-O, ELF, PE/COFF, and host-native executable formats
belong to compatibility and finalization backends. The core linker should be
able to consume Nuis-owned link graphs, lifecycle/clock metadata, section
manifests, heterogeneous payloads, and container metadata directly, then choose
whether to emit a Nuis container, a host-native wrapper, or a compatibility
object/executable format.

The same rule applies to the larger C world: C ABI, libc, native object files,
and the classic von-Neumann host stack should be modeled as a CFFI /
host-compat capability domain inside Nuis, not as the implicit substrate that
defines all linker semantics.

## Core Responsibilities

Future `nsld-core` or equivalent galaxy-style capability should own:

* deterministic link graphs and link-unit registration
* lifecycle hook ordering and global clock metadata
* deterministic section and data-segment layout
* object-plan target identity metadata for optional platform compatibility bytes
* native object output verification when a compatibility object is emitted
* unified heterogeneous container metadata
* final executable layout and internal image dry-run metadata
* lifecycle-scoped native-object lanes for CFFI/host compatibility payloads
* static C-world compatibility wrapper policy
* host-compat domain metadata that keeps C/von-Neumann execution explicit,
  scheduled, and verifiable

Current `prepare`, `check`, `closure`, `container`, and `verify-container`
reports expose this host-compat domain metadata as compatibility-domain
summary fields. JSON output keeps legacy flat fields for compatibility and also
provides object-shaped summaries such as `compatibility_domain_summary`,
`container_compatibility_domain_summary`, and verify-container
expected/actual summaries. Treat those fields as linker protocol, not as
cosmetic CLI output.

The CLI should remain a human and script entry point for those capabilities.

Current early-beta pressure is not to special-case Metal, CoreML, CUDA, or
other providers in the linker. Nsld should consume registered heterogeneous
payload and execution-capsule metadata after the worker boundary verifies it,
then place those records according to the existing lifecycle, clock, and
deterministic data-layout contracts.

## Current Module Map

The current final executable pipeline is split by protocol boundary rather than
by terminal command formatting:

* `final_stage.rs` owns only the final-stage plan report, emitter, and verifier.
* `final_executable_summary.rs` owns readiness and writer-plan summaries.
* `final_executable_writer_input.rs` owns writer input emission and verification.
* `final_executable_finalizer_registry.rs` owns canonical target selection,
  static finalizer registration, registry hashing, command planning, and the
  provider execution callback.
* `final_executable_macho_artifact.rs` validates and atomically materializes
  thin or universal arm64 Mach-O images embedded in compiled artifacts.
* `final_executable_macho_object.rs` validates the relocatable program/runtime
  object handoff, including roles, LinkPlan hashes, `MH_OBJECT`, and load
  command structure.
* `final_executable_macho_layout.rs`, `final_executable_macho_relocation.rs`,
  `final_executable_macho_materialization.rs`, and
  `final_executable_macho_application.rs` own placement through direct writes.
* `final_executable_macho_platform.rs` and
  `final_executable_macho_platform_application.rs` own registered stub/GOT
  allocation, platform writes, and unresolved bind preservation.
* `final_executable_macho_shell.rs` coordinates the audited shell plan while
  its layout and linkedit modules own final addresses and metadata requirements.
* `final_executable_host.rs` owns host finalizer dry-run and invoke-plan gates.
* `final_executable_layout_stage.rs` owns the Nsld final executable layout plan.
* `final_executable_image_stage.rs` owns the `NUIFIMG` dry-run image checkpoint.
* `final_executable_emit.rs` owns blocked final-executable emission and verify.
* `content_hash_cache.rs` bounds repeated file-hash work during one linker
  process without retaining artifact bytes.
* `final_executable_paths.rs`, `final_executable_render.rs`,
  `final_executable_verify_helpers.rs`, `final_executable_layout.rs`, and
  `final_executable_image.rs` hold shared path/render/verify/layout/image
  helpers.

Keep future linker execution work behind the same stage boundaries so Mach-O,
ELF, PE/COFF, and future Nuis-native writers can evolve without coupling the
front-door plan to one backend.

The current `nuis-nsld-executable-finalizer-registry-v1` route selects an
internal Mach-O arm64 artifact-image provider for `native-cpu-llvm`. Nuisc
hands it separate LLVM program and runtime shim objects through the compiled
artifact; Nsld verifies their roles, hashes, and Mach-O object structure before
parsing their sections, symbol/string tables, and ARM64 relocation records.
The provider projects `nuis-nsld-macho-host-object-linkage-v1`, including
internal cross-object resolutions and explicit unresolved C/system symbols,
then derives `nuis-nsld-macho-placement-binding-v3`. The latter
deterministically merges compatible sections, applies checked alignment,
assigns contribution offsets, binds section-backed cross-object symbols, and
coalesces tentative common definitions by maximum size/alignment into the
reserved provider-owned `__DATA,__nuis_common` zero-fill region. Strong
definitions override common declarations; duplicate strong definitions,
incompatible flags, reserved-section claims, missing indirect targets, and
alias cycles fail closed. Absolute values remain explicit coordinates and
indirect aliases resolve to a terminal section or absolute definition.
Unresolved C/system symbols remain an explicit compatibility boundary. The nested
`nuis-nsld-macho-arm64-relocation-application-v1` plan then maps each checked
relocation to its placed source and target offsets. Its static registry covers
the eight ARM64 relocation kinds emitted by the real Nuisc objects, preserves
ADDEND and SUBTRACTOR pair identity, distinguishes direct writes from
GOT/external platform structures, and fails closed on malformed or unknown
kinds. JSON, text, and persisted invoke-plan surfaces expose the same
placement-bound plan hash and records before the provider atomically
materializes the embedded compatibility executable without a second Nsld-side
clang invocation.
`nuis-nsld-macho-arm64-materialization-preview-v1` now copies every verified
section contribution into a deterministic provider-owned merged image, keeps
alignment gaps and zero-fill bytes explicit, hashes each merged section and the
complete source image, and previews every direct ARM64 write without mutating
that image. `UNSIGNED`/`SUBTRACTOR`, `BRANCH26`, and
`PAGE21`/`PAGEOFF12` with paired `ADDEND` records use checked instruction,
alignment, displacement, and range encoding. Each preview exposes source and
encoded bytes plus independent byte and audit hashes; GOT and unresolved
external records remain explicit deferred platform-structure work.
`nuis-nsld-macho-arm64-patch-application-v1` then reconstructs the source image
from placement inputs and accepts only the audited preview contract. It verifies
the placement, relocation, image, source-byte, encoded-byte, and preview-audit
hashes; rejects duplicate or overlapping spans and source drift; writes every
direct patch exactly once; and emits a stable applied-image hash plus per-patch
write audits and an application-ledger hash. JSON, text, and persisted invoke
plans project the same application report. This working image is not mislabeled
as an executable while deferred platform structures still exist.
`nuis-nsld-macho-arm64-platform-structure-plan-v1` consumes the applied-image
ledger and deferred relocation records through a provider-owned rule registry.
It deduplicates structured target identities, maps GOT loads to shared GOT
entries and external branches to stub-plus-GOT targets, assigns checked
working-image-relative offsets with 12-byte/4-aligned stub and 8-byte/8-aligned
GOT policies, and hashes the registry, layout, targets, bindings, applied image,
and ledger. Unsupported deferred forms fail closed rather than receiving a
guessed structure. The report is identical across JSON, text, and persisted
invoke plans.
`nuis-nsld-macho-arm64-platform-patch-application-v1` now consumes that exact
plan, extends the direct-patched image to the checked span, emits standard
`ADRP x16`/`LDR x16`/`BR x16` 12-byte stubs and 8-byte GOT entries, and rewrites
every deferred `BRANCH26` or GOT page/pageoff relocation exactly once. Internal
GOT entries contain audited image-relative targets; unresolved external entries
remain zero placeholders with explicit symbol-bound bind records. Inherited
direct spans, structure writes, and deferred patches share a fail-closed
write-once occupancy map. Plan drift, source drift, malformed instructions,
overlap, and incomplete coverage block publication. The resulting image and
ordered write/patch/bind ledgers are projected identically through JSON, text,
and persisted invoke plans.
`nuis-nsld-macho-arm64-shell-layout-plan-v1` now translates that exact
platform ledger into a deterministic 16 KiB-page Mach-O shell plan. A static
entry registry selects `_main`, `_nuis_entry`, or `_nuis_yir_entry` by object
role; merged sections plus provider-created `__stubs` and `__got` regions are
remapped into checked `__PAGEZERO`, `__TEXT`, `__DATA_CONST`, and `__LINKEDIT`
segments; and section-backed definitions, unresolved symbols, indirect-symbol
slots, rebase/bind stream requirements, symbol/string tables, and ordered load
commands receive final file and VM addresses. External binds use the static
libSystem compatibility registration rather than symbol-specific linker
branches; the executable shell always carries that platform baseline even when
the private program has no external binds. Every segment, section, symbol,
indirect entry, bind, rebase, and command has an audit hash. The provider also
derives a deterministic `LC_UUID` from the shell-plan identity and records an
explicit pending code-signature boundary.
`nuis-nsld-macho-arm64-shell-image-serialization-v2` now consumes that exact
plan and platform ledger. It emits a private `mach_header_64`, ordered load
commands, copied file-backed sections, dyld rebase/bind streams, `nlist_64`
records, indirect-symbol entries, and the UTF-8 string table. Direct and
platform relocations, provider stubs, and internal GOT pointers are re-encoded
against final VM addresses with per-write source/prewrite/output hashes and a
complete serialization ledger. Internal-rebase and external-bind fixtures both
pass, and JSON, text, and persisted invoke-plan TOML expose the same report.
`nuis-nsld-macho-arm64-ad-hoc-signature-v1` appends a standard SuperBlob and
CodeDirectory with SHA-256 hashes over every 4 KiB code slot, extends
`__LINKEDIT`, and gives `LC_CODE_SIGNATURE` its final non-zero payload size.
`nuis-nsld-macho-arm64-signed-image-validation-v1` then independently reparses
the header, every load-command boundary, deterministic UUID, signature
envelope, reserved fields, padding, and every signed range. Signed-content,
command-boundary, UUID, and padding mutations all fail closed. The base
`nuis-nsld-macho-arm64-publication-eligibility-v1` report deliberately remains
private and pending until runtime evidence is supplied.
`nuis-nsld-macho-arm64-os-loader-probe-v1` provides that evidence without
publishing the image. Its default mode is plan-only; explicit `--apply`
materializes an owner-only temporary executable only for a signed,
zero-unresolved, zero-external-bind image, verifies the exact bytes, invokes the
host kernel under bounded time and output limits, records process and cleanup
results without leaking the temporary path, and removes all probe files. The
fully internal ARM64 fixture is accepted by the real macOS kernel and dyld,
exits zero with no output, and earns probe-local publication eligibility. The
real external-compatibility CLI fixture remains blocked before materialization.
Successful apply persists
`nuis-nsld-macho-arm64-publication-admission-v1` at the stable relative file
`nuis.nsld.macho-arm64-publication-admission.toml`. The canonical strict
flat-TOML receipt binds the finalizer registry and target, private-image FNV and
SHA-256 identities, signature ledger and cdhash, zero-bind closure, and complete
probe/cleanup evidence under its own SHA-256 digest. It is written through an
owner-only create-new temporary file, sync, atomic rename, and directory sync.
`verify-final-executable-private-image-admission` independently reparses that
receipt, rejects missing, duplicate, unknown, noncanonical, or hash-drifted
fields, rebuilds the current private product, and replays registry, target,
image, signature, and probe identities. A real ordinary compiled-artifact
fixture passes; receipt tamper and rebuilt-artifact drift fail closed.
`final-executable-private-image-publication` then asks only the selected
finalizer registry entry for its optional private-image publication capability.
The default plan is non-mutating. Explicit `--apply` requires a freshly valid
receipt, writes the same verified bytes to a create-new owner-executable
temporary file, syncs and rereads them, atomically renames the file, syncs the
directory, and verifies final size, SHA-256 identity, and executable mode. An
invalid receipt leaves the compatibility image unchanged; a valid real fixture
publishes and executes directly through macOS with exit code 0.
The ordinary `final-executable-output` boundary now owns a separate
`nuis-nsld-final-output-selection-registry-v1`. With no policy it selects the
existing compatibility output without mutation. An explicit
`--output-policy admitted-private-image` request is plan-only unless paired with
`--apply`; application reuses the selected finalizer callback and therefore
cannot bypass receipt replay. The nested
`nuis-nsld-final-output-selection-evidence-v1` record binds the registry hash,
provider capability, receipt and verification identities, publication ledger,
candidate SHA-256, and installed size/SHA-256/executable identity. The real
common-symbol fixture now takes this ordinary route before execution.
A gated host-command provider remains as a fallback. PE/COFF is still an
explicit `registered-not-implemented` provider; the first registered ELF AMD64
provider now validates and publishes a compatibility image while independently
parsing its two `ET_REL` inputs and deriving
`nuis-nsld-elf-amd64-placement-binding-v1`. That canonical plan groups
allocatable text, read-only data, data, zero-fill, and common storage into
page-separated permission classes; assigns checked file/image/virtual
coordinates; coalesces commons; preserves absolute values; and binds internal
symbols, zero-valued unmatched weak references, or compatibility-external
symbols without mutating bytes.
`nuis-nsld-elf-amd64-relocation-application-v1` consumes that exact plan hash,
maps registered `R_X86_64_NONE/64/PC32/PLT32/32/32S` records to placed sources
and bound targets, checks `S+A` or `S+A-P` against the relocation width, and
emits deterministic little-endian previews. Overflow and placement drift fail
before mutation; unresolved system targets remain platform-structure work.
`nuis-nsld-elf-amd64-materialization-preview-v1` reparses the hash- and
size-bound `ET_REL` inputs, copies each file-backed placed section into one
deterministic merged memory image, audits zero-fill ranges separately, and
binds source-object, file-image, memory-image, placement, and relocation hashes
into one canonical report. Direct relocations become non-overlapping
write-once patch spans with source-byte and encoded-byte hashes; unresolved
system targets remain deferred. This stage deliberately previews writes and
does not mutate or publish the image.
`nuis-nsld-elf-amd64-patch-application-v1` independently rebuilds the exact
source image and preview, rejects any plan, source, order, width, or byte-hash
drift, and writes every direct relocation once into a separate buffer. Its
ledger binds source and applied file/memory hashes, every write span, and every
preview/write audit. Deferred compatibility relocations remain unchanged, and
the applied image is not yet passed to a shell serializer or publisher.
`nuis-nsld-elf-amd64-platform-structure-plan-v1` validates that ledger before
using a static provider rule registry to group external PLT32 calls. It assigns
shared 16-byte nonlazy PLT entries, 8-byte GOT slots, deduplicated dynamic
symbol/string records, and `R_X86_64_JUMP_SLOT` `Elf64_Rela` records across
page-separated RX, RW, and RO regions. Each deferred source is bound to exactly
one checked i32 patch preview; unsupported shapes, overlapping sources,
ambiguous registration, and ledger drift fail before any platform byte write.
The Mach-O route proves relocatable input,
table parsing, placement,
binding, merged-image construction, direct and
platform relocation encoding, deterministic GOT/stub allocation, platform byte
synthesis, unresolved-bind preservation, audited Mach-O shell planning, and
signed private final-address byte serialization with an independent structural
validator, one real isolated OS-loader execution, durable replay admission, and
registered opt-in publication and ordinary explicit selection. The real internal fixture now executes
`ADRP`/`ADD`/`STR` against provider-allocated common storage before passing
signature, loader, receipt replay, ordinary final-output selection, and direct
execution. ELF platform byte application, shell serialization, load admission,
and PE/COFF remain incomplete. The
ordinary default compatibility executable remains byte-for-byte unchanged;
private installation requires both the explicit policy and `--apply`.

## Current Early-Beta Rule

Do not add new linker semantics only as formatted command output. If a command
needs to expose new information, add or preserve a structured representation so
`nuisc`, future IDE tools, `yalivia`, Nuis OS surfaces, and test/benchmark
flows can consume the same contract without shelling through text.
