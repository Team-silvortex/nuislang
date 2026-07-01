# `nuis` `alpha-0.4.*` System Inventory

This file is the current system-level inventory for the `alpha-0.4.*` line.

It complements the hardening plan:

* the hardening plan says what to optimize for
* this inventory says what exists, what is usable, and what still needs work

Short rule:

`alpha-0.4.*` is no longer just feature expansion; it is the first serious
inventory pass where docs, examples, runtime probes, and compile artifacts need
to describe the same system.

## Current Spine

The most important working route is:

```text
nuis project / source
  -> frontend
  -> NIR
  -> YIR
  -> verify
  -> LLVM / package / artifact
  -> artifact-doctor
  -> run-artifact / host-YIR probe
```

This route is not yet a final product pipeline, but it is now concrete enough
to be used as the main integration spine.

Current entrypoints:

* [../current-mainline-map.md](/Users/Shared/chroot/dev/nuislang/docs/current-mainline-map.md)
* [nuis-alpha-0.4-mainline-hardening-plan.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-alpha-0.4-mainline-hardening-plan.md)
* [nuis-alpha-0.4-doc-sync-inventory.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-alpha-0.4-doc-sync-inventory.md)
* [../reference/nuis-native-artifact-workflow.md](/Users/Shared/chroot/dev/nuislang/docs/reference/nuis-native-artifact-workflow.md)
* [../reference/nuis-binary-format-protocol.md](/Users/Shared/chroot/dev/nuislang/docs/reference/nuis-binary-format-protocol.md)

## What Is Real Enough To Lean On

These surfaces should be treated as current implementation truth, with normal
alpha caution:

* multi-file `nuis.toml` projects
* source parsing into NIR and YIR
* project validation around links, ABI declarations, std/galaxy imports, and
  domain surfaces
* registered `nustar` manifests for CPU, data, shader, kernel, network, NPU,
  and architecture-specific CPU packages
* ABI target metadata in `nustar` manifests
* YIR verifier coverage for GLM, data fabric, result-family states, CPU heap
  protocol, scheduler contracts, and lowering contracts
* LLVM text emission for the current CPU slice
* AOT artifact emission with build manifest, envelope, lifecycle contract,
  payload blobs, bridge registry, host bridge plan index, and lowering index
  alignment checks
* `nuis build-report`, `artifact-doctor`, `inspect-artifact`,
  `verify-artifact`, `verify-build-manifest`, and `run-artifact`
* runtime-side artifact loading through `nuis-runtime`
* host-consumable summaries for payload-backed CPU fallback domain units
* host YIR execution probes through the registered YIR executor, including
  real kernel tensor evaluation for kernel-focused artifact YIR sidecars
* std source assets for text, IO, filesystem, task/thread, net, errors, and
  host-runtime facade style examples
* std filesystem contract consumers that build and run as process-style smoke
  demos: read, write, copy, roundtrip, output, directory create/remove,
  filesystem report, report-to-file, and filesystem/console report routes
* official galaxy scaffolding for `pixelmagic`, `witsage`, and `ns-nova`

## Current Integration Proofs

These are the proof shapes that matter most right now:

* `cargo test -p nuis-runtime`
* `cargo test -p nuis build_report_json_executes_host_yir_kernel_values --bin nuis`
* `cargo test -p nuis build_report_json_exposes_host_cpu_fallback_runtime_events --bin nuis`
* `cargo test -p nuis build_report_json_exposes_real_heterogeneous_runtime_summary --bin nuis`
* `cargo test -p nuis run_artifact_json --bin nuis`
* `cargo test -p nuisc galaxy_resolution --lib`
* `cargo test -p nuisc stdlib_registry --lib`
* `cargo test -p nuisc stdlib_docs --lib`
* `cargo test -p nuisc compiles_filesystem_mainline_examples --test examples_mainline_compile`
* `cargo test -p nuisc file_copy_demo --test memory_compile`
* `cargo run -p nuis -- build examples/projects/filesystem/file_copy_demo <tmp-output>`
* `cargo run -p nuis -- run-artifact <tmp-output>`
* `cargo fmt --check`
* `git diff --check`

The host-YIR probe is important because it crosses:

`artifact manifest -> YIR sidecar -> yir-syntax -> yir-exec -> registered kernel mod -> numeric result summary`

It is not the final heterogeneous binary execution model, and full mixed
data-fabric projects can still outrun the current reference executor. It does
prove that runtime-facing artifact reports can consume real YIR semantics
instead of only reporting scaffold metadata.

## What Is Still Soft

These surfaces exist, but should not be described as complete:

* final self-hosting
* final linker ownership and native container format stability
* final raw-pointer/unsafe story across GLM and ownership
* final GPU vendor backend maturity
* final CoreML / ANE / NPU backend maturity
* final network syscall/service runtime
* final ns-nova engine maturity
* full source-level execution for every YIR-domain capability
* full standard-library runtime behavior behind every facade helper

Current safe wording:

* say "probe", "contract", "sidecar", "fallback", or "reference execution"
  when that is what exists
* say "runs" only for paths that are actually exercised by tests or checked
  commands
* say "registered capability" when the compiler is consulting `nustar`
  metadata rather than knowing a backend directly

## Documentation Drift Found In This Pass

The main drift pattern was not that docs were absent. It was that old anchors
kept their front-door wording after the project moved on. The current cleanup
anchor is
[nuis-alpha-0.4-doc-sync-inventory.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-alpha-0.4-doc-sync-inventory.md).

High-priority cleanup targets:

* README links that still call `alpha-0.1.*` or `alpha-0.0.1` the current line
* example audit language that still reads like alpha closeout instead of
  `alpha-0.4.*` hardening
* older kernel companion demos that still spell explicit
  `kernel_target_config(...)` even though the current project build path can
  materialize target config from registered ABI metadata
* generated/demo artifact outputs under `examples/bins` that should be treated
  as rebuildable snapshots, not current source truth
* long example inventories that should route through frontdoor ladders instead
  of flat lists

## Current Example Policy

Use examples as proof routes, not as an encyclopedia.

Current preferred routes:

* core project path:
  [examples/projects/window_controls_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/window_controls_demo)
* kernel/tensor path:
  [examples/projects/kernel_tensor_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/kernel_tensor_demo)
* domain ladders:
  [examples/projects/domains/README.md](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/README.md)
* tooling CLI path:
  [examples/projects/tooling/README.md](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/README.md)
* source-level anchors:
  [examples/ns/README.md](/Users/Shared/chroot/dev/nuislang/examples/ns/README.md)
* handwritten YIR anchors:
  [examples/yir/README.md](/Users/Shared/chroot/dev/nuislang/examples/yir/README.md)

Demotion rule:

* old examples can remain if they still provide regression value
* old examples should not remain frontdoor if a project-form or tested route is
  stronger
* old generated artifacts should be regenerated locally when needed instead of
  treated as canonical truth

## Next Mainline Work

The best next work before `alpha-0.7.0` is:

1. tighten artifact-to-runtime execution probes around YIR sidecars and
   host-consumable domain units
2. keep replacing scaffold-only claims with tests or explicit "contract only"
   wording
3. update long-tail kernel examples away from explicit `kernel_target_config(...)`
   where automatic registered-ABI target config is now the clearer frontdoor
4. continue widening the std IO/filesystem/text CLI proving ladder from the
   now-running filesystem smoke set into broader CLI tools
5. make PixelMagic and WitSage prove CPU plus shader/kernel cooperation through
   real examples
6. keep `nustar` coupling under audit so compiler code knows contract shapes,
   not backend internals

## Reading Rule

When documents conflict:

1. prefer this file and the hardening plan for current alpha line intent
2. prefer reference docs and tests for exact implementation truth
3. treat `alpha-0.1.*`, `alpha-0.0.1`, and `0.20.*` docs as predecessor
   anchors unless they are explicitly linked from the current map
