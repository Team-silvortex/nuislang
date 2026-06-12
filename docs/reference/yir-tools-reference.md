---

# YIR Tools Reference

## Draft Reference v0.01

---

# 1. Purpose

This document is the working tool reference for the current handwritten `YIR`
prototype in this repository.

It is the closest thing the project currently has to an early `LLVM tools`
reference for the `YIR` layer.

It should evolve together with the implementation.

---

# 2. Scope

This reference currently covers:

* reference command-line entry points
* current `nuis / nuisc` tool split
* current LLVM path
* current AOT packaging path
* current preview/export helpers

This reference does not yet attempt to freeze:

* final `nuisc` CLI
* final `nustar` package ABI
* final runtime packaging format

---

# 3. Reference Tools

## Front-door workflow tool

[tools/nuis/src/main.rs](/Users/Shared/chroot/dev/nuislang/tools/nuis/src/main.rs)

```text
cargo run -p nuis -- <command>
```

Current reference commands:

* `status`
* `registry`
* `fmt <input>`
* `bindings <input.ns>`
* `pack-nustar <package-id> <output.nustar>`
* `inspect-nustar <input.nustar>`
* `loader-contract <package-id>`
* `verify-build-manifest <nuis.build.manifest.toml>`
* `cache-status [--all] [--verbose-cache] [--json] [input]`
* `clean-cache [--all] [--json] [input]`
* `cache-prune [--all] [--keep N] [--json] [input]`
* `workflow [--json] [input.ns|project-dir|nuis.toml]`
* `project-status [--json] <input.ns|project-dir|nuis.toml>`
* `project-doctor [--json] <project-dir|nuis.toml>`
* `project-lock-abi <project-dir|nuis.toml>`
* `scheduler-view <input.ns|project-dir|nuis.toml>`
* `release-check <input> <output-dir>`
* `check <input.ns|project-dir|nuis.toml>`
* `test [--list] [--ignored|--include-ignored] [--exact] <input.ns|project-dir|nuis.toml> [filter]`
* `build <input.ns|project-dir|nuis.toml> <output-dir>`
* `build --cpu-abi <abi> <input> <output-dir>`
* `build --target <triple> <input> <output-dir>`
* `dump-ast <input.ns>`
* `dump-nir <input.ns>`
* `dump-yir <input.ns>`
* `rc ...`
* `galaxy ...`

`nuis` is the current front-door workflow tool. It should grow into the
user-facing toolchain surface while reusing `nuisc` as the compiler core.

### Default compile workflow

For `0.16.0`, the default command order should be read as:

```text
workflow -> project-doctor -> check -> test -> build -> release-check
```

Use `workflow` when you want the tool itself to restate the shortest route for
the current input before you start deeper compile work. It also emits a
`recommended_next_step` and `recommended_command` surface that can be consumed
by scripts or editor helpers:

```bash
cargo run -p nuis -- workflow <project-dir|nuis.toml>
cargo run -p nuis -- workflow <input.ns> --json
```

The stable compile order remains:

```text
project-doctor -> check -> test -> build -> release-check
```

Shortest project route:

```bash
cargo run -p nuis -- project-doctor <project-dir|nuis.toml>
cargo run -p nuis -- check <project-dir|nuis.toml>
cargo run -p nuis -- test <project-dir|nuis.toml>
cargo run -p nuis -- build <project-dir|nuis.toml> <output-dir>
cargo run -p nuis -- release-check <project-dir|nuis.toml> <output-dir>
```

Shortest single-source route:

```bash
cargo run -p nuis -- check <input.ns>
cargo run -p nuis -- test <input.ns>
cargo run -p nuis -- build <input.ns> <output-dir>
```

### Debug workflow

When `check` fails and you need compiler-shape visibility, use this order:

```text
dump-ast -> dump-nir -> dump-yir -> scheduler-view
```

Use:

* `dump-ast` for parser, annotation, and surface-shape issues
* `dump-nir` for frontend typing, generic rewrite, and pattern lowering issues
* `dump-yir` for lowering/result-family/verifier-adjacent issues
* `scheduler-view` for lane and scheduling inspection

Current `nustar` loading policy is:

* static index
* lazy manifest loading
* binding only for families actually required by the current `YIR` graph
* each loaded `nustar` now also declares its `AST surface`, `NIR surface`, `YIR lowering`, and `part verify` responsibilities so `nuisc` can stay mod-agnostic while still seeing the full package role
* each loaded `nustar` also declares unified entry names for those four surfaces, so future `nuisc` loading can bind through stable entry points instead of hard-coded per-mod assumptions
* each loaded `nustar` can now also declare a small domain-owned clock contract through `clock_domain_id`, `clock_kind`, `clock_epoch_kind`, `clock_resolution`, and `clock_bridge_default`
  * lowering now also materializes the current scheduler registration into a
  short contract stack:
  * placement
    * `scheduler_contract_cpu_lane_policy_type`
    * `scheduler_contract_cpu_lane_capability_type`
    * `scheduler_contract_cpu_bridge_capability_type`
  * timing
    * `scheduler_contract_cpu_clock_type`
  * result observation
    * `scheduler_contract_cpu_result_lane_type`
    * `scheduler_contract_cpu_result_capability_type`
    * `scheduler_contract_cpu_observer_role_variant_type`
  * async summary observation
    * `scheduler_contract_cpu_summary_capability_type`
    * `scheduler_contract_cpu_summary_class_type`
  * observer classification
    * `scheduler_contract_cpu_observer_source_class_type`
    * `scheduler_contract_cpu_observer_stage_class_type`
    * `scheduler_contract_cpu_observer_scope_class_type`
    * `scheduler_contract_cpu_observer_branch_class_type`
* `nuis registry` now also prints the short scheduler-facing reading view for
  each package:
  * `scheduler_contract_stack`
  * `scheduler_result_roles`
  * `scheduler_sample_navigation`
  * `scheduler_result_samples`
  * `scheduler_transport_samples`
  * `scheduler_summary_api`
  * `scheduler_summary_samples`
  * `scheduler_observer_classes`
* `scheduler_sample_navigation` is the shortest CLI ordering hint for how to
  read the sample lines that follow
  * `official.shader`
    * `policy -> windowed`
  * `official.kernel`
    * `policy -> windowed`
  * `official.network`
    * `result_ladder -> transport_split_ladder -> transport_summary_ladder -> summary_classes`
* `nuis scheduler-view` now renders multi-segment sample hints as small
  indented blocks, while `nuis registry` keeps the same content on one line
  for compact scanning
* `nuis project-status` and `nuis project-doctor` now also print a lightweight
  `std net` reading hint for projects that resolve the `network` domain:
  * `std_net_navigation`
    * `profile_core -> result_spine -> task_spine`
  * `std_net_samples`
    * `profile_core`
      * `net_endpoint_recipe`
      * `net_endpoint_recipe_demo`
    * `result_spine`
      * `net_result_recipe`
      * `net_result_bridge_recipe`
      * `net_result_recipe_demo`
      * `net_result_bridge_recipe_demo`
    * `task_spine`
      * `net_task_policy_recipe`
      * `net_task_batch_recipe`
      * `net_task_windowed_recipe`
      * `net_task_windowed_bridge_recipe`
      * `net_task_policy_recipe_demo`
      * `net_task_batch_recipe_demo`
      * `net_task_windowed_recipe_demo`
      * `net_task_windowed_bridge_recipe_demo`
* for result-facing ladders, `scheduler_result_samples` now acts as the
  shortest CLI hint for “which checked-in samples best represent this domain's
  current result reading order”
  * `official.network` currently points to:
    * `result_ladder`
      * `network_result_profile_demo`
      * `network_connect_result_demo`
      * `network_accept_result_demo`
      * `network_result_task_policy_demo`
      * `network_result_task_batch_demo`
      * `network_result_task_windowed_batch_demo`
      * `network_result_session_bridge_demo`
    * `control_ladder`
      * `network_connect_result_demo`
      * `network_accept_result_demo`
      * `network_connect_accept_task_policy_demo`
      * `network_connect_accept_task_batch_demo`
      * `network_connect_accept_task_windowed_batch_demo`
* for transport-facing ladders, `scheduler_transport_samples` now acts as the
  shortest CLI hint for “which checked-in samples best represent this domain's
  current transport runtime/split/summary progression”
  * `official.network` currently points to:
    * `transport_runtime`
      * `network_host_transport_runtime_demo`
      * `network_transport_result_demo`
    * `transport_split_ladder`
      * `network_transport_result_policy_split_demo`
      * `network_transport_result_batch_split_demo`
      * `network_transport_result_windowed_split_demo`
      * `network_transport_result_session_bridge_split_demo`
    * `transport_summary_ladder`
      * `network_transport_result_task_policy_demo`
      * `network_transport_result_task_batch_demo`
      * `network_transport_result_task_windowed_batch_demo`
      * `network_transport_result_session_bridge_demo`
* for domain-owned split summaries, `scheduler_summary_samples` now acts as the
  shortest CLI hint for “which checked-in sample ladder best represents this
  summary class”
  * `official.shader` currently points to:
    * `policy`
      * `shader_async_policy_profile_demo`
      * `shader_async_fallback_profile_demo`
    * `windowed`
      * `shader_async_batch_profile_demo`
      * `shader_async_windowed_batch_profile_demo`
  * `official.kernel` currently points to:
    * `policy`
      * `kernel_async_tensor_policy_profile_demo`
      * `kernel_async_tensor_fallback_profile_demo`
    * `windowed`
      * `kernel_async_tensor_batch_profile_demo`
      * `kernel_async_tensor_windowed_batch_profile_demo`
  * `official.network` currently points to:
    * `transport_split`
      * `network_transport_result_policy_split_demo`
      * `network_transport_result_batch_split_demo`
      * `network_transport_result_windowed_split_demo`
      * `network_transport_result_session_bridge_split_demo`
    * `control_split`
      * `network_connect_accept_task_policy_demo`
      * `network_connect_accept_task_batch_demo`
      * `network_connect_accept_task_windowed_batch_demo`
* `nuis loader-contract official.network` now also exposes the minimal control
  host symbol reservation through `required_metadata`, currently:
  * `host_symbol=network.connect:host_network_connect_probe`
  * `host_symbol=network.accept:host_network_accept_probe`
  * `host_symbol=network.close:host_network_close`
  * `host_symbol=network.send:host_network_send_probe`
  * `host_symbol=network.recv:host_network_recv_probe`
* source-level annotation guidance now prefers
  `extern "c" @host_symbol("network.*") fn ...;`
  for std-owned host-boundary declarations; the function-body bridge-stub form
  remains available as an MVP compatibility path
* the current narrow project-first bridge sample is
  [network_host_control_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_host_control_runtime_demo),
  whose `dump-yir` output currently shows:
  * `cpu.extern_call_i64 ... c host_network_connect_probe ...`
  * `cpu.extern_call_i64 ... c host_network_accept_probe ...`
  * `cpu.extern_call_i64 ... c host_network_close ...`
* the current narrow transport companion is
  [network_host_transport_runtime_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_host_transport_runtime_demo),
  whose `dump-yir` output currently shows:
  * `cpu.extern_call_i64 ... c host_network_send_probe ...`
  * `cpu.extern_call_i64 ... c host_network_recv_probe ...`
* the current narrow result-facing companion is
  [network_transport_result_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/network_transport_result_demo),
  which now verifies that those same host transport probes may be wrapped by:
  * `network_result(...)`
  * `network_config_ready(...)`
  * `network_send_ready(...)`
  * `network_recv_ready(...)`
  * `network_value(...)`
* `nuis project-status` and `nuis project-doctor` now also print the same
  scheduler-facing reading view per resolved domain requirement, plus the
  resolved clock/bridge line for that domain

Current `nustar` packaging prototype is:

* one standard binary envelope
* manifest segment
* format version
* ABI tag
* implementation-format tag
* implementation blob segment
* implementation checksum

Current loading-contract direction is:

* `native-dylib` and `llvm-bc` both bind through the same canonical loader ABI
* packages declare a canonical `loader_entry`
* the canonical bootstrap symbol is `nustar.bootstrap.v1`
* the canonical bootstrap signature is `extern "C" fn(*const NustarHostAbiV1, *const u8, usize, *mut NustarBootstrapResultV1) -> i32`
* host/runtime bootstrap stays machine-ABI aware: current `.nustar` packages carry `machine_arch / machine_os / object_format / calling_abi`
* package implementations are intended to be replaceable
  * a platform-facing package such as an `x86_64-cpu-linux` `nustar` may be swapped
    as long as registration completeness, standards legality, and loader-contract
    requirements still validate
  * this is also why frontend annotation spelling should be treated as a managed
    convenience surface, not as the strongest semantic truth in the stack
* `loader-contract` now also defines per-kind implementation-segment requirements, including container kind, implementation section name, required exports, required metadata, and link mode
* machine ABI compatibility is explicit and inspectable
* `abi_targets` now also participate in package inspection, loader-contract metadata, project auto-resolution, and CPU-target override validation

## Project Workflow Notes

The front-door workflow is now project-aware:

* `check`, `test`, `build`, `dump-ast`, `dump-nir`, `dump-yir`, `bindings`, and cache commands all accept single-file `.ns`, project directories, or direct `nuis.toml` inputs where applicable
* `project-status` prints the resolved project graph, declared `tests = [...]`, effective ABI mode, and per-domain ABI target details
* `project-doctor` prints a higher-level health summary covering project ABI state, declared/missing test inputs, `galaxy.toml`, `nuis.galaxy.lock`, dependency materialization state, `ns-nova.toml`, and current `stdlib/ns-nova` source-asset visibility
* `project-lock-abi` materializes the currently recommended host-matching ABI set into the project manifest
* `test` runs `check` first, collects language-level `test fn` declarations, can list them with `--list`, can restrict execution to a substring filter on the test function name or declared label, supports `--exact`, supports `--ignored` / `--include-ignored`, and currently understands the MVP metadata `ignored`, `should_fail`, `reason`, `timeout_ms`, and `clock_domain`
* `verify-build-manifest` now reports CPU target metadata including ABI, machine arch/os, object format, calling ABI, clang triple, and cross-build flag

### Recommended Project Management Flow

For the current repository shape, the most useful front-door sequence is:

```text
project-doctor
  -> project-status
  -> scheduler-view
  -> project-lock-abi    (when ABI is still auto-resolved)
  -> check
  -> test
  -> build
  -> release-check
```

Read that as:

* `project-doctor`
  first health check for missing `galaxy.toml`, `nuis.galaxy.lock`, synced
  deps, test inputs, or `ns-nova` metadata
* `project-status`
  structural and ABI-resolution view of the same normalized project model used
  by build metadata
* `check` / `build` / `release-check`
  all derive from one shared `ProjectCompilationPlan`, so organization,
  exchange routes, ABI resolution, and synthetic input naming stay unified
* `scheduler-view`
  focused `placement / timing / result / summary / observer` projection;
  `--json` mirrors the same view as structured data
* `project-lock-abi`
  optional freeze step for the current host-matching ABI set
* `check`
  semantic/project validation
* `test`
  front-door test pass for single-file or project inputs; it supports
  `--list`, substring or exact filtering, `--ignored`, and
  `--include-ignored`, and the current runner status labels are `PASS`, `FAIL`,
  `SKIP`, `XFAIL`, and `XPASS`
  timed tests already support `timeout_ms`, `clock_domain`, and
  `clock_policy="bridge"`; see
  [cpu-task-scheduler-clock.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-task-scheduler-clock.md)
  for the current clock/bridge contract
* `build`
  artifact generation
  emits the concrete output directory plus project-side indexes such as
  `nuis.project.plan.txt`, `nuis.project.organization.txt`,
  `nuis.project.exchange.txt`, `nuis.project.packet.txt`,
  `nuis.project.host_ffi.txt`, and `nuis.project.abi.txt`
  packet metadata now records shape, field order, coarse field role, and a
  first encode skeleton; packet validation is still intentionally narrow, and
  async-carrier families like `Task<...>` / `*Result<...>` are still rejected
* `release-check`
  final release-facing pass; use this when you want the default compile route
  to end in a reproducible output directory plus build-manifest verification

For framework/package-aware projects, the current companion `galaxy` flow is:

```text
galaxy init
  -> galaxy check
  -> galaxy lock-deps
  -> galaxy sync-deps
  -> project-doctor
```

`nuis project-status` and `nuis project-doctor` now also print this same route
as lightweight front-door hints, using the clearer
`project_compile_*` / `project_test_workflow` naming:

* `project_compile_workflow`
  * `health -> structure -> scheduler -> abi_lock -> check -> test -> build -> release_check`
* `project_compile_samples`
  * `health=nuis project-doctor <project-dir>; structure=nuis project-status <project-dir>; scheduler=nuis scheduler-view <project-dir>; abi_lock=nuis project-lock-abi <project-dir>; compile=nuis check <project-dir> -> nuis test <project-dir> -> nuis build <project-dir> -> nuis release-check <project-dir> <output-dir>`
* `project_test_workflow`
  * `list=nuis test --list <project-dir>; exact=nuis test --exact <project-dir> <test-name>; ignored=nuis test --ignored <project-dir>; include_ignored=nuis test --include-ignored <project-dir>`
* `project_galaxy_workflow`
  * printed when the project already has `galaxy.toml` or declared galaxy deps:
    `nuis galaxy init <project-dir> -> nuis galaxy check <project-dir> -> nuis galaxy lock-deps <project-dir> -> nuis galaxy sync-deps <project-dir> -> nuis project-doctor <project-dir>`

Current project-aware front doors now also share a normalized compiler-side
plan summary:

* `project_plan`
  * printed by `project-status`, `project-doctor`, `scheduler-view`,
    `project-lock-abi`, and the `galaxy` project-management commands
  * current shape: `entry=<entry> domains=<...> exchanges=<n> abi_mode=<...>`
  * `nuis.project.plan.txt` also records `dependencies`,
    `synthetic_input_kind/synthetic_input`, and `output_intents`
  * dependency rows now also carry a category; current project-managed package
    deps land in `package-registry`
  * `scheduler-view --json <project-dir>` now also emits the project-side
    classification fields directly: `project_plan_dependency_categories`,
    `project_plan_output_categories`, `project_exchange_route_classes`
  * `project-status --json` and `project-doctor --json` emit the same
    project-plan structure fields directly, plus their current workbench state

Typical commands:

```bash
cargo run -p nuis -- project-doctor examples/projects/window_controls_demo
cargo run -p nuis -- test examples/projects/window_controls_demo
cargo run -p nuis -- test --exact examples/projects/window_controls_demo smoke_add
cargo run -p nuis -- galaxy init examples/projects/window_controls_demo --framework ns-nova
cargo run -p nuis -- dump-yir examples/projects/domains/network_host_control_runtime_demo
```
## Build Override Notes
Current CPU build override surface:

```text
nuis build --cpu-abi <registered-cpu-abi> <input> <output-dir>
nuis build --target <clang-target-triple> <input> <output-dir>
nuis release-check --cpu-abi <registered-cpu-abi> <input> <output-dir>
nuis release-check --target <clang-target-triple> <input> <output-dir>
nuisc compile --cpu-abi <registered-cpu-abi> <input> <output-dir>
nuisc compile --target <clang-target-triple> <input> <output-dir>
```

Important current rule:

* CPU ABI support is intended to come from `nustar` registration, not hardcoded `nuisc` tables
* explicit CPU overrides and project auto-ABI selection are both checked against registered `abi_targets`
* current window-hosted AppKit bundle packaging still rejects cross-target output instead of pretending to support it

## Core compiler

[tools/nuisc/src/main.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/main.rs)

```text
cargo run -p nuisc -- <command>
```

`nuisc` is the current compiler-core CLI. It still exposes the same minimal
pipeline surface directly while the higher-level `nuis` workflow matures.
Its current command surface broadly mirrors the relevant `nuis` compiler-facing
subcommands:

* `status`
* `registry`
* `fmt`
* `bindings`
* `pack-nustar`
* `inspect-nustar`
* `loader-contract`
* `verify-build-manifest`
* `cache-status`
* `clean-cache`
* `cache-prune`
* `dump-ast`
* `dump-nir`
* `dump-yir`
* `check`
* `compile`

## Parse + verify + execute

[tools/yir-run/src/main.rs](/Users/Shared/chroot/dev/nuislang/tools/yir-run/src/main.rs)

```text
cargo run -p yir-run -- <module.yir>
```

This is the main handwritten `YIR` execution entry.

It performs:

* parse
* verify
* graph execution
* trace/lane/value reporting

## Emit LLVM IR text

[tools/yir-emit-llvm/src/main.rs](/Users/Shared/chroot/dev/nuislang/tools/yir-emit-llvm/src/main.rs)

```text
cargo run -p yir-emit-llvm -- <module.yir>
```

This currently lowers the `cpu` slice to LLVM IR text.

## Build AOT bundle

[tools/yir-pack-aot/src/main.rs](/Users/Shared/chroot/dev/nuislang/tools/yir-pack-aot/src/main.rs)

```text
cargo run -p yir-pack-aot -- <module.yir> <output-dir> [frame-scale]
```

This packages a small AOT artifact using LLVM/clang where possible and cooked or
prerendered artifacts where necessary.

## Export frame

[tools/yir-export-frame/src/main.rs](/Users/Shared/chroot/dev/nuislang/tools/yir-export-frame/src/main.rs)

```text
cargo run -p yir-export-frame -- <module.yir> <output.ppm> [scale]
```

This is the current reference render-artifact path.

## Export UI plan

[tools/yir-export-ui-plan/src/main.rs](/Users/Shared/chroot/dev/nuislang/tools/yir-export-ui-plan/src/main.rs)

This extracts host-side preview plan data from current `cpu` host extension ops.

## macOS preview adapter

[tools/yir-preview-macos/PreviewFrame.swift](/Users/Shared/chroot/dev/nuislang/tools/yir-preview-macos/PreviewFrame.swift)

This is a tool-layer adapter.

It is not part of `YIR` core semantics.

---

# 4. LLVM Path

The current LLVM path is intentionally narrow:

* it lowers the `cpu` slice
* it already supports arithmetic
* it already supports the current heap-node prototype
* it already emits `malloc/free + gep/load/store` for the linked-list model

Current examples:

* [examples/yir/ball_cpu_driver.yir](/Users/Shared/chroot/dev/nuislang/examples/yir/ball_cpu_driver.yir)
* [examples/yir/cpu_linked_list.yir](/Users/Shared/chroot/dev/nuislang/examples/yir/cpu_linked_list.yir)
* [examples/yir/cpu_linked_list_rustish.yir](/Users/Shared/chroot/dev/nuislang/examples/yir/cpu_linked_list_rustish.yir)

The current hetero render path may still package prerendered or cooked artifacts
for non-CPU slices.

This is expected at the current stage.

---

# 5. AOT Packaging Notes

The current `yir-pack-aot` path is reference-quality packaging, not final
distribution design.

Current behavior:

* pure CPU graphs can be compiled to a native binary through LLVM/clang
* hetero/window demos with `cpu.tick_i64` can now also be packaged into a
  macOS AppKit-hosted single binary with an embedded `YIR` runtime path:
  generated hosts link `libyir_runtime_host.a`, embed the `.yir` module bytes,
  and generate live framebuffer updates in-process instead of shelling out to a
  sidecar exporter
* the `window_controls_demo` project route now builds successfully through this path
* shader packaging already has a contract/package skeleton for future backend
  variants
* shader package manifests may now include per-stage binding layout entries, texture/sampler/geometry binding kinds, minimal render-state metadata, sampler/texture binding details such as filter, address mode, and texture shape, plus top-level fabric handle-table metadata, per-stage fabric table association, and Fabric worker core binding metadata
* current macOS AppKit host stubs read `fabric_worker_core`, start a dedicated
  Fabric worker thread, and apply it as that thread's startup affinity hint;
* current Fabric host booting stays AOT-first: generated host stubs embed a
  static typed action table derived from `data.*` nodes instead of constructing
  a heavyweight dynamic metadata graph at runtime
* the current typed action table also carries a minimal class/slot ABI tag for
  each Fabric action, so host-side dispatch can remain static without falling
  back to string-only pattern matching
* this is still weaker than a strict reserved-core runtime model
* CPU build manifests and `project-status` output now also expose per-domain ABI target details such as backend family and host-adaptive selection

This is the beginning of a `YIR`-native toolchain, not the final shape.

---

# 6. Stability Notes

Most stable current reference tool surfaces:

* `yir-run`
* `yir-emit-llvm`
* `yir-pack-aot`
* CPU-slice LLVM lowering

Clearly provisional current tool surfaces:

* preview adapter details
* exact shader package schema
* final bundle/manifest format
* final in-process runtime embedding strategy

---

# 7. Sync Policy

This file should be updated whenever one of these changes:

* a reference tool is added or removed
* CLI behavior changes in a user-visible way
* LLVM lowering meaning changes
* AOT packaging meaning changes
* preview/export adapter meaning changes

The goal is for this file to remain a living reference, not a stale description.
