# Nsld Linker Frontdoor

`Nsld` is the Nuis linker toolchain member introduced on the `alpha-0.6.0`
line.

At this stage, `Nsld` is intentionally a frontdoor over the existing linker
contract logic in `nuisc::linker`. It does not yet claim to be the final
self-owned object linker. Its job is to give linker work a stable tool
boundary before the implementation is split out further.

For the current `alpha-0.10.*` line, the emphasis is executable-artifact
closure: keep the frontdoor/reporting discipline, but drive it toward the
smallest runnable host-assisted route or explicit blocked executable artifact.

Longer-term, `Nsld` should be read as a CLI adapter over a future reusable
linker core / galaxy capability boundary, not as a CLI-only tool. See
[toolchain-galaxy-core-boundary.md](toolchain-galaxy-core-boundary.md).
For the current gap between the deterministic Nsld container and a runnable
Nuis-owned heterogeneous executable, see [nsld-binary-assembly-gap-map.md](nsld-binary-assembly-gap-map.md).
For the automation frontdoor, see [nsld-driver-frontdoor.md](nsld-driver-frontdoor.md).

## Current Role

`Nsld` currently owns:

* link-plan inspection from `nuis.build.manifest.toml`
* heterogeneous calculate plan visibility
* clock protocol visibility
* lowering sidecar capability validation for domains that declare IR sidecars
* deterministic link-unit reporting across registered domain units
* object-plan target identity metadata for optional Mach-O/ELF/COFF-family
  compatibility writers
* final-stage reporting
* the first independent CLI boundary for future linker work

`Nsld` does not yet own:

* final host-native executable wrapping for Mach-O, ELF, or PE
* replacement of the host toolchain wrapper
* binary section assembly independent from `nuisc`
* stable linker script or relocation formats
* finished `nsld-core` galaxy-style API for direct compiler/runtime consumers

The long-term Nsld design is not `.o`-first. Nuis-native linking should be able
to consume structured Nsld/YIR/Nustar link units, lifecycle/clock metadata,
GLM-compatible ownership metadata, deterministic sections, and heterogeneous
payloads directly. Mach-O, ELF, PE/COFF, and traditional object files are
compatibility/finalization backends: useful for host operating systems and C ABI
bridges, but not the required internal representation of the linker core.

This also means the C ABI, libc, and classic von-Neumann host stack should be
treated as a CFFI / host-compat execution domain inside Nuis. They are not the
semantic root of the linker. See
[cffi-von-neumann-domain-contract.md](cffi-von-neumann-domain-contract.md).

## Commands

```sh
cargo run -p nsld -- status
cargo run -p nsld -- plan <nuis.build.manifest.toml>
cargo run -p nsld -- plan <artifact-output-dir> --json
cargo run -p nsld -- check <artifact-output-dir>
cargo run -p nsld -- check <artifact-output-dir> --json
cargo run -p nsld -- artifact-chain <artifact-output-dir>
cargo run -p nsld -- artifact-chain <artifact-output-dir> --json
cargo run -p nsld -- closure <artifact-output-dir>
cargo run -p nsld -- closure <artifact-output-dir> --json
cargo run -p nsld -- emit-closure <artifact-output-dir>
cargo run -p nsld -- verify-closure <artifact-output-dir> --json
cargo run -p nsld -- final-stage-plan <artifact-output-dir>
cargo run -p nsld -- emit-final-stage-plan <artifact-output-dir>
cargo run -p nsld -- verify-final-stage-plan <artifact-output-dir> --json
cargo run -p nsld -- final-executable-readiness <artifact-output-dir>
cargo run -p nsld -- final-executable-writer-plan <artifact-output-dir>
cargo run -p nsld -- emit-final-executable-writer-input <artifact-output-dir>
cargo run -p nsld -- verify-final-executable-writer-input <artifact-output-dir> --json
cargo run -p nsld -- final-executable-host-dry-run <artifact-output-dir>
cargo run -p nsld -- final-executable-host-invoke-plan <artifact-output-dir>
cargo run -p nsld -- emit-final-executable-host-invoke-plan <artifact-output-dir>
cargo run -p nsld -- verify-final-executable-host-invoke-plan <artifact-output-dir> --json
cargo run -p nsld -- final-executable-layout <artifact-output-dir>
cargo run -p nsld -- emit-final-executable-layout <artifact-output-dir>
cargo run -p nsld -- verify-final-executable-layout <artifact-output-dir> --json
cargo run -p nsld -- final-executable-image-dry-run <artifact-output-dir>
cargo run -p nsld -- emit-final-executable-image-dry-run <artifact-output-dir>
cargo run -p nsld -- verify-final-executable-image-dry-run <artifact-output-dir> --json
cargo run -p nsld -- emit-final-executable <artifact-output-dir>
cargo run -p nsld -- verify-final-executable-emit <artifact-output-dir> --json
cargo run -p nsld -- final-executable-output <artifact-output-dir>
cargo run -p nsld -- final-executable-output <artifact-output-dir> --json
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
cargo run -p nsld -- verify-object-output <artifact-output-dir>
cargo run -p nsld -- verify-object-output <artifact-output-dir> --json
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
cargo run -p nsld -- object-file-layout <artifact-output-dir>
cargo run -p nsld -- object-file-layout <artifact-output-dir> --json
cargo run -p nsld -- emit-object-file-layout <artifact-output-dir>
cargo run -p nsld -- emit-object-file-layout <artifact-output-dir> --json
cargo run -p nsld -- verify-object-file-layout <artifact-output-dir>
cargo run -p nsld -- verify-object-file-layout <artifact-output-dir> --json
cargo run -p nsld -- object-image-dry-run <artifact-output-dir>
cargo run -p nsld -- object-image-dry-run <artifact-output-dir> --json
cargo run -p nsld -- emit-object-image-dry-run <artifact-output-dir>
cargo run -p nsld -- emit-object-image-dry-run <artifact-output-dir> --json
cargo run -p nsld -- verify-object-image-dry-run <artifact-output-dir>
cargo run -p nsld -- verify-object-image-dry-run <artifact-output-dir> --json
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

`nsld prepare` emits and immediately verifies the current required Nsld-owned
preparation artifacts in dependency order, then reports canonical paths for
the optional final-executable boundary artifacts:

* `nuis.nsld.link-inputs.toml`
* `nuis.nsld.link-units.toml`
* `nuis.nsld.link-bundle.toml`
* `nuis.nsld.assemble-plan.toml`
* `nuis.nsld.section-manifest.toml`
* `nuis.nsld.object-plan.toml`
* `nuis.nsld.object-writer-input.toml`
* `nuis.nsld.object-byte-layout.toml`
* `nuis.nsld.object-file-layout.toml`
* `nuis.nsld.object-image-dry-run.toml`
* `nuis.nsld.object-image-dry-run.bin`
* `nuis.nsld.object.blocked.toml`
* optional `nuis.nsld.mach-o` when the Mach-O arm64 writer can emit
* `nuis.nsld.object-writer-dry-run.toml`
* `nuis.nsld.container-plan.toml`
* `nuis.nsld.container`
* `nuis.nsld.container.payload`
* `nuis.nsld.closure.toml`
* `nuis.nsld.final-stage-plan.toml`
* optional `nuis.nsld.final-executable-writer-input.toml`
* optional `nuis.nsld.final-executable-host-invoke-plan.toml`
* optional `nuis.nsld.final-executable-layout.toml`
* optional `nuis.nsld.final-executable.blocked.toml`

`nsld emit-inputs` is the explicit materialization command for the link-input
table. `nsld inputs` remains accepted as the alpha-era compatibility alias.

This gives later linker, cache, and debugger stages one reproducible
preparation step without hiding the lower-level `inputs`, `emit-units`, or
`emit-bundle` commands. It also emits `nuis.nsld.object-image-dry-run.bin`
when the registered object-image backend can construct a deterministic dry-run
image. `nsld check` treats the writer input snapshot, blocked emit report, and
writer dry-run report as first-class chain artifacts, so stale object-writer
state is caught before container or future native object emission. When native
object emission succeeds, `prepare` also verifies the emitted object output
against the deterministic image bytes before treating the preparation as valid.
The prepare report also returns the final container `metadata_table_hash`,
`container_layout_hash`, `container_hash`, `payload_size_bytes`, and
`payload_hash`, so later stages can key off the prepared binary-contract
summary without re-opening every artifact.
It also emits and verifies `nuis.nsld.closure.toml`, a closure-level linker
contract snapshot keyed by `linker_contract_hash`. The closure snapshot is a
prepared output and check-time validation target; it is intentionally excluded
from closure's own prepared-prefix contract so the snapshot does not hash or
verify itself recursively.
`prepare` also emits and verifies `nuis.nsld.final-stage-plan.toml`, the
deterministic executable-boundary plan. A final-stage plan can be valid while
`ready = false`; for example, the current host-assisted route intentionally
records `self-owned-final-native-linker` as a blocker until Nsld owns the final
native executable step.
It also surfaces object-image relocation lowering status through
`object_image_relocation_lowering_valid`,
`object_image_relocation_lowering_rule_count`, and
`object_image_relocation_lowering_issues`, matching the top-level check view.
The prepare JSON report also exposes
`object_image_relocation_lowering_rules`,
`object_image_relocation_record_count`, and
`object_image_relocation_record_table_hash`, and
`object_image_relocation_records`, so orchestration tools can inspect the
active relocation registry and actual backend relocation records without
opening the dry-run TOML separately.

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
alignment, planned file offset seed, and planned file size seed. The object
plan remains a non-mutating contract even when a registered writer is ready.
The report also includes `writer_target_id`, `writer_backend_kind`,
`object_family`, `writer_status`, and `unsupported_features` so future byte
writers can consume the plan without guessing target support.
`[[object_relocation_seed]]` entries describe Nsld-owned relocation intent
before it is lowered into Mach-O, ELF, PE, shader, or kernel relocation forms.
`nsld verify-object-plan` checks the plan hash, section count,
`[[object_section]]` field presence/types, `[[object_relocation_seed]]` field
presence/types, and mapping/seed drift.
`nsld object-writer-readiness` is a non-mutating readiness view over the same
plan. It reports whether object emission is currently allowed for the selected
writer target.
`nsld emit-object` is the first compatibility object emission frontdoor. For
the registered Mach-O arm64 writer, once the prerequisite Nsld artifacts are
prepared, it writes optional `nuis.nsld.mach-o` from the deterministic image
bytes and reports `emitted = true`. For unprepared input or backends that
remain blocked, it still reports `emitted = false` with blockers. This is not a
claim that Nsld's core must consume `.o` files; it is a host-system
compatibility lane alongside the Nuis-native container/link graph lane.
The command also materializes the current diagnostic artifacts:
`nuis.nsld.object-writer-input.toml`, `nuis.nsld.object.blocked.toml`,
`nuis.nsld.object-image-dry-run.toml`, and
`nuis.nsld.object-image-dry-run.bin`. The `object.blocked.toml` path is kept as
the alpha compatibility emit-report path while the report schema evolves.
`nsld verify-object-emit` checks that the emit report and image dry-run
artifacts still agree on the object plan hash and dry-run image hash.
`nsld verify-object-output` separately checks the emitted native object bytes:
today that means `nuis.nsld.mach-o` must have the same size and content hash as
`nuis.nsld.object-image-dry-run.bin`. Keeping this as its own frontdoor lets
later ELF/PE writers, Nsld's future linker core, and nsdb consume the native
object compatibility contract without re-running or parsing `check`.
`nsld verify-object-writer-input` checks that this snapshot still matches the
current object plan hashes, writer identity, section count, relocation seed
count, and required writer table field types.
`nsld object-writer-dry-run` is a non-mutating writer preflight report. It
summarizes the writer input path, planned native object path, section and
relocation seed counts, whether the writer input is valid, and the blockers
that still prevent real byte emission.
`nsld emit-object-writer-dry-run` writes this preflight state to
`nuis.nsld.object-writer-dry-run.toml`; `nsld verify-object-writer-dry-run`
checks that artifact against the current writer input, writer identity, and
object plan state.
`nsld object-byte-layout` then derives the deterministic byte-level section
layout: file offsets, byte sizes, alignment, total byte span, and a
`byte_layout_hash`. `emit-object-byte-layout` writes
`nuis.nsld.object-byte-layout.toml`, and `verify-object-byte-layout` checks it
before any future platform-specific object writer emits bytes.
`nsld object-file-layout` wraps that byte layout in the selected native object
container family and records file-level records, final offsets, and a
`file_layout_hash`. `emit-object-file-layout` writes
`nuis.nsld.object-file-layout.toml`, and `verify-object-file-layout` keeps that
file-level contract locked to the current byte layout.
`nsld object-image-dry-run` is the current native-image boundary. It asks the
registered object-image backend to encode the selected file layout into an
in-memory image without treating it as a final emitted object. Today the
Mach-O arm64 backend can construct dry-run bytes; ELF and COFF slots are
registered but intentionally report `not-implemented`. `emit-object-image-dry-run`
writes both `nuis.nsld.object-image-dry-run.toml` and
`nuis.nsld.object-image-dry-run.bin`, while `verify-object-image-dry-run`
checks the report, image size, and image hash.
The dry-run report also includes the relocation lowering registry and the
actual backend relocation records through `relocation_lowering_rules`,
`relocation_record_count`, `relocation_record_table_hash`, and
`relocation_records`. For Mach-O arm64 this records the native symbol index and
relocation flags derived from each seed, giving later linker/debugger stages a
structured and hashable view before Nsld owns final object linking.
The `emit-object` frontdoor also writes `nuis.nsld.object.blocked.toml` as its
current emit report, so CI and later linker stages can consume emitted or
blocked emission state without scraping stderr. That report preserves
`writer_backend_kind` and `object_family` from the object-plan chain, and
`verify-object-emit` rejects writer identity drift.

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
relocation finalization, compatibility object finalization, or host executable
wrapping.
When `nuis.nsld.mach-o` has been emitted and passes the same validation exposed
by `verify-object-output`, the container plan appends it as a
`native-object-output` section so the native object participates in the
container section table, payload hash, and loader-facing metadata instead of
remaining a side artifact. If the object output exists but no longer matches
the deterministic dry-run bytes, the container plan reports
`object-output:*` blockers and does not package the invalid native object.
This section is best understood as a CFFI Nustar / host-compatibility lane, not
as the Nuis-native linker core. A future Nuis binary format can reserve a
dedicated lifecycle phase for these native-object payloads: enter through an
explicit compatibility hook, run under the CFFI whitelist and wrapper policy,
then return results through Nuis-owned metadata and memory-safety contracts.
That keeps C ABI execution visible to the scheduler instead of turning it into
an unstructured side call.
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
compatibility_domain_count = 1
compatibility_domain_table_hash = "0x..."
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
lifecycle_hook = "on_lifecycle_bootstrap"
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

[[compatibility_domain]]
domain_id = "compat0000.cffi-von-neumann"
domain_kind = "cffi-host-compat"
paradigm = "classic-von-neumann-host"
lifecycle_hook = "on_cffi_native_object"
abi_family = "mach-o"
wrapper_policy = "wrapped"
required = true

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
`compatibility_domain_count`, `compatibility_domain_table_hash`,
`external_import_count`, `external_import_table_hash`, `payload_size_bytes`,
`payload_hash`, or `container_hash` no longer match. It also parses and checks
every `[[section]]`, `[[loader_symbol]]`, `[[relocation]]`,
`[[compatibility_domain]]`, and `[[external_import]]` table entry by index.
Field-level table diagnostics are grouped into `container_section_issues`,
`loader_symbol_issues`, `relocation_issues`,
`compatibility_domain_issues`, and `external_import_issues`; malformed entries report
missing or invalid fields such as
`relocation[0].relocation_kind missing` or
`relocation[0].source_offset invalid`. Loader symbols also carry
`lifecycle_hook`, so bootstrap, heterogeneous dispatch, and CFFI native-object
lanes are visible to the scheduler-facing metadata. Section payload ranges are checked
separately in `section_range_issues` against each section's `payload_hash`, so a
corrupted payload segment can be reported without waiting for later relocation
or final native linking.
When a native object is packaged, the verification report also exposes a
native-object summary: whether the `native-object-output` section exists,
whether the `native-object-output` loader symbol exists, and whether the
`native-object-binding` relocation exists, with their ids for debugger/linker
consumers.
JSON reports keep the legacy flat fields for compatibility, but also expose
object-shaped compatibility-domain summaries for tooling:
`compatibility_domain_summary` on container, prepare, and closure reports,
`container_compatibility_domain_summary` on check reports, and
`expected_compatibility_domain_summary` /
`actual_compatibility_domain_summary` on verify-container reports.

The `[[compatibility_domain]]` table is the current Nsld metadata hook for
explicit CFFI / host-compat execution domains. The default entry is
`compat0000.cffi-von-neumann`, with `domain_kind = "cffi-host-compat"`,
`paradigm = "classic-von-neumann-host"`, and
`lifecycle_hook = "on_cffi_native_object"`. This table is intentionally
separate from `[[external_import]]`: external imports describe dependencies
outside the self-owned container, while compatibility domains describe the
execution paradigm admitted into the Nuis container and schedule.

The loader entry fields and `[[loader_symbol]]` table are the first
loader-facing bootstrap records in the Nsld container. They currently bind the
lifecycle bootstrap symbol from the link plan to the compiled artifact section
and its payload range with `on_lifecycle_bootstrap`; heterogeneous nodes use
their declared lifecycle hook; native-object compatibility payloads use
`on_cffi_native_object`. Future loader/runtime work can extend that into richer
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

## Artifact Chain

`nsld artifact-chain` is a non-mutating diagnostic view of the registered Nsld
artifact sequence. It lists each stage in deterministic order with its
`stage_id`, file name, full path, `required` flag, and `present` state. The
summary exposes `stage_count`, `present_count`, `required_count`,
`missing_required_count`, `optional_present_count`,
`first_missing_required_stage`, `next_required_stage`, and
`suggested_command_id`, `suggested_command`, plus `suggested_command_resolved`,
`suggested_command_reason`, and the same ordering issues used by `nsld check`.
When all required stages are present, it also reports
`next_optional_stage`, `next_optional_command_id`, `next_optional_command`,
`next_optional_command_resolved`, and `next_optional_command_reason`, so the
final executable boundary can suggest the next explicit optional step without
making that step required.
The command id is stable for scripts and UI actions. The suggested command
template uses `<input>` as the same manifest-or-output-dir argument accepted by
the rest of the Nsld frontdoor; the resolved form replaces that placeholder
with the manifest path known to the current report.

This command is intentionally not a gate: it can describe half-built output
directories without exiting as a failure. Use `nsld check` when the artifact
chain should participate in the broader linker health result.

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
* an emitted `nuis.nsld.object-writer-input.toml` is still valid when that file
  is present
* emitted object byte/file/image dry-run reports are still valid when present
* an emitted `nuis.nsld.object.blocked.toml` still agrees with the object plan
  and image dry-run hash when present
* an emitted `nuis.nsld.mach-o` still matches the deterministic image dry-run
  bytes when present, using the same check exposed by `verify-object-output`;
  the check report includes expected/actual object-output size and hash fields
  when the object output exists
* object-image relocation lowering status is surfaced through
  `object_image_relocation_lowering_valid`,
  `object_image_relocation_lowering_rule_count`, and
  `object_image_relocation_lowering_issues`; check JSON also exposes
  `object_image_relocation_lowering_rules`,
  `object_image_relocation_record_count`, and
  `object_image_relocation_record_table_hash`, and
  `object_image_relocation_records` for rule- and record-level auditability
* an emitted `nuis.nsld.object-writer-dry-run.toml` is still valid when that
  file is present
* an emitted `nuis.nsld.container-plan.toml` is still valid when that file is
  present
* an emitted `nuis.nsld.container` is still valid when that file is present
* an emitted `nuis.nsld.container.payload` is paired with the container metadata
  when either side exists
* the container loader readiness can be surfaced from the top-level check when
  a container is present
* the container native-object summary can be surfaced from the top-level check,
  including native object section, loader symbol, and relocation ids
* the container compatibility-domain summary can be surfaced from the top-level
  check, including the CFFI/host-compat domain id, paradigm, lifecycle hook,
  ABI family, wrapper policy, table hash, and required flag
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
`container_metadata_table_hash`, `container_external_import_count`, and the
`container_compatibility_domain_*` summary fields. Those fields let CI and
future tooling distinguish "this route still has external host dependencies"
from "this Nuis container explicitly admits the CFFI / classic-von-Neumann
compatibility execution domain."
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
* `linker_contract_hash`: deterministic hash of the closure-level linker truth
* `link_inputs`: validated lowering sidecars that can be consumed as linker
  inputs
* `external_dependencies`: host or wrapper dependencies still outside Nsld
* `unresolved`: missing pieces before a fully self-owned linker closure

This command is intentionally conservative. A report with `closed = false`
does not mean the build is unusable; it means the current route still depends
on non-Nsld stages such as the host launcher wrapper or final native link.
`linker_contract_hash` is not a native object hash and not the container hash;
it fingerprints the current linker contract surface: internal contracts, link
input table hash, container metadata/layout/hash state, payload hash, loader
readiness, relocation record table hash, external dependencies, unresolved
entries, and final-stage mode.
`nsld emit-closure` materializes that surface to `nuis.nsld.closure.toml`;
`nsld verify-closure` compares the snapshot against the current closure report
and reports focused drift such as `linker_contract_hash mismatch`,
`container_hash mismatch`, `payload_size_bytes mismatch`, or
`payload_hash mismatch`.
`nsld check` performs the same closure snapshot verification when the snapshot
is present and exposes `closure_snapshot_present`, `closure_snapshot_valid`,
`closure_snapshot_issues`, `closure_snapshot_linker_contract_hash`,
`closure_snapshot_container_hash`, `closure_snapshot_payload_size_bytes`, and
`closure_snapshot_payload_hash`.

If `nuis.nsld.link-inputs.toml` exists, closure also verifies it. A valid table
adds `verified-link-input-table` to `internal_contracts`; an invalid table adds
`link-input-table:*` entries to `unresolved`. If the table is absent, closure
reports the table state as absent without treating that absence as unresolved.

Closure also checks the prepared Nsld artifact prefix, including the object
plan, writer input snapshot, object byte/file/image dry-run reports, blocked
object emit report, optional emitted object output, writer dry-run report, and
container artifacts. Missing required artifacts are tolerated when the prefix is
contiguous, because `closure` is a route-feasibility view rather than
`prepare`; optional object output may be absent for targets that cannot yet
emit native bytes. If a later required artifact appears without its
prerequisite, `prepared_artifact_chain:*` is added to `unresolved`. If an
existing object-writer artifact no longer verifies against the current plan, or
the object output exists but no longer matches the image dry-run bytes, the
matching `object-*:*` unresolved entry is reported; verified artifacts are
surfaced as `verified-*` internal contracts.
The closure snapshot, final-stage plan, and final-executable blocked report are
not part of this self-checking prepared-prefix view; they are chain-tail
snapshots or diagnostics validated by their own `verify-*` commands and by
`check` when present. This keeps `linker_contract_hash` from recursively
depending on reports derived from the closure itself.
The closure report and snapshot also expose `container_layout_hash`,
`container_hash`, `payload_size_bytes`, and `payload_hash` as linker assembly
anchors. These are not backend-specific object records; they are stable Nsld
container/payload summaries that later nsld, nsdb, cache, and linker steps can
compare before deciding whether to reuse or inspect heavier artifacts.
When an object-image dry-run exists, closure also surfaces the relocation
lowering registry through `object_image_relocation_lowering_valid`,
`object_image_relocation_lowering_rule_count`,
`object_image_relocation_lowering_rules`, and
`object_image_relocation_lowering_issues`, plus
`object_image_relocation_record_count` and
`object_image_relocation_record_table_hash` and
`object_image_relocation_records`, so the linker feasibility view can audit
seed-kind-to-target relocation policy and actual backend relocation records
without re-opening the dry-run TOML.
When that dry-run verifies and the record table hash is present, closure also
adds `verified-object-image-relocation-record-table` to `internal_contracts`.

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
and `container_loader_readiness`. It also reports the expected compatibility
domain summary: count, table hash, domain id, domain kind, paradigm, lifecycle
hook, ABI family, wrapper policy, and required flag. These are derived from the
current link plan and do not require `nuis.nsld.container` to have been emitted
yet; they give route-planning tools the same container fingerprint and
CFFI/host-compat domain identity used by `container`, `emit-container`,
`prepare`, and `check`.

## Final-Stage Plan

`nsld final-stage-plan` is the deterministic boundary immediately before a
runnable host or Nuis-native executable exists. It does not invoke clang, lld,
or a future self-owned Nsld linker. Instead, it records the final-stage
contract that such a linker must consume:

* final-stage kind, driver, link mode, and output path
* required inputs: Nsld container, container payload, closure snapshot, and
  optional/required native object output
* container hash, payload hash, and linker contract hash
* compatibility mode, currently `host-assisted-wrapper` when the host toolchain
  remains part of finalization
* blockers such as `self-owned-final-native-linker` or missing final-stage
  inputs

`nsld emit-final-stage-plan` materializes this to
`nuis.nsld.final-stage-plan.toml`; `nsld verify-final-stage-plan` re-computes
the current plan and reports focused drift such as `plan_hash mismatch` or
`input_count mismatch`. It also checks the `[[final_stage_input]]` entry count
and input id list so final-stage inputs cannot be removed or retargeted without
an explicit verification failure. The final-stage `blockers` and `notes`
arrays are compared as arrays as well, so gate reasons and compatibility notes
cannot drift silently. This gives Nsld, cache tooling, and future nsdb a stable
surface for the executable boundary before real linker execution is owned.
Internally, the alpha implementation now mirrors that protocol boundary:
`final_stage.rs` is only the final-stage-plan frontdoor, while readiness,
writer input, host finalizer gates, layout, dry-run image assembly, and blocked
emission live in separate `final_executable_*` stage modules. That split is not
just cleanup; it keeps future Mach-O, ELF, PE/COFF, and Nuis-native writers from
turning the final-stage plan into a backend-specific dispatcher.
`nsld final-executable-readiness` is a non-mutating query for the next boundary:
it reports the would-be output path, blocked-report path, final-stage plan hash,
driver, link mode, writer kind/status, writer blockers, blocker list, and notes
without writing any artifact.
`nsld final-executable-writer-plan` expands that readiness boundary into the
non-mutating writer contract: final-stage inputs, writer steps, writer
kind/status, writer blockers, and notes. It is intentionally a plan rather than
an emitter so alpha-0.8.x can make the final executable writer explicit before
committing Mach-O, ELF, or PE-specific object assembly behavior.
`nsld emit-final-executable-writer-input` materializes that writer contract as
`nuis.nsld.final-executable-writer-input.toml`; `nsld
verify-final-executable-writer-input` re-renders it and checks the writer input
hash, final-stage plan hash, writer identity, writer status, and planned command
argument count and values, plus the `writer_blockers` array. The current host-assisted command input records the
selected driver, target triple, native object, and output path, but still does
not invoke the host linker.
`nsld final-executable-host-dry-run` consumes the verified writer input,
resolves the selected host driver through an explicit path or `PATH`, and
reports `environment_ready`, `driver_available`, `driver_resolved_path`, and
the exact command arguments. It is non-mutating and never invokes the host
driver; alpha-0.8.x reports `invocation_policy = "dry-run-only"` with
`alpha-host-finalizer-execution-disabled` by default so Nsld cannot
accidentally execute a host linker before the execution policy is deliberately
changed. `NUIS_NSLD_HOST_FINALIZER_POLICY=allow-host-invoke` (or `allow`) lets
the report recognize the next policy state as `allow-host-invoke`; an empty,
missing, or `dry-run-only` value stays blocked, and any other value is reported
as `host-finalizer-policy-env:invalid:NUIS_NSLD_HOST_FINALIZER_POLICY`. This
policy switch only changes the audited plan state: the dry-run command remains
non-mutating and still never spawns the host driver. The command exists to make
the final host-assisted call boundary auditable before Nsld is allowed to
execute it.
`nsld final-executable-host-invoke-plan` sits one step after dry-run and before
any future mutating host finalizer invocation. It reuses the dry-run command
shape, reports `invocation_kind = "host-finalizer-command"`, requires an
explicit allow gate, and reports whether `NUIS_NSLD_ALLOW_HOST_FINALIZER` is
present with an allow-like value (`1`, `true`, `yes`, or `allow`). By default
`explicit_allow_present = false` and `would_invoke = false`. Even when the
explicit allow marker and `NUIS_NSLD_HOST_FINALIZER_POLICY=allow-host-invoke`
are both present, alpha-0.8.x remains non-mutating because the writer blockers
still prevent host linker execution. The command records
`host-invoke-plan-is-non-mutating` and
`host-finalizer-process-is-not-spawned`, adds
`host-finalizer-explicit-allow:missing` to blockers when that marker is absent,
and never spawns the host driver. This gives alpha-0.8.x a stable audit surface
for the future transition from “known finalizer command” to “deliberately
invoked finalizer command” without weakening the current execution gate.
`nsld emit-final-executable-host-invoke-plan` writes the same gate as
`nuis.nsld.final-executable-host-invoke-plan.toml`, and
`nsld verify-final-executable-host-invoke-plan` re-renders it to catch content
drift plus focused gate drift in `invocation_policy`,
`requires_explicit_allow`, `explicit_allow_present`, `would_invoke`,
`command_arg_count`, `command_args`, `blocker_count`, and `blockers`. This artifact is
optional in the chain for now, like the
writer input and blocked executable report, but it gives CI and future nsdb a
stable file-backed record of why the host finalizer was not allowed to run.
`nsld final-executable-layout` describes the Nsld-owned executable assembly
boundary without writing a native executable. It records the platform envelope
as a compatibility shell, the internal `nuis-hetero-unified-binary` format,
the `on_process_start` lifecycle entry hook, deterministic scheduler and
data-segment ordering contracts, the native-object compatibility domain, and
the payload list that will eventually be assembled into the unified binary.
The layout keeps a simple payload-name array for scripting and also emits
structured `[[payload]]` entries with stable payload id, kind, lifecycle hook,
path, content hash, required flag, and present flag for future linker and nsdb
consumers. It also derives a deterministic internal byte map with
`byte_alignment`, `byte_span`, `byte_map_hash`, and `[[byte_map_entry]]`
records containing payload offset, size, alignment, and content hash.
`nsld emit-final-executable-layout` writes this protocol snapshot as
`nuis.nsld.final-executable-layout.toml`; `nsld verify-final-executable-layout`
re-renders it and checks focused drift in `layout_hash`, `payload_count`,
the payload-name array, `[[payload]]` entry count, `[[byte_map_entry]]` entry
count, `byte_span`, `byte_map_hash`, `lifecycle_entry_hook`, and
`platform_envelope_family`. This keeps Nsld's
internal executable protocol decoupled from Mach-O, ELF, PE/COFF, or any
future Nuis-native shell writer.
`nsld final-executable-image-dry-run` consumes that layout protocol and builds
the current Nsld-owned internal image bytes without pretending to be a platform
executable. Required payloads must be present; optional payloads may stay absent.
The emitted `.bin` starts with a fixed 64-byte `NUIFIMG` dry-run header carrying
version, header size, payload count, byte alignment, payload span, payload
offset, `layout_hash`, and `byte_map_hash`. Payload bytes then begin at
`payload_byte_offset` and keep the offsets described by the layout byte map.
The report records `image_format`, `image_magic`, `image_header_size`,
`payload_byte_offset`, `payload_byte_span`, `layout_hash`, `byte_map_hash`,
`payload_count`, `byte_span`, `image_size_bytes`, and `image_hash`.
`nsld emit-final-executable-image-dry-run` writes both
`nuis.nsld.final-executable-image-dry-run.toml` and
`nuis.nsld.final-executable-image-dry-run.bin`; `verify-final-executable-image-dry-run`
re-renders the report, parses the `.bin` header, and hashes the full image to
catch byte drift. Focused verification fields include header magic, version,
header size, payload offset, payload span, header `layout_hash`, and header
`byte_map_hash`. It also verifies the payload region behind the header against
the layout byte map, reporting payload-region count/hash drift and focused
payload entry hash drift when a payload byte range no longer matches its source
payload hash. The TOML report-side `blockers` array is also compared against
the expected dry-run blockers so report drift cannot hide why an image was not
ready. This is the first file-backed final-image assembly checkpoint:
still not a runnable Mach-O/ELF/PE binary, but no longer only a layout plan.
`nsld emit-final-executable` materializes the same readiness shape. For
host-assisted routes it still does not fabricate a runnable executable and does
not call the host toolchain; instead it writes
`nuis.nsld.final-executable.blocked.toml` with `emitted = false`. For
self-contained routes, once writer input, layout, and image dry-run snapshots
all verify, it copies the Nsld-owned final executable image bytes to the
final-stage output path and records `emitted = true`. This first mutating
writer path produces a Nuis-owned unified binary image, not a Mach-O/ELF/PE
host executable. The emit path now also consumes
`verify-final-executable-writer-input`: if the writer input is missing or has
drifted, the blocked report records `writer_input_valid = false`,
`writer_input_issues`, and a `final-executable-writer-input:invalid` blocker.
It also consumes `final-executable-host-dry-run`, recording
`host_dry_run_environment_ready`, `host_dry_run_driver_available`,
`host_dry_run_driver_resolved_path`, `host_dry_run_can_invoke`, and
`host_dry_run_blockers` in the same blocked report. The exact dry-run command
shape is also copied as `host_dry_run_command_arg_count` and
`host_dry_run_command_args`. The dry-run invocation policy is also copied into
the blocked report as
`host_dry_run_invocation_policy` and
`host_dry_run_invocation_policy_reason`, preserving the alpha execution gate at
the executable boundary. The blocked report now also consumes
`verify-final-executable-host-invoke-plan`: missing or drifted invoke-plan
snapshots add `host-finalizer-invoke-plan:invalid`, and the current missing
explicit allow gate adds `host-finalizer-invoke-plan:not-allowed`. The report
records `host_invoke_plan_valid`, `host_invoke_plan_hash`,
`host_invoke_plan_invocation_policy`,
`host_invoke_plan_requires_explicit_allow`,
`host_invoke_plan_explicit_allow_present`, `host_invoke_plan_would_invoke`,
`host_invoke_plan_blocker_count`, and `host_invoke_plan_issues` so the final
blocked executable boundary proves it did not bypass the invocation gate and
does not require consumers to reopen the invoke-plan artifact just to understand
which gate remained closed.
The blocked report also consumes `verify-final-executable-layout`: missing or
drifted layout snapshots add `final-executable-layout-plan:invalid`, and the
report records `layout_plan_valid`, `layout_plan_hash`, and
`layout_plan_issues` so final executable emission is tied to the Nsld-owned
layout protocol rather than only to host linker arguments.
It also consumes `verify-final-executable-image-dry-run`: missing or drifted
internal image snapshots add `final-executable-image-dry-run:invalid`, and the
report records `image_dry_run_valid`, `image_dry_run_hash`,
`image_dry_run_size_bytes`, and `image_dry_run_issues`. This makes the final
blocked boundary depend on the actual Nsld-owned image checkpoint, not only on
the abstract byte-map layout.
`nsld verify-final-executable-emit` re-computes that blocked report and catches
drift in fields such as `final_stage_plan_hash`, `emitted`, and
`blocker_count` plus the final `blockers` array; it also reports focused drift for writer input fields such as
`writer_input_valid`, `writer_input_hash`, and `writer_input_issues`, host
dry-run readiness fields such as `host_dry_run_environment_ready` and
`host_dry_run_driver_available`, plus invocation fields such as
`host_dry_run_can_invoke`, `host_dry_run_driver_resolved_path`,
`host_dry_run_invocation_policy`, `host_dry_run_invocation_policy_reason`, and
`host_dry_run_command_args`, plus `host_dry_run_blockers`, host invoke-plan
fields such as `host_invoke_plan_valid`, `host_invoke_plan_hash`,
`host_invoke_plan_invocation_policy`,
`host_invoke_plan_requires_explicit_allow`,
`host_invoke_plan_explicit_allow_present`,
`host_invoke_plan_would_invoke`, `host_invoke_plan_blocker_count`, and
`host_invoke_plan_issues`, and layout fields such
as `layout_plan_valid`, `layout_plan_hash`, and `layout_plan_issues`, plus
image dry-run fields such as `image_dry_run_valid`, `image_dry_run_hash`,
`image_dry_run_size_bytes`, and `image_dry_run_issues`. The
blocked report also includes
`host_dry_run_blocker_count` so scripts can check the host-finalizer blocker
summary without parsing the blocker array. The readiness report currently exposes
`writer_kind = "host-assisted-final-executable"`,
`writer_status = "blocked"`, and
`final-executable-writer:host-assisted:not-implemented` as the writer-level
blocker for the default host-toolchain route. This keeps final executable
readiness scriptable without binding Nsld to one platform object format.
Self-contained routes report `writer_status = "ready"` when the final-stage
inputs are ready and the writer has no own blockers.
`nsld final-executable-output` is the read-only boundary for the real runnable
candidate. It inspects the final output path from the final-stage plan and
reports presence, size, and hash when a file exists. It keeps explicit blockers
when the final-stage plan or final-executable emit report is invalid, when the
blocked emit report still says `emitted = false`, or when the output file is
missing. That separates "protocol snapshots are internally consistent" from "a
controlled executable file actually exists".
`nsld artifact-chain` also mirrors this as a read-only final-output boundary
hint with `final_output_boundary_ready`,
`final_output_boundary_command_id`,
`final_output_boundary_command_resolved`, `final_output_boundary_reason`, and
`final_output_boundary_blockers`. This hint intentionally does not replace the
mutating `next_action_*` fields consumed by `nsld drive --apply`; it points
operators and scripts to `nsld final-executable-output <manifest>` when the
artifact chain has no more safe apply step but the final output boundary is
still blocked.
`nsld check` mirrors the same hint with the
`artifact_chain_final_output_boundary_*` fields so check-oriented scripts can
see the read-only output boundary without treating it as the drive next action.
The final-stage plan file is also checked by `nsld check`, which exposes
`final_stage_plan_present`, `final_stage_plan_valid`,
`final_stage_plan_ready`, `final_stage_plan_hash`,
`final_stage_plan_blocker_count`, and `final_stage_plan_issues`. As with the
standalone plan report, `ready = false` is informational; only verification
drift fails the top-level check.
If `nuis.nsld.final-executable-writer-input.toml` is present, `nsld check`
verifies it independently and exposes
`final_executable_writer_input_present`,
`final_executable_writer_input_valid`,
`final_executable_writer_input_hash`,
`final_executable_writer_input_command_arg_count`, and
`final_executable_writer_input_issues`. This keeps the final host-assisted
command input auditable before the invoke-plan gate consumes it.
If `nuis.nsld.final-executable-host-invoke-plan.toml` is present, `nsld check`
verifies it independently and exposes
`final_executable_host_invoke_plan_present`,
`final_executable_host_invoke_plan_valid`,
`final_executable_host_invoke_plan_hash`,
`final_executable_host_invoke_plan_invocation_policy`,
`final_executable_host_invoke_plan_requires_explicit_allow`,
`final_executable_host_invoke_plan_explicit_allow_present`,
`final_executable_host_invoke_plan_would_invoke`,
`final_executable_host_invoke_plan_blocker_count`, and
`final_executable_host_invoke_plan_issues`. This keeps the execution gate
observable even before `emit-final-executable` folds the gate into the blocked
final executable report.
If `nuis.nsld.final-executable-layout.toml` is present, `nsld check` verifies
it independently and exposes `final_executable_layout_plan_present`,
`final_executable_layout_plan_valid`, `final_executable_layout_plan_hash`,
`final_executable_layout_plan_payload_count`, and
`final_executable_layout_plan_issues`. This gives CI a stable check for the
Nsld-owned binary layout protocol before real platform executable emission
exists.
If `nuis.nsld.final-executable-image-dry-run.toml` is present, `nsld check`
verifies it independently and exposes
`final_executable_image_dry_run_present`,
`final_executable_image_dry_run_valid`,
`final_executable_image_dry_run_hash`,
`final_executable_image_dry_run_size_bytes`, and
`final_executable_image_dry_run_issues`. Verification also checks the paired
`nuis.nsld.final-executable-image-dry-run.bin` when the report contains an
expected image hash.
If `nuis.nsld.final-executable.blocked.toml` is present, `nsld check` also
verifies it and exposes `final_executable_blocked_present`,
`final_executable_blocked_valid`, `final_executable_blocked_emitted`,
`final_executable_blocked_plan_hash`,
`final_executable_blocked_blocker_count`, and
`final_executable_blocked_issues`. The file remains optional so `prepare` can
stay a deterministic artifact preparation step while final executable emission
remains an explicit boundary command.
If the Nsld-owned final-stage output is present, `nsld check` runs the same
read-only output boundary exposed by `final-executable-output` and reports
`final_executable_output_path_present`,
`final_executable_output_boundary_status`,
`final_executable_output_materialization_status`,
`final_executable_output_execution_handoff_status`,
`final_executable_output_execution_handoff_target`,
`final_executable_output_execution_handoff_evidence_status`,
`final_executable_output_recommended_next_action`,
`final_executable_output_nsld_owned`,
`final_executable_output_present`,
`final_executable_output_size_bytes`,
`final_executable_output_hash`,
`final_executable_output_runnable_candidate`,
`final_executable_output_blocker_count`,
`final_executable_output_blockers`, and
`final_executable_output_issues`. The standalone `final-executable-output`
report distinguishes `path_present` from `nsld_owned_output`: a host-native
binary can exist at the final-stage path without being treated as a Nuis image
emitted by Nsld. Output blockers distinguish `final-executable-output:missing`
from `final-executable-output:not-nsld-owned` and
`final-executable-output:unreadable`, because those are different linker
states. A missing Nsld-owned final output remains non-fatal so ordinary
preparation workflows can stop before the explicit final emit step, but a
present Nsld-owned output must be consistent with the emitted final-executable
boundary.
Scripts should prefer the normalized `boundary_status` /
`final_executable_output_boundary_status` field for high-level branching, use
`materialization_status` /
`final_executable_output_materialization_status` to distinguish host-native from
self-contained image readiness, use `execution_handoff_status` /
`final_executable_output_execution_handoff_status` to distinguish runner-ready
outputs from outputs that still need an entrypoint materializer, use
`execution_handoff_target` /
`final_executable_output_execution_handoff_target` to name the abstract owner of
that handoff, use `execution_handoff_evidence_status` /
`final_executable_output_execution_handoff_evidence_status` to identify the
proof class backing the handoff, and use
`recommended_next_action` /
`final_executable_output_recommended_next_action` for the next scripted boundary
step. Blockers and issues remain diagnostic detail.

When `nuis.nsld.final-executable-pipeline.toml` is present, `nsld check` also
mirrors the pipeline's self-owned image summary through
`final_executable_pipeline_self_owned_image_status` and the next entrypoint
layer through
`final_executable_pipeline_entrypoint_materialization_status`. The pipeline
emit/verify JSON uses the shorter `self_owned_image_status` /
`actual_self_owned_image_status` and `entrypoint_materialization_status` /
`actual_entrypoint_materialization_status` fields. Image status is intentionally
narrower than entrypoint readiness: it only answers whether the internal `.nsb`
image layer is present, hashed, and header-valid.

`nsld prepare` also returns the same compatibility-domain summary after it has
emitted and verified the full artifact chain. This makes the prepare result a
single frontdoor for both "which files were written?" and "which host-compat
execution paradigm did the prepared container admit?" It also reports canonical
paths for `final_executable_writer_input_path`,
`final_executable_host_invoke_plan_path`, and
`final_executable_layout_plan_path`, and
`final_executable_image_dry_run_path`,
`final_executable_image_dry_run_bytes_path`, and
`final_executable_blocked_path`, but it does not emit those final-executable
boundary files. Those remain explicit boundary commands so `prepare` cannot
accidentally advance the host finalizer gate.

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

## Alpha-0.8.* Meaning

For `alpha-0.10.*`, success means:

* Nsld artifact-chain diagnostics can explain missing stages and suggested
  commands
* object/container/closure/final-stage artifacts remain deterministic and
  checkable
* final executable readiness describes the exact blocker before real execution
* the next implementation work is pointed at a minimal runnable host-assisted
  route or explicit blocked executable artifact

This is the convergence line for binary linking. It should still avoid claiming
that the self-owned production linker is finished.
