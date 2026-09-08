# Current Mainline Map

This is the short reading map for current implementation, not a history catalog.
When documents disagree, inspect the code/tests and current tensor evidence
before changing a capability claim.

## Fast Reading Order

The current `beta-0.12.*` priority is the ns-nova application-led dependency
chain agreed in [beta 0.11](versioning/nuis-beta-0.11-application-led-mainline.md).
The [beta-0.12 snapshot](versioning/nuis-beta-0.12.0-snapshot.md) records
`505c820c` (`beta-0.12.2`); Git remains authoritative for later patches.

1. [Repository overview](../README.md)
2. [Mainline selection and acceptance coordinates](reference/nuis-development-tensor-mainline.md)
3. [Machine-readable application boundary](reference/nuis-ns-nova-application-lifecycle-v1.toml)
4. [Stateful window contract](reference/nuis-yir-window-session-v3.md)
5. [Cancellation and host retirement](reference/nuis-yir-application-cancellation-v1.md)
6. [Runnable image application](../examples/projects/domains/ns_nova_image_showcase/README.md)
7. [Focused validation checklist](versioning/nuis-beta-0.12.0-release-checklist.md)

Use the [versioning index](versioning/README.md) for the earlier
[beta-0.10 migration entry](versioning/nuis-beta-0.10.0-self-hosting-entry.md),
[beta-0.6 foundation](versioning/nuis-beta-0.6.0-mainline-entry.md) and older
alpha/pre-alpha anchors. Those checkpoints must not replace current behavior.

## Start Here

Run from the repository root:

```sh
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run -p nuis -- dev-tensor --json
```

The current goal is `standard-library/ns-nova/interactive-image-workflow`.
Its selected prerequisite is
`standard-library/ns-nova/persistent-application-session`.
Selection follows the [declared dependency plan](reference/nuis-development-tensor.mainline.toml),
not the globally lowest percentage. Correctness regressions may interrupt it.

Current session evidence:

- Registered Nuis open/event/close helpers retain scalar aggregate state and
  one provider connection across independent host events.
- Compiled AppKit/Metal regressions check input, actual image pixels, replay,
  worker/cache reuse, explicit cleanup and failure-preserving exit.
- An explicitly selected CPU parent opens before its child and receives one
  owned terminal outcome; its successful cleanup cannot certify child success.
- Pump, window and C ABI expose one independent cancellation ticket.
  It survives window destruction without implicit close, a fabricated terminal
  outcome or parent delivery. The packaged host now consumes it through explicit
  `--window-cancel-after-events`, with real Metal and compiled replay regressions.
- Cancellation admission, host-scope retirement and provider/device resource
  retirement are distinct. Explicit registered-worker drain now has provider-owned
  evidence; an opt-in host-library cancellation ticket separately observes it.

The [provider-session drain boundary](reference/nuis-yir-provider-session-drain-v1.md)
now distinguishes explicit worker-scope retirement from successful completion,
without publishing replacement replay. Pump/window/C ABI drain admission is now
connected through the generic provider-scope boundary, with first-fault and damaged
exchange guards. The packaged script now accepts opt-in `--drain-provider` behind
an exact bundle capability; the shared policy requires a typed Drained terminal
and cancellation exit, rejecting Finish before publication. It never publishes
cancelled work as successful launch/trace evidence. The terminal contract and
launch policy have no OS/window/backend dependency. AppKit and Unix IPC are
current adapters; non-AppKit/headless packaged entry is the next conformance
boundary and Windows transport is not yet claimed.
Cross-session resource reuse remains a separate, unproven boundary.
The default host-only path acknowledges its own scope, while the live provider
still treats EOF without Finish as incomplete execution. Ordinary quit still uses
close/Finish; parent cancellation remains unsupported. Do not import provider
registries, transports or concrete cancellation state into the window adapter.

This does not close resource-capability state, peer recovery, general multi-child
routing, richer image bindings, sustained performance, native CPU frame dispatch
or a self-contained Nsld application image. The broad engine coordinate is not
an engine-completion percentage.

## Current Truth By Layer

| Area | Current Reference |
| --- | --- |
| Development model | [Tensor protocol](reference/nuis-development-tensor.md), [mainline selection](reference/nuis-development-tensor-mainline.md) |
| Frontend/lowering | [Control flow](reference/control-flow-lowering-contract.md), [generic diagnostics](reference/generic-diagnostic-ownership-contract.md), [source encoding](reference/source-text-encoding-contract.md) |
| Memory/task safety | [NIR memory](reference/nir-memory-model.md), [task/GLM](reference/cpu-task-glm-contract.md), [thread/lock boundary](reference/cpu-thread-lock-boundary.md), [FFI pointers](reference/ffi-pointer-safety-boundary.md) |
| Runtime sessions | [Application lifecycle](reference/nuis-yir-application-session-v1.md), [window](reference/nuis-yir-window-session-v3.md), [terminal outcomes](reference/nuis-yir-application-outcome-v1.md), [parent pump](reference/nuis-yir-application-outcome-pump-v1.md), [cancellation](reference/nuis-yir-application-cancellation-v1.md) |
| Nustars | [Capability ownership](reference/nustar-capability-split-boundary.md), [multi-backend artifacts](reference/nustar-multi-backend-artifact-contract.md), [CFFI domain](reference/cffi-von-neumann-domain-contract.md), [provider IPC](reference/nuis-yir-provider-runtime-ipc-v4.md) |
| Linking/launch | [Native workflow](reference/nuis-native-artifact-workflow.md), [NSB protocol](reference/nuis-binary-format-protocol.md), [Nsld](reference/nsld-linker-frontdoor.md), [assembly gaps](reference/nsld-binary-assembly-gap-map.md) |
| Debugging/distribution | [Nsdb](reference/nsdb-yir-debugger-frontdoor.md), [Nsbdr](reference/nsbdr-bundler-frontdoor.md), [reusable capability boundary](reference/toolchain-galaxy-core-boundary.md) |
| Official Galaxies | [Std](../stdlib/std/README.md), [PixelMagic](../stdlib/pixelmagic/README.md), [WitSage](../stdlib/witsage/README.md), [ns-nova](../stdlib/ns-nova/README.md) |
| Compiler migration | [Readiness](reference/nuis-self-hosting-readiness.md), [data model](reference/nuis-compiler-data-model.md), [stage handoff](reference/nuis-compiler-stage-handoff.md), [candidate production](reference/nuis-compiler-candidate-production.md), [candidate-to-Nsld boundary](reference/nuis-compiler-candidate-nsld-materialization.md) |

The five bounded migration-preparation gates remain closed; actual compiler
responsibility transfer is separate. Use the readiness/component references for
canonical payloads, trust pins, differential/reproducibility and rollback
commands rather than reproducing their entire history in this router.

## Fast Example Routes

| Task | Start With | Evidence Boundary |
| --- | --- | --- |
| Basic host CLI | [filesystem_io_report_demo](../examples/projects/tooling/filesystem_io_report_demo), [stdin_runtime_demo](../examples/projects/tooling/stdin_runtime_demo), [cli_report_file_demo](../examples/projects/tooling/cli_report_file_demo) | Native executables with observable I/O and output-file checks. |
| Text statistics | [cli_wc_demo](../examples/projects/tooling/cli_wc_demo) | One 4096-byte read and ASCII separators, not complete streaming `wc`. |
| Simple file/image transform | [cli_pgm_invert_demo](../examples/projects/tooling/cli_pgm_invert_demo) | Checked small PGM input/output, not a general image-codec library. |
| Current application goal | [ns_nova_image_showcase](../examples/projects/domains/ns_nova_image_showcase) | Real Metal processing, compiled export and explicit stateful AppKit window; embedded YIR host runtime remains. |
| Small render/uniform route | [ns_nova_showcase](../examples/projects/domains/ns_nova_showcase) | Bounded frame loop, registered render resources and live/replay evidence. |
| Linker closure | [native_artifact_closure_demo](../examples/projects/tooling/native_artifact_closure_demo) | Inspect the selected finalizer and admission evidence; a planned image is not automatically runnable. |
| Compiler component | [bootstrap_structural_projection_candidate](../examples/projects/tooling/bootstrap_structural_projection_candidate) | Bounded candidate proof, not general self-compilation or replacement authority. |
| Linux device work | [CUDA bring-up](reference/linux-cuda-provider-bringup.md), [domain examples](../examples/projects/domains/README.md) | Run provider-specific gates on suitable hardware; absent hardware is unverified, not fallback success. |

The [observable CLI regression](../tools/nuis/tests/std_filesystem_smoke.rs)
builds and runs seven project routes, including PixelMagic/WitSage reports.
Those reports alone are not device-execution evidence. The separate Metal tests
check actual dispatched image data.

The packaged AppKit application now exposes standalone scripted cancellation
through `--window-cancel-after-events`. This does not add a Nuis intrinsic,
CFFI signature grant, parent cancellation route or device-retirement authority.

## Deep Routers

- [Documentation index](README.md) and [reference index](reference/README.md)
- [Project examples](../examples/projects/README.md), [source examples](../examples/ns/README.md), [freshness audit](examples-freshness-audit.md)
- [Std tooling](../stdlib/std/tooling/README.md), [filesystem](../stdlib/std/filesystem/README.md), [network](../stdlib/std/network/README.md)
- [Repository layout](repo-layout.md) and [file-line policy](repo-file-line-policy.md)
- [Application-led design](versioning/nuis-beta-0.11-application-led-mainline.md) and [long-range OS roadmap](versioning/nuis-long-range-heterogeneous-os-roadmap.md)

Rendering, control, ML and audio are an engine horizon, not four completed
workflows. Shared YIR/GLM/time contracts preserve independent Galaxy/Nustar
ownership. Hardware extensibility and scheduling efficiency require separate
measurements; a smoke test is not an advantage over MLIR or a mature language.

## Cleanup Rule

Keep this map short. Current behavior belongs in focused reference documents;
minor checkpoints belong in versioning, and examples state their own input,
backend and execution limits. Keep paths repository-relative. Update tensor
evidence and documentation drift guards after each accepted change, without
raising capability scores for documentation work.
