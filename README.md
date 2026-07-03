# nuislang

> AOT-first heterogeneous systems language and toolchain built around `nuis -> NIR -> YIR -> LLVM/AOT`, with `nustar` packages providing per-domain parsing, lowering, ABI contracts, and verification surfaces.

## Current Status

The repository is currently on the `alpha-0.7.*` line. It is still in an active
architecture-building stage, but the mainline is now expected to describe one
connected toolchain rather than a pile of promising surfaces.

The most stable current spine is:

```text
nuis source / project
  -> nuisc
  -> NIR
  -> YIR
  -> LLVM / AOT packaging
```

The key thing that is already real today is not “all language features are done”, but that the project now has one increasingly consistent execution model across:

* `cpu`
* `data`
* `shader`
* `kernel`
* `network`

That model is increasingly enforced through `YIR` contracts, project validation, per-domain `nustar` manifests, and verifier checks rather than only ad hoc frontend rules.

The newest `alpha-0.7.*` emphasis is:

* std-backed tooling demos as the default smoke surface across CLI, IO,
  filesystem, text/JSON, time/benchmark, result/diagnostic, and hetero proxy
  lanes
* examples promoted from raw probe totals to contract-level `ok/error` exits
  while still preserving report totals for inspection
* continued linker/lowering pressure through `nsld`, native artifacts, and
  heterogeneous closure metadata

Current versioning entrypoints:

* current `alpha-0.7.*` mainline entry:
  [docs/versioning/nuis-alpha-0.7-mainline-entry.md](docs/versioning/nuis-alpha-0.7-mainline-entry.md)
* predecessor `alpha-0.6.*` linker/std smoke entry:
  [docs/versioning/nuis-alpha-0.6-mainline-entry.md](docs/versioning/nuis-alpha-0.6-mainline-entry.md)
* current mainline router:
  [docs/current-mainline-map.md](docs/current-mainline-map.md)
* `alpha-0.4.*` hardening baseline system inventory:
  [docs/versioning/nuis-alpha-0.4-system-inventory.md](docs/versioning/nuis-alpha-0.4-system-inventory.md)
* `alpha-0.4.*` hardening baseline plan:
  [docs/versioning/nuis-alpha-0.4-mainline-hardening-plan.md](docs/versioning/nuis-alpha-0.4-mainline-hardening-plan.md)
* `alpha-0.4.*` documentation sync baseline:
  [docs/versioning/nuis-alpha-0.4-doc-sync-inventory.md](docs/versioning/nuis-alpha-0.4-doc-sync-inventory.md)
* long-range heterogeneous OS roadmap:
  [docs/versioning/nuis-long-range-heterogeneous-os-roadmap.md](docs/versioning/nuis-long-range-heterogeneous-os-roadmap.md)
* predecessor `alpha-0.1.*` status anchor:
  [docs/versioning/nuis-alpha-0.1-mainline-status.md](docs/versioning/nuis-alpha-0.1-mainline-status.md)
* predecessor alpha closeout board:
  [docs/versioning/nuis-alpha-0.0.1-closeout-board.md](docs/versioning/nuis-alpha-0.0.1-closeout-board.md)
* predecessor alpha closeout checklist:
  [docs/versioning/nuis-alpha-0.0.1-closeout-checklist.md](docs/versioning/nuis-alpha-0.0.1-closeout-checklist.md)
* previous pre-alpha snapshot anchor:
  [docs/versioning/nuis-0.19.0-snapshot.md](docs/versioning/nuis-0.19.0-snapshot.md)
* previous pre-alpha workflow anchor:
  [docs/versioning/nuis-0.19.0-compile-workflow.md](docs/versioning/nuis-0.19.0-compile-workflow.md)
* predecessor ABI compile vocabulary bridge into `0.20.*`:
  [docs/versioning/nuis-0.20.0-abi-compile-vocabulary.md](docs/versioning/nuis-0.20.0-abi-compile-vocabulary.md)
* previous pre-alpha regression gate:
  [docs/versioning/nuis-0.19.0-mainline-regression-matrix.md](docs/versioning/nuis-0.19.0-mainline-regression-matrix.md)
* previous pre-alpha address/pointer anchor:
  [docs/versioning/nuis-0.19.0-address-pointer-mainline.md](docs/versioning/nuis-0.19.0-address-pointer-mainline.md)
* previous minor-line snapshot:
  [docs/versioning/nuis-0.18.0-snapshot.md](docs/versioning/nuis-0.18.0-snapshot.md)
* historical index:
  [docs/versioning/README.md](docs/versioning/README.md)

If you want the current line first, start with
[docs/versioning/nuis-alpha-0.7-mainline-entry.md](docs/versioning/nuis-alpha-0.7-mainline-entry.md),
then use [docs/current-mainline-map.md](docs/current-mainline-map.md)
and
[docs/reference/std-mainline-layering-contract.md](docs/reference/std-mainline-layering-contract.md).
For the hardening baseline behind that current line, use
[docs/versioning/nuis-alpha-0.4-system-inventory.md](docs/versioning/nuis-alpha-0.4-system-inventory.md),
[docs/versioning/nuis-alpha-0.4-mainline-hardening-plan.md](docs/versioning/nuis-alpha-0.4-mainline-hardening-plan.md)
and
[docs/versioning/nuis-alpha-0.4-doc-sync-inventory.md](docs/versioning/nuis-alpha-0.4-doc-sync-inventory.md).

If you want the older pre-alpha history anchor after that, start with
[`0.19.0` snapshot](docs/versioning/nuis-0.19.0-snapshot.md), then use
[docs/versioning/README.md](docs/versioning/README.md) only when you intentionally need older lines.

Current source-style note:

* ordinary `.ns` examples and `std` modules now prefer address surface syntax
  such as `ptr.value`, `ptr.next`, `buffer.len`, and `buffer[index]`
* builtin names like `load_value(...)` and `store_at(...)` remain the lowered
  implementation truth in NIR/YIR-facing docs
* source-facing contract:
  [docs/reference/address-surface-contract.md](docs/reference/address-surface-contract.md)

## Toolchain

The visible workspace toolchain now includes:

* `nuis`: front-door workflow command
* `nuisc`: compiler core and AOT artifact producer
* `nsld`: alpha-0.6.0 linker frontdoor for link-plan, clock protocol, and
  heterogeneous calculate contract inspection
* `nsdb`: YIR-layer debugger metadata frontdoor; native debuggers can still
  attach to the host shell, but Nsdb owns Nuis semantic debug visibility
* `nuis-rc`: local resident control prototype
* `yir-*`: lower-level YIR inspection, packing, running, and export tools

`nsld` is currently a separate tool boundary over `nuisc::linker`, not the
finished self-owned object linker. The point of the split is to make linker
ownership explicit before the implementation is fully extracted.
Longer-term, `nsld` and `nsdb` should be CLI adapters over reusable
galaxy-style core toolchain capabilities, not CLI-only command surfaces.

Reference:
[docs/reference/nsld-linker-frontdoor.md](docs/reference/nsld-linker-frontdoor.md),
[docs/reference/nsdb-yir-debugger-frontdoor.md](docs/reference/nsdb-yir-debugger-frontdoor.md),
[docs/reference/toolchain-galaxy-core-boundary.md](docs/reference/toolchain-galaxy-core-boundary.md)

```text
nuis     -> front-door workflow tool
nuis-rc  -> resident control tool (later-stage, still intentionally thin)
nuisc    -> compiler/scheduler core
yalivia  -> hosted future JIT/runtime subproject under `subprojects/yalivia`
vulpoya  -> hosted future analyzer/verifier subproject under `subprojects/vulpoya`
```

Current responsibility split:

* `nuis` is the main workflow surface for `check`, `build`, caches, projects, and package inspection.
* `nuisc` is the compiler/scheduler core that consumes `.ns` or project inputs and emits `NIR`, `YIR`, LLVM IR, and AOT outputs.
* `nustar` packages are where per-domain ABI support, default lanes, frontend/lowering entrypoints, and package contracts are registered.
* `nustar` packages are also starting to declare per-domain clock contracts such as `clock_domain_id`, `clock_kind`, `clock_epoch_kind`, `clock_resolution`, and `clock_bridge_default`.
* `subprojects/yalivia` and `subprojects/vulpoya` are now hosted inside this repository tree as sibling ecosystem projects while their boundaries are still evolving.

## Quick Start

Recommended default compile workflow:

```bash
cargo run -p nuis -- project-doctor examples/projects/window_controls_demo
cargo run -p nuis -- check examples/projects/window_controls_demo
cargo run -p nuis -- test examples/projects/window_controls_demo
cargo run -p nuis -- build examples/projects/window_controls_demo examples/bins/window_controls_demo_project
cargo run -p nuis -- release-check examples/projects/window_controls_demo examples/bins/window_controls_demo_project_release
```

If you want the CLI to restate the shortest route for the current input first:

```bash
cargo run -p nuis -- workflow examples/projects/window_controls_demo
cargo run -p nuis -- workflow stdlib/std/net_session_recipe.ns --json
```

Useful follow-up variants:

```bash
cargo run -p nuis -- test --list examples/projects/window_controls_demo
cargo run -p nuis -- test --ignored examples/projects/window_controls_demo
cargo run -p nuis -- test --include-ignored examples/projects/window_controls_demo
cargo run -p nuis -- test --exact examples/projects/window_controls_demo smoke_add
cargo run -p nuis -- test --ignored --exact examples/projects/window_controls_demo smoke_skip
cargo run -p nuis -- project-status examples/projects/window_controls_demo
cargo run -p nuis -- project-lock-abi examples/projects/window_controls_demo
```

If you want the current native artifact closure route specifically, use:

```bash
cargo run -p nuis -- workflow examples/projects/tooling/native_artifact_closure_demo
cargo run -p nuis -- project-status examples/projects/tooling/native_artifact_closure_demo
cargo run -p nuis -- build examples/projects/tooling/native_artifact_closure_demo examples/bins/native_artifact_closure_demo_project
cargo run -p nuis -- artifact-doctor examples/bins/native_artifact_closure_demo_project
cargo run -p nuis -- run-artifact examples/bins/native_artifact_closure_demo_project/nuis.build.manifest.toml
```

Current native artifact closure reference:
[docs/reference/nuis-native-artifact-workflow.md](docs/reference/nuis-native-artifact-workflow.md)

The current rule of thumb should be:

* `project-doctor` before deep work on a project
* `check` for compile/validation truth
* `test` for language-level behavior
* `build` for artifact emission
* `release-check` before calling the result release-ready

Current language-level test declarations use:

```ns
test("smoke_add") fn smoke_add() -> i64 { return 1; }
test(ignored=true) fn skipped_case() -> i64 { return 1; }
test(should_fail=true) fn expected_failure() -> i64 { return 0; }
test("expected_failure", should_fail=true, reason="must reject zero") fn expected_failure() -> i64 { return 0; }
test("slow_async", timeout_ms=25, clock_domain="global", clock_policy="bridge") async fn slow_async() -> i64 { return 1; }
```

Timed tests already support `timeout_ms`, `clock_domain`, and
`clock_policy="bridge"`. `nuis test` reports both declared and resolved clock
metadata at run time. For the current contract and bridge semantics, read
[docs/reference/cpu-task-scheduler-clock.md](docs/reference/cpu-task-scheduler-clock.md).

Current workflow reading rule:

* use `nuis workflow` when you want the CLI to restate the shortest route for
  a specific input
* use [docs/current-mainline-map.md](docs/current-mainline-map.md)
  when you want the repo-wide current route
* use
  [docs/versioning/nuis-alpha-0.7-mainline-entry.md](docs/versioning/nuis-alpha-0.7-mainline-entry.md)
  when the question is whether a lane is already current, still active, or only
  an intentional alpha boundary
* use
  [docs/versioning/nuis-alpha-0.0.1-closeout-board.md](docs/versioning/nuis-alpha-0.0.1-closeout-board.md)
  only when you intentionally need the first alpha closeout history

Useful inspection commands:

```bash
cargo run -p nuis -- dump-ast examples/projects/window_controls_demo
cargo run -p nuis -- dump-nir examples/projects/window_controls_demo
cargo run -p nuis -- dump-yir examples/projects/window_controls_demo
cargo run -p nuis -- inspect-artifact examples/bins/window_controls_demo_project/nuis.build.manifest.toml
cargo run -p nuis -- verify-artifact examples/bins/window_controls_demo_project/nuis.compiled.artifact
cargo run -p nuis -- artifact-doctor examples/bins/window_controls_demo_project
cargo run -p nuis -- verify-build-manifest examples/bins/window_controls_demo_project/nuis.build.manifest.toml
cargo run -p nuis -- run-artifact examples/bins/window_controls_demo_project/nuis.build.manifest.toml
```

If you want the shortest current native artifact closure route specifically:

```bash
cargo run -p nuis -- build \
  examples/projects/tooling/native_artifact_closure_demo \
  examples/bins/native_artifact_closure_demo_project
cargo run -p nuis -- inspect-artifact \
  examples/bins/native_artifact_closure_demo_project/nuis.build.manifest.toml
cargo run -p nuis -- verify-artifact \
  examples/bins/native_artifact_closure_demo_project/nuis.compiled.artifact
cargo run -p nuis -- artifact-doctor \
  examples/bins/native_artifact_closure_demo_project
cargo run -p nuis -- verify-build-manifest \
  examples/bins/native_artifact_closure_demo_project/nuis.build.manifest.toml
cargo run -p nuis -- run-artifact \
  examples/bins/native_artifact_closure_demo_project/nuis.build.manifest.toml
```

CPU target override is now explicit:

```bash
cargo run -p nuis -- build --cpu-abi cpu.arm64.apple_aapcs64 \
  examples/projects/kernel_tensor_demo \
  examples/bins/kernel_tensor_demo_project

cargo run -p nuis -- build --target aarch64-apple-darwin \
  examples/projects/kernel_tensor_demo \
  examples/bins/kernel_tensor_demo_project
```

## What Is Working Well Right Now

High-signal implemented surfaces:

* multi-file `nuis.toml` projects with project-level `links`
* lazy `nustar` loading and per-domain ABI resolution
* ABI auto-selection from registered `abi_targets`
* explicit `--cpu-abi` and `--target` overrides for CPU builds
* compile-cache inspection and pruning through `nuis`
* AOT bundle generation for current CPU-only and macOS window-hosted demo paths
* runtime-side artifact loading and host-consumable summary reporting through
  `nuis-runtime`
* host-YIR execution probes that read artifact YIR sidecars and execute
  registered YIR domain mods, including real kernel tensor result summaries
* repository source hygiene with non-test `tools/nuisc/src` implementation
  files kept below the current 600-line policy threshold
* source visibility boundaries through minimal `pub/private`
* `project-status` / `project-doctor` public-surface reporting
* intrinsic frontend annotations for `@test`, `@export`, `@inline`,
  `@noinline`, and `@host_symbol`
* first constrained trait/generic monomorphization slices
* packet schema/contract metadata through `@packet`,
  `@packet_field`, and `@packet_control_field`
* executable `while` subsets for counted/carry/flow-style loops, including
  representative native compile/launch smoke
* `std net` low-level syscall/socket/flow layering
* project-level host FFI contract indexing
* `ns-nova` framework manifests with family/render/selection assembly metadata for `core / ui / future scene` layering
* `cpu/data/shader/kernel` result-family validation in `YIR`
* task-style async primitives with `spawn / join / cancel / timeout / join_result`
* core-level async/result metadata beginning to move into `yir-core`

## Mainline Vs Experimental

Current mainline, meaning “good default places to stand on today”:

* `nuis -> NIR -> YIR -> LLVM/AOT` build path
* multi-file `nuis.toml` projects
* `nustar`-driven ABI and lane policy registration
* artifact-to-runtime inspection through `nuis-runtime`, including host-YIR
  reference execution for payload-backed YIR sidecars
* `examples/projects` as the primary runnable/compile-contract example layer
* `docs/reference` as the primary implementation-truth documentation layer
* `stdlib/std`, `stdlib/pixelmagic`, and `stdlib/witsage` as the current
  source-asset/library proving path
* `stdlib/ns-nova` as an official future GUI/render galaxy that should remain
  behind AOT/std/PixelMagic/WitSage hardening for now

Current experimental or intentionally still soft-edged tracks:

* `Task<T>` / `TaskResult<T>` ownership and future `GLM` elevation
* external-handle bridge-object direction for resource-bearing task payloads
* hot-sync contraction of local async regions
* `YIR`-level global clock negotiation and multi-`nustar` time conversion
* native CPU task execution beyond the current compile/contract staging path

Reading rule that matches this split:

* when current examples/docs and future sketches differ, prefer the current
  project/examples/reference path first
* treat sketches, probes, and future notes as design direction, not as already
  promised repository behavior

## Fast Orientation

If you want the shortest path by goal:

* consolidated current mainline map
  - [docs/current-mainline-map.md](docs/current-mainline-map.md)
* current alpha line and hardening baseline
  - [docs/versioning/nuis-alpha-0.7-mainline-entry.md](docs/versioning/nuis-alpha-0.7-mainline-entry.md)
  - [docs/versioning/nuis-alpha-0.6-mainline-entry.md](docs/versioning/nuis-alpha-0.6-mainline-entry.md)
  - [docs/versioning/nuis-alpha-0.4-system-inventory.md](docs/versioning/nuis-alpha-0.4-system-inventory.md)
  - [docs/versioning/nuis-alpha-0.4-mainline-hardening-plan.md](docs/versioning/nuis-alpha-0.4-mainline-hardening-plan.md)
  - [docs/versioning/nuis-alpha-0.4-doc-sync-inventory.md](docs/versioning/nuis-alpha-0.4-doc-sync-inventory.md)
* current user-facing project path
  - [examples/projects/README.md](examples/projects/README.md)
  - [docs/examples-freshness-audit.md](docs/examples-freshness-audit.md)
  - [docs/reference/yir-tools-reference.md](docs/reference/yir-tools-reference.md)
* current source-level language and host examples
  - [examples/ns/README.md](examples/ns/README.md)
  - [examples/ns/ffi/README.md](examples/ns/ffi/README.md)
  - [examples/ns/memory/README.md](examples/ns/memory/README.md)
* current implementation-facing semantic contracts
  - [docs/reference/README.md](docs/reference/README.md)
* current `std` and framework source assets
  - [stdlib/README.md](stdlib/README.md)
  - [stdlib/std/README.md](stdlib/std/README.md)
  - [stdlib/pixelmagic/README.md](stdlib/pixelmagic/README.md)
  - [stdlib/witsage/README.md](stdlib/witsage/README.md)
  - [stdlib/ns-nova/README.md](stdlib/ns-nova/README.md)
* quick repo map
  - [docs/repo-layout.md](docs/repo-layout.md)

## Key Architectural Notes

Current high-signal architectural facts:

* `YIR` is the main semantic execution boundary in this repository.
* `nuisc` is intentionally becoming more mod-agnostic; per-domain support should come from registered `nustar` contracts.
* `abi_targets` now live in `nustar` manifests and drive auto ABI selection, CLI overrides, packaging metadata, and loader contracts.
* default lane policy also belongs to `nustar` manifests; `nuisc` should only apply that policy plus narrow fallbacks.
* `data.handle_table` remains an indirection/resource-binding surface, not a place to own large payload blobs directly.
* `data.fabric` is being kept on a strict seven-family primitive model: `bind`, `handle`, `marker`, `move`, `window`, `pipe`, and `observe`. Higher-level helpers must lower into those families rather than invent new primitive classes.
* current Fabric host integration is intentionally thin and AOT-first, with static typed action tables rather than a heavy runtime metadata graph.
* async/result semantics are being normalized into `yir-core`, even though the concrete entry ops are still currently surfaced through `cpu.*`.

Historical architecture material lives under:

* [docs/historical/README.md](docs/historical/README.md)
