# Nsld Linker Frontdoor

`Nsld` is the Nuis linker toolchain member introduced on the `alpha-0.6.0`
line.

At this stage, `Nsld` is intentionally a frontdoor over the existing linker
contract logic in `nuisc::linker`. It does not yet claim to be the final
self-owned object linker. Its job is to give linker work a stable tool
boundary before the implementation is split out further.

Longer-term, `Nsld` should be read as a CLI adapter over a future reusable
linker core / galaxy capability boundary, not as a CLI-only tool. See
[toolchain-galaxy-core-boundary.md](toolchain-galaxy-core-boundary.md).
For the current gap between the deterministic Nsld container and a runnable
Nuis-owned heterogeneous executable, see
[nsld-binary-assembly-gap-map.md](nsld-binary-assembly-gap-map.md).

## Current Role

`Nsld` currently owns:

* link-plan inspection from `nuis.build.manifest.toml`
* heterogeneous calculate plan visibility
* clock protocol visibility
* lowering sidecar capability validation for domains that declare IR sidecars
* deterministic link-unit reporting across registered domain units
* final-stage reporting
* the first independent CLI boundary for future linker work

`Nsld` does not yet own:

* final native object linking
* replacement of the host toolchain wrapper
* binary section assembly independent from `nuisc`
* stable linker script or relocation formats
* finished `nsld-core` galaxy-style API for direct compiler/runtime consumers

## Commands

```sh
cargo run -p nsld -- status
cargo run -p nsld -- plan <nuis.build.manifest.toml>
cargo run -p nsld -- plan <artifact-output-dir> --json
cargo run -p nsld -- check <artifact-output-dir>
cargo run -p nsld -- check <artifact-output-dir> --json
cargo run -p nsld -- closure <artifact-output-dir>
cargo run -p nsld -- closure <artifact-output-dir> --json
cargo run -p nsld -- prepare <artifact-output-dir>
cargo run -p nsld -- prepare <artifact-output-dir> --json
cargo run -p nsld -- assemble-plan <artifact-output-dir>
cargo run -p nsld -- assemble-plan <artifact-output-dir> --json
cargo run -p nsld -- emit-assemble-plan <artifact-output-dir>
cargo run -p nsld -- emit-assemble-plan <artifact-output-dir> --json
cargo run -p nsld -- verify-assemble-plan <artifact-output-dir>
cargo run -p nsld -- verify-assemble-plan <artifact-output-dir> --json
cargo run -p nsld -- section-manifest <artifact-output-dir>
cargo run -p nsld -- section-manifest <artifact-output-dir> --json
cargo run -p nsld -- emit-section-manifest <artifact-output-dir>
cargo run -p nsld -- emit-section-manifest <artifact-output-dir> --json
cargo run -p nsld -- verify-section-manifest <artifact-output-dir>
cargo run -p nsld -- verify-section-manifest <artifact-output-dir> --json
cargo run -p nsld -- object-plan <artifact-output-dir>
cargo run -p nsld -- object-plan <artifact-output-dir> --json
cargo run -p nsld -- emit-object-plan <artifact-output-dir>
cargo run -p nsld -- emit-object-plan <artifact-output-dir> --json
cargo run -p nsld -- verify-object-plan <artifact-output-dir>
cargo run -p nsld -- verify-object-plan <artifact-output-dir> --json
cargo run -p nsld -- object-writer-readiness <artifact-output-dir>
cargo run -p nsld -- object-writer-readiness <artifact-output-dir> --json
cargo run -p nsld -- emit-object <artifact-output-dir>
cargo run -p nsld -- emit-object <artifact-output-dir> --json
cargo run -p nsld -- verify-object-emit <artifact-output-dir>
cargo run -p nsld -- verify-object-emit <artifact-output-dir> --json
cargo run -p nsld -- verify-object-writer-input <artifact-output-dir>
cargo run -p nsld -- verify-object-writer-input <artifact-output-dir> --json
cargo run -p nsld -- object-writer-dry-run <artifact-output-dir>
cargo run -p nsld -- object-writer-dry-run <artifact-output-dir> --json
cargo run -p nsld -- emit-object-writer-dry-run <artifact-output-dir>
cargo run -p nsld -- emit-object-writer-dry-run <artifact-output-dir> --json
cargo run -p nsld -- verify-object-writer-dry-run <artifact-output-dir>
cargo run -p nsld -- verify-object-writer-dry-run <artifact-output-dir> --json
cargo run -p nsld -- object-byte-layout <artifact-output-dir>
cargo run -p nsld -- object-byte-layout <artifact-output-dir> --json
cargo run -p nsld -- emit-object-byte-layout <artifact-output-dir>
cargo run -p nsld -- emit-object-byte-layout <artifact-output-dir> --json
cargo run -p nsld -- verify-object-byte-layout <artifact-output-dir>
cargo run -p nsld -- verify-object-byte-layout <artifact-output-dir> --json
cargo run -p nsld -- container-plan <artifact-output-dir>
cargo run -p nsld -- container-plan <artifact-output-dir> --json
cargo run -p nsld -- emit-container-plan <artifact-output-dir>
cargo run -p nsld -- emit-container-plan <artifact-output-dir> --json
cargo run -p nsld -- verify-container-plan <artifact-output-dir>
cargo run -p nsld -- verify-container-plan <artifact-output-dir> --json
cargo run -p nsld -- container <artifact-output-dir>
cargo run -p nsld -- container <artifact-output-dir> --json
cargo run -p nsld -- emit-container <artifact-output-dir>
cargo run -p nsld -- emit-container <artifact-output-dir> --json
cargo run -p nsld -- verify-container <artifact-output-dir>
cargo run -p nsld -- verify-container <artifact-output-dir> --json
cargo run -p nsld -- bundle <artifact-output-dir>
cargo run -p nsld -- bundle <artifact-output-dir> --json
cargo run -p nsld -- emit-bundle <artifact-output-dir>
cargo run -p nsld -- emit-bundle <artifact-output-dir> --json
cargo run -p nsld -- verify-bundle <artifact-output-dir>
cargo run -p nsld -- verify-bundle <artifact-output-dir> --json
cargo run -p nsld -- units <artifact-output-dir>
cargo run -p nsld -- units <artifact-output-dir> --json
cargo run -p nsld -- emit-units <artifact-output-dir>
cargo run -p nsld -- emit-units <artifact-output-dir> --json
cargo run -p nsld -- verify-units <artifact-output-dir>
cargo run -p nsld -- verify-units <artifact-output-dir> --json
cargo run -p nsld -- inputs <artifact-output-dir>
cargo run -p nsld -- inputs <artifact-output-dir> --json
cargo run -p nsld -- emit-inputs <artifact-output-dir>
cargo run -p nsld -- emit-inputs <artifact-output-dir> --json
cargo run -p nsld -- verify-inputs <artifact-output-dir>
cargo run -p nsld -- verify-inputs <artifact-output-dir> --json
```

When given an output directory, `Nsld` resolves
`nuis.build.manifest.toml` inside that directory.

For the normal linker-preparation workflow, prefer:

```sh
cargo run -p nsld -- prepare <artifact-output-dir>
cargo run -p nsld -- check <artifact-output-dir>
```

`nsld prepare` emits and immediately verifies the nine current Nsld-owned
artifacts in dependency order:

* `nuis.nsld.link-inputs.toml`
* `nuis.nsld.link-units.toml`
* `nuis.nsld.link-bundle.toml`
* `nuis.nsld.assemble-plan.toml`
* `nuis.nsld.section-manifest.toml`
* `nuis.nsld.object-plan.toml`
* `nuis.nsld.container-plan.toml`
* `nuis.nsld.container`
* `nuis.nsld.container.payload`

`nsld emit-inputs` is the explicit materialization command for the link-input
table. `nsld inputs` remains accepted as the alpha-era compatibility alias.

This gives later linker, cache, and debugger stages one reproducible
preparation step without hiding the lower-level `inputs`, `emit-units`, or
`emit-bundle` commands.
The prepare report also returns the final container `metadata_table_hash`,
`container_layout_hash`, `container_hash`, `payload_size_bytes`, and
`payload_hash`, so later stages can key off the prepared binary-contract
summary without re-opening every artifact.

`nsld assemble-plan` is the first dry-run view of binary assembly. It consumes
the prepared bundle state and lists the sections that a future Nsld-owned
container writer would need to assemble in deterministic order. It currently
reports:

* compiled artifact section
* Nsld link input table section
* Nsld link unit table section
* Nsld link bundle section
* validated lowering sidecar input sections
* hetero data segment sections when source paths are present

The command is intentionally non-mutating: it does not write a binary and does
not replace the host finalizer. Its purpose is to make the future self-owned
section assembly route visible and testable before relocation/container writing
lands.

`nsld emit-assemble-plan` materializes this dry-run view to:

```text
nuis.nsld.assemble-plan.toml
```

The emitted plan currently uses:

```toml
schema = "nuis-nsld-assemble-plan-v1"
schema_version = 1
plan_kind = "deterministic-section-assembly-plan"
producer = "nsld"
producer_phase = "alpha-0.6.0"
ready = true
bundle_id = "lb..."
bundle_hash = "0x..."
assemble_plan_hash = "0x..."
section_count = 6
blockers = []

[[section]]
order_index = 0
section_id = "sec0000.compiled-artifact"
section_kind = "compiled-artifact"
source_path = "/.../nuis.compiled.artifact"
source_hash = "0x..."
required = true
```

`nsld verify-assemble-plan` re-computes the section plan from the current
manifest, prepared Nsld metadata, and known sidecar/data segment paths.
Verification fails if the file is missing, if the full content differs, or if
`assemble_plan_hash` or `section_count` no longer match.

`nsld section-manifest` derives the container-writer-facing section table from
the assemble plan. It keeps the same deterministic section order while
emphasizing section identity, source hashes, and `section_table_hash`.

`nsld emit-section-manifest` materializes this table to:

```text
nuis.nsld.section-manifest.toml
```

The emitted manifest currently uses:

```toml
schema = "nuis-nsld-section-manifest-v1"
schema_version = 1
manifest_kind = "deterministic-section-manifest"
producer = "nsld"
producer_phase = "alpha-0.6.0"
ready = true
assemble_plan_hash = "0x..."
section_count = 6
section_table_hash = "0x..."
blockers = []

[[section]]
order_index = 0
section_id = "sec0000.compiled-artifact"
section_kind = "compiled-artifact"
source_path = "/.../nuis.compiled.artifact"
source_hash = "0x..."
required = true
```

`nsld verify-section-manifest` re-computes the section manifest and fails if
the file is missing, if the full content differs, or if `section_count` or
`section_table_hash` no longer match.

`nsld object-plan` derives the first object-writer-facing plan from the section
manifest. It maps each Nsld section to a stable object section name and role
and now records deterministic writer layout seeds such as source size,
alignment, planned file offset seed, and planned file size seed while keeping
native byte emission and relocation application explicitly blocked behind
`plan-only` status.
The report also includes `writer_target_id`, `writer_status`, and
`unsupported_features` so future byte writers can consume the plan without
guessing target support.
`[[object_relocation_seed]]` entries describe Nsld-owned relocation intent
before it is lowered into Mach-O, ELF, PE, shader, or kernel relocation forms.
`nsld verify-object-plan` checks the plan hash, section count,
`[[object_section]]` field presence/types, `[[object_relocation_seed]]` field
presence/types, and mapping/seed drift.
`nsld object-writer-readiness` is a non-mutating readiness view over the same
plan. It reports whether object emission is currently allowed for the selected
writer target.
`nsld emit-object` already exists as a structured frontdoor, but in the current
alpha it intentionally reports `emitted = false` while the object byte emitter
and native relocation applier are still blocked. The command succeeds when it
materializes the current diagnostic artifacts: `nuis.nsld.object-writer-input.toml`,
`nuis.nsld.object.blocked.toml`, `nuis.nsld.object-image-dry-run.toml`, and
`nuis.nsld.object-image-dry-run.bin`.
`nsld verify-object-emit` checks that the blocked emit report and image dry-run
artifacts still agree on the object plan hash and dry-run image hash.
`nsld verify-object-writer-input` checks that this snapshot still matches the
current object plan hashes, section count, relocation seed count, and required
writer table field types.
`nsld object-writer-dry-run` is a non-mutating writer preflight report. It
summarizes the writer input path, planned native object path, section and
relocation seed counts, whether the writer input is valid, and the blockers
that still prevent real byte emission.
`nsld emit-object-writer-dry-run` writes this preflight state to
`nuis.nsld.object-writer-dry-run.toml`; `nsld verify-object-writer-dry-run`
checks that artifact against the current writer input and object plan state.
`nsld object-byte-layout` then derives the deterministic byte-level section
layout: file offsets, byte sizes, alignment, total byte span, and a
`byte_layout_hash`. `emit-object-byte-layout` writes
`nuis.nsld.object-byte-layout.toml`, and `verify-object-byte-layout` checks it
before any future platform-specific object writer emits bytes.
When blocked, it also writes `nuis.nsld.object.blocked.toml` so CI and later
linker stages can consume the failed emission state without scraping stderr.

`nsld container-plan` derives the first Nuis-owned binary container layout
plan. It consumes the section manifest, records the container magic/version,
the deterministic section table hash, the planned output path, and a
`container_layout_hash` that future writer/linker stages can use as a stable
layout identity.

`nsld emit-container-plan` materializes this layout plan to:

```text
nuis.nsld.container-plan.toml
```

The emitted plan currently uses:

```toml
schema = "nuis-nsld-container-plan-v1"
schema_version = 1
plan_kind = "deterministic-container-layout-plan"
producer = "nsld"
producer_phase = "alpha-0.6.0"
ready = true
container_magic = "NUISNSLD"
container_version = 1
section_count = 6
section_table_hash = "0x..."
container_layout_hash = "0x..."
output_path = "/.../nuis.nsld.container"
blockers = []

[[section]]
order_index = 0
section_id = "sec0000.compiled-artifact"
section_kind = "compiled-artifact"
source_path = "/.../nuis.compiled.artifact"
source_hash = "0x..."
required = true
```

`nsld verify-container-plan` re-computes the container plan and fails if the
file is missing, if the full content differs, or if `section_count` or
`container_layout_hash` no longer match.

`nsld container` derives the first deterministic Nuis-owned container file
view. It is intentionally still a metadata container shell: it records the
container magic/version, `container_layout_hash`, `container_hash`, blockers,
aggregate `payload_size_bytes` / `payload_hash`, and section table with
deterministic `offset` / `size_bytes` entries without claiming to replace
relocation, final native object linking, or host executable wrapping.
The preview report exposes `metadata_table_hash`,
`container_section_table_hash`, `loader_symbol_table_hash`,
`relocation_table_hash`, and `external_import_table_hash` so loader, release,
and debugger tooling can key off the same table summaries before
`emit-container` writes files.

`nsld emit-container` materializes this view and its contiguous payload blob
to:

```text
nuis.nsld.container
nuis.nsld.container.payload
```

The emitted container currently uses TOML-compatible metadata:

```toml
schema = "nuis-nsld-container-v1"
schema_version = 1
container_kind = "deterministic-hetero-container"
producer = "nsld"
producer_phase = "alpha-0.6.0"
ready = true
container_magic = "NUISNSLD"
container_version = 1
metadata_table_hash = "0x..."
section_count = 6
container_section_table_hash = "0x..."
container_layout_hash = "0x..."
container_hash = "0x..."
loader_readiness = "host-assisted"
loader_blockers = ["external-import:final-stage-driver:cc", "external-import:clang-target:arm64-apple-macosx", "external-import:c-world-policy:wrapped"]
loader_entry_kind = "lifecycle-bootstrap"
loader_entry_symbol = "nustar.bootstrap.v1"
loader_entry_section_id = "sec0000.compiled-artifact"
loader_symbol_count = 2
loader_symbol_table_hash = "0x..."
relocation_count = 2
relocation_table_hash = "0x..."
external_import_count = 3
external_import_table_hash = "0x..."
payload_size_bytes = 1234
payload_hash = "0x..."
payload_path = "/.../nuis.nsld.container.payload"
blockers = []

[[loader_symbol]]
symbol_id = "sym0000.loader-entry"
symbol_kind = "lifecycle-bootstrap"
symbol_name = "nustar.bootstrap.v1"
section_id = "sec0000.compiled-artifact"
offset = 0
size_bytes = 1234
payload_hash = "0x..."

[[relocation]]
relocation_id = "rel0000.lifecycle-entry"
relocation_kind = "lifecycle-entry-binding"
source_section_id = "sec0000.compiled-artifact"
source_offset = 0
target_symbol_id = "sym0000.loader-entry"
addend = 0

[[relocation]]
relocation_id = "rel0001.hetero-node"
relocation_kind = "hetero-node-binding"
source_section_id = "sec0004.lowering-sidecar-input"
source_offset = 1234
target_symbol_id = "sym0001.hetero-node.shader.official.shader"
addend = 0

[[external_import]]
import_id = "imp0000.final-stage-driver"
import_kind = "final-stage-driver"
import_name = "cc"
provider = "host-toolchain"
required = true

[[external_import]]
import_id = "imp0001.clang-target"
import_kind = "clang-target"
import_name = "arm64-apple-macosx"
provider = "host-toolchain"
required = true

[[external_import]]
import_id = "imp0002.c-world-policy"
import_kind = "c-world-policy"
import_name = "wrapped"
provider = "c-world-wrapper"
required = true

[[section]]
order_index = 0
section_id = "sec0000.compiled-artifact"
section_kind = "compiled-artifact"
source_path = "/.../nuis.compiled.artifact"
source_hash = "0x..."
payload_hash = "0x..."
required = true
offset = 0
size_bytes = 1234
```

`nsld verify-container` re-computes the container shell and payload blob. It
fails if either file is missing, if the metadata content differs, or if
`metadata_table_hash`, `section_count`, `container_section_table_hash`,
`container_layout_hash`, `loader_entry_kind`, `loader_entry_symbol`,
`loader_entry_section_id`, `loader_readiness`, `loader_symbol_count`,
`loader_symbol_table_hash`, `relocation_count`, `relocation_table_hash`,
`external_import_count`, `external_import_table_hash`, `payload_size_bytes`,
`payload_hash`, or `container_hash` no longer match. It also parses and checks
every `[[section]]`, `[[loader_symbol]]`, `[[relocation]]`, and
`[[external_import]]` table entry by index. Field-level table diagnostics are
grouped into `container_section_issues`, `loader_symbol_issues`,
`relocation_issues`, and `external_import_issues`; malformed entries report
missing or invalid fields such as
`relocation[0].relocation_kind missing` or
`relocation[0].source_offset invalid`. Section payload ranges are checked
separately in `section_range_issues` against each section's `payload_hash`, so a
corrupted payload segment can be reported without waiting for later relocation
or final native linking.

The loader entry fields and `[[loader_symbol]]` table are the first
loader-facing bootstrap records in the Nsld container. They currently bind the
lifecycle bootstrap symbol from the link plan to the compiled artifact section
and its payload range; future loader/runtime work can extend that into richer
symbol and relocation tables without changing the container's basic entry
contract.

`relocation_count` and `relocation_table_hash` describe the loader-facing
relocation table. The current metadata container emits a deterministic
`lifecycle-entry-binding` record that binds the compiled artifact bootstrap
section to the loader entry symbol. When heterogeneous link-plan nodes are
present, it also emits `hetero-node-binding` records that bind each node's
`link_input` section to a loader-visible dispatch symbol. These are not final
native object relocations yet; they are Nsld-owned loader relocation seeds that
future Mach-O, ELF, PE, shader, and kernel relocation phases can extend without
inventing a new top-level container concept.

`[[external_import]]` records host or compatibility dependencies still outside
the self-owned Nsld container. Today that normally includes the host final-stage
driver, the selected clang target, and any non-`none` C-world wrapper policy.
These entries make the remaining non-native closure explicit to a future loader
or release gate.

`loader_readiness` summarizes that state for loader and release tooling:
`self-contained` means the container has no required external imports,
`host-assisted` means it is structurally loadable but still depends on host or
compatibility providers, and `blocked` means the container has unresolved
assembly blockers. `loader_blockers` records the exact reason strings used to
derive that readiness.

`nsld check` also exposes `container_payload_present` and
`container_payload_issues`, so missing or orphaned payload state is visible in
the top-level linker health report.

## Linker Check

`nsld check` is the first dedicated linker gate. It currently verifies:

* artifact lowering alignment is consistent
* clock protocol validation passed
* hetero calculate validation passed
* host FFI validation passed and `host_ffi.link_allowed` remains true
* host FFI ABI groups expose per-ABI entries and local validation summaries
* hetero calculate plan is static-link
* hetero calculate plan is lifecycle-driven
* lowering sidecar capabilities are readable and link-ready for domains that
  declare `artifact_ir_sidecar_path`
* an emitted `nuis.nsld.link-inputs.toml` is still valid when that file is
  present
* an emitted `nuis.nsld.link-units.toml` is still valid when that file is
  present
* an emitted `nuis.nsld.link-bundle.toml` is still valid when that file is
  present
* an emitted `nuis.nsld.assemble-plan.toml` is still valid when that file is
  present
* an emitted `nuis.nsld.section-manifest.toml` is still valid when that file is
  present
* an emitted `nuis.nsld.object-plan.toml` is still valid when that file is
  present
* an emitted `nuis.nsld.container-plan.toml` is still valid when that file is
  present
* an emitted `nuis.nsld.container` is still valid when that file is present
* an emitted `nuis.nsld.container.payload` is paired with the container metadata
  when either side exists
* the container loader readiness can be surfaced from the top-level check when
  a container is present
* the emitted Nsld artifact chain is a contiguous prepared prefix, so a later
  artifact such as `nuis.nsld.container` cannot appear without its prerequisite
  metadata artifacts

The command exits with failure when any linker gate fails. JSON output is
intended for CI and future toolchain orchestration.

Host FFI is treated as static linker input, not as a dynamic fast path. The
current link plan carries parsed host FFI entries, ABI groups, validation
issues/notes, and the derived `link_allowed` decision. Duplicate whitelist
entries or policy/count drift are blocking issues. Multiple signatures for the
same ABI symbol are reported as notes so the linker can surface overload-like
shape without rejecting an otherwise valid registered footprint.

`nsld check` does not require `nuis.nsld.link-inputs.toml`,
`nuis.nsld.link-units.toml`, `nuis.nsld.link-bundle.toml`, or
`nuis.nsld.assemble-plan.toml`, `nuis.nsld.section-manifest.toml`,
`nuis.nsld.object-plan.toml`, `nuis.nsld.container-plan.toml`, or
`nuis.nsld.container` to exist. If any file
is absent, the corresponding gate is reported as absent and the check still
uses the core linker gates. If a file is present, it is verified with the same
rules as `nsld verify-inputs`, `nsld verify-units`, `nsld verify-bundle`,
`nsld verify-assemble-plan`, `nsld verify-section-manifest`,
`nsld verify-container-plan`, or `nsld verify-container`; any mismatch fails
the check.

The check report exposes `artifact_chain_valid` and `artifact_chain_issues`
for this prepare-order state.

When `nuis.nsld.container` exists, the check report also exposes
`container_loader_readiness`, `container_loader_blockers`,
`container_metadata_table_hash`, and `container_external_import_count`.
`host-assisted` is reported as an explicit remaining dependency state, not as a
check failure; `blocked` fails the check because it means the container still
has unresolved assembly blockers.

The check report also exposes linker diagnostics for:

* `domains`: package, domain family, lowering target, backend, and alignment
  issues
* `sidecar_capabilities`: owning Nustar, frontend IR, native IR, dispatch
  lowering, validation contracts, and per-sidecar issues
* `clock_edges`: clock bridge edges and happens-before edges
* `data_segments`: deterministic segment order, owner package, access phase,
  and payload source

## Linker Closure

`nsld closure` is a route-feasibility report. It separates:

* `internal_contracts`: contracts Nsld can already consume as linker truth
* `link_inputs`: validated lowering sidecars that can be consumed as linker
  inputs
* `external_dependencies`: host or wrapper dependencies still outside Nsld
* `unresolved`: missing pieces before a fully self-owned linker closure

This command is intentionally conservative. A report with `closed = false`
does not mean the build is unusable; it means the current route still depends
on non-Nsld stages such as the host launcher wrapper or final native link.

If `nuis.nsld.link-inputs.toml` exists, closure also verifies it. A valid table
adds `verified-link-input-table` to `internal_contracts`; an invalid table adds
`link-input-table:*` entries to `unresolved`. If the table is absent, closure
reports the table state as absent without treating that absence as unresolved.

When every declared lowering IR sidecar has a valid capability block, closure
adds `lowering-sidecar-capabilities` to `internal_contracts`. Domains without
IR sidecars, such as data/fabric domains, are not treated as sidecar capability
failures.

Closure also builds a `link-input-sidecar-table` from valid sidecar
capabilities. Each entry records a stable input id, input kind, domain family,
package id, sidecar path, native IR, dispatch lowering, validation contract
count, byte length, and a deterministic FNV-1a content hash. Link inputs are
ordered by domain family, package id, and sidecar path before `liNNNN` ids are
assigned.

Closure also reports `link_input_count`, `link_input_total_bytes`, and
`link_input_table_hash`. The table hash is derived from the ordered linker
input identities and their content hashes, so a future linker/cache/debugger
can cheaply detect whether the complete heterogeneous input set is unchanged.

Closure also reports the expected container `container_metadata_table_hash`
and `container_loader_readiness`. These are derived from the current link plan
and do not require `nuis.nsld.container` to have been emitted yet; they give
route-planning tools the same container fingerprint used by `container`,
`emit-container`, `prepare`, and `check`.

This is not final object linking yet; it is the linker-owned input table that
future binary assembly, cache reuse, debug-symbol correlation, and closure
verification can consume.

## Link Units

`nsld units` builds the first deterministic link-unit view over the current
link plan. It groups registered domain units by stable domain/package/role
order and attaches any validated lowering sidecar inputs owned by that unit.

Each reported unit includes:

* `unit_id`: stable `luNNNN.<domain>.<package>` identity
* `unit_kind`: currently `native-domain` or `hetero-domain`
* domain family, package id, backend family, lowering target, and packaging
  role
* attached `link_input_ids` from the Nsld sidecar input table
* clock-edge and data-segment counts visible to the unit
* whether the unit still requires the host wrapper path
* a deterministic order key

The report also exposes `unit_table_hash`, derived from the ordered unit
material. This is deliberately a link-contract hash, not a final object hash:
it lets future Nsld, Nsdb, cache reuse, and hetero binary assembly detect when
the domain-unit skeleton has changed without peeking into a Nustar's private
lowering logic.

`nsld emit-units` materializes this report to:

```text
nuis.nsld.link-units.toml
```

The emitted table currently uses:

```toml
schema = "nuis-nsld-link-unit-table-v1"
schema_version = 1
table_kind = "deterministic-link-units"
producer = "nsld"
producer_phase = "alpha-0.6.0"
unit_count = 2
hetero_unit_count = 1
link_input_count = 1
clock_edge_count = 3
data_segment_count = 1
unit_table_hash = "0x..."

[[link_unit]]
order_index = 1
unit_id = "lu0001.shader.official.shader"
unit_kind = "hetero-domain"
domain_family = "shader"
package_id = "official.shader"
backend_family = "metal"
lowering_target = "metal.apple-silicon-gpu"
packaging_role = "hetero-contract"
link_input_ids = ["li0000.shader.official.shader"]
clock_edge_count = 2
data_segment_count = 1
requires_host_wrapper = false
deterministic_order_key = "0001.shader.official.shader"
```

`nsld verify-units` re-computes the expected unit table from the current
manifest and link plan. Verification fails if the file is missing, if the full
table content differs, or if `unit_count`, `hetero_unit_count`,
`link_input_count`, or `unit_table_hash` no longer match.

## Link Bundle

`nsld bundle` folds the input table, link-unit table, clock/data counts,
final-stage mode, and artifact paths into one linker-owned bundle view. This
is still not final object linking; it is the single manifest a future Nsld
assembler, cache, or YIR-level debugger can consume before section assembly.

`nsld emit-bundle` materializes this view to:

```text
nuis.nsld.link-bundle.toml
```

The emitted bundle currently uses:

```toml
schema = "nuis-nsld-link-bundle-v1"
schema_version = 1
bundle_kind = "hetero-static-link-bundle"
producer = "nsld"
producer_phase = "alpha-0.6.0"
bundle_id = "lb..."
bundle_hash = "0x..."
bundle_ready = true
unit_count = 2
hetero_unit_count = 1
link_input_count = 1
link_input_total_bytes = 1987
link_input_table_hash = "0x..."
unit_table_hash = "0x..."
clock_edge_count = 3
data_segment_count = 1
final_stage_link_mode = "host-toolchain-finalize"
host_wrapper_required = true
compiled_artifact_path = "/.../nuis.compiled.artifact"
native_output_path = "/.../shader_profile_demo"
issues = []
```

`nsld verify-bundle` re-computes the bundle from the current manifest and link
plan. Verification fails if the file is missing, if the full content differs,
or if `bundle_id` or `bundle_hash` no longer match. `bundle_ready = false`
does not by itself mean the file is invalid; it means the bundle faithfully
records unresolved linker inputs that future stages must not ignore.

## Link Input Table Artifact

`nsld emit-inputs` materializes the closure link input table to:

```text
nuis.nsld.link-inputs.toml
```

The emitted table currently uses:

```toml
schema = "nuis-nsld-link-input-table-v1"
schema_version = 1
table_kind = "lowering-sidecar-link-inputs"
producer = "nsld"
producer_phase = "alpha-0.6.0"
link_input_count = 1
link_input_total_bytes = 1987
link_input_table_hash = "0x..."

[[link_input]]
order_index = 0
input_id = "li0000.shader.official.shader"
input_kind = "lowering-ir-sidecar"
domain_family = "shader"
package_id = "official.shader"
path = "/.../nuis.domain.shader.lowering.ir.txt"
native_ir = "msl2.4"
dispatch_lowering = "command-encoder-draw-dispatch"
contract_count = 3
content_bytes = 1987
content_hash = "0x..."
```

This file is still alpha-stage metadata, but it is deliberately shaped as a
future linker/cache/debugger contract rather than a human-only report.
`nsld inputs` remains accepted as a compatibility alias for the same operation.

`nsld verify-inputs` re-computes the expected table from the current manifest
and declared lowering sidecars, then checks the emitted
`nuis.nsld.link-inputs.toml`. Verification fails if the file is missing, if the
full table content differs, or if `link_input_count`,
`link_input_total_bytes`, or `link_input_table_hash` no longer match. This is
the first self-checking form of the Nsld-owned linker input contract.

## Boundary Rule

The compiler may know the shared structure of `nustar` registration,
artifact manifests, lifecycle metadata, and YIR contracts. It should not grow
hard-coded knowledge of each domain's private linker behavior.

`Nsld` should therefore evolve toward this shape:

```text
nuisc produces verified artifacts and manifests
  -> nsld consumes the link contract
  -> nsld freezes hetero clock/data order
  -> nsld assembles the Nuis-owned binary container
  -> host toolchain is used only as a wrapper when required
```

## Alpha-0.6.0 Meaning

For `alpha-0.6.0`, success means:

* linker truth has a named tool boundary
* existing `nuisc::linker` behavior remains reusable
* `Nsld` can inspect real build outputs
* clock protocol and hetero calculate metadata are visible from the linker
  frontdoor
* Nsld can derive a deterministic link-unit skeleton from registered domain
  units and validated lowering sidecars

This is the beginning of linker independence, not the end of linker work.
