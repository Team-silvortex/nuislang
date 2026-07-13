# nuislang

> AOT-first heterogeneous systems language and toolchain built around
> `nuis -> NIR -> YIR -> LLVM/AOT`, with `nustar` packages registering
> per-domain parsing, lowering, ABI contracts, artifacts, and verification
> surfaces.

## Current Status

The repository is on the `alpha-0.10.*` line. This is still an architecture
building line, not a beta-stability line, but the project now has one connected
compiler/toolchain spine instead of separate experimental islands.

Current spine:

```text
nuis source / nuis.toml project
  -> nuis frontdoor
  -> nuisc
  -> NIR
  -> YIR
  -> LLVM / AOT artifacts
  -> nsld binary-linking convergence
```

The current `alpha-0.10.*` goal is an honest minimal executable-artifact loop:

```text
project
  -> compiled artifact
  -> object image / compatibility object
  -> container + payload
  -> closure snapshot
  -> final-stage plan
  -> executable writer input
  -> runnable host-assisted artifact or explicit blocked executable artifact
```

This does not yet mean final self-hosting, final std API stability, or a fully
self-owned linker. Safe current wording is `binary-linking convergence`,
`executable-artifact closure`, `minimal runnable route`, and
`host-assisted finalization` where applicable.

Start here for the current line:

* [docs/current-mainline-map.md](docs/current-mainline-map.md)
* [docs/versioning/nuis-alpha-0.10-mainline-entry.md](docs/versioning/nuis-alpha-0.10-mainline-entry.md)
* [docs/versioning/nuis-alpha-0.8-mainline-entry.md](docs/versioning/nuis-alpha-0.8-mainline-entry.md)
* [docs/versioning/nuis-alpha-0.8-doc-sync-inventory.md](docs/versioning/nuis-alpha-0.8-doc-sync-inventory.md)
* [docs/reference/nsld-linker-frontdoor.md](docs/reference/nsld-linker-frontdoor.md)
* [docs/reference/nsld-binary-assembly-gap-map.md](docs/reference/nsld-binary-assembly-gap-map.md)
* [docs/reference/nuis-native-artifact-workflow.md](docs/reference/nuis-native-artifact-workflow.md)
* [docs/reference/nustar-multi-backend-artifact-contract.md](docs/reference/nustar-multi-backend-artifact-contract.md)
* [docs/reference/toolchain-galaxy-core-boundary.md](docs/reference/toolchain-galaxy-core-boundary.md)

Older alpha and pre-alpha files are still useful, but they are predecessor or
baseline context. Use [docs/versioning/README.md](docs/versioning/README.md)
when you intentionally need history.

## What Exists Now

Implemented or actively usable surfaces:

* `nuis` project workflow commands for `workflow`, `project-doctor`, `check`,
  `test`, `build`, artifact inspection, and release checks.
* `nuisc` compiler core with parser/frontend, NIR/YIR generation, verifier
  checks, LLVM lowering, AOT artifact emission, and project metadata.
* `nustar` registration for the main domain set: `cpu`, `data`, `shader`,
  `kernel`, `network`, plus CFFI/host-compatibility boundaries.
* `YIR` as the central semantic execution boundary, with result-family,
  artifact, clock, ABI, and domain validation surfaces.
* `nsld` as the linker frontdoor over the current linker core, with object,
  container, closure, final-stage, and readiness diagnostics.
* `nsdb` as the YIR-level debugger metadata frontdoor.
* `nsbdr` as the future OS bundle/distribution frontdoor over final `nsld`
  outputs.
* Standard-library source assets under `stdlib/core`, `stdlib/std`,
  `stdlib/pixelmagic`, `stdlib/witsage`, and `stdlib/ns-nova`.
* Project examples covering control flow, task/thread/lock surfaces,
  filesystem/IO/text/tooling, PixelMagic image lanes, WitSage classical ML
  contracts, network profiles, shader/kernel domain artifacts, and native
  artifact closure.

Still incomplete or intentionally soft:

* final self-owned executable linking
* stable std import/autoinjection semantics
* complete unsafe/raw-pointer policy
* final GPU/NPU backend maturity
* beta-level public API stability
* self-hosting

## Quick Start

Use `nuis workflow` first when you are not sure which command should own the
next step:

```bash
cargo run -p nuis -- workflow examples/projects/window_controls_demo
cargo run -p nuis -- project-doctor examples/projects/window_controls_demo
cargo run -p nuis -- check examples/projects/window_controls_demo
cargo run -p nuis -- test examples/projects/window_controls_demo
cargo run -p nuis -- build examples/projects/window_controls_demo examples/bins/window_controls_demo_project
cargo run -p nuis -- release-check examples/projects/window_controls_demo examples/bins/window_controls_demo_project_release
```

Native artifact closure route:

```bash
cargo run -p nuis -- workflow examples/projects/tooling/native_artifact_closure_demo
cargo run -p nuis -- build \
  examples/projects/tooling/native_artifact_closure_demo \
  examples/bins/native_artifact_closure_demo_project
cargo run -p nuis -- inspect-artifact \
  examples/bins/native_artifact_closure_demo_project/nuis.build.manifest.toml
cargo run -p nuis -- verify-artifact \
  examples/bins/native_artifact_closure_demo_project/nuis.compiled.artifact
cargo run -p nuis -- artifact-doctor \
  examples/bins/native_artifact_closure_demo_project
cargo run -p nsld -- drive \
  examples/bins/native_artifact_closure_demo_project/nuis.build.manifest.toml --json
cargo run -p nsld -- drive \
  examples/bins/native_artifact_closure_demo_project/nuis.build.manifest.toml --apply
cargo run -p nsld -- drive \
  examples/bins/native_artifact_closure_demo_project/nuis.build.manifest.toml --apply --json
cargo run -p nsld -- drive \
  examples/bins/native_artifact_closure_demo_project/nuis.build.manifest.toml --apply --until-clean
cargo run -p nsld -- drive \
  examples/bins/native_artifact_closure_demo_project/nuis.build.manifest.toml --apply --until-clean --json
cargo run -p nuis -- run-artifact \
  examples/bins/native_artifact_closure_demo_project/nuis.build.manifest.toml
```

`nsld drive` without `--apply` is a non-mutating dry run. `--apply` writes at
most one whitelisted linker artifact step, while `--apply --until-clean` keeps
applying whitelisted steps until the chain is clean, blocked, repeated, or
capped. Add `--json` to any drive mode when a script needs the structured
`mutates_artifacts` and next-action/status fields.

`nuis release-check` also reports the `nsld-drive-command-set-v1` summary after
build and artifact self-checks pass. It does not mutate linker artifacts on its
own; use the reported dry-run JSON command before handing off to an applying
`nsld drive` mode.

Explicit CPU target examples:

```bash
cargo run -p nuis -- build --cpu-abi cpu.arm64.apple_aapcs64 \
  examples/projects/kernel_tensor_demo \
  examples/bins/kernel_tensor_demo_project

cargo run -p nuis -- build --target aarch64-apple-darwin \
  examples/projects/kernel_tensor_demo \
  examples/bins/kernel_tensor_demo_project
```

Useful inspection commands:

```bash
cargo run -p nuis -- dump-ast examples/projects/window_controls_demo
cargo run -p nuis -- dump-nir examples/projects/window_controls_demo
cargo run -p nuis -- dump-yir examples/projects/window_controls_demo
cargo run -p nuis -- project-status examples/projects/window_controls_demo
cargo run -p nuis -- artifact-doctor examples/bins/window_controls_demo_project
cargo run -p nuis -- verify-build-manifest examples/bins/window_controls_demo_project/nuis.build.manifest.toml
```

## Toolchain Boundaries

```text
nuis   -> workflow/project frontdoor
nuisc  -> compiler core and AOT artifact producer
nsld   -> linker frontdoor and binary assembly convergence surface
nsdb   -> YIR semantic debugger frontdoor
nsbdr  -> OS bundle/distribution adapter for final nsld outputs
yir-*  -> lower-level YIR inspection, packing, running, and export tools
```

`nsld`, `nsdb`, and `nsbdr` should remain command adapters over reusable
toolchain capabilities. They should not become isolated CLI-only logic piles.
See [docs/reference/toolchain-galaxy-core-boundary.md](docs/reference/toolchain-galaxy-core-boundary.md).

The C world is treated as a compatibility domain, not as the hidden default
machine model. Current CFFI direction is documented in
[docs/reference/cffi-von-neumann-domain-contract.md](docs/reference/cffi-von-neumann-domain-contract.md)
and [docs/reference/ffi-pointer-safety-boundary.md](docs/reference/ffi-pointer-safety-boundary.md).

## Nustar And Heterogeneous Domains

`nustar` packages are the registration boundary for domain-specific knowledge.
The compiler may know the shape of `nustar` contracts, but it should avoid
hardcoding every domain's internal logic.

Current core domain set:

* `cpu`: scalar/control-flow/task/thread/lock/host-compatible execution spine
* `data`: fabric, handles, markers, movement, windows, pipes, observers
* `shader`: shader artifact and graphics/image-processing pressure surface
* `kernel`: compute-kernel and future ML/NPU pressure surface
* `network`: profile/session/flow and host-backed networking contracts
* `cffi`: compatibility boundary for libc/C ABI/classic host object support

Important references:

* [docs/reference/nustar-capability-split-boundary.md](docs/reference/nustar-capability-split-boundary.md)
* [docs/reference/nustar-multi-backend-artifact-contract.md](docs/reference/nustar-multi-backend-artifact-contract.md)
* [docs/reference/std-shader-kernel-project-contract.md](docs/reference/std-shader-kernel-project-contract.md)
* [docs/reference/network-domain-contract.md](docs/reference/network-domain-contract.md)

## Standard Library And Official Galaxies

The standard library is not a final crate-like automatic import tree yet, but it
is no longer empty scaffolding.

Current layers:

```text
core -> std -> pixelmagic
core -> std -> witsage
core -> std -> ns-nova
```

* `core`: smallest semantic/source-contract base
* `std`: systems helpers for task, host, IO, filesystem, text, networking,
  tooling, persistence, result/error, and benchmark/report lanes
* `pixelmagic`: official image/resource Galaxy and shader-facing image pipeline
  pressure surface
* `witsage`: official classical ML Galaxy and kernel-facing model/statistics
  pressure surface
* `ns-nova`: future GUI/render/application framework Galaxy, intentionally
  later than AOT/std/PixelMagic/WitSage hardening

Read [stdlib/README.md](stdlib/README.md) first, then
[stdlib/std/README.md](stdlib/std/README.md),
[stdlib/pixelmagic/README.md](stdlib/pixelmagic/README.md),
[stdlib/witsage/README.md](stdlib/witsage/README.md), and
[stdlib/ns-nova/README.md](stdlib/ns-nova/README.md).

## Examples

Use [examples/README.md](examples/README.md) as the router. The current default
example layer is `examples/projects`, not every older source snippet.

High-signal routes:

* control-flow: `examples/projects/state/*`
* task/thread/lock: `examples/projects/task/*`
* tooling/std: `examples/projects/tooling/*`
* domain demos: `examples/projects/domains/*`
* source-level companions: `examples/ns/*`
* handwritten YIR anchors: `examples/yir/*`
* invalid/verifier cases: `examples/invalid/*`

## Development Checks

Common checks for this workspace:

```bash
cargo test -q -p nuisc --lib --no-run
cargo test -q -p nuisc --test file_line_limit --no-fail-fast
cargo test -q -p yir-lower-llvm --lib --no-run
cargo clippy -q -p yir-lower-llvm --lib --tests -- -D warnings
cargo fmt --check
git diff --check
```

The repository has an active file-size hygiene policy. Current large-file work
is reducing `crates/yir-lower-llvm/src/lib.rs` by extracting focused lowering
modules such as scalar arithmetic, scalar comparisons, guard returns, simple
loops, carry payload parsing, value types, bitwise lowering, params, and select.
See [docs/repo-file-line-policy.md](docs/repo-file-line-policy.md).

## Long-Range Direction

Nuis is intentionally not designed as a classic C-shaped systems language with a
thin new syntax layer. The long-range direction is a Nuis-owned heterogeneous
computing stack: AOT-first today, linker/debugger/bundler convergence next,
eventually self-hosting before beta, and later Nuis OS / XR heterogeneous
workstation ideas without binding the architecture too tightly to libc or a
single von-Neumann host model.

Long-range design notes:

* [docs/versioning/nuis-long-range-heterogeneous-os-roadmap.md](docs/versioning/nuis-long-range-heterogeneous-os-roadmap.md)
* [docs/glm-spec/glm-heterogeneous-flow-graph-positioning.md](docs/glm-spec/glm-heterogeneous-flow-graph-positioning.md)
* [docs/glm-spec/vulpoya-yir-secondary-review-positioning.md](docs/glm-spec/vulpoya-yir-secondary-review-positioning.md)
* [docs/historical/README.md](docs/historical/README.md)
