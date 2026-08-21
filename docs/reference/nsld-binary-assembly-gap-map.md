# Nsld Binary Assembly Gap Map

This note maps the current gap between the Nsld-owned deterministic container
pipeline and a truly runnable Nuis-owned heterogeneous executable.

## Current Assembly Chain

Nsld already has a deterministic preparation chain:

```text
link plan
  -> link inputs
  -> link units
  -> link bundle
  -> assemble plan
  -> section manifest
  -> object plan
  -> object writer input
  -> object byte layout
  -> object file layout
  -> object image dry run
  -> container plan
  -> container metadata
  -> container payload
  -> closure snapshot
  -> final-stage plan
  -> final executable writer input
  -> final executable layout and image dry-run
  -> self-contained NSB image output or host-assisted boundary report
```

`nsld prepare` can emit and verify this chain today. This is already useful
because it gives linker, cache, release, and debugger work one reproducible
artifact boundary. The final closure snapshot records the current
`linker_contract_hash`, plus container and payload hash anchors, making later
Nsld, cache, and debugger work able to detect linker-contract or assembly-input
drift without treating the snapshot as part of its own self-verification
material.

## Current Artifact Meaning

The emitted `nuis.nsld.container` is a Nuis-owned binary-contract container,
not the final host executable.

It currently owns:

* deterministic section order
* section hashes and payload ranges
* loader-facing entry metadata
* loader symbol seeds
* relocation seeds
* external import records
* an open immutable metadata-binding table for versioned identity claims
* payload hash and container hash
* verification of metadata and payload consistency
* deterministic final-stage plan before executable finalization
* deterministic final executable writer input
* deterministic self-contained final image dry-run and image bytes
* Nsld-owned self-contained `.nsb` image output for the internal image route
* deterministic blocked/final-output boundary report for host-assisted routes
* normalized final-output materialization status and recommended next action for
  scripts that need to advance the binary assembly chain deterministically

The first immutable binding is
`identity.selected-provider-bundle-set`. Nsld accepts it only after
independently verifying the provider-sample contract, count, and hash. The
binding-table hash participates in the metadata root and each complete record
participates in the container root; disagreement blocks container emission.
The final NSB image embeds those canonical container bytes. Both the Nsld
container loader and `nuis-host-runner` read the binding table from actual
image payload bytes, independently recompute its table hash, validate required
records and selected-set structure, and block handoff when the final image is
mutated. Nuis launch evidence requires the host-runner-observed proof rather
than trusting only Nsld's handoff-ready result. Run-artifact now persists that
proof under `nuis-final-image-binding-proof-v1`; Nuis and Nsdb independently
recompute its canonical hash, Nsdb preserves it while merging later completion
records, and replay blocks on drift. The direct Nsld final-output writer now
passes the same loader-observed, provider-neutral claim through Nsdb's public
persistence API. Nsdb computes the proof, rejects a conflicting final-image
claim, and Nsld mirrors the verified status through final-output JSON/text.
Older proof-less records remain readable as `legacy-unbound`, but the public
replay summary, the concrete Nsdb replay plan and transcript, and Nuis's
independent final-output closure all reject them before execution. They expose
the deterministic migration action `rebuild-final-output-binding-proof`
instead of silently trusting pre-proof history. The remaining continuity gap
is no longer debugger identity: `nsdb-yir-replay-identity-v1` now carries the
verified proof hash through debugger transcripts, persisted cursors, resume
validation, and every cursor-lineage event. Nuis independently compares the
handoff and cursor before exposing resume readiness, while its lineage mirror
separately verifies all three identities. The official PixelMagic route now
keeps the provider-complete intermediate non-replayable, rebuilds the project
under its declared `nuis-self-contained-image` packaging mode, and calls the
provider-neutral `nsld seal` frontdoor. That command preflights the open
provider manifest and selected-set identity, then executes exactly one
`prepare`, one final executable pipeline, and one final-output publish stage.
It fails before mutation for host-finalized packaging or incomplete provider
samples. The resulting Metal/CoreML NSB upgrades the same handoff with a
loader-observed proof and proves three-frame stop, cursor persistence, resume,
and lineage against that final-image identity. Acquisition and self-contained
rebuild remain Nuis-level orchestration. The sealed container now also carries
`nuis-final-image-provider-dispatch-v1`: an open, ordered table of package,
bundle, provider family, runner contract, adapter contract, and adapter ID.
Its count and canonical hash are required through
`runtime.provider-dispatch-table`, enter the metadata root and container hash,
and are independently recomputed by the final-image loader. A same-length
adapter mutation is rejected from final NSB bytes even when mutable sidecars
remain unchanged. The container capsule has an explicit protocol terminator,
so future tables do not depend on a historical field-position boundary. The
`nuis-host-runner` now independently parses the complete table, recomputes its
dispatch and selected-set hashes, cross-checks both metadata bindings, exposes
every open entry through JSON, and blocks lifecycle handoff on drift. Nsdb has
a separate final-image parser. Pre-seal provider acquisition is explicitly
reported as `pre-seal-acquisition`; once a launcher exists, missing or damaged
NSB state cannot fall back. Before worker launch Nsdb matches sidecar
package/bundle/family and actual runner/adapter identity against the final
image, then compares the registered runtime adapter. The official CoreML/Metal
route re-enters provider execution after seal and proves every selected bundle
was authorized by that table. Request sidecars now supply payload details, not
dispatch selection. The remaining continuity gap is persisting the direct
dispatch table hash and matched entry in each completion/replay record rather
than relying only on the broader final-image binding proof.

The Mach-O arm64 compatibility provider now also owns a private final-address
shell image. `nuis-nsld-macho-arm64-shell-image-serialization-v2` emits the
header, load commands, copied sections, dyld streams, symbol/indirect/string
tables, and audited relocation/stub/internal-GOT rewrites from the deterministic
shell plan. It also appends a SHA-256 ad-hoc SuperBlob/CodeDirectory, extends
`__LINKEDIT`, and independently validates every load-command and signed-range
boundary, including the provider-derived `LC_UUID`.
`nuis-nsld-macho-arm64-os-loader-probe-v1` now exercises that exact signed image
without publishing it. The plan-only default is side-effect free; explicit
application admits only a zero-unresolved/zero-bind input, verifies temporary
materialization, bounds execution and output, records cleanup, and exposes no
temporary path. A fully internal ARM64 fixture is accepted by the real macOS
kernel and dyld and exits zero. Explicit success now persists a canonical,
SHA-256-bound `nuis-nsld-macho-arm64-publication-admission-v1` receipt. An
independent verifier rebuilds and checks registry, target, image, signature,
probe, cleanup, and zero-bind identities; receipt tamper and regenerated-
artifact drift fail closed in the real CLI fixture. A registry-declared,
provider-neutral publication frontdoor now stays plan-only by default and, on
explicit apply, atomically installs those exact admitted bytes. The resulting
private Mach-O executes through macOS and exits 0. Invalid admission does not
touch the compatibility output, so the private image is never silently
substituted.

`nuis-nsld-macho-placement-binding-v3` owns tentative common storage plus
non-section definition resolution. It coalesces declarations by symbol name,
honors strong-definition precedence, emits canonical `__DATA,__nuis_common`
allocations, preserves `N_ABS` values, and resolves multi-hop `N_INDR` aliases.
Cycles and missing targets fail before byte mutation. Direct and GOT
relocations, shell symbols, and final-address rewriting consume the same
image-offset-or-absolute evidence. The real published fixture includes an alias
and absolute symbol, executes `ADRP`/`ADD`/`STR` against VM-only common storage,
and exits zero while ordinary compatibility output remains unchanged by
default.

`nuis-nsld-final-output-selection-registry-v1` now makes that private product an
explicit ordinary final-output option rather than a separate architecture path.
`compatibility-default` remains non-mutating. `admitted-private-image` delegates
through the selected finalizer only after receipt replay, stays plan-only
without `--apply`, and publishes the exact candidate under an installed-output
SHA-256 identity. The final report binds registry, policy, receipt, publication,
candidate, and installed-image evidence under
`nuis-nsld-final-output-selection-evidence-v1`. A damaged receipt cannot start
installation, while the valid common-symbol fixture selects and executes the
private Mach-O through the ordinary command.

The finalizer registry now also owns one narrow Linux route:
`nsld.finalizer.elf.amd64.artifact-image-v1`. It validates exact LinkPlan and
compiled-artifact target identity, both hash-bound ELF64 `ET_REL` host objects,
their bounded section/name/symbol tables, registered explicit-addend
`R_X86_64` relocations, cross-object strong-definition uniqueness and internal
symbol closure. `nuis-nsld-elf-amd64-placement-binding-v1` now groups
allocatable program/runtime contributions into deterministic text, read-only
data, data, zero-fill, and common page-separated permission classes; assigns
checked file/image/virtual coordinates; coalesces common declarations;
preserves absolute values; maps unmatched weak references to zero; and binds
internal references under one canonical plan hash. The provider also
validates an ELF64 `ET_EXEC`/PIE image whose entry belongs to a bounded
executable `PT_LOAD`. The accepted compatibility image is atomically published
without a Clang/LLD process. This is deliberate staging evidence, not a
self-owned ELF link claim: the embedded executable remains host-toolchain-linked
until Nsld consumes its owned placement plan for relocation application and
shell emission.

It does not yet own:

* complete architecture parity across Mach-O, ELF, and PE/COFF
* provider-owned ELF relocation application/shell serialization and complete
  PE/COFF object merging
* a durable embedded Nsdb/YIR debug metadata section

## Gap 1: Compatibility Object Writer

Nsld needs a self-owned binary assembly layer. A traditional object writer can
be one backend of that layer, but it is not required to be the core
representation. Nuis-native linking can consume structured link units, section
manifests, clock/lifecycle metadata, GLM-compatible ownership metadata, and
heterogeneous payloads directly.

Minimum first target:

* consume the section manifest and payload
* optionally write a host object container for one platform
* preserve Nsld section identity in object metadata where possible
* keep Nsld hashes as source-of-truth verification metadata
* avoid special C-world shortcuts outside the external import contract

This layer should be deterministic and testable before it claims to replace the
host finalizer.

## Gap 2: Relocation Applier

The current relocation table is a loader-facing seed, not a native relocation
engine.

The next relocation layer needs:

* relocation kind registry
* section-relative offset model
* symbol resolution over loader symbols and hetero dispatch symbols
* deterministic unresolved-symbol diagnostics
* host ABI-specific lowering only behind the registered target profile

This should stay below the YIR contract and above the final platform object
format.

## Closed Foundation: Loader Bootstrap

The final NSB now carries architecture-specific native entry thunks, a
hash-bound lifecycle context, one-shot invocation authorization, deterministic
hook order, and independently verified host-runner/Nsdb evidence. Linux and
Windows execution remain cross-host validation lanes, but loader bootstrap is
no longer an unimplemented architectural gap.

## Closed Foundation: Heterogeneous Dispatch Bridge

Registered provider bundles now materialize an open final-image dispatch table,
bind provider selection into the NSB identity, execute through capability-owned
worker adapters, and preserve clock, GLM, completion, and replay evidence.
Backend breadth remains ongoing work, but the provider-neutral bridge itself is
an established runtime slice rather than a missing foundation.

## Gap 5: Debug Metadata Section

Nsdb can inspect YIR metadata through the current manifest/link-plan route, but
the final executable needs a durable debug metadata section.

Minimum shape:

* YIR domain index
* clock edge table
* section-to-YIR mapping
* loader symbol map
* lowering sidecar references or embedded summaries
* GLM/debug state handles when ready

Native debuggers may still see the shell binary. Nsdb should own the Nuis
semantic view.

## Practical Next Milestone

The next useful milestone is not "replace the system linker immediately".

Nsld can already emit a self-contained Nuis image for the internal image route.
That route is selected by `packaging_mode = "nuis-self-contained-image"` and is
a real Nsld-owned final-output boundary, but it is not yet the same as a
host-shell executable or an OS-native entrypoint.

The current next milestone is now represented by:

```text
nsld prepare / drive
  -> object image dry-run
  -> container + payload
  -> closure snapshot
  -> final executable layout
  -> self-contained NSB image output
  -> launcher manifest / dry-run
```

That gives the self-owned image route a deterministic final-output layer before
the host-shell and OS-native entrypoint layers are finished. It keeps the
project moving without pretending the native executable story is complete. The
current `object-plan` and object image dry-run remain compatibility planning
layers; native object bytes and relocation application are optional
compatibility/finalization layers rather than the mandatory internal form of
Nsld.

The final executable pipeline now carries a normalized
`self_owned_image_status` field and a separate
`entrypoint_materialization_status` field. For the self-contained internal image
route, `self_owned_image_status = ready` means the `.nsb` image layer is present,
hash-visible, and header-valid. `entrypoint_materialization_status` then says
whether the next entrypoint layer is `host-launcher-ready`,
`image-ready-entrypoint-pending`, or `blocked`. This keeps host-shell and
OS-native entrypoint work separate from the internal binary assembly layer.
The generated host-shell entrypoint identifies itself with
`NUIS_HOST_ENTRYPOINT_STUB_PROTOCOL=nuis-nsld-host-entrypoint-v1` before
delegating to `NUIS_HOST_RUNNER`, giving nsdb, nsbdr, and future runners a
stable protocol hook without baking runner implementation details into Nsld.

The plan already assigns each Nsld section a writer-facing object section
record with a stable object section name, object section role, source section
id, source hash, source size, alignment, payload offset seed, file offset seed,
and file size seed. The future byte writer should consume that mapping instead
of rediscovering object layout from the section manifest.
It also emits `[[object_relocation_seed]]` records, which are Nsld-owned
relocation intent seeds and not yet native Mach-O, ELF, PE, shader, or kernel
relocation records.
The plan also exposes a writer summary with `writer_target_id`,
`writer_backend_kind`, `object_family`, `writer_status`, and
`unsupported_features`, so future byte-emission commands can distinguish
"target known, writer blocked" from "target unknown" without hardcoding one
platform family into the linker frontdoor.
`verify-object-plan` now validates required object-section and relocation-seed
fields plus semantic drift in both tables.
`object-writer-readiness` exposes the same information as a command-level
readiness gate before `emit-object` attempts compatibility byte emission.
`emit-object` is wired to the first compatibility object writers: prepared
Mach-O arm64 and ELF AMD64 inputs can be emitted from deterministic image bytes.
Unprepared input, ELF AArch64, and COFF still report blockers. The command also
materializes diagnostic artifacts: the future byte
writer's deterministic input snapshot, the alpha emit report at
`nuis.nsld.object.blocked.toml`, and the object image dry-run report/bin pair.
Artifact-chain and drive recommendations use the `emit-native-object` alias for
this lane so native-object stages read naturally while preserving the same
deterministic object-output contract.
`verify-object-emit` checks that
those artifacts still agree on the object plan hash and dry-run image hash.
`verify-object-output` checks the emitted native object bytes themselves by
comparing the object output path, currently `nuis.nsld.mach-o`, against
`nuis.nsld.object-image-dry-run.bin` by size and content hash. `nsld check`
additionally runs that verification when the object output is present, and
`nsld closure` can surface it as `verified-object-output`.
Container planning also uses this validation as the native-object admission
gate: an invalid object output becomes an `object-output:*` blocker instead of
being repackaged as a `native-object-output` section.
When the object is admitted, the Nsld container now also emits a
`[[compatibility_domain]]` metadata entry for the CFFI / host-compat execution
domain. That entry records the compatibility domain id, domain kind, classic
von-Neumann host paradigm, lifecycle hook, ABI family, wrapper policy, and
required flag, and its table hash participates in the container metadata hash.
This object lane is intentionally optional. A future self-owned Nsld linker may
emit a Nuis heterogeneous container or a host-native executable wrapper without
round-tripping every internal unit through `.o`.
For CFFI specifically, this native-object output can become a dedicated Nustar
artifact lane inside the Nuis binary format: a compatibility payload admitted
by hash, scheduled through explicit lifecycle hooks such as
`on_cffi_native_object`, constrained by the CFFI signature whitelist, and
wrapped by Nuis-owned memory/ownership metadata rather than being treated as an
arbitrary native side call.
`verify-object-writer-input` closes that snapshot loop by validating the writer
input hashes, section and relocation-seed counts, and required writer table
field types before a future byte writer consumes it.
`object-writer-dry-run` then gives the future byte writer's preflight view:
planned object path, writer input validity, consumed section/relocation counts,
and blockers, still without writing platform object bytes.
`emit-object-writer-dry-run` materializes that preflight view as
`nuis.nsld.object-writer-dry-run.toml`, and `verify-object-writer-dry-run`
keeps it locked to the current object plan and writer input snapshot.
`object-byte-layout` adds the next deterministic layer: byte offsets, byte
sizes, alignment, total byte span, and `byte_layout_hash`, materialized as
`nuis.nsld.object-byte-layout.toml` before native object bytes exist.
It carries the same `writer_target_id`, `writer_backend_kind`, `object_family`,
and `object_format` identity from `object-plan`, and includes those fields in
the byte-layout hash so backend-family changes cannot accidentally reuse stale
layout cache entries.
`object-file-layout` continues that identity into writer-family-specific file
records, including the file-layout hash, while still keeping Mach-O/ELF/COFF
families behind registered writer metadata rather than ad hoc linker branches.
`object-image-dry-run` then preserves `writer_backend_kind` and `object_family`
alongside its image-backend status fields, and verification rejects identity
drift before any future real object writer treats the dry-run image as an
emission input.
For the Mach-O arm64 backend, the image encoder now writes readable Nsld
section source bytes into the corresponding `section-payload` file-layout
records. Missing source files still remain upstream readiness blockers rather
than causing the dry-run image encoder to invent payload content.
Mach-O section headers also now point at their deterministic relocation table
slots with `reloff` / `nreloc`, so relocation seeds are visible through normal
object-file section metadata instead of existing only as a detached Nsld table.
The Mach-O relocation encoder resolves each relocation seed's source section id
to the matching section symbol table index, instead of deriving the symbol index
only from seed order. This is still not a complete native relocation applier,
but it moves the object image closer to ordinary linker-visible semantics.
If a relocation seed cannot resolve its source section to a Mach-O section
symbol, the object-image backend now reports a structured
`mach-o-relocation:*:unresolved-section-symbol:*` blocker instead of silently
treating symbol index `0` as acceptable ready output.
Mach-O arm64 also has the first seed-kind lowering registry: current
bootstrap, metadata, data, and extension address seeds lower to conservative
external pointer-sized unsigned relocations, while unknown seed kinds are
reported as `mach-o-relocation:*:unsupported-seed-kind:*` blockers.
`object-image-dry-run` reports this as structured metadata via
`relocation_lowering_valid`, `relocation_lowering_rule_count`, and
`relocation_lowering_issues`, so future linker gates do not need to infer
relocation health from raw bytes.
It also emits a machine-readable `[[relocation_lowering_rule]]` table and JSON
`relocation_lowering_rules` array with source seed kind, target relocation kind,
PC-relative mode, length power, external flag, and native relocation type.
The dry-run report now also emits `relocation_record_count`,
`relocation_record_table_hash`, `[[relocation_record]]`, and JSON
`relocation_records`, capturing the actual backend relocation records derived
from the seeds: source section, source offset, seed id, seed kind, target
relocation kind, native symbol index, and encoded relocation flags. This gives
Nsld, nsdb, and later link metadata passes a structured, hashable audit surface
instead of forcing them to decode raw Mach-O bytes.
`verify-object-image-dry-run` also checks those fields directly, so relocation
lowering drift is reported as a focused mismatch instead of only as a whole-file
content change.
The verify step now parses and compares each relocation lowering rule entry as
well, so a rule can drift while keeping the same count and still produce a
field-level diagnostic such as
`relocation_lowering_rule[0].target_relocation_kind mismatch`.
It also compares relocation records field-by-field; for example, a changed
symbol index is reported as `relocation_record[0].symbol_index mismatch`.

The final-image dispatch handoff now continues past loader verification.
Post-seal Nsdb execution writes
`nuis-provider-completion-dispatch-authority-v1` into each completion, binding
the immutable dispatch table, selected set, matched entry, and actual runner
adapter. Completion-set hashes cover that authority. Nsdb and Nuis then
independently carry the aggregate identity into replay transcripts and
`nsdb-yir-replay-cursor-record-v2`; a cursor from another dispatch table is
rejected even when its frame IDs and manifest still match. Cursor lineage v2
and repair journal v6 now retain that identity across bounded generations,
repair events, prefix rotation, and the canonical window hash. Nsdb and Nuis
independently reject transplanted ancestry even when the broader final-image
proof still matches. Nuis final-output text/JSON and closure JSON expose the
identity accepted by the lineage mirror without rerunning authority selection.

## Success Boundary

Nsld reaches the first real binary assembly milestone when:

* the object plan is derived from the verified container state
* every object section is traceable back to an Nsld section id
* object-plan hashes are stable
* unsupported native targets fail with structured diagnostics
* no domain-specific shortcut is hardcoded into the linker frontdoor

After that, byte emission and loader bootstrap can evolve against a stable
plan instead of a moving pile of ad hoc linker code.
