# Nsld

`nsld` is the Nuis linker front-door.

In the current `beta-0.1.0` line it is still a CLI wrapper over repository-owned
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
then derives `nuis-nsld-macho-placement-binding-v1`. The latter
deterministically merges compatible sections, applies checked alignment,
assigns contribution offsets, binds section-backed cross-object symbols, and
rejects duplicate definitions, incompatible flags, and referenced definitions
without a section placement. Unresolved C/system symbols remain an explicit
compatibility boundary. The nested
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
A gated host-command provider remains as a fallback; ELF and PE/COFF are
explicit `registered-not-implemented` providers. This proves relocatable input,
table parsing, placement, binding, merged-image construction, and complete
direct relocation encoding previews, but not committed ARM64 byte writes,
GOT/stub synthesis, or complete Mach-O shell emission independently of Nuisc's
compatibility link.

## Current Early-Beta Rule

Do not add new linker semantics only as formatted command output. If a command
needs to expose new information, add or preserve a structured representation so
`nuisc`, future IDE tools, `yalivia`, Nuis OS surfaces, and test/benchmark
flows can consume the same contract without shelling through text.
