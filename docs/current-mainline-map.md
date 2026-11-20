# Current Mainline Map

This router connects current tasks, execution routes and their evidence.
Use code/tests and the live development tensor to resolve implementation claims.

## Current Direction

The current `beta-0.18.*` priority is ns-nova application-led development.
Git history is authoritative; dated committed identity and worktree follow-ups
are separated in the [snapshot](versioning/nuis-beta-0.18.0-snapshot.md).
The [validation checklist](versioning/nuis-beta-0.18.0-release-checklist.md) gives
operating gates; the [proof index](reference/nuis-native-session-proof-index.md)
catalogues detailed native-session receipts without duplicating them here.

The [application-led agreement](versioning/nuis-beta-0.11-application-led-mainline.md)
keeps rendering, control, ML and audio as engine goals. Foundation hardening,
measured optimization and module-by-module compiler migration serve that goal.
The [beta-0.10 migration entry](versioning/nuis-beta-0.10.0-self-hosting-entry.md)
is a phase anchor, not completed self-hosting.

The [direct effectful record-return follow-up](reference/nuis-native-direct-effectful-records-v1.md)
removes the loop-wrapper requirement from the verified conditional helper path.
It keeps pure discovery and backend effect grants separate.

## Selected Prerequisite

Run from the repository root:

```sh
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run -p nuis -- dev-tensor --json
```

The goal is `standard-library/ns-nova/interactive-image-workflow`.
The [declared dependency plan](reference/nuis-development-tensor.mainline.toml)
selects this chain, not the globally lowest percentage:

```text
native-resource-session-callbacks (active/95)
  -> native CPU frame dispatch (early/10)
  -> standard-library/ns-nova/persistent-application-session (active/60)
  -> interactive-image-workflow
```

The [2026-11-13 recalibration](reference/nuis-development-tensor-recalibration-2026-11-13.md)
retains the preceding early/0 checkpoint. Current resource evidence is the
[short-circuit proof](reference/nuis-native-resource-short-circuit-v1.md):
fixed-extent synchronous Buffer/Bytes callbacks, flat i64-plus-Bytes State,
checked errors and early owned returns execute natively. Both nonempty arms
can rejoin one shared suffix after an untaken inner return, preserving ownership
and once-only effects. Explicit, inferred and aliased i64 local declarations
preserve lexical identities through shared semantic type resolution; true
assignments remain distinct. Flat and nested i64/bool record helpers compose with held
Bytes through checked calls and allocation-free private value returns. Projections
check complete records, including unselected subtrees, without consuming owners.
Boolean validation, negation and same-kind equality/inequality retain exact types
and nonpublishing checked error paths. Exact bool/i64 parameters compose through
direct and helper-to-helper calls without integer coercion in native value calls.
Immutable bool locals cross resource-return outlines as typed value snapshots;
branch-local flags remain separate from the shared root continuation's scope.
Same-kind bool shadows retain separate identities across explicit, inferred,
aliased and mutable/redeclared locals; each enclosing value is restored on exit.
Bounded i64/bool scalar helpers share the record-helper call graph. Pure logical
initializers preserve selected-only RHS evaluation; unoutlined non-total RHS
expressions now fail rather than falling back to eager execution.
Registration, cache, source-free NSB relocation,
binary-only launch and final accepted CPU pixel export have scoped regressions.

Canonical negation such as `!(a || helper(...))` now uses the same guarded
normalization directly, including nested/double negation and original budgets.

Nested resource-read declarations now reuse owned branch pairs and shared
suffixes. Left/right logical subtrees and negation evaluate reads only on the
selected path; resources do not enter the pure helper ABI. Lazy left results
have independent lexical snapshots, and existing terminal continuations are
forwarded rather than wrapped in redundant helpers.

Read/helper composition now keeps argument reads and pure scalar/record calls
inside selected owned regions, without widening pure helper signatures.
Short-circuit bool arguments now preserve left-to-right snapshots and selected
reads, including nested and multiple arguments.
Inlined conditional helpers now admit exact bool-controlled `select` over two
ready same-kind i64/bool values, without selecting records/resources or coercing types.
Comparison/XOR wrappers now reuse ordered scalar snapshots and guarded logical
children, including helper arguments and exact i64 ordering comparisons.
Direct i64 local initialization now uses the same typed snapshots and guarded
arguments, including inferred and const-alias declarations. Complete source
proof still precedes extraction, and the existing closure limits remain.
Direct inline bool conditions now reuse that path through one hygienic snapshot,
without requiring a source local or duplicating either branch. The snapshot still
counts toward the original capture limit; both arms retain existing owner joins.
Directly returned mixed-State fields now use ordered snapshots, including
`width: width + word(mode >= 0 && bytes_len(saved) == 48)` beside `pixels: move(saved)`.
Plain State and checked Ok layouts share the contract; terminal cleanup bindings
retain field order and exact Bytes transfer.
Direct checked Err codes now use one typed snapshot before terminal cleanup.
Owned continuations remain guarded even when they capture no Bytes; they are
not candidates for eager pure-helper return selection.
Borrowed `buffer_len` now participates in the same scalar vocabulary, such as
`width: buffer_len(pixels) + word(mode >= 0 && bytes_len(saved) == 48)`.
Exact `ref Buffer` proof, original expression budgets and scalar snapshot order remain.
Helper-returned value-record locals now stage their scalar arguments and retain
the exact nominal binding, including nested projections and const aliases.
Project record fields to scalar locals before crossing later resource continuations.
Nested record literal locals now stage scalar leaves in source field order,
including `let local: Flags = Flags { valid: mode == 0 && buffer_len(pixels) == 6 };`.
Native local construction now distinguishes pure values from owned mixed State,
while retaining the shared value-layout limits and exact State rules.
Record-valued helper fields now execute once and immediately freeze scalar leaves,
including projected nested results and multiple helper fields. Only scalar
snapshots cross subsequent resource branches; reconstructed shapes are bounded.
Previously bound records now use a separate block-local availability proof:
`let prior: Packet = packet(0, true, 0); let local: Envelope = Envelope { packet: prior, unused: mode == 0 && buffer_len(pixels) == 6 };`.
Exact nominal nested projections freeze scalar leaves before later resource branches.
Rebindings and control-flow boundaries invalidate the local table, not the capture ABI.
Scalar liveness now also handles a lazy field before an existing record field:
`let local: Envelope = Envelope { unused: mode == 0 && buffer_len(pixels) == 6, packet: prior };`.
Only completed record projections move before evaluation; repeated/overlapping
paths share scalar snapshots. New calls, reads and checked arithmetic stay ordered.
Local-record scalar operands now participate in constructor predicates:
`let local: Envelope = Envelope { unused: prior.flags.valid && bytes_len(saved) == 48, packet: prior };`.
The original tree is bounded before exact type-witness proof; witnesses are never
emitted. Shared pure projections serve both predicates and later record fields,
including nested paths, indices, helper arguments and comparison/XOR wrappers.
Standalone scalar declarations now reuse that proof and snapshot planner:
`let valid: bool = prior.flags.valid && bytes_len(saved) == 48;`.
Bool/i64 declarations keep their original lexical identity; reads, calls and
checked arithmetic remain inside the existing owned-region normalization.
Unchanged or already rewritten declarations stay on the ordinary scalar route.
Direct resource conditions now reuse that proof without an intermediate declaration:
`if prior.flags.valid && bytes_len(saved) == 48 { return Result.Ok(Canvas { width: width + 1, pixels: move(saved) }); }`.
Original arms and suffix remain intact. Checked-error arms use registered outcome
identity even without held Bytes, rather than selecting owned Results as scalars.
Generated temporaries remain inside the unchanged 33-input guard budget.
**Next:** admit local-record operands in directly returned State fields:
`return Result.Ok(Canvas { width: word(prior.flags.valid && bytes_len(saved) == 48), pixels: move(saved) });`.
This return still rejects with `short-circuit RHS requires guarded lowering`.
Preserve selected-only reads,
borrow lifetimes, lexical identities, whole-state publication and native traps.
Captured writes,
resource-changing joins, general resource loops, broader layouts and provider/device
completion remain separate. CPU export does not establish interactive frame dispatch.

Use the [resource example ladder](../examples/projects/domains/README.md#native-cpu-resource-sessions)
for runnable entrypoints and the [implementation owner map](repo-layout.md#native-resource-session-ownership)
for changes across compiler, contracts, lowering, host and delivery boundaries.

### Retained Scalar Baseline

The bounded scalar/record session profile remains stable/100, not all CPU
lowering or the whole application. Its
[direct effectful-record checkpoint](reference/nuis-native-direct-effectful-records-v1.md)
and [proof catalog](reference/nuis-native-session-proof-index.md) preserve the
full return, loop, print and field-transport history. Further combinations only
become selected work when a concrete application witness requires them. The
backlog of further changing backedges and wide private captures retains its own gates.

Keep effects once-only and source-ordered, failure before State publication,
and independent storage fallback for unproved reductions. Correctness
regressions may interrupt the selected feature. Documentation and storage
maintenance do not advance functional tensor scores.

## Execution Routes

| Route | Existing Evidence | Remaining Boundary |
| --- | --- | --- |
| Embedded-YIR application | Nuis callbacks, real Metal image work, explicit AppKit/headless sessions and provider observations. | The host embeds the YIR lifecycle runtime; not every callback executes through native CPU lowering. |
| Native CPU session | Bounded registered callbacks, current State, selected effects, loops/exits, cache and source-free restoration. | Not arbitrary resource/I/O/async capture or a complete native engine. |
| Native resource session | Fixed-extent Buffer/Bytes and flat i64-plus-Bytes records, bounded ownership, exact registration/NSB/export checkpoints, source-free launch and State-driven CPU canvas export. | Not yet resource loops/helpers, arbitrary mixed layouts, native ns-nova windows or provider/device completion; not pure Nsld finalization. |
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
| Native CPU canvas | [native_canvas_session_demo](../examples/projects/domains/native_canvas_session_demo) | Nuis pixel callbacks and final accepted RGBA8 State exported as PPM; source-free, CPU-only, no window/provider dispatch. |
| Native mixed-State canvas | [native_mixed_canvas_session_demo](../examples/projects/domains/native_mixed_canvas_session_demo) | Nuis-owned dimensions, event count, cursor and pixels; complete State publication and source-free final image export. |
| Native checked canvas | [native_checked_canvas_session_demo](../examples/projects/domains/native_checked_canvas_session_demo) | Nuis-side dimension checks and Result errors; no state/frame publication on rejection, including source-free execution. |
| Native guarded canvas | [native_guarded_canvas_session_demo](../examples/projects/domains/native_guarded_canvas_session_demo/README.md) | Held Bytes across both arms' early returns and continuations; one shared suffix, no interactive window or GPU dispatch. |
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
- [Architecture/storage audit](repo-architecture-audit.md) and [workspace boundary check](../scripts/check-workspace-architecture.py)
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
