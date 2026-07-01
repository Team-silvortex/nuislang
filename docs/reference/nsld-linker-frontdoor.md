# Nsld Linker Frontdoor

`Nsld` is the Nuis linker toolchain member introduced on the `alpha-0.6.0`
line.

At this stage, `Nsld` is intentionally a frontdoor over the existing linker
contract logic in `nuisc::linker`. It does not yet claim to be the final
self-owned object linker. Its job is to give linker work a stable tool
boundary before the implementation is split out further.

## Current Role

`Nsld` currently owns:

* link-plan inspection from `nuis.build.manifest.toml`
* heterogeneous calculate plan visibility
* clock protocol visibility
* lowering sidecar capability validation for domains that declare IR sidecars
* final-stage reporting
* the first independent CLI boundary for future linker work

`Nsld` does not yet own:

* final native object linking
* replacement of the host toolchain wrapper
* binary section assembly independent from `nuisc`
* stable linker script or relocation formats

## Commands

```sh
cargo run -p nsld -- status
cargo run -p nsld -- plan <nuis.build.manifest.toml>
cargo run -p nsld -- plan <artifact-output-dir> --json
cargo run -p nsld -- check <artifact-output-dir>
cargo run -p nsld -- check <artifact-output-dir> --json
cargo run -p nsld -- closure <artifact-output-dir>
cargo run -p nsld -- closure <artifact-output-dir> --json
cargo run -p nsld -- inputs <artifact-output-dir>
cargo run -p nsld -- inputs <artifact-output-dir> --json
cargo run -p nsld -- verify-inputs <artifact-output-dir>
cargo run -p nsld -- verify-inputs <artifact-output-dir> --json
```

When given an output directory, `Nsld` resolves
`nuis.build.manifest.toml` inside that directory.

## Linker Check

`nsld check` is the first dedicated linker gate. It currently verifies:

* artifact lowering alignment is consistent
* clock protocol validation passed
* hetero calculate validation passed
* hetero calculate plan is static-link
* hetero calculate plan is lifecycle-driven
* lowering sidecar capabilities are readable and link-ready for domains that
  declare `artifact_ir_sidecar_path`

The command exits with failure when any linker gate fails. JSON output is
intended for CI and future toolchain orchestration.

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

This is not final object linking yet; it is the linker-owned input table that
future binary assembly, cache reuse, debug-symbol correlation, and closure
verification can consume.

## Link Input Table Artifact

`nsld inputs` materializes the closure link input table to:

```text
nuis.nsld.link-inputs.toml
```

The emitted table currently uses:

```toml
schema = "nuis-nsld-link-input-table-v1"
table_kind = "lowering-sidecar-link-inputs"
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

This is the beginning of linker independence, not the end of linker work.
