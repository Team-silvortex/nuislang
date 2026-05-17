# nuislang

> AOT-first heterogeneous systems language and toolchain built around `nuis -> NIR -> YIR -> LLVM/AOT`, with `nustar` packages providing per-domain parsing, lowering, ABI contracts, and verification surfaces.

## Current Status

The repository is in an active architecture-building stage. The most stable current spine is:

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

That model is increasingly enforced through `YIR` contracts, project validation, per-domain `nustar` manifests, and verifier checks rather than only ad hoc frontend rules.

## Toolchain

```text
nuis     -> front-door workflow tool
nuis-rc  -> resident control tool (later-stage, still intentionally thin)
nuisc    -> compiler/scheduler core
yalivia  -> separate future JIT/runtime project
vulpoya  -> separate future analyzer/verifier project
```

Current responsibility split:

* `nuis` is the main workflow surface for `check`, `build`, caches, projects, and package inspection.
* `nuisc` is the compiler/scheduler core that consumes `.ns` or project inputs and emits `NIR`, `YIR`, LLVM IR, and AOT outputs.
* `nustar` packages are where per-domain ABI support, default lanes, frontend/lowering entrypoints, and package contracts are registered.
* `nustar` packages are also starting to declare per-domain clock contracts such as `clock_domain_id`, `clock_kind`, `clock_epoch_kind`, `clock_resolution`, and `clock_bridge_default`.

## Quick Start

Recommended first commands:

```bash
cargo run -p nuis -- project-doctor examples/projects/window_controls_demo
cargo run -p nuis -- check examples/projects/window_controls_demo
cargo run -p nuis -- test examples/projects/window_controls_demo
cargo run -p nuis -- test --list examples/projects/window_controls_demo
cargo run -p nuis -- test --ignored examples/projects/window_controls_demo
cargo run -p nuis -- test --include-ignored examples/projects/window_controls_demo
cargo run -p nuis -- test --exact examples/projects/window_controls_demo smoke_add
cargo run -p nuis -- test --ignored --exact examples/projects/window_controls_demo smoke_skip
cargo run -p nuis -- project-status examples/projects/window_controls_demo
cargo run -p nuis -- build examples/projects/window_controls_demo examples/bins/window_controls_demo_project
```

Current language-level test declarations use:

```ns
test("smoke_add") fn smoke_add() -> i64 { return 1; }
test(ignored=true) fn skipped_case() -> i64 { return 1; }
test(should_fail=true) fn expected_failure() -> i64 { return 0; }
test("expected_failure", should_fail=true, reason="must reject zero") fn expected_failure() -> i64 { return 0; }
test("slow_async", timeout_ms=25, clock_domain="global", clock_policy="bridge") async fn slow_async() -> i64 { return 1; }
```

When a timed test uses `clock_domain`, `nuis test` now shows both the declared
domain and the runner-resolved domain in execution output, including the current
canonical staging codes. In the current MVP, `global (2)` still resolves to the
host monotonic clock `monotonic (0)`, and the output also reports the current
host deadline source such as `host_monotonic_deadline` or `host_wall_deadline`.
The current explicit way to acknowledge that bridge is
`clock_policy="bridge"` alongside `clock_domain="global"`.

Useful inspection commands:

```bash
cargo run -p nuis -- dump-ast examples/projects/window_controls_demo
cargo run -p nuis -- dump-nir examples/projects/window_controls_demo
cargo run -p nuis -- dump-yir examples/projects/window_controls_demo
cargo run -p nuis -- verify-build-manifest examples/bins/window_controls_demo_project/nuis.build.manifest.toml
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
* project-level host FFI contract indexing
* `ns-nova` framework manifests with family/render/selection assembly metadata for `core / ui / future scene` layering
* `cpu/data/shader/kernel` result-family validation in `YIR`
* task-style async primitives with `spawn / join / cancel / timeout / join_result`
* core-level async/result metadata beginning to move into `yir-core`

## Current Reference Examples

Start here:

* [examples/README.md](/Users/Shared/chroot/dev/nuislang/examples/README.md)
* [examples/ns/README.md](/Users/Shared/chroot/dev/nuislang/examples/ns/README.md)
* [examples/projects/README.md](/Users/Shared/chroot/dev/nuislang/examples/projects/README.md)
* [examples/yir/README.md](/Users/Shared/chroot/dev/nuislang/examples/yir/README.md)
* [examples/bins/README.md](/Users/Shared/chroot/dev/nuislang/examples/bins/README.md)
* [stdlib/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/README.md)
* [docs/README.md](/Users/Shared/chroot/dev/nuislang/docs/README.md)
* [docs/repo-layout.md](/Users/Shared/chroot/dev/nuislang/docs/repo-layout.md)

Recommended current examples:

* [examples/ns/core/hello_world.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/core/hello_world.ns)
* [examples/ns/types/hello_ref_struct.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/types/hello_ref_struct.ns)
* [examples/ns/data/hello_data.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/data/hello_data.ns)
* [examples/ns/ffi/hello_ffi.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/hello_ffi.ns)
* [examples/ns/demos/window_controls_demo.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/demos/window_controls_demo.ns)
* [examples/projects/window_controls_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/window_controls_demo)
* [examples/projects/kernel_tensor_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/kernel_tensor_demo)
* [examples/yir/demos/window_controls_demo.yir](/Users/Shared/chroot/dev/nuislang/examples/yir/demos/window_controls_demo.yir)
* [examples/yir/data/data_fabric_demo.yir](/Users/Shared/chroot/dev/nuislang/examples/yir/data/data_fabric_demo.yir)
* [examples/yir/shader/shader_bindings_demo.yir](/Users/Shared/chroot/dev/nuislang/examples/yir/shader/shader_bindings_demo.yir)
* [examples/yir/kernel/kernel_auto_broadcast_demo.yir](/Users/Shared/chroot/dev/nuislang/examples/yir/kernel/kernel_auto_broadcast_demo.yir)

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

## Notes

This repository now keeps current implementation guidance and historical design material in separate places.

For current reality, stay with:

* [README.md](/Users/Shared/chroot/dev/nuislang/README.md)
* [docs/README.md](/Users/Shared/chroot/dev/nuislang/docs/README.md)
* [docs/repo-layout.md](/Users/Shared/chroot/dev/nuislang/docs/repo-layout.md)
* [docs/reference/yir-reference.md](/Users/Shared/chroot/dev/nuislang/docs/reference/yir-reference.md)
* [docs/reference/yir-langref.md](/Users/Shared/chroot/dev/nuislang/docs/reference/yir-langref.md)
* [docs/reference/yir-tools-reference.md](/Users/Shared/chroot/dev/nuislang/docs/reference/yir-tools-reference.md)

For historical architecture arguments and the older long-form whitepaper, go to:

* [docs/historical/README.md](/Users/Shared/chroot/dev/nuislang/docs/historical/README.md)
* [docs/historical/nuislang-whitepaper-v0.44b.md](/Users/Shared/chroot/dev/nuislang/docs/historical/nuislang-whitepaper-v0.44b.md)
