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
  -> final executable readiness / blocked report
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
* payload hash and container hash
* verification of metadata and payload consistency
* deterministic final-stage plan before executable finalization
* deterministic blocked final-executable emission report

It does not yet own:

* ELF or PE compatibility object emission
* native relocation application
* final executable entrypoint generation
* host-shell or Nuis-native executable materialization
* Nuis lifecycle runtime bootstrapping
* heterogeneous dispatch at runtime

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

## Gap 3: Loader Bootstrap

The container records a lifecycle bootstrap symbol, but there is no finished
Nuis loader runtime yet.

The loader needs:

* one native entrypoint shim
* bootstrap of the Nuis lifecycle loop
* deterministic hook execution order
* access to container metadata and payload sections
* bridge points for clock protocol and GLM checks
* failure reporting that can be consumed by Nsdb later

This is where the container stops being just inspectable and starts becoming
runnable.

## Gap 4: Heterogeneous Dispatch Bridge

Shader, kernel, network, and future Nustar domains already contribute lowering
sidecars and link units. They still need a runtime dispatch bridge.

The bridge needs:

* per-domain dispatch table materialization
* backend target selection without hardcoding finite combinations
* capability-driven sidecar loading
* deterministic clock-edge handoff
* GLM/resource validation at the boundary
* fallback or host-proxy mode when real hardware backend is unavailable

The bridge should consume Nustar registrations, not compiler hardcoded domain
logic.

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

The current next milestone is now represented by:

```text
nsld prepare
  -> nsld object-plan
  -> nsld emit-object-plan
  -> nsld verify-object-plan
  -> nsld object-byte-layout
  -> nsld object-file-layout
  -> nsld object-image-dry-run
```

That gives the compatibility object writer a deterministic planning layer
before bytes are emitted. It mirrors the existing assemble/container path and
keeps the project moving without pretending the native executable story is
finished. The current `object-plan` is intentionally non-mutating; native object
bytes and relocation application remain optional compatibility/finalization
layers rather than the mandatory internal form of Nsld.

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
`emit-object` is now wired to the first minimal compatibility object writer:
prepared Mach-O arm64 input can be emitted as optional `nuis.nsld.mach-o` from
the deterministic image bytes. Unprepared input, ELF, and COFF still report
blockers. The command also materializes diagnostic artifacts: the future byte
writer's deterministic input snapshot, the alpha emit report at
`nuis.nsld.object.blocked.toml`, and the object image dry-run report/bin pair.
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

## Success Boundary

Nsld reaches the first real binary assembly milestone when:

* the object plan is derived from the verified container state
* every object section is traceable back to an Nsld section id
* object-plan hashes are stable
* unsupported native targets fail with structured diagnostics
* no domain-specific shortcut is hardcoded into the linker frontdoor

After that, byte emission and loader bootstrap can evolve against a stable
plan instead of a moving pile of ad hoc linker code.
