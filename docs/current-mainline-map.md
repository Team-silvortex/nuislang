# Current Mainline Map

This is the short reading map for current implementation, not a history catalog.
When documents disagree, inspect the code/tests and current tensor evidence
before changing a capability claim.

## Fast Reading Order

The current `beta-0.17.*` priority is ns-nova application-led development.
Git records `beta-0.17.0` at `ccc16af9` and `beta-0.17.3` at `81dc1669`.
Start with the [current snapshot](versioning/nuis-beta-0.17.0-snapshot.md),
[current checklist](versioning/nuis-beta-0.17.0-release-checklist.md) and
[native session proof index](reference/nuis-native-session-proof-index.md).

The governing direction remains the
[application-led agreement](versioning/nuis-beta-0.11-application-led-mainline.md).
The [beta-0.10 migration entry](versioning/nuis-beta-0.10.0-self-hosting-entry.md)
and [beta-0.6 foundation entry](versioning/nuis-beta-0.6.0-mainline-entry.md)
retain their historical role. They are not new acceptance results.

## Start Here

Run from the repository root:

```sh
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run -p nuis -- dev-tensor --json
```

The current goal is `standard-library/ns-nova/interactive-image-workflow`.
Its selected prerequisite is
`standard-library/ns-nova/persistent-application-session`, still active/99.
Selection follows the [declared dependency plan](reference/nuis-development-tensor.mainline.toml),
not the globally lowest percentage. Correctness regressions may interrupt it.

The latest [mixed-parent-exit receipt](reference/nuis-native-stateful-loop-return-parent-mixed-v1.md)
checks ordinary parent break/continue and selected typed child returns in the
same function: actual prior-State handoff, exact source steps, real false/zero
returns, reached reservations and sixteen delivery images.
This bounded CPU worktree acceptance is not a release, GPU acceptance or a
completed native application.

The [effect-prefix certificate](reference/nuis-native-return-effect-prefix-v1.md)
now checks selected work, exact steps and pending propagation in the independent
return rewrite. The separate [storage-sharing proof](reference/nuis-native-return-effect-storage-v1.md)
now removes a duplicate record payload for eligible initial loop carries.
The [initialization-prefix follow-up](reference/nuis-native-return-effect-storage-prefix-v1.md)
now admits interleaved literal prints without moving them. The
[candidate-attempt proof](reference/nuis-native-return-effect-storage-candidates-v1.md)
tries at most eight in declaration order, publishing only a fully certified plan.
The [reassignment proof](reference/nuis-native-return-effect-storage-reassignment-v1.md)
now admits ordered same-type prefix writes while retaining fresh initialization.
Next validate record-returning effectful helper calls in those prefix writes.
Keep checks/calls once-only and source-ordered,
failure before pending/result publication, a separate pending signal and
independent payload fallback for every unproved reduction.
The separate backlog of further changing backedges and wide private captures
retains its own proof gates and does not replace the selected frontier.

| Execution Route | Existing Evidence | Remaining Boundary |
| --- | --- | --- |
| Embedded-YIR application | Nuis callbacks, real Metal image processing, explicit AppKit/headless session and replay/provider observations | Not all CPU callbacks execute through the native-session profile; provider and packaging gates remain explicit. |
| Native CPU session | Bounded typed callbacks, current State, selected effects, loops/exits, cache and source-free restoration | Not arbitrary resource/I/O/asynchronous capture or general effectful snapshot coalescing. |
| Nsld private image | Scoped Mach-O/ELF assembly and explicit admission/final-output selection | Not a self-contained heterogeneous application or full backend/PE parity. |

Cancellation admission, host-scope retirement and provider/device retirement are
distinct. Follow the [cancellation contract](reference/nuis-yir-application-cancellation-v1.md)
and [provider drain](reference/nuis-yir-provider-session-drain-v1.md); neither a
host acknowledgement nor a smoke report proves device completion.

Use the proof index for source shape, exact budgets and test counts. Do not add
totals from different receipts or turn a historical boundary into a current veto
without checking its later follow-up.

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
