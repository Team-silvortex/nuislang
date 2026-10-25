# Current Mainline Map

This router connects current tasks, execution routes and their evidence.
Use code/tests and the live development tensor to resolve implementation claims.

## Current Direction

The current `beta-0.17.*` priority is ns-nova application-led development.
Git history is authoritative; dated committed identity and worktree follow-ups
are separated in the [snapshot](versioning/nuis-beta-0.17.0-snapshot.md).
The [validation checklist](versioning/nuis-beta-0.17.0-release-checklist.md) gives
operating gates; the [proof index](reference/nuis-native-session-proof-index.md)
catalogues detailed native-session receipts without duplicating them here.

The [application-led agreement](versioning/nuis-beta-0.11-application-led-mainline.md)
keeps rendering, control, ML and audio as engine goals. Foundation hardening,
measured optimization and module-by-module compiler migration serve that goal.
The [beta-0.10 migration entry](versioning/nuis-beta-0.10.0-self-hosting-entry.md)
is a phase anchor, not completed self-hosting.

## Selected Prerequisite

Run from the repository root:

```sh
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run -p nuis -- dev-tensor --json
```

The goal is `standard-library/ns-nova/interactive-image-workflow`.
Its selected prerequisite is
`standard-library/ns-nova/persistent-application-session`, active/99 at this audit.
The [declared dependency plan](reference/nuis-development-tensor.mainline.toml)
selects that prerequisite, not the globally lowest percentage. Correctness
regressions may interrupt it; documentation maintenance does not advance it.

| Current Native CPU Scope | Read First |
| --- | --- |
| Ordered effectful returns and complete State publication | [Effect-prefix certificate](reference/nuis-native-return-effect-prefix-v1.md), [mixed loop exits](reference/nuis-native-stateful-loop-return-parent-mixed-v1.md). |
| Bounded record storage reuse with independent fallback | [Storage certificate](reference/nuis-native-return-effect-storage-v1.md), [payload/field execution](reference/nuis-native-return-effect-storage-field-payload-v1.md). |
| Guarded scalar/record joins and owned value provenance | [Logical fields](reference/nuis-native-logical-record-fields-v1.md), [two-arm origins](reference/nuis-native-origin-record-join-v1.md). |
| Explicit pure-i64 print values and root entry reads | [Computed prints](reference/nuis-native-computed-iteration-print-v1.md), [root epochs](reference/nuis-native-carry-epoch-print-v1.md). |
| Fresh child entry reads without ancestor-grant inheritance | [Child epochs](reference/nuis-native-child-epoch-print-v1.md), a separate worktree follow-up at the dated audit. |
| Pre-update nested i64 fields without whole-record readiness | [Original-path field epochs](reference/nuis-native-field-epoch-print-v1.md), using the same root/child entry rules. |
| Pre-update bool fields at exact pure-i64 helper arguments | [Typed bool arguments](reference/nuis-native-bool-field-epoch-print-v1.md), not ambient bool expression permission. |
| Pre-update i32 fields with exact signed State words | [Shared typed arguments](reference/nuis-native-i32-field-epoch-print-v1.md), retaining bool and ordinary-read boundaries. |
| Pre-update f32 fields with raw callee/State words | [IEEE field arguments](reference/nuis-native-f32-field-epoch-print-v1.md), separating raw payload proof from finite CLI delivery. |
| Pre-update f64 fields with complete 64-bit words | [Full-width field arguments](reference/nuis-native-f64-field-epoch-print-v1.md), retaining high bits, low-bit distinctions and entry budgets. |

The next selected gate is **typed unary-float arguments from pre-update fields**,
starting with f32/f64 sign negation inside a completed pure-i64 print value.
It needs original-leaf/entry/type proof and raw sign-bit/fault controls; direct
field permission does not imply compound arguments or whole-record readiness.
General effects, resources, references and arbitrary
outer writes remain outside these bounded CPU admissions.

Keep checks/calls once-only and source-ordered, failure before result/pending
publication, and independent fallback for every unproved storage reduction.
The backlog of further changing backedges and wide private captures retains
its own gates. Full history and numerical test results belong in receipts.

## Execution Routes

| Route | Existing Evidence | Remaining Boundary |
| --- | --- | --- |
| Embedded-YIR application | Nuis callbacks, real Metal image work, explicit AppKit/headless sessions and provider observations. | The host embeds the YIR lifecycle runtime; not every callback executes through native CPU lowering. |
| Native CPU session | Bounded registered callbacks, current State, selected effects, loops/exits, cache and source-free restoration. | Not arbitrary resource/I/O/async capture or a complete native engine. |
| Nsld private image | Scoped Mach-O/ELF assembly and explicit admission/final-output selection. | Not a self-contained heterogeneous application or full provider/PE parity. |

Do not transfer acceptance between these routes. Cancellation admission,
host-scope retirement and provider/device retirement are distinct; use the
[cancellation contract](reference/nuis-yir-application-cancellation-v1.md) and
[provider drain](reference/nuis-yir-provider-session-drain-v1.md) for their owners.
Neither a host acknowledgement nor a smoke report proves device completion.

## Current Truth By Layer

| Area | Current Reference |
| --- | --- |
| Development model | [Tensor protocol](reference/nuis-development-tensor.md), [mainline selection](reference/nuis-development-tensor-mainline.md) |
| Production architecture | [Software manufacturing direction](reference/nuis-software-manufacturing-architecture.md), [RC policy and current limits](reference/nuis-rc-cache-lifecycle.md) |
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
