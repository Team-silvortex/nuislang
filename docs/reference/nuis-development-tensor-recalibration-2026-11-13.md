# Development Tensor Recalibration: 2026-11-13

This is a scope and dependency audit at the beta-0.18.8 Git baseline, with the
existing uncommitted lowering work preserved. It is not a release, a fresh
all-platform certification, or a rewrite of historical execution receipts.

The subsequent [native Buffer/Bytes proof](nuis-native-resource-session-v1.md)
first advanced the resource coordinate to active/35; the later
[artifact proof](nuis-native-resource-session-artifacts-v1.md) advances it to active/60.
The [CPU canvas follow-up](nuis-native-resource-frame-export-v1.md) then advances
only that resource coordinate to active/65; frame dispatch remains early/10.
The [mixed-State proof](nuis-native-mixed-resource-state-v1.md) subsequently advances
the same coordinate to active/75 without promoting the dependent frame/window gates.
The [checked-State proof](nuis-native-checked-resource-state-v1.md) advances it to
active/80 for Nuis-owned validation and guarded Err, retaining the same dependent gates.
The [guarded resource return proof](nuis-native-guarded-resource-return-v1.md)
advances it to active/85 for branch-local early Ok and cleanup, without promoting
the frame/window gates.
The [retained-Bytes return proof](nuis-native-retained-bytes-return-v1.md) advances
it to active/90 for taken-edge caller cleanup and surviving-path ownership, still
without promoting the frame/window gates.
The [transferred-Bytes return proof](nuis-native-transferred-bytes-return-v1.md)
advances it to active/95 for selected-arm ownership transfer under independently
verified helper contracts, retaining the same dependent gates.
The [nested owned-return proof](nuis-native-nested-owned-return-v1.md) adds nested
terminal regions while retaining active/95. Its next task is a nonterminal outer
resource join; the dependent gates and other coordinate records remain unchanged.
The [owned-continuation proof](nuis-native-owned-continuation-v1.md) subsequently
closes empty-else outer joins through one shared suffix, retaining active/95 and
the same dependent gates. Nonempty else joins are its next concrete task.
The subsequent [paired-continuation proof](nuis-native-paired-owned-continuation-v1.md)
closes that task without a new opcode or ABI: both arms retain independent early
returns and share one suffix. Active/95 and the other coordinates remain unchanged;
branch-local scalar capture identities are the next boundary.
The [scalar-shadowing follow-up](nuis-native-resource-scalar-shadowing-v1.md)
now preserves explicit i64 declaration identities before NIR, including nested
branches and source-free project delivery. Inferred/aliased shadows remain next;
active/95 and all other coordinates are unchanged.
The [inferred-shadowing follow-up](nuis-native-resource-inferred-shadowing-v1.md)
then resolves inferred/aliased declarations through shared semantics and checks
canonical i64 identities before optimization. The reproduced next gap is an
ordinary scalar-record helper inside a resource callback (`call_owned_struct`);
active/95 and the dependent coordinates remain unchanged.
The [record-helper follow-up](nuis-native-resource-record-helpers-v1.md) closes
that flat-i64 case through shared admission and allocation-free value returns.
Both helper families share the original closure bounds; nested scalar-record
projections are the next reproduced gap. Active/95 remains unchanged.
The [nested-record follow-up](nuis-native-resource-nested-records-v1.md) then
admits nested i64 value projections with whole-record resource checks, shared
leaf/depth bounds and unchanged private transport. Boolean helper validation
flags remain next; active/95 and the dependent coordinates remain unchanged.
The [boolean-record follow-up](nuis-native-resource-boolean-records-v1.md) then
admits bool result fields, exact predicates and same-kind equality/inequality,
with checked errors and unchanged ownership. Bool helper parameters remain the
next reproduced boundary; active/95 and other coordinates remain unchanged.
The [boolean-parameter follow-up](nuis-native-resource-boolean-parameters-v1.md)
then admits exact bool/i64 value arguments through both callback and helper calls,
and rejects helper-internal bool-to-integer coercion. Immutable bool captures
across outlined resource branches remain the next reproduced gap; active/95
and the dependent coordinates remain unchanged.
The [boolean-capture follow-up](nuis-native-resource-boolean-captures-v1.md)
then preserves immutable bool snapshots through guarded arms and terminal
continuations, with exact argument kinds and unchanged owner partitions.
Same-kind bool shadowing was the next reproduced gap; active/95 and the
dependent coordinates remain unchanged.
The [boolean-shadowing follow-up](nuis-native-resource-boolean-shadowing-v1.md)
then admits matching canonical i64/bool shadows before optimization, retaining
outer initializer reads, nested lexical restoration and failure cleanup. It
reproduces short-circuit bool initializer rejection at `call_bool` as the next
boundary; active/95 and all other coordinate values remain unchanged.
The [short-circuit follow-up](nuis-native-resource-short-circuit-v1.md) then
admits pure i64/bool helper calls through the shared bounded closure and strict
value checks, preserving selected-only RHS execution. It also closes the unsafe
eager fallback for unoutlined resource RHS reads; selected-only resource capture
remains open. Its negation follow-up now handles `!(a || helper(...))` directly
in resource prefixes, with nested/double negation and unchanged source budgets.
Active/95 and the dependent coordinates remain unchanged.
The subsequent resource-read follow-up admits one lazy edge in bool declarations:
Buffer loads and live Bytes length reads stay in selected owned regions, sharing
one continuation without a new pure-helper resource ABI. Four predicates across
three declaration forms each retain 120-event native evidence and failure cleanup.
Nested resource logical trees were next. That follow-up now covers left/right
subtrees and canonical negation, preserving selected-only reads, lexical
snapshots, shared suffixes and failure cleanup. Redundant identity continuations
are removed rather than raising the source/native closure bounds. Read/helper
composition now reuses completed pure signatures and selected owned regions;
logical bool arguments now retain left-to-right scalar snapshots, nested selection
and existing owner cleanup. Inlined conditional scalar helpers producing `select`
now admit exact bool predicates and ready same-kind i64/bool arms; no resource
selection or speculative reads are granted. Comparison/XOR wrappers now use
ordered snapshots and existing owned branches for their logical children.
Direct i64 local initialization containing resource short-circuit arguments
now shares that normalization with exact declaration/result kinds, unchanged
ownership and closure limits. Eight compositions across three declaration forms
each run 120 native events; full-width values and failure cleanup remain covered.
Direct inline resource-bearing bool conditions now reuse the same path through
one hygienic snapshot. Eight conditions retain 120-event evidence; paired arms
preserve branch-local and incoming owners through early returns and shared suffixes.
The snapshot counts toward the original capture bound. Returned mixed-State
fields now normalize through shared layout contracts and source-ordered snapshots,
including terminal cleanup bindings. Direct checked Err code expressions now
share this route, including zero/full-width codes and borrowed-only callbacks.
Inline `buffer_len` composition is the next reproduced gap; this does not admit
arbitrary expressions or relax pure-helper resource isolation.
Active/95 and the other
coordinates remain unchanged.
The early/0 decision and
4515-check receipt below remain this audit's historical baseline, not the latest
implementation status.

## Decision

The live inventory grows from 35 to 37 coordinates. All 35 previous coordinates
remain registered; only the persistent-session score changes. Two explicit
compiler coordinates separate the already admitted scalar/value-record profile
from the still-missing registered native resource callback boundary.

* `native-scalar-record-session-profile`: `stable/100` for the admitted finite
  scalar/value-record profile, not arbitrary control or resource-bearing state.
* `native-resource-session-callbacks`: `early/0`. Existing rejection tests
  establish the boundary; passing them is not implementation success.
* `persistent-application-session`: `active/99` becomes `active/60`. This is a
  scope correction, not lost functionality or a measured 60% completion claim.
* `native-cpu-frame-dispatch`: retains `early/10`, but now precedes the
  persistent application-session goal instead of depending on it.

Progress is a conservative scope-specific triage label. A `stable/100` cell
closes its declared milestone, not its containing module, architecture, or the
whole language. The mean score is not a forecast of remaining engineering time.
Coverage means coverage of the declared inventory, not automatic discovery of
every missing feature in the repository.

## Actionable Dependency Path

The selected goal remains `standard-library/ns-nova/interactive-image-workflow`.
Its current goal-to-prerequisite path is:

`interactive-image-workflow -> persistent-application-session -> native-cpu-frame-dispatch -> native-resource-session-callbacks`

Execution proceeds in the reverse direction. Resource callbacks additionally
depend on the closed scalar/record profile and registered domain contracts.
Reopening the scalar profile reopens that dependency chain. Native frame dispatch
is a prerequisite of application closure, not proof that the application already
runs entirely as native CPU callbacks.

After those prerequisites close, the existing lifecycle failure/resource-safety
and shader-resource gates still block the interactive image goal. Sustained
performance and compiler-component ownership transfer remain later declared
goals. An unrelated low score, such as the future cache daemon, does not silently
replace the application mainline. A correctness interrupt can still preempt it.

The next implementation slice is one generic owned Buffer/Bytes callback path:
admission, capability/descriptor validation, invocation, completion, publication
and once-only cleanup. It must compare exact bytes against the reference path,
reject invalid capability entry, and account for trap/failure ownership. It must
not encode ns-nova logic into the compiler, smuggle pointers into scalar slots,
or silently fall back to interpreted execution.

## Evidence Boundary

The existing [direct effectful record receipt](nuis-native-direct-effectful-records-v1.md)
records 224 selected compiler tests, 15 native/reference tests and 120 CLI
launches. Those are prior local receipts, not 224 newly rerun tests in this audit.
The native scalar bridge admits bounded value records; its instruction gate
excludes resource and provider operations. Resource-bearing native State is also
rejected. Buffer-writing native function tests elsewhere and an embedded-YIR
image application are useful adjacent evidence, not this missing session contract.

The audit adds a focused rejection test in
`crates/yir-lower-llvm/src/native_session/resource_boundary_tests.rs` and retains
the runtime's native-session rejection coverage. The current selected validation
command is a baseline safety gate. Closing the coordinate requires a new positive
native-vs-reference resource witness in addition to these negative tests.

Real Metal/CoreML and Linux CUDA/Vulkan evidence remains in its original receipts.
This calibration does not rerun GPU hardware or the remote Linux server and does
not upgrade cross-host parity, physical data-fabric certification, resource
session acceptance or compiler replacement authority.

## Inventory Review

The table reviews all 35 existing coordinates and their scope boundaries. It does
not claim to rerun every historical test. Existing stable milestones are retained
rather than downgraded because the containing system has broader future work.

| Coordinate | Calibrated Triage | Scope And Remaining Boundary |
| --- | --- | --- |
| `compiler-frontdoor/nuis-cli/workflow-and-project-orientation` | stable/100 | CLI reading order, status and handoff projections; not whole-program execution. |
| `compiler-frontdoor/nuis-cli/artifact-runtime-closure` | stable/100 | Readiness and artifact execution surfaces; release-check itself remains read-only and is not runtime proof. |
| `linker-toolchain/nsld/artifact-chain-drive` | stable/100 | Whitelisted drive and replay boundaries; not arbitrary host link mutation. |
| `linker-toolchain/nsld/final-output-boundary` | stable/100 | Typed final-output and debugger projections; no universal platform claim. |
| `linker-toolchain/nsld/os-native-executable-finalization` | stable/100 | Recorded Mach-O arm64 and bounded GNU ELF amd64 lanes; broader ELF and PE/COFF remain open. |
| `heterogeneous-runtime/nustar/registered-domain-contracts` | stable/100 | Registered domain contracts and recorded provider chains; not every hardware capability. |
| `heterogeneous-runtime/data/provider-neutral-data-fabric` | usable/74 | Reference data fabric only; physical DPU/IPU/RDMA certification remains unclaimed. |
| `heterogeneous-runtime/linux-cuda/cuda-provider-bringup` | stable/100 | Recorded RTX 4050 provider closure; not a fresh remote run or universal NVIDIA coverage. |
| `heterogeneous-runtime/linux-vulkan/vulkan-provider-bringup` | stable/100 | Recorded Vulkan provider/debugger closure; not a fresh remote run or all GPU families. |
| `standard-library/std/host-io-filesystem-text` | stable/100 | Bootstrap IO/fs/text and CLI workload slice; richer codecs and async IO remain open. |
| `standard-library/std/concurrency-task-thread-lock` | stable/100 | Bounded slot synchronization; dynamic cardinality, fairness and a mature parallel executor remain open. |
| `standard-library/pixelmagic/image-processing-lane` | stable/100 | Early-beta image plan selection; richer resource ABIs, codecs and texture upload remain open. |
| `standard-library/witsage/classical-ml-lane` | stable/100 | Bounded classical inference; native training and engine integration remain open. |
| `standard-library/ns-nova/application-rendering-framework` | active/20 | Nuis-owned lifecycle plus embedded-YIR image execution; integrated rendering/control/ML/audio is not closed. |
| `standard-library/ns-nova/persistent-application-session` | active/60 | Active/99 becomes active/60: scalar native proofs do not close the native resource/frame path. |
| `standard-library/ns-nova/interactive-image-workflow` | early/0 | Load, edit, redraw and export in one persistent application are not yet verified. |
| `heterogeneous-runtime/application-session/lifecycle-failure-resource-safety` | early/0 | Existing scalar/provider fault checks do not establish owned-resource cancellation and retirement. |
| `heterogeneous-runtime/shader/shader-resource-bindings` | early/15 | Fixed uniform OR storage binding on Metal; mixed bindings, textures, samplers and dynamic extents remain open. |
| `standard-library/ns-nova/sustained-runtime-performance` | early/0 | No representative sustained interactive frame-time and memory-growth baseline. |
| `compiler-toolchain/nuisc/native-cpu-frame-dispatch` | early/10 | Bounded native aggregates exist; complete production CPU lifecycle/frame execution still uses embedded YIR. |
| `compiler-toolchain/bootstrap/compiler-component-ownership-transfer` | early/10 | Preparation and narrow candidate proofs do not replace a general production compiler module. |
| `language-core/nuisc/type-control-flow-generics` | stable/100 | Existing language milestone retained; affine enum-backedge i32/float and resource codec gaps are distinct from recent scalar-record support. |
| `host-compatibility/cffi/registered-pointer-string-object-boundary` | stable/100 | Registered bounded C memory capabilities; retained borrows, arbitrary pointers and general object layouts remain closed. |
| `package-system/galaxy/source-import-and-lock-resolution` | stable/100 | Local/offline import, lock and trust slice; remote acquisition, distribution and GC remain open. |
| `package-system/nuis-rc/heap-stack-cache-lifecycle` | early/20 | Synthetic-snapshot GC policy only; physical CAS inventory, real leases, crash-safe collection and daemon remain unimplemented. |
| `native-binary-system/nsb-nsld/self-owned-binary-assembly` | stable/100 | Bounded self-owned assembly and filesystem trust adapters; not all backends or OS-protected trust. |
| `native-binary-system/nuis-runtime/lifecycle-loader-bootstrap` | stable/100 | Loader/bootstrap and invocation permit slice; not every native service. |
| `native-binary-system/nuis-runtime/lifecycle-context-dispatch` | stable/100 | Registered context-dispatch slice; Linux SysV and Windows Win64 execution remain explicit cross-host items. |
| `developer-system/dev-tensor/architecture-module-function-progress-model` | stable/100 | Repository-local hierarchy/selection protocol; evidence-anchor checks are not semantic validation of all features. |
| `language-core/nuisc/bootstrap-language-subset` | stable/100 | Frozen bootstrap subset v8; larger grammar/capability sets require new contracts. |
| `compiler-toolchain/bootstrap/stage0-stage1-driver` | stable/100 | Bounded source-to-equivalent-Nsld-input driver; no claim of candidate-owned native object production. |
| `developer-system/bootstrap/differential-reproducibility-gate` | stable/100 | Local differential gate; not remote physical-independence attestation. |
| `standard-library/std/compiler-data-model` | stable/100 | Bounded compiler collections/arenas and payloads; not general unbounded compiler storage. |
| `language-core/nuisc/stage-neutral-ir-boundary` | stable/100 | Bounded producer-neutral three-page handoff; full-stream pagination and general aggregate backedges remain open. |
| `developer-system/dev-tensor/self-hosting-phase-roadmap` | stable/100 | Migration-entry roadmap; self-compilation, convergence and replacement authority remain open. |
| `compiler-toolchain/nuisc/native-scalar-record-session-profile` | stable/100 | New bounded compiler coordinate; existing scalar/record receipts, not resource or whole-application closure. |
| `compiler-toolchain/nuisc/native-resource-session-callbacks` | early/0 | New registered native ownership/completion boundary; rejection baseline only. |

The new cells belong to the existing application-led milestone, with
`required = false` and `bootstrap_critical = false`. These flags exclude them
from the old preparation-gate aggregate, not from the mandatory dependency path
of the current application goal. The five bounded self-hosting preparation gates
and their scores are unchanged. Full compiler ownership transfer remains open.
Rendering/control/ML/audio engine integration remains a horizon, not an implied
closure of the independently tested PixelMagic or WitSage packages.

## Readiness And Lineage

A valid dependency plan alone no longer makes an actionable card ready. Coverage,
recursive hierarchy, task/handoff lineage and evidence drift must all be clean.
A structurally ready plan with dirty or unknown drift produces a blocked card
without redirecting the selected coordinate. Clean anchors do not mean the
referenced runtime tests just ran or that their full semantics were reviewed.

All six complete preceding persistent-session fields are retained byte-for-byte
behind SHA-256 markers. The prior validation command is quoted as inert shell
history, not appended as another executable command. The previous 4509 drift IDs
remain in their original order; calibration guards are appended. Existing
historical proof documents and earlier task directions are not deleted or
rewritten to look current. The live plan, short task head and this audit take
precedence over dated historical next-step text.

Regression coverage now checks the resource -> frame -> session -> failure-safety
-> shader -> image sequence, reopening the scalar dependency, drift blocking,
unchanged preparation aggregates and all six predecessor hashes. New inventory
entries are registered in both milestone ownership and the derived fallback
inventory, keeping recursive coverage and task-card ancestry intact.

## Reproduce

Use the repository root; no host-specific absolute path, OS version or remote
credentials are part of the tensor contract.

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor mainline --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-lower-llvm --lib --locked -j1 -- native_session_resource_boundary --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-runtime-host --lib --locked -j1 -- native_application_session --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- state_return_direct_eager_pair --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run -p nuis --locked -j1 -- dev-tensor --json
```

A successful audit reports 37 covered coordinates, clean drift/hierarchy/lineage
and the resource-callback coordinate selected above. It must not report the new
resource coordinate stable merely because its rejection tests pass.

## Local Validation Receipt

The final ordinary CLI reports 37 covered cells and 4515 clean drift checks.
All 4509 preceding IDs retain their order; 34 prior cells are byte-identical in
the live JSON. The remaining prior cell retains all six complete predecessor
fields and their hashes behind its recalibrated task head. Coverage, manifest,
milestone, hierarchy and task lineage are clean, with the four-node path above
and unchanged historical bootstrap-critical count and average.

Fresh local checks passed: 51 tensor/mainline tests, one resource-admission
rejection test, seven runtime native-session tests, two direct record-return
native/reference tests, 40 script tests, two file-budget tests and one host-path
policy test. Ordinary CLI build, formatting, UTF-8, documentation links,
workspace architecture and diff checks also pass. No GPU or remote Linux test
was run for this receipt; prior 224-test and 120-launch results remain historical.
